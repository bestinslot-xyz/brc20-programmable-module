//! Bounded in-memory pool of signed raw transactions awaiting inclusion.
//!
//! Entries are keyed by transaction hash and expire after a TTL. When the entry
//! count or the total raw byte size would exceed its cap, the oldest entries
//! are evicted first. Each `(from, nonce)` pair holds a bounded number of
//! distinct transactions. Nothing is persisted.

use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::time::{Duration, Instant};

use alloy::primitives::{Address, Bytes, B256};

/// How long a pooled transaction stays live.
pub const TX_POOL_TTL: Duration = Duration::from_secs(60);
/// Maximum number of pooled transactions.
pub const TX_POOL_MAX_ENTRIES: usize = 10_000;
/// Maximum total raw bytes across pooled transactions.
pub const TX_POOL_MAX_BYTES: usize = 64 * 1024 * 1024;
/// Maximum number of distinct live transactions for one `(from, nonce)` pair.
pub const TX_POOL_MAX_PER_NONCE: usize = 4;
/// Maximum total raw bytes in one listing of unsent transactions.
pub const UNSENT_LIST_MAX_BYTES: usize = 16 * 1024 * 1024;

/// A signed raw transaction held in the pool.
#[derive(Debug, Clone)]
pub struct PooledTx {
    /// `keccak256(raw)`.
    pub hash: B256,
    /// Shared buffer, so cloning an entry does not copy the transaction.
    pub raw: Bytes,
    pub from: Address,
    pub nonce: u64,
    pub received_at: Instant,
}

/// Why the pool refused a transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PoolError {
    /// The sender already has [`TX_POOL_MAX_PER_NONCE`] live transactions for this nonce.
    TooManyForNonce,
}

impl fmt::Display for PoolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PoolError::TooManyForNonce => {
                write!(f, "too many pending transactions for this nonce")
            }
        }
    }
}

impl std::error::Error for PoolError {}

/// Pool of signed raw transactions with a TTL, entry and byte caps, and a
/// per-nonce cap.
#[derive(Debug)]
pub struct TxPool {
    ttl: Duration,
    max_entries: usize,
    max_bytes: usize,
    entries: HashMap<B256, PooledTx>,
    /// Hashes in insertion order, oldest first.
    order: VecDeque<B256>,
    bytes: usize,
    /// Live entry count for each `(from, nonce)` pair.
    per_nonce: HashMap<(Address, u64), usize>,
}

impl TxPool {
    pub fn new(ttl: Duration, max_entries: usize, max_bytes: usize) -> Self {
        Self {
            ttl,
            max_entries,
            max_bytes,
            entries: HashMap::new(),
            order: VecDeque::new(),
            bytes: 0,
            per_nonce: HashMap::new(),
        }
    }

    /// Inserts `tx` and returns its hash. A transaction whose hash is already
    /// pooled returns that hash and keeps the original `received_at`. Expired
    /// entries are purged first, a transaction beyond the per-nonce cap is
    /// refused, and the oldest entries are evicted until both size caps hold.
    pub fn insert(&mut self, tx: PooledTx, now: Instant) -> Result<B256, PoolError> {
        self.purge_expired(now);
        let hash = tx.hash;
        if self.entries.contains_key(&hash) {
            return Ok(hash);
        }
        let key = (tx.from, tx.nonce);
        let count = self.per_nonce.entry(key).or_insert(0);
        if *count >= TX_POOL_MAX_PER_NONCE {
            return Err(PoolError::TooManyForNonce);
        }
        *count += 1;
        self.bytes += tx.raw.len();
        self.entries.insert(hash, tx);
        self.order.push_back(hash);
        while self.entries.len() > self.max_entries || self.bytes > self.max_bytes {
            let Some(oldest) = self.order.pop_front() else {
                break;
            };
            self.take(&oldest);
        }
        Ok(hash)
    }

    pub fn get(&mut self, hash: &B256, now: Instant) -> Option<&PooledTx> {
        self.purge_expired(now);
        self.entries.get(hash)
    }

    pub fn remove(&mut self, hash: &B256) {
        if self.take(hash) {
            self.order.retain(|h| h != hash);
        }
    }

    /// Live entries, oldest first, stopping before the total raw bytes would
    /// exceed `max_bytes`.
    pub fn list(&mut self, now: Instant, max_bytes: usize) -> Vec<PooledTx> {
        self.purge_expired(now);
        let mut total = 0usize;
        let mut listed = Vec::new();
        for hash in &self.order {
            let Some(tx) = self.entries.get(hash) else {
                continue;
            };
            total = total.saturating_add(tx.raw.len());
            if total > max_bytes {
                break;
            }
            listed.push(tx.clone());
        }
        listed
    }

    /// Highest nonce among live entries from `from`, if any.
    pub fn max_nonce_from(&mut self, from: &Address, now: Instant) -> Option<u64> {
        self.purge_expired(now);
        self.entries
            .values()
            .filter(|t| &t.from == from)
            .map(|t| t.nonce)
            .max()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Total raw bytes of the pooled transactions.
    pub fn bytes(&self) -> usize {
        self.bytes
    }

    fn purge_expired(&mut self, now: Instant) {
        while let Some(front) = self.order.front() {
            let expired = self
                .entries
                .get(front)
                .is_none_or(|t| now.saturating_duration_since(t.received_at) > self.ttl);
            if !expired {
                break;
            }
            let hash = self.order.pop_front().expect("front exists");
            self.take(&hash);
        }
    }

    /// Removes the entry for `hash` from `entries` and its byte and per-nonce
    /// counts, leaving `order` to the caller. Returns whether an entry existed.
    fn take(&mut self, hash: &B256) -> bool {
        let Some(removed) = self.entries.remove(hash) else {
            return false;
        };
        self.bytes -= removed.raw.len();
        let key = (removed.from, removed.nonce);
        if let Some(count) = self.per_nonce.get_mut(&key) {
            *count -= 1;
            if *count == 0 {
                self.per_nonce.remove(&key);
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use alloy::primitives::{Address, Bytes, B256};

    use super::*;

    fn hash(n: u8) -> B256 {
        B256::repeat_byte(n)
    }

    fn tx(n: u8, from: Address, nonce: u64, size: usize, at: Instant) -> PooledTx {
        PooledTx {
            hash: hash(n),
            raw: Bytes::from(vec![n; size]),
            from,
            nonce,
            received_at: at,
        }
    }

    fn pool() -> TxPool {
        TxPool::new(TX_POOL_TTL, 100, 1024 * 1024)
    }

    fn hashes(txs: &[PooledTx]) -> Vec<B256> {
        txs.iter().map(|t| t.hash).collect()
    }

    #[test]
    fn insert_then_get_returns_tx() {
        let t0 = Instant::now();
        let mut p = pool();
        let h = p.insert(tx(1, Address::ZERO, 0, 10, t0), t0).unwrap();
        assert_eq!(h, hash(1));
        let got = p.get(&h, t0).expect("present");
        assert_eq!(got.raw, Bytes::from(vec![1u8; 10]));
        assert_eq!(p.len(), 1);
        assert_eq!(p.bytes(), 10);
    }

    #[test]
    fn duplicate_insert_returns_same_hash_and_keeps_received_at() {
        let t0 = Instant::now();
        let mut p = pool();
        p.insert(tx(1, Address::ZERO, 0, 10, t0), t0).unwrap();
        let t1 = t0 + Duration::from_secs(30);
        let h = p.insert(tx(1, Address::ZERO, 0, 10, t1), t1).unwrap();
        assert_eq!(h, hash(1));
        assert_eq!(p.len(), 1);
        assert_eq!(p.bytes(), 10);
        assert!(p.get(&h, t0 + Duration::from_secs(61)).is_none());
    }

    #[test]
    fn same_sender_same_nonce_different_hash_both_kept() {
        let t0 = Instant::now();
        let mut p = pool();
        let a = Address::repeat_byte(7);
        p.insert(tx(1, a, 5, 10, t0), t0).unwrap();
        p.insert(tx(2, a, 5, 10, t0), t0).unwrap();
        assert_eq!(p.len(), 2);
        assert!(p.get(&hash(1), t0).is_some());
        assert!(p.get(&hash(2), t0).is_some());
    }

    #[test]
    fn per_nonce_cap_refuses_extra_tx_but_accepts_duplicates() {
        let t0 = Instant::now();
        let mut p = pool();
        let a = Address::repeat_byte(7);
        for n in 1..=TX_POOL_MAX_PER_NONCE as u8 {
            p.insert(tx(n, a, 5, 10, t0), t0).unwrap();
        }
        let extra = TX_POOL_MAX_PER_NONCE as u8 + 1;
        assert_eq!(
            p.insert(tx(extra, a, 5, 10, t0), t0),
            Err(PoolError::TooManyForNonce)
        );
        assert!(p.get(&hash(extra), t0).is_none());
        // A duplicate of a live entry still returns its hash
        assert_eq!(p.insert(tx(1, a, 5, 10, t0), t0), Ok(hash(1)));
        // Other nonces and other senders are not affected
        p.insert(tx(extra, a, 6, 10, t0), t0).unwrap();
        p.insert(tx(extra + 1, Address::repeat_byte(8), 5, 10, t0), t0)
            .unwrap();
        // Removing one entry frees a slot for that nonce
        p.remove(&hash(1));
        p.insert(tx(extra + 2, a, 5, 10, t0), t0).unwrap();
        assert_eq!(p.len(), TX_POOL_MAX_PER_NONCE + 2);
    }

    #[test]
    fn per_nonce_cap_frees_slots_on_expiry_and_eviction() {
        let t0 = Instant::now();
        let a = Address::repeat_byte(7);
        let mut p = pool();
        for n in 1..=TX_POOL_MAX_PER_NONCE as u8 {
            p.insert(tx(n, a, 5, 10, t0), t0).unwrap();
        }
        let t1 = t0 + Duration::from_secs(61);
        p.insert(tx(50, a, 5, 10, t1), t1).unwrap();

        let mut p = TxPool::new(TX_POOL_TTL, TX_POOL_MAX_PER_NONCE, 1024 * 1024);
        for n in 1..=TX_POOL_MAX_PER_NONCE as u8 {
            p.insert(tx(n, a, 5, 10, t0), t0).unwrap();
        }
        // An unrelated insert evicts the oldest entry for nonce 5
        p.insert(tx(60, a, 6, 10, t0), t0).unwrap();
        p.insert(tx(61, a, 5, 10, t0), t0).unwrap();
    }

    #[test]
    fn entry_expires_after_ttl() {
        let t0 = Instant::now();
        let mut p = pool();
        p.insert(tx(1, Address::ZERO, 0, 10, t0), t0).unwrap();
        assert!(p.get(&hash(1), t0 + Duration::from_secs(59)).is_some());
        assert!(p.get(&hash(1), t0 + Duration::from_secs(61)).is_none());
        assert!(p.list(t0 + Duration::from_secs(61), usize::MAX).is_empty());
        assert_eq!(p.len(), 0);
        assert_eq!(p.bytes(), 0);
    }

    #[test]
    fn entry_cap_evicts_oldest() {
        let t0 = Instant::now();
        let mut p = TxPool::new(TX_POOL_TTL, 3, 1024 * 1024);
        for n in 1..=4 {
            p.insert(tx(n, Address::ZERO, n as u64, 10, t0), t0)
                .unwrap();
        }
        assert!(p.get(&hash(1), t0).is_none());
        for n in 2..=4 {
            assert!(p.get(&hash(n), t0).is_some());
        }
        assert_eq!(p.len(), 3);
    }

    #[test]
    fn byte_cap_evicts_oldest() {
        let t0 = Instant::now();
        let mut p = TxPool::new(TX_POOL_TTL, 100, 250);
        for n in 1..=3 {
            p.insert(tx(n, Address::ZERO, n as u64, 100, t0), t0)
                .unwrap();
        }
        assert!(p.get(&hash(1), t0).is_none());
        assert!(p.get(&hash(2), t0).is_some());
        assert!(p.get(&hash(3), t0).is_some());
        assert!(p.bytes() <= 250);
    }

    #[test]
    fn remove_drops_entry_and_bytes() {
        let t0 = Instant::now();
        let mut p = pool();
        p.insert(tx(1, Address::ZERO, 0, 10, t0), t0).unwrap();
        p.insert(tx(2, Address::ZERO, 1, 20, t0), t0).unwrap();
        p.remove(&hash(1));
        assert!(p.get(&hash(1), t0).is_none());
        assert_eq!(p.len(), 1);
        assert_eq!(p.bytes(), 20);
        assert_eq!(p.list(t0, usize::MAX).len(), 1);
    }

    #[test]
    fn list_returns_live_entries_oldest_first() {
        let t0 = Instant::now();
        let mut p = pool();
        for n in 1..=3 {
            p.insert(tx(n, Address::ZERO, n as u64, 10, t0), t0)
                .unwrap();
        }
        assert_eq!(
            hashes(&p.list(t0, usize::MAX)),
            vec![hash(1), hash(2), hash(3)]
        );
    }

    #[test]
    fn list_returns_oldest_prefix_within_byte_budget() {
        let t0 = Instant::now();
        let mut p = pool();
        for n in 1..=4 {
            p.insert(tx(n, Address::ZERO, n as u64, 100, t0), t0)
                .unwrap();
        }
        assert_eq!(hashes(&p.list(t0, 250)), vec![hash(1), hash(2)]);
        assert_eq!(hashes(&p.list(t0, 300)), vec![hash(1), hash(2), hash(3)]);
        assert!(p.list(t0, 99).is_empty());
        assert_eq!(p.len(), 4);
    }

    #[test]
    fn max_nonce_from_ignores_expired_and_other_senders() {
        let t0 = Instant::now();
        let mut p = pool();
        let a = Address::repeat_byte(1);
        let b = Address::repeat_byte(2);
        p.insert(tx(1, a, 0, 10, t0), t0).unwrap();
        p.insert(tx(2, a, 1, 10, t0), t0).unwrap();
        p.insert(tx(3, a, 1, 10, t0), t0).unwrap();
        p.insert(tx(4, b, 9, 10, t0), t0).unwrap();
        assert_eq!(p.max_nonce_from(&a, t0), Some(1));
        assert_eq!(p.max_nonce_from(&a, t0 + Duration::from_secs(61)), None);
    }
}
