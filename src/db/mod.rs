mod cached_database;
mod database;

mod brc20_prog_database;
pub mod types;

#[cfg(feature = "server")]
pub use brc20_prog_database::Brc20ProgDatabase;

#[cfg(all(test, feature = "server"))]
thread_local! {
    /// Writes a test lets through before `write_batch` fails as a crash would, or `None` for no limit.
    pub(crate) static WRITES_BEFORE_CRASH: std::cell::Cell<Option<usize>> =
        const { std::cell::Cell::new(None) };
}

/// Writes `batch` to `db` unless it is empty. With `sync`, the WAL is synced before returning.
/// In tests, `WRITES_BEFORE_CRASH` can make it fail after the write, as a crash would.
#[cfg(feature = "server")]
pub(crate) fn write_batch(
    db: &rocksdb::DB,
    batch: rocksdb::WriteBatch,
    sync: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if batch.is_empty() {
        return Ok(());
    }
    let mut options = rocksdb::WriteOptions::default();
    options.set_sync(sync);
    db.write_opt(batch, &options)?;
    #[cfg(test)]
    if let Some(left) = WRITES_BEFORE_CRASH.get() {
        WRITES_BEFORE_CRASH.set(Some(left.saturating_sub(1)));
        if left <= 1 {
            return Err("Injected crash".into());
        }
    }
    Ok(())
}
