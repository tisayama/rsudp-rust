use chrono::{DateTime, Utc, TimeZone};
use std::collections::{HashSet, VecDeque};

/// Generate a deterministic deduplication key from station and timestamp.
/// Floors timestamp to 500ms boundary and formats as `{station}:{iso8601}`.
pub fn generate_dedup_key(station: &str, timestamp_ms: i64) -> String {
    let floored_ms = (timestamp_ms / 500) * 500;
    let dt: DateTime<Utc> = Utc.timestamp_millis_opt(floored_ms).unwrap();
    format!("{}:{}", station, dt.format("%Y-%m-%dT%H:%M:%S%.3fZ"))
}

/// LRU-style deduplication checker using HashSet + VecDeque.
pub struct DedupChecker {
    seen: HashSet<String>,
    order: VecDeque<String>,
    max_entries: usize,
}

impl DedupChecker {
    pub fn new(max_entries: usize) -> Self {
        Self {
            seen: HashSet::with_capacity(max_entries),
            order: VecDeque::with_capacity(max_entries),
            max_entries,
        }
    }

    /// Returns `true` if the key is new (not seen before).
    /// Returns `false` if it's a duplicate.
    pub fn check_and_insert(&mut self, key: &str) -> bool {
        if self.seen.contains(key) {
            return false;
        }
        if self.seen.len() >= self.max_entries {
            if let Some(oldest) = self.order.pop_front() {
                self.seen.remove(&oldest);
            }
        }
        self.seen.insert(key.to_string());
        self.order.push_back(key.to_string());
        true
    }

    pub fn len(&self) -> usize {
        self.seen.len()
    }

    pub fn is_empty(&self) -> bool {
        self.seen.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_dedup_key_floors_to_500ms() {
        // 2025-11-25T09:01:23.730Z -> floored to 2025-11-25T09:01:23.500Z
        let key = generate_dedup_key("AM.R6E01", 1732525283730);
        assert_eq!(key, "AM.R6E01:2024-11-25T09:01:23.500Z");
    }

    #[test]
    fn test_same_window_same_key() {
        let key1 = generate_dedup_key("AM.R6E01", 1732525283500);
        let key2 = generate_dedup_key("AM.R6E01", 1732525283730);
        let key3 = generate_dedup_key("AM.R6E01", 1732525283999);
        assert_eq!(key1, key2);
        assert_eq!(key2, key3);
    }

    #[test]
    fn test_adjacent_windows_different_keys() {
        let key1 = generate_dedup_key("AM.R6E01", 1732525283499);
        let key2 = generate_dedup_key("AM.R6E01", 1732525283500);
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_dedup_checker_new_key_returns_true() {
        let mut checker = DedupChecker::new(100);
        assert!(checker.check_and_insert("key1"));
        assert_eq!(checker.len(), 1);
    }

    #[test]
    fn test_dedup_checker_duplicate_returns_false() {
        let mut checker = DedupChecker::new(100);
        assert!(checker.check_and_insert("key1"));
        assert!(!checker.check_and_insert("key1"));
        assert_eq!(checker.len(), 1);
    }

    #[test]
    fn test_dedup_checker_lru_eviction() {
        let mut checker = DedupChecker::new(3);
        assert!(checker.check_and_insert("key1"));
        assert!(checker.check_and_insert("key2"));
        assert!(checker.check_and_insert("key3"));
        assert_eq!(checker.len(), 3);

        // Adding key4 should evict key1
        assert!(checker.check_and_insert("key4"));
        assert_eq!(checker.len(), 3);

        // key1 was evicted, so it should be treated as new
        assert!(checker.check_and_insert("key1"));
        // key2 was evicted to make room
        assert!(checker.check_and_insert("key2"));
    }
}
