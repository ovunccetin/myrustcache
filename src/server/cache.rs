use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::utils::current_monotime;

/// A basic cache interface allowing put, get, and remove operations.
pub trait Cache: Send + Sync {
    /// Puts a key-value pair into the cache with an optional time-to-live (TTL) in seconds.
    ///
    /// # Arguments
    /// * `key` - The key to associate with the value.
    /// * `value` - The string value to store in the cache.
    /// * `ttl` - An optional time-to-live (TTL) in seconds for the key-value pair. If `None`, the key-value
    ///           pair lives indefinitely. Otherwise, it is invalidated after the TTL has elapsed.
    ///
    fn put(&self, key: String, value: String, ttl: Option<u64>) -> ();

    /// Gets the value associated with the given key from the cache.
    ///
    /// # Arguments
    /// * `key` - The key to look up in the cache.
    ///
    /// # Returns
    /// * An `Option` containing the value associated with the key if it exists and has not expired.
    fn get(&self, key: &str) -> Option<Arc<String>>;

    /// Removes the key-value pair associated with the given key from the cache.
    ///
    /// # Arguments
    /// * `key` - The key to remove from the cache.
    ///
    /// # Returns
    /// * An `Option` containing the value associated with the key if it existed and was removed.
    fn remove(&self, key: &str) -> Option<Arc<String>>;
}

/// Cached value with an optional time of expiration (i.e. when the value is no longer valid).
struct CacheEntry {
    value: Arc<String>,      // Use an `Arc` to allow multiple immutable references to the value across threads.
    expires_at: Option<u64>, // The time at which the value expires (in milliseconds since the Unix epoch).
}

impl CacheEntry {
    fn new(value: String, ttl: Option<u64>) -> CacheEntry {
        CacheEntry {
            value: Arc::new(value),
            expires_at: ttl.map(|t| current_monotime() + t * 1000),
        }
    }

    fn is_expired(&self) -> bool {
        self.expires_at.map_or(false, |t| current_monotime() >= t)
    }
}

struct SimpleCache {
    /// A HashMap to store key-value pairs in memory. By using a `RwLock`, we can allow multiple readers
    /// concurrently. Write operations are exclusive. So, only one writer can modify the cache at a time.
    /// For a more fine-grained locking mechanism, we can consider external crates like `dashmap` or `flurry`.
    cache: RwLock<HashMap<String, CacheEntry>>,
}

impl Cache for SimpleCache {
    fn put(&self, key: String, value: String, ttl: Option<u64>) -> () {
        self.cache
            .write()
            .unwrap()
            .insert(key, CacheEntry::new(value, ttl));
    }

    fn get(&self, key: &str) -> Option<Arc<String>> {
        self.cache
            .read()
            .unwrap()
            .get(key)
            .filter(|entry| !entry.is_expired())
            .map(|entry| Arc::clone(&entry.value))
    }

    fn remove(&self, key: &str) -> Option<Arc<String>> {
        self.cache
            .write()
            .unwrap()
            .remove(key)
            .map(|entry| entry.value)
    }
}

pub struct CacheFactory;

impl CacheFactory {
    pub fn new_cache() -> Arc<dyn Cache> {
        Arc::new(SimpleCache {
            cache: RwLock::new(HashMap::new()),
        })
    }
}
