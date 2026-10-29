#![cfg(feature = "server")]

//! The engine's on-disk store.
//!
//! The engine keeps every database in one RocksDB at `<data dir>/engine`. Each database is a
//! column family with the database's name, each cached database keeps its reorg history in a
//! `<name>_cache` column family, and the `meta` column family holds `storage_version`. A commit
//! writes one `WriteBatch` across all column families, so a block is persisted whole or not at all.
//!
//! A data dir in the per-database layout (one RocksDB directory per database name) migrates
//! into this layout the first time it is opened. The migration builds the store in
//! `<data dir>/engine.migrating`, verifies every column family against its source, records the
//! source directories in `meta` under `migrated_from`, writes `storage_version`, renames the
//! directory to `engine`, and then deletes the listed source directories and the list. The
//! source directories stay untouched until the rename, so an interrupted migration restarts from
//! scratch on the next open, and an interrupted cleanup finishes from the list on it.

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use alloy::primitives::{Keccak256, B256};
use rocksdb::{
    ColumnFamily, ColumnFamilyDescriptor, IteratorMode, Options, WriteBatch, WriteOptions, DB,
};
use tracing::{info, warn};

/// Directory of the engine store inside the data dir.
pub(crate) const ENGINE_DIR: &str = "engine";

/// Directory an engine store is built in before it is renamed to [`ENGINE_DIR`].
pub(crate) const MIGRATING_DIR: &str = "engine.migrating";

/// Column family holding the store's own metadata.
pub(crate) const META_CF: &str = "meta";

/// Key of the storage format version in [`META_CF`].
const STORAGE_VERSION_KEY: &[u8] = b"storage_version";

/// Key in [`META_CF`] listing the per-database directories a migration consumed, one name per
/// line, present until the cleanup has deleted them.
const MIGRATED_FROM_KEY: &[u8] = b"migrated_from";

/// Lock file in the data dir, held while a process opens or migrates the store.
pub(crate) const LOCK_FILE: &str = "engine.lock";

/// Free space a migration needs, as a multiple of the source databases' size, in tenths.
const MIGRATION_SPACE_TENTHS: u64 = 11;

/// Storage format version this build reads and writes.
pub(crate) const STORAGE_VERSION: u32 = 2;

/// Databases with a reorg history, each also stored as a `<name>_cache` column family.
pub(crate) const CACHED_DATABASES: [&str; 12] = [
    "account_memory",
    "code",
    "account",
    "number_and_index_to_tx_hash",
    "tx_receipt",
    "inscription_id_to_tx_hash",
    "contract_address_to_inscription_id",
    "tx",
    "account_and_nonce_to_tx_hash",
    "pending_tx_hash_to_tx_id",
    "tx_trace",
    "block_hash_to_number",
];

/// Databases keyed by block number.
pub(crate) const BLOCK_DATABASES: [&str; 3] = [
    "block_number_to_block",
    "block_number_to_raw_block",
    "block_number_to_hash",
];

/// Key-value databases for global values and the data dir's configuration.
pub(crate) const PLAIN_DATABASES: [&str; 2] = ["global", "config"];

/// Largest number of keys copied in one migration batch.
const MIGRATION_BATCH_KEYS: usize = 10_000;

/// Largest number of key and value bytes copied in one migration batch.
const MIGRATION_BATCH_BYTES: usize = 32 * 1024 * 1024;

/// Keys copied between two progress lines of one column family.
const MIGRATION_PROGRESS_KEYS: u64 = 200_000;

/// Open file limit of the engine store, within the process limit set at open.
const MAX_OPEN_FILES: i32 = 2048;

/// Total write-ahead log size at which the engine store flushes the oldest column families.
const MAX_TOTAL_WAL_SIZE: u64 = 512 * 1024 * 1024;

/// Name of the column family that holds a cached database's history.
pub(crate) fn cache_name(name: &str) -> String {
    format!("{}_cache", name)
}

/// Every database name, which is both a column family of the engine store and a directory of
/// the per-database layout.
pub(crate) fn database_names() -> Vec<String> {
    let mut names = Vec::new();
    for name in CACHED_DATABASES {
        names.push(name.to_string());
        names.push(cache_name(name));
    }
    names.extend(BLOCK_DATABASES.iter().map(|name| name.to_string()));
    names.extend(PLAIN_DATABASES.iter().map(|name| name.to_string()));
    names
}

/// Every column family of the engine store.
fn column_family_names() -> Vec<String> {
    let mut names = database_names();
    names.push(META_CF.to_string());
    names
}

/// Returns the handle of a column family, or an error naming it.
pub(crate) fn column_family<'a>(
    db: &'a DB,
    name: &str,
) -> Result<&'a ColumnFamily, Box<dyn Error>> {
    db.cf_handle(name)
        .ok_or_else(|| format!("Column family {} is missing from the engine store", name).into())
}

/// Points where tests inject a failure. Builds outside tests never fail at them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FailPoint {
    /// A commit's batch is built and not yet written.
    CommitBeforeWrite,
    /// A commit's batch is written and the in-memory caches are not yet cleared.
    CommitAfterWrite,
    /// The migration copied this many column families.
    MigrationAfterCopying(usize),
    /// The migration verified every column family and has not written `storage_version`.
    MigrationBeforeStorageVersion,
    /// The migrated store is complete and not yet renamed to `engine`.
    MigrationBeforeRename,
    /// The migrated store is renamed to `engine` and no source directory is deleted yet.
    MigrationAfterRename,
    /// The cleanup deleted this many source directories.
    CleanupAfterDeleting(usize),
}

#[cfg(test)]
pub(crate) mod test_hooks {
    use std::cell::Cell;
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::{Path, PathBuf};

    use rocksdb::{IteratorMode, DB};

    use super::{column_family, database_names, open_per_database_dir, FailPoint};

    /// Every key-value pair of every database, by database name, in iteration order.
    pub(crate) type StoreDump = BTreeMap<String, Vec<(Vec<u8>, Vec<u8>)>>;

    /// A data dir written by brc20-prog v0.16.6, the last release with the per-database layout:
    /// the mainnet consensus fixture workload, committed.
    pub(crate) fn fixture_path() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/v0_16_6_mainnet_db")
    }

    pub(crate) fn copy_dir(from: &Path, to: &Path) {
        fs::create_dir_all(to).unwrap();
        for entry in fs::read_dir(from).unwrap() {
            let entry = entry.unwrap();
            let target = to.join(entry.file_name());
            if entry.file_type().unwrap().is_dir() {
                copy_dir(&entry.path(), &target);
            } else {
                fs::copy(entry.path(), &target).unwrap();
            }
        }
    }

    /// Every file under the per-database directories of `base_path` with its bytes, by path
    /// relative to `base_path`.
    pub(crate) fn file_tree(base_path: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
        let mut files = BTreeMap::new();
        for name in database_names() {
            let path = base_path.join(&name);
            if path.is_dir() {
                walk(base_path, &path, &mut files);
            }
        }
        files
    }

    /// Every file under `path` with its bytes, by path relative to `path`.
    pub(crate) fn dir_tree(path: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
        let mut files = BTreeMap::new();
        walk(path, path, &mut files);
        files
    }

    fn walk(root: &Path, dir: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(root, &path, files);
            } else {
                let relative = path.strip_prefix(root).unwrap().to_path_buf();
                files.insert(relative, fs::read(&path).unwrap());
            }
        }
    }

    /// Contents of every database column family of an engine store.
    pub(crate) fn dump_store(db: &DB) -> StoreDump {
        database_names()
            .into_iter()
            .map(|name| {
                let pairs = db
                    .full_iterator_cf(column_family(db, &name).unwrap(), IteratorMode::Start)
                    .map(|item| {
                        let (key, value) = item.unwrap();
                        (key.to_vec(), value.to_vec())
                    })
                    .collect();
                (name, pairs)
            })
            .collect()
    }

    /// Contents of every database directory of a data dir in the per-database layout. A missing
    /// directory reads as an empty database.
    pub(crate) fn dump_per_database_dirs(base_path: &Path) -> StoreDump {
        database_names()
            .into_iter()
            .map(|name| {
                let path = base_path.join(&name);
                let pairs = if path.is_dir() {
                    open_per_database_dir(&path)
                        .unwrap()
                        .full_iterator(IteratorMode::Start)
                        .map(|item| {
                            let (key, value) = item.unwrap();
                            (key.to_vec(), value.to_vec())
                        })
                        .collect()
                } else {
                    Vec::new()
                };
                (name, pairs)
            })
            .collect()
    }

    thread_local! {
        static ARMED: Cell<Option<FailPoint>> = const { Cell::new(None) };
        static FREE_SPACE: Cell<Option<u64>> = const { Cell::new(None) };
    }

    /// Makes the next pass through `point` on this thread fail.
    pub(crate) fn arm(point: FailPoint) {
        ARMED.with(|armed| armed.set(Some(point)));
    }

    /// Disarms the fail point of this thread.
    pub(crate) fn disarm() {
        ARMED.with(|armed| armed.set(None));
    }

    /// Reports `bytes` as the free space of every path on this thread, or the real value for None.
    pub(crate) fn set_free_space(bytes: Option<u64>) {
        FREE_SPACE.with(|free| free.set(bytes));
    }

    pub(super) fn take(point: FailPoint) -> bool {
        ARMED.with(|armed| {
            if armed.get() == Some(point) {
                armed.set(None);
                true
            } else {
                false
            }
        })
    }

    pub(super) fn free_space() -> Option<u64> {
        FREE_SPACE.with(|free| free.get())
    }
}

/// Fails if a test armed `point`.
pub(crate) fn fail_point(point: FailPoint) -> Result<(), Box<dyn Error>> {
    #[cfg(test)]
    if test_hooks::take(point) {
        return Err(format!("Injected failure at {:?}", point).into());
    }
    #[cfg(not(test))]
    let _ = point;
    Ok(())
}

fn store_options() -> Options {
    let mut opts = Options::default();
    opts.create_if_missing(true);
    opts.create_missing_column_families(true);
    opts.set_max_open_files(MAX_OPEN_FILES);
    // Column families share the write-ahead log, so a rarely written one would keep old log
    // files alive; past this total, RocksDB flushes the column families that hold them.
    opts.set_max_total_wal_size(MAX_TOTAL_WAL_SIZE);
    opts
}

/// Opens a RocksDB with every engine column family, plus any other column family already in it.
fn open_with_column_families(path: &Path) -> Result<DB, Box<dyn Error>> {
    let opts = store_options();
    let mut names = column_family_names();
    if path.join("CURRENT").exists() {
        for existing in DB::list_cf(&opts, path)? {
            if existing != rocksdb::DEFAULT_COLUMN_FAMILY_NAME && !names.contains(&existing) {
                names.push(existing);
            }
        }
    }
    let descriptors = names
        .into_iter()
        .map(|name| ColumnFamilyDescriptor::new(name, Options::default()));
    Ok(DB::open_cf_descriptors(&opts, path, descriptors)?)
}

fn read_storage_version(db: &DB) -> Result<Option<u32>, Box<dyn Error>> {
    let Some(bytes) = db.get_cf(column_family(db, META_CF)?, STORAGE_VERSION_KEY)? else {
        return Ok(None);
    };
    let version = std::str::from_utf8(&bytes)
        .ok()
        .and_then(|text| text.parse::<u32>().ok())
        .ok_or("The engine store has an unreadable storage_version")?;
    Ok(Some(version))
}

/// Writes `storage_version` durably: a synced WAL write followed by a flush of the meta column family.
fn write_storage_version(db: &DB) -> Result<(), Box<dyn Error>> {
    let meta = column_family(db, META_CF)?;
    let mut write_opts = WriteOptions::default();
    write_opts.set_sync(true);
    db.put_cf_opt(
        meta,
        STORAGE_VERSION_KEY,
        STORAGE_VERSION.to_string(),
        &write_opts,
    )?;
    db.flush_cf(meta)?;
    Ok(())
}

/// Opens the engine store of the data dir at `base_path`.
///
/// A data dir with neither layout gets a fresh store. A data dir in the per-database layout
/// migrates first. Once the store has opened, the source directories still listed in its
/// `migrated_from` are deleted, which finishes an interrupted cleanup, and an empty `config`
/// directory left by an earlier release is removed.
///
/// The whole open runs under an exclusive lock on `<data dir>/engine.lock`, so a second process
/// opening the same data dir fails instead of removing or racing a store being built.
///
/// Fails if another process is opening the data dir, if the store's `storage_version` is missing
/// or not one this build reads, or if a migration is needed and cannot complete.
pub(crate) fn open(base_path: &Path) -> Result<Arc<DB>, Box<dyn Error>> {
    // A migration opens a RocksDB per source directory next to the store, so the limit goes up
    // before anything is opened.
    rlimit::Resource::NOFILE.set(4096, 8192)?;
    fs::create_dir_all(base_path)?;
    // Held until the store is open; from then on RocksDB's own LOCK file guards it. Holding it
    // longer would also block the next open in this process, since every open locks anew.
    let _lock = lock_data_dir(base_path)?;

    let engine_path = base_path.join(ENGINE_DIR);
    if !engine_path.exists() {
        let sources = per_database_dirs(base_path);
        remove_leftover_build(base_path)?;
        if sources.is_empty() {
            create_fresh(base_path)?;
        } else {
            migrate(base_path, &sources)?;
        }
    }

    // Checked read-only first, so a store this build cannot read is never written.
    check_storage_version(&engine_path)?;
    let db = open_with_column_families(&engine_path)?;
    remove_migrated_dirs(base_path, &db)?;
    remove_stray_config_dir(base_path)?;
    warn_about_stray_dirs(base_path);
    Ok(Arc::new(db))
}

/// Takes the exclusive, non-blocking lock on `<data dir>/engine.lock` (`flock` on unix). The lock
/// is released when the returned file is dropped.
fn lock_data_dir(base_path: &Path) -> Result<fs::File, Box<dyn Error>> {
    let path = base_path.join(LOCK_FILE);
    let file = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(&path)?;
    match file.try_lock() {
        Ok(()) => Ok(file),
        Err(fs::TryLockError::WouldBlock) => Err(format!(
            "Another brc20-prog is opening this data dir ({}); try again once it has started or \
             stopped.",
            base_path.display()
        )
        .into()),
        Err(fs::TryLockError::Error(error)) => {
            Err(format!("Cannot lock {}: {}", path.display(), error).into())
        }
    }
}

/// Reads the store's `storage_version` through a read-only open and fails unless this build
/// reads it.
fn check_storage_version(engine_path: &Path) -> Result<(), Box<dyn Error>> {
    let opts = Options::default();
    let names = DB::list_cf(&opts, engine_path)?;
    let version = if names.iter().any(|name| name == META_CF) {
        let db = DB::open_cf_for_read_only(&opts, engine_path, &names, false)?;
        read_storage_version(&db)?
    } else {
        None
    };
    match version {
        Some(STORAGE_VERSION) => {}
        Some(version) if version > STORAGE_VERSION => {
            return Err(format!(
                "The engine store at {} has storage_version {}, written by a newer brc20-prog. \
                 This build reads storage_version {}; upgrade brc20-prog to open it.",
                engine_path.display(),
                version,
                STORAGE_VERSION
            )
            .into());
        }
        Some(version) => {
            return Err(format!(
                "The engine store at {} has unsupported storage_version {}; this build reads \
                 storage_version {}.",
                engine_path.display(),
                version,
                STORAGE_VERSION
            )
            .into());
        }
        None => {
            return Err(format!(
                "The engine store at {} has no storage_version, so it is not a complete store.",
                engine_path.display()
            )
            .into());
        }
    }
    Ok(())
}

/// Whether opening the data dir at `base_path` migrates it from the per-database layout.
pub(crate) fn needs_migration(base_path: &Path) -> bool {
    !base_path.join(ENGINE_DIR).exists() && !per_database_dirs(base_path).is_empty()
}

/// Directories of the per-database layout present in the data dir.
fn per_database_dirs(base_path: &Path) -> Vec<String> {
    database_names()
        .into_iter()
        .filter(|name| base_path.join(name).is_dir())
        .collect()
}

fn remove_leftover_build(base_path: &Path) -> Result<(), Box<dyn Error>> {
    let building = base_path.join(MIGRATING_DIR);
    if building.exists() {
        warn!(
            "Removing {}, left over from an interrupted run",
            building.display()
        );
        fs::remove_dir_all(&building)?;
    }
    Ok(())
}

/// Syncs a directory so a rename inside it survives power loss. Best effort where unsupported.
fn sync_dir(path: &Path) {
    if let Ok(dir) = fs::File::open(path) {
        let _ = dir.sync_all();
    }
}

/// Creates an empty store, built under [`MIGRATING_DIR`] so `engine` always has `storage_version`.
fn create_fresh(base_path: &Path) -> Result<(), Box<dyn Error>> {
    let building = base_path.join(MIGRATING_DIR);
    {
        let db = open_with_column_families(&building)?;
        write_storage_version(&db)?;
    }
    fs::rename(&building, base_path.join(ENGINE_DIR))?;
    sync_dir(base_path);
    Ok(())
}

/// Total size in bytes of the files under `path`.
///
/// RocksDB info logs (`LOG*`) are left out, since the migration does not copy them.
fn dir_size(path: &Path) -> Result<u64, Box<dyn Error>> {
    let mut total = 0;
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            total += dir_size(&entry.path())?;
        } else if !entry.file_name().to_string_lossy().starts_with("LOG") {
            total += metadata.len();
        }
    }
    Ok(total)
}

/// Free space in bytes available to this process on the filesystem holding `path`.
fn free_space(path: &Path) -> Result<Option<u64>, Box<dyn Error>> {
    #[cfg(test)]
    if let Some(bytes) = test_hooks::free_space() {
        return Ok(Some(bytes));
    }
    free_space_of(path)
}

#[cfg(unix)]
fn free_space_of(path: &Path) -> Result<Option<u64>, Box<dyn Error>> {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;

    let c_path = CString::new(path.as_os_str().as_bytes())?;
    let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };
    // SAFETY: c_path is a valid NUL-terminated string and stat is a valid out pointer.
    let result = unsafe { libc::statvfs(c_path.as_ptr(), &mut stat) };
    if result != 0 {
        return Err(format!(
            "Cannot read the free space of {}: {}",
            path.display(),
            std::io::Error::last_os_error()
        )
        .into());
    }
    Ok(Some(
        (stat.f_bavail as u64).saturating_mul(stat.f_frsize as u64),
    ))
}

#[cfg(not(unix))]
fn free_space_of(_path: &Path) -> Result<Option<u64>, Box<dyn Error>> {
    Ok(None)
}

/// Running key count and digest of a column family's key-value pairs in iteration order.
struct Digest {
    keys: u64,
    hasher: Keccak256,
}

impl Digest {
    fn new() -> Self {
        Self {
            keys: 0,
            hasher: Keccak256::new(),
        }
    }

    fn add(&mut self, key: &[u8], value: &[u8]) {
        self.keys += 1;
        self.hasher.update((key.len() as u64).to_be_bytes());
        self.hasher.update(key);
        self.hasher.update((value.len() as u64).to_be_bytes());
        self.hasher.update(value);
    }

    fn finish(self) -> (u64, B256) {
        (self.keys, self.hasher.finalize())
    }
}

/// Opens a database directory of the per-database layout without modifying it.
///
/// A read-only open creates no info log and writes no file, so the directory stays
/// byte-identical.
pub(crate) fn open_per_database_dir(path: &Path) -> Result<DB, Box<dyn Error>> {
    let mut opts = Options::default();
    opts.set_max_open_files(256);
    Ok(DB::open_for_read_only(&opts, path, false)?)
}

/// Copies the database at `source` into the `name` column family in bounded batches.
///
/// Returns the key count and digest of what was read.
fn copy_database(source: &Path, target: &DB, name: &str) -> Result<(u64, B256), Box<dyn Error>> {
    let source_db = open_per_database_dir(source)?;
    let cf = column_family(target, name)?;
    let mut write_opts = WriteOptions::default();
    // The store is flushed and verified before it is used, so the copy skips the WAL.
    write_opts.disable_wal(true);

    let mut digest = Digest::new();
    let mut batch = WriteBatch::default();
    let mut batch_bytes = 0;
    for item in source_db.full_iterator(IteratorMode::Start) {
        let (key, value) = item?;
        digest.add(&key, &value);
        batch_bytes += key.len() + value.len();
        batch.put_cf(cf, &key, &value);
        if batch.len() >= MIGRATION_BATCH_KEYS || batch_bytes >= MIGRATION_BATCH_BYTES {
            target.write_opt(std::mem::take(&mut batch), &write_opts)?;
            batch_bytes = 0;
        }
        if digest.keys.is_multiple_of(MIGRATION_PROGRESS_KEYS) {
            info!("Migrating {}: {} keys copied", name, digest.keys);
        }
    }
    if !batch.is_empty() {
        target.write_opt(batch, &write_opts)?;
    }
    Ok(digest.finish())
}

/// Key count and digest of a column family.
fn digest_column_family(db: &DB, name: &str) -> Result<(u64, B256), Box<dyn Error>> {
    let mut digest = Digest::new();
    for item in db.full_iterator_cf(column_family(db, name)?, IteratorMode::Start) {
        let (key, value) = item?;
        digest.add(&key, &value);
    }
    Ok(digest.finish())
}

/// Migrates the per-database layout in `base_path` into a new engine store.
///
/// `sources` names the per-database directories present; a database without a directory
/// becomes an empty column family.
fn migrate(base_path: &Path, sources: &[String]) -> Result<(), Box<dyn Error>> {
    let started = Instant::now();
    let building = base_path.join(MIGRATING_DIR);

    let mut source_size = 0;
    for name in sources {
        source_size += dir_size(&base_path.join(name))?;
    }
    // The copy is about the size of its sources; the margin covers RocksDB's own files and the
    // memtables flushed during the copy.
    let needed = source_size / 10 * MIGRATION_SPACE_TENTHS;
    match free_space(base_path)? {
        Some(available) if available < needed => {
            return Err(format!(
                "Migrating the engine data in {} to the single-store layout needs about {} MB of \
                 free disk space, and {} MB is available. Free up space and start again; the \
                 existing data is unchanged.",
                base_path.display(),
                needed.div_ceil(1024 * 1024),
                available / (1024 * 1024)
            )
            .into());
        }
        Some(_) => {}
        None => warn!("Cannot read free disk space; migrating without the space check"),
    }

    info!(
        "Migrating the engine data in {} ({} MB in {} databases) into {}",
        base_path.display(),
        source_size.div_ceil(1024 * 1024),
        sources.len(),
        building.display()
    );

    {
        let db = open_with_column_families(&building)?;
        let mut copied = Vec::new();
        for (index, name) in database_names().iter().enumerate() {
            let source = base_path.join(name);
            let summary = if sources.contains(name) {
                copy_database(&source, &db, name)?
            } else {
                Digest::new().finish()
            };
            info!("Migrated {}: {} keys", name, summary.0);
            copied.push((name.clone(), summary));
            fail_point(FailPoint::MigrationAfterCopying(index + 1))?;
        }

        for name in database_names() {
            db.flush_cf(column_family(&db, &name)?)?;
        }

        for (name, (source_keys, source_digest)) in copied {
            let (keys, digest) = digest_column_family(&db, &name)?;
            if keys != source_keys || digest != source_digest {
                return Err(format!(
                    "Migration check failed for {}: the source has {} keys (digest {}), the \
                     copy has {} keys (digest {}). The existing data is unchanged.",
                    name, source_keys, source_digest, keys, digest
                )
                .into());
            }
            info!("Verified {}: {} keys", name, keys);
        }
        info!("Verified every column family against its source");

        // The cleanup deletes exactly these directories, even when a later open finishes it.
        db.put_cf(
            column_family(&db, META_CF)?,
            MIGRATED_FROM_KEY,
            sources.join("\n"),
        )?;

        fail_point(FailPoint::MigrationBeforeStorageVersion)?;
        write_storage_version(&db)?;
    }

    fail_point(FailPoint::MigrationBeforeRename)?;
    fs::rename(&building, base_path.join(ENGINE_DIR))?;
    sync_dir(base_path);
    fail_point(FailPoint::MigrationAfterRename)?;

    info!(
        "Migrated the engine data in {:.1?}; removing the per-database directories",
        started.elapsed()
    );
    Ok(())
}

/// The per-database directories a migration consumed, recorded in [`META_CF`] until the cleanup
/// has deleted them.
fn read_migrated_from(db: &DB) -> Result<Option<Vec<String>>, Box<dyn Error>> {
    let Some(bytes) = db.get_cf(column_family(db, META_CF)?, MIGRATED_FROM_KEY)? else {
        return Ok(None);
    };
    let names = std::str::from_utf8(&bytes)
        .map_err(|_| "The engine store has an unreadable migrated_from list")?
        .split('\n')
        .filter(|name| !name.is_empty())
        .map(|name| name.to_string())
        .collect::<Vec<_>>();
    if let Some(unknown) = names.iter().find(|name| !database_names().contains(name)) {
        return Err(format!(
            "The engine store's migrated_from list names {}, which is not a database",
            unknown
        )
        .into());
    }
    Ok(Some(names))
}

/// Deletes the per-database directories listed in `migrated_from`, then the list itself.
///
/// Runs only once `engine` holds a complete store, and finishes a cleanup an earlier open left
/// unfinished. Directories not on the list are left alone.
fn remove_migrated_dirs(base_path: &Path, db: &DB) -> Result<(), Box<dyn Error>> {
    let Some(names) = read_migrated_from(db)? else {
        return Ok(());
    };
    let mut removed = 0;
    for name in &names {
        let path: PathBuf = base_path.join(name);
        if path.is_dir() {
            fs::remove_dir_all(&path)?;
            removed += 1;
            fail_point(FailPoint::CleanupAfterDeleting(removed))?;
        }
    }
    sync_dir(base_path);
    db.delete_cf(column_family(db, META_CF)?, MIGRATED_FROM_KEY)?;
    info!(
        "Removed {} per-database directories the migration consumed",
        removed
    );
    Ok(())
}

/// Warns about per-database directories next to a complete `engine` that no `migrated_from` list
/// names. The engine does not read them, and they are left in place. A kept `config` directory has
/// its own warning.
fn warn_about_stray_dirs(base_path: &Path) {
    let stray: Vec<String> = per_database_dirs(base_path)
        .into_iter()
        .filter(|name| name != "config")
        .collect();
    if !stray.is_empty() {
        warn!(
            "Leaving per-database directories next to the engine store in {} that no migration \
             listed: {}. The engine does not read them.",
            base_path.display(),
            stray.join(", ")
        );
    }
}

/// Handles a `config` directory next to a complete `engine`, which an earlier release creates
/// when it fails to open a migrated data dir. An empty one (no files, or a RocksDB without keys)
/// is removed; any other is left in place with a warning.
fn remove_stray_config_dir(base_path: &Path) -> Result<(), Box<dyn Error>> {
    let path = base_path.join("config");
    if !path.is_dir() {
        return Ok(());
    }
    let empty = fs::read_dir(&path)?.next().is_none()
        || open_per_database_dir(&path)
            .map(|db| db.iterator(IteratorMode::Start).next().is_none())
            .unwrap_or(false);
    if empty {
        fs::remove_dir_all(&path)?;
        info!(
            "Removed {}, an empty config database next to the engine store, as created by an \
             earlier brc20-prog that cannot open this data dir",
            path.display()
        );
    } else {
        warn!(
            "{} is next to the engine store and is not empty; leaving it in place. The engine \
             keeps its configuration in the engine store and does not read this directory.",
            path.display()
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::test_hooks::{
        self, copy_dir, dir_tree, dump_per_database_dirs, dump_store, file_tree, fixture_path,
        StoreDump,
    };
    use super::*;

    /// A copy of the v0.16.6 fixture and the contents it must migrate to.
    fn old_layout_data_dir() -> (TempDir, StoreDump) {
        let dir = TempDir::new().unwrap();
        copy_dir(&fixture_path(), dir.path());
        (dir, dump_per_database_dirs(&fixture_path()))
    }

    fn read_migrated_from_at(path: &Path) -> Option<Vec<String>> {
        read_migrated_from(&open_with_column_families(&path.join(ENGINE_DIR)).unwrap()).unwrap()
    }

    /// Entries of a data dir, without the lock file every open leaves behind.
    fn entries(path: &Path) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(path)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().into_string().unwrap())
            .filter(|name| name != LOCK_FILE)
            .collect();
        names.sort();
        names
    }

    /// The data dir holds no `engine`, and its per-database directories are byte-identical to
    /// the fixture's.
    fn assert_untouched(path: &Path, expected: &StoreDump) {
        assert!(!path.join(ENGINE_DIR).exists());
        assert_eq!(file_tree(path), file_tree(&fixture_path()));
        assert_eq!(&dump_per_database_dirs(path), expected);
    }

    /// The data dir holds only a migrated store with the expected contents and no pending cleanup.
    fn assert_migrated(path: &Path, expected: &StoreDump) {
        let db = open(path).unwrap();
        assert_eq!(read_storage_version(&db).unwrap(), Some(STORAGE_VERSION));
        assert_eq!(read_migrated_from(&db).unwrap(), None);
        assert_eq!(&dump_store(&db), expected);
        assert_eq!(entries(path), vec![ENGINE_DIR.to_string()]);
    }

    #[test]
    fn test_fresh_data_dir_opens_in_the_single_store_layout() {
        let dir = TempDir::new().unwrap();
        let db = open(dir.path()).unwrap();
        assert_eq!(read_storage_version(&db).unwrap(), Some(STORAGE_VERSION));
        for name in column_family_names() {
            assert!(db.cf_handle(&name).is_some(), "missing {}", name);
        }
        assert_eq!(entries(dir.path()), vec![ENGINE_DIR.to_string()]);
    }

    #[test]
    fn test_fixture_covers_every_kind_of_database() {
        let expected = dump_per_database_dirs(&fixture_path());
        for name in [
            "account",
            "account_cache",
            "account_memory",
            "code",
            "tx",
            "tx_receipt",
            "tx_trace",
            "block_number_to_block",
            "block_number_to_raw_block",
            "block_number_to_hash",
            "block_hash_to_number",
            "global",
            "config",
        ] {
            assert!(
                !expected[name].is_empty(),
                "{} is empty in the fixture",
                name
            );
        }
    }

    #[test]
    fn test_migration_copies_every_key_and_value() {
        let (dir, expected) = old_layout_data_dir();
        assert_migrated(dir.path(), &expected);
    }

    #[test]
    fn test_migration_interrupted_after_each_column_family_completes_on_next_open() {
        let count = database_names().len();
        for copied in 1..=count {
            let (dir, expected) = old_layout_data_dir();
            test_hooks::arm(FailPoint::MigrationAfterCopying(copied));
            assert!(open(dir.path()).is_err());
            test_hooks::disarm();
            assert!(dir.path().join(MIGRATING_DIR).exists());
            assert_untouched(dir.path(), &expected);

            assert_migrated(dir.path(), &expected);
        }
    }

    #[test]
    fn test_migration_interrupted_before_the_rename_completes_on_next_open() {
        for point in [
            FailPoint::MigrationBeforeStorageVersion,
            FailPoint::MigrationBeforeRename,
        ] {
            let (dir, expected) = old_layout_data_dir();
            test_hooks::arm(point);
            assert!(open(dir.path()).is_err());
            test_hooks::disarm();
            assert_untouched(dir.path(), &expected);

            assert_migrated(dir.path(), &expected);
        }
    }

    #[test]
    fn test_cleanup_interrupted_after_the_rename_finishes_on_next_open() {
        let (dir, expected) = old_layout_data_dir();
        test_hooks::arm(FailPoint::MigrationAfterRename);
        assert!(open(dir.path()).is_err());
        test_hooks::disarm();
        assert!(dir.path().join(ENGINE_DIR).exists());
        assert!(!dir.path().join(MIGRATING_DIR).exists());
        assert_eq!(dump_per_database_dirs(dir.path()), expected);
        assert_eq!(read_migrated_from_at(dir.path()), Some(database_names()));

        assert_migrated(dir.path(), &expected);
    }

    #[test]
    fn test_cleanup_interrupted_midway_finishes_from_the_list_on_next_open() {
        let count = database_names().len();
        for deleted in [1, count / 2, count - 1] {
            let (dir, expected) = old_layout_data_dir();
            test_hooks::arm(FailPoint::CleanupAfterDeleting(deleted));
            assert!(open(dir.path()).is_err());
            test_hooks::disarm();
            assert_eq!(per_database_dirs(dir.path()).len(), count - deleted);
            assert_eq!(read_migrated_from_at(dir.path()), Some(database_names()));

            assert_migrated(dir.path(), &expected);
        }
    }

    #[test]
    fn test_cleanup_deletes_only_the_listed_dirs() {
        // A data dir missing some databases migrates and lists only the ones it had.
        let (dir, _) = old_layout_data_dir();
        fs::remove_dir_all(dir.path().join("tx_trace_cache")).unwrap();
        fs::create_dir_all(dir.path().join("notes")).unwrap();
        fs::write(dir.path().join("notes/readme"), b"keep").unwrap();
        test_hooks::arm(FailPoint::MigrationAfterRename);
        assert!(open(dir.path()).is_err());
        test_hooks::disarm();
        let listed = read_migrated_from_at(dir.path()).unwrap();
        assert_eq!(listed.len(), database_names().len() - 1);
        assert!(!listed.contains(&"tx_trace_cache".to_string()));

        // A database-named dir that appears after the migration is not on the list.
        fs::create_dir_all(dir.path().join("tx_trace_cache")).unwrap();
        fs::write(dir.path().join("tx_trace_cache/readme"), b"keep").unwrap();
        drop(open(dir.path()).unwrap());
        assert_eq!(
            entries(dir.path()),
            vec![
                ENGINE_DIR.to_string(),
                "notes".to_string(),
                "tx_trace_cache".to_string()
            ]
        );

        // With the list gone, later opens leave every dir alone.
        fs::create_dir_all(dir.path().join("tx")).unwrap();
        fs::write(dir.path().join("tx/readme"), b"keep").unwrap();
        drop(open(dir.path()).unwrap());
        assert_eq!(fs::read(dir.path().join("tx/readme")).unwrap(), b"keep");
        assert_eq!(fs::read(dir.path().join("notes/readme")).unwrap(), b"keep");
    }

    #[test]
    fn test_empty_config_dir_next_to_the_engine_is_removed() {
        for kind in ["empty dir", "empty database"] {
            let (dir, expected) = old_layout_data_dir();
            drop(open(dir.path()).unwrap());
            let config = dir.path().join("config");
            if kind == "empty dir" {
                fs::create_dir_all(&config).unwrap();
            } else {
                // What v0.16.6 leaves when it fails to open a migrated data dir.
                let mut opts = Options::default();
                opts.create_if_missing(true);
                drop(DB::open(&opts, &config).unwrap());
            }
            assert_migrated(dir.path(), &expected);
        }
    }

    #[test]
    fn test_config_dir_with_data_next_to_the_engine_is_kept() {
        let (dir, _) = old_layout_data_dir();
        drop(open(dir.path()).unwrap());
        let config = dir.path().join("config");
        {
            let mut opts = Options::default();
            opts.create_if_missing(true);
            DB::open(&opts, &config)
                .unwrap()
                .put(b"key", b"value")
                .unwrap();
        }
        drop(open(dir.path()).unwrap());
        assert!(config.is_dir());
        let kept = open_per_database_dir(&config).unwrap();
        assert_eq!(kept.get(b"key").unwrap().as_deref(), Some(&b"value"[..]));
    }

    #[test]
    fn test_leftover_migration_dir_is_removed_and_redone() {
        let (dir, expected) = old_layout_data_dir();
        {
            // A store that looks complete but was never renamed, holding a stray key.
            let leftover = open_with_column_families(&dir.path().join(MIGRATING_DIR)).unwrap();
            leftover
                .put_cf(
                    column_family(&leftover, "account").unwrap(),
                    b"stray",
                    b"key",
                )
                .unwrap();
            write_storage_version(&leftover).unwrap();
        }

        assert_migrated(dir.path(), &expected);
    }

    #[test]
    fn test_insufficient_free_space_fails_without_changes() {
        let (dir, expected) = old_layout_data_dir();
        test_hooks::set_free_space(Some(1024));
        let error = open(dir.path()).err().unwrap().to_string();
        test_hooks::set_free_space(None);
        assert!(error.contains("free disk space"), "{}", error);
        assert!(!dir.path().join(MIGRATING_DIR).exists());
        assert_untouched(dir.path(), &expected);

        assert_migrated(dir.path(), &expected);
    }

    #[test]
    fn test_newer_storage_version_is_rejected() {
        let dir = TempDir::new().unwrap();
        drop(open(dir.path()).unwrap());
        {
            let mut db = open_with_column_families(&dir.path().join(ENGINE_DIR)).unwrap();
            db.put_cf(
                column_family(&db, META_CF).unwrap(),
                STORAGE_VERSION_KEY,
                (STORAGE_VERSION + 1).to_string(),
            )
            .unwrap();
            db.create_cf("added_by_a_newer_version", &Options::default())
                .unwrap();
        }
        let engine_path = dir.path().join(ENGINE_DIR);
        let before = dir_tree(&engine_path);
        let error = open(dir.path()).err().unwrap().to_string();
        assert!(error.contains("newer brc20-prog"), "{}", error);
        // The store is only read, never opened for writing.
        assert_eq!(dir_tree(&engine_path), before);
    }

    #[test]
    fn test_open_fails_while_another_process_holds_the_lock() {
        let (dir, expected) = old_layout_data_dir();
        let holder = lock_data_dir(dir.path()).unwrap();
        let error = open(dir.path()).err().unwrap().to_string();
        assert!(
            error.contains("Another brc20-prog is opening this data dir"),
            "{}",
            error
        );
        assert!(!dir.path().join(MIGRATING_DIR).exists());
        assert_untouched(dir.path(), &expected);
        assert_eq!(
            entries(dir.path()),
            fixture_entries(),
            "nothing but the lock file was added"
        );

        drop(holder);
        assert_migrated(dir.path(), &expected);
    }

    #[test]
    fn test_lock_contention_keeps_a_store_being_built() {
        // A second open while the first is still building must not remove engine.migrating.
        let (dir, _) = old_layout_data_dir();
        test_hooks::arm(FailPoint::MigrationBeforeRename);
        assert!(open(dir.path()).is_err());
        test_hooks::disarm();
        let building = dir.path().join(MIGRATING_DIR);
        let before = dir_tree(&building);

        let holder = lock_data_dir(dir.path()).unwrap();
        assert!(open(dir.path()).is_err());
        assert_eq!(dir_tree(&building), before);
        drop(holder);
    }

    fn fixture_entries() -> Vec<String> {
        entries(&fixture_path())
    }

    #[test]
    fn test_store_without_storage_version_is_rejected() {
        let dir = TempDir::new().unwrap();
        drop(open_with_column_families(&dir.path().join(ENGINE_DIR)).unwrap());
        let error = open(dir.path()).err().unwrap().to_string();
        assert!(error.contains("no storage_version"), "{}", error);
    }
}
