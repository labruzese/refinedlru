use std::hash::{Hash, BuildHasher};
use crate::LruCache;

impl<K: Hash + Eq, V, S: BuildHasher> LruCache<K, V, S> {
    /// Resizes the cache. If the new capacity is smaller than the size of the current
    /// cache any entries past the new capacity are discarded.
    ///
    /// # Example
    ///
    /// ```
    /// use vlru::LruCache;
    /// let mut cache: LruCache<isize, &str> = LruCache::new(2);
    ///
    /// cache.put(1, "a");
    /// cache.put(2, "b");
    /// cache.resize(4);
    /// cache.put(3, "c");
    /// cache.put(4, "d");
    ///
    /// assert_eq!(cache.len(), 4);
    /// assert_eq!(cache.get(&1), Some(&"a"));
    /// assert_eq!(cache.get(&2), Some(&"b"));
    /// assert_eq!(cache.get(&3), Some(&"c"));
    /// assert_eq!(cache.get(&4), Some(&"d"));
    /// ```
    pub fn resize(&mut self, cap: usize) {
        // return early if capacity doesn't change
        if cap == self.cap {
            return;
        }

        while self.map.len() > cap {
            self.pop_lru();
        }
        self.map.shrink_to_fit();

        self.cap = cap;
    }

    /// Clears the contents of the cache.
    ///
    /// # Example
    ///
    /// ```
    /// use vlru::LruCache;
    /// let mut cache: LruCache<isize, &str> = LruCache::new(2);
    /// assert_eq!(cache.len(), 0);
    ///
    /// cache.put(1, "a");
    /// assert_eq!(cache.len(), 1);
    ///
    /// cache.put(2, "b");
    /// assert_eq!(cache.len(), 2);
    ///
    /// cache.clear();
    /// assert_eq!(cache.len(), 0);
    /// ```
    pub fn clear(&mut self) {
        while self.pop_lru().is_some() {}
    }
}