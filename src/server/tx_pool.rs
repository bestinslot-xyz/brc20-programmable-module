//! Bounded in-memory pool of signed raw transactions awaiting inclusion.
//!
//! Entries are keyed by transaction hash and expire after a TTL. When the entry
//! count or the total raw byte size would exceed its cap, the oldest entries
//! are evicted first. Nothing is persisted.

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

use alloy::primitives::{Address, B256};

/// How long a pooled transaction stays live.
pub const TX_POOL_TTL: Duration = Duration::from_secs(60);
/// Maximum number of pooled transactions.
pub const TX_POOL_MAX_ENTRIES: usize = 10_000;
/// Maximum total raw bytes across pooled transactions.
pub const TX_POOL_MAX_BYTES: usize = 64 * 1024 * 1024;

/// A signed raw transaction held in the pool.
#[derive(Debug, Clone)]
pub struct PooledTx {
    /// `keccak256(raw)`.
    pub hash: B256,
    pub raw: Vec<u8>,
    pub from: Address,
    pub nonce: u64,
    pub received_at: Instant,
}

/// Pool of signed raw transactions with a TTL and entry and byte caps.
#[derive(Debug)]
pub struct TxPool {
    ttl: Duration,
    max_entries: usize,
    max_bytes: usize,
    entries: HashMap<B256, PooledTx>,
    /// Hashes in insertion order, oldest first.
    order: VecDeque<B256>,
    bytes: usize,
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
        }
    }

    /// Inserts `tx`, or returns the hash of the existing entry with the same
    /// hash (keeping its original `received_at`). Expired entries are purged
    /// first and the oldest entries are evicted until both caps hold.
    pub fn insert(&mut self, tx: PooledTx, now: Instant) -> B256 {
        self.purge_expired(now);
        let hash = tx.hash;
        if self.entries.contains_key(&hash) {
            return hash;
        }
        self.bytes += tx.raw.len();
        self.entries.insert(hash, tx);
        self.order.push_back(hash);
        while self.entries.len() > self.max_entries || self.bytes > self.max_bytes {
            let Some(oldest) = self.order.pop_front() else {
                break;
            };
            if let Some(evicted) = self.entries.remove(&oldest) {
                self.bytes -= evicted.raw.len();
            }
        }
        hash
    }

    pub fn get(&mut self, hash: &B256, now: Instant) -> Option<&PooledTx> {
        self.purge_expired(now);
        self.entries.get(hash)
    }

    pub fn remove(&mut self, hash: &B256) {
        if let Some(removed) = self.entries.remove(hash) {
            self.bytes -= removed.raw.len();
            self.order.retain(|h| h != hash);
        }
    }

    /// Live entries, oldest first.
    pub fn list(&mut self, now: Instant) -> Vec<PooledTx> {
        self.purge_expired(now);
        self.order
            .iter()
            .filter_map(|h| self.entries.get(h).cloned())
            .collect()
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
            if let Some(removed) = self.entries.remove(&hash) {
                self.bytes -= removed.raw.len();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use alloy::primitives::{Address, B256};

    use super::*;

    fn hash(n: u8) -> B256 {
        B256::repeat_byte(n)
    }

    fn tx(n: u8, from: Address, nonce: u64, size: usize, at: Instant) -> PooledTx {
        PooledTx {
            hash: hash(n),
            raw: vec![n; size],
            from,
            nonce,
            received_at: at,
        }
    }

    fn pool() -> TxPool {
        TxPool::new(TX_POOL_TTL, 100, 1024 * 1024)
    }

    #[test]
    fn insert_then_get_returns_tx() {
        let t0 = Instant::now();
        let mut p = pool();
        let h = p.insert(tx(1, Address::ZERO, 0, 10, t0), t0);
        assert_eq!(h, hash(1));
        let got = p.get(&h, t0).expect("present");
        assert_eq!(got.raw, vec![1u8; 10]);
        assert_eq!(p.len(), 1);
        assert_eq!(p.bytes(), 10);
    }

    #[test]
    fn duplicate_insert_returns_same_hash_and_keeps_received_at() {
        let t0 = Instant::now();
        let mut p = pool();
        p.insert(tx(1, Address::ZERO, 0, 10, t0), t0);
        let t1 = t0 + Duration::from_secs(30);
        let h = p.insert(tx(1, Address::ZERO, 0, 10, t1), t1);
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
        p.insert(tx(1, a, 5, 10, t0), t0);
        p.insert(tx(2, a, 5, 10, t0), t0);
        assert_eq!(p.len(), 2);
        assert!(p.get(&hash(1), t0).is_some());
        assert!(p.get(&hash(2), t0).is_some());
    }

    #[test]
    fn entry_expires_after_ttl() {
        let t0 = Instant::now();
        let mut p = pool();
        p.insert(tx(1, Address::ZERO, 0, 10, t0), t0);
        assert!(p.get(&hash(1), t0 + Duration::from_secs(59)).is_some());
        assert!(p.get(&hash(1), t0 + Duration::from_secs(61)).is_none());
        assert!(p.list(t0 + Duration::from_secs(61)).is_empty());
        assert_eq!(p.len(), 0);
        assert_eq!(p.bytes(), 0);
    }

    #[test]
    fn entry_cap_evicts_oldest() {
        let t0 = Instant::now();
        let mut p = TxPool::new(TX_POOL_TTL, 3, 1024 * 1024);
        for n in 1..=4 {
            p.insert(tx(n, Address::ZERO, n as u64, 10, t0), t0);
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
            p.insert(tx(n, Address::ZERO, n as u64, 100, t0), t0);
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
        p.insert(tx(1, Address::ZERO, 0, 10, t0), t0);
        p.insert(tx(2, Address::ZERO, 1, 20, t0), t0);
        p.remove(&hash(1));
        assert!(p.get(&hash(1), t0).is_none());
        assert_eq!(p.len(), 1);
        assert_eq!(p.bytes(), 20);
        assert_eq!(p.list(t0).len(), 1);
    }

    #[test]
    fn list_returns_live_entries_oldest_first() {
        let t0 = Instant::now();
        let mut p = pool();
        for n in 1..=3 {
            p.insert(tx(n, Address::ZERO, n as u64, 10, t0), t0);
        }
        let hashes: Vec<B256> = p.list(t0).into_iter().map(|t| t.hash).collect();
        assert_eq!(hashes, vec![hash(1), hash(2), hash(3)]);
    }

    #[test]
    fn max_nonce_from_ignores_expired_and_other_senders() {
        let t0 = Instant::now();
        let mut p = pool();
        let a = Address::repeat_byte(1);
        let b = Address::repeat_byte(2);
        p.insert(tx(1, a, 0, 10, t0), t0);
        p.insert(tx(2, a, 1, 10, t0), t0);
        p.insert(tx(3, a, 1, 10, t0), t0);
        p.insert(tx(4, b, 9, 10, t0), t0);
        assert_eq!(p.max_nonce_from(&a, t0), Some(1));
        assert_eq!(p.max_nonce_from(&a, t0 + Duration::from_secs(61)), None);
    }
}
