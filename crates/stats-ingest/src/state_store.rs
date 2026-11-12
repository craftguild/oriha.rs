use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub trait ShortStateStore: Send + Sync {
    fn reserve_value(
        &self,
        pending_key: String,
        committed_key: &str,
        value: String,
        ttl: Duration,
    ) -> ValueReservation;
    fn commit_value(&self, pending_key: &str, committed_key: String, value: String, ttl: Duration);
    fn remove(&self, key: &str);
    fn increment_with_limit(&self, key: String, ttl: Duration, limit: u64) -> bool;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueReservation {
    Reserved,
    Duplicate,
    Pending,
    Conflict,
}

#[derive(Debug, Default)]
pub struct InMemoryShortStateStore {
    entries: Mutex<HashMap<String, Entry>>,
}

#[derive(Clone, Debug)]
struct Entry {
    expires_at: Instant,
    count: u64,
    value: Option<String>,
}

impl InMemoryShortStateStore {
    pub fn new() -> Self {
        Self::default()
    }

    fn prune_expired(entries: &mut HashMap<String, Entry>, now: Instant) {
        entries.retain(|_, entry| entry.expires_at > now);
    }
}

impl ShortStateStore for InMemoryShortStateStore {
    fn reserve_value(
        &self,
        pending_key: String,
        committed_key: &str,
        value: String,
        ttl: Duration,
    ) -> ValueReservation {
        let now = Instant::now();
        let mut entries = self.entries.lock().expect("short-state lock poisoned");
        Self::prune_expired(&mut entries, now);

        if let Some(entry) = entries.get(committed_key) {
            return if entry.value.as_deref() == Some(value.as_str()) {
                ValueReservation::Duplicate
            } else {
                ValueReservation::Conflict
            };
        }

        if let Some(entry) = entries.get(&pending_key) {
            return if entry.value.as_deref() == Some(value.as_str()) {
                ValueReservation::Pending
            } else {
                ValueReservation::Conflict
            };
        }

        entries.insert(
            pending_key,
            Entry {
                expires_at: now + ttl,
                count: 1,
                value: Some(value),
            },
        );
        ValueReservation::Reserved
    }

    fn commit_value(&self, pending_key: &str, committed_key: String, value: String, ttl: Duration) {
        let now = Instant::now();
        let mut entries = self.entries.lock().expect("short-state lock poisoned");
        Self::prune_expired(&mut entries, now);
        entries.insert(
            committed_key,
            Entry {
                expires_at: now + ttl,
                count: 1,
                value: Some(value),
            },
        );
        entries.remove(pending_key);
    }

    fn remove(&self, key: &str) {
        let mut entries = self.entries.lock().expect("short-state lock poisoned");
        entries.remove(key);
    }

    fn increment_with_limit(&self, key: String, ttl: Duration, limit: u64) -> bool {
        let now = Instant::now();
        let mut entries = self.entries.lock().expect("short-state lock poisoned");
        Self::prune_expired(&mut entries, now);

        let entry = entries.entry(key).or_insert(Entry {
            expires_at: now + ttl,
            count: 0,
            value: None,
        });
        entry.count = entry.count.saturating_add(1);
        entry.count <= limit
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn reserve_value_observes_ttl_and_conflicts() {
        let store = InMemoryShortStateStore::new();
        assert_eq!(
            store.reserve_value(
                "pending".to_string(),
                "committed",
                "digest-a".to_string(),
                Duration::from_millis(5)
            ),
            ValueReservation::Reserved
        );
        assert_eq!(
            store.reserve_value(
                "pending".to_string(),
                "committed",
                "digest-a".to_string(),
                Duration::from_millis(5)
            ),
            ValueReservation::Pending
        );
        assert_eq!(
            store.reserve_value(
                "pending".to_string(),
                "committed",
                "digest-b".to_string(),
                Duration::from_millis(5)
            ),
            ValueReservation::Conflict
        );
        thread::sleep(Duration::from_millis(10));
        assert_eq!(
            store.reserve_value(
                "pending".to_string(),
                "committed",
                "digest-b".to_string(),
                Duration::from_secs(1)
            ),
            ValueReservation::Reserved
        );
    }

    #[test]
    fn commit_value_is_observed_atomically_with_value() {
        let store = InMemoryShortStateStore::new();
        assert_eq!(
            store.reserve_value(
                "pending".to_string(),
                "committed",
                "digest-a".to_string(),
                Duration::from_secs(1)
            ),
            ValueReservation::Reserved
        );
        store.commit_value(
            "pending",
            "committed".to_string(),
            "digest-a".to_string(),
            Duration::from_secs(1),
        );
        assert_eq!(
            store.reserve_value(
                "pending".to_string(),
                "committed",
                "digest-a".to_string(),
                Duration::from_secs(1)
            ),
            ValueReservation::Duplicate
        );
        assert_eq!(
            store.reserve_value(
                "pending".to_string(),
                "committed",
                "digest-b".to_string(),
                Duration::from_secs(1)
            ),
            ValueReservation::Conflict
        );
    }

    #[test]
    fn increment_with_limit_observes_limit() {
        let store = InMemoryShortStateStore::new();
        assert!(store.increment_with_limit("a".to_string(), Duration::from_secs(1), 2));
        assert!(store.increment_with_limit("a".to_string(), Duration::from_secs(1), 2));
        assert!(!store.increment_with_limit("a".to_string(), Duration::from_secs(1), 2));
    }
}
