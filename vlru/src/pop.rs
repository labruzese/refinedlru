use core::borrow::Borrow;
use std::hash::{Hash, BuildHasher};
use alloc::boxed::Box;
use crate::{
    LruCache,
    entry::LruEntry,
    keys::KeyWrapper,
};

impl<K: Hash + Eq, V, S: BuildHasher> LruCache<K, V, S> {
    /// Removes and returns the value corresponding to the key from the cache or
    /// `None` if it does not exist.
    ///
    /// # Example
    ///
    /// ```
    /// use vlru::LruCache;
    /// let mut cache = LruCache::new(2);
    ///
    /// cache.put(2, "a");
    ///
    /// assert_eq!(cache.pop(&1), None);
    /// assert_eq!(cache.pop(&2), Some("a"));
    /// assert_eq!(cache.pop(&2), None);
    /// assert_eq!(cache.len(), 0);
    /// ```
    pub fn pop<Q>(&mut self, k: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        match self.map.remove(KeyWrapper::from_ref(k)) {
            None => None,
            Some(old_node) => {
                let mut old_node = unsafe { *Box::from_raw(old_node.as_ptr()) };

                self.detach(&mut old_node);

                let LruEntry { val, .. } = old_node;
                val
            }
        }
    }

    /// Removes and returns the key and the value corresponding to the key from the cache or
    /// `None` if it does not exist.
    ///
    /// # Example
    ///
    /// ```
    /// use vlru::LruCache;
    /// let mut cache = LruCache::new(2);
    ///
    /// cache.put(1, "a");
    /// cache.put(2, "a");
    ///
    /// assert_eq!(cache.pop(&1), Some("a"));
    /// assert_eq!(cache.pop_entry(&2), Some((2, "a")));
    /// assert_eq!(cache.pop(&1), None);
    /// assert_eq!(cache.pop_entry(&2), None);
    /// assert_eq!(cache.len(), 0);
    /// ```
    pub fn pop_entry<Q>(&mut self, k: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        match self.map.remove(KeyWrapper::from_ref(k)) {
            None => None,
            Some(old_node) => {
                let mut old_node = unsafe { *Box::from_raw(old_node.as_ptr()) };

                self.detach(&mut old_node);

                let LruEntry { key, val, .. } = old_node;
                key.zip(val)
            }
        }
    }

    /// Removes and returns the key and value corresponding to the least recently
    /// used item or `None` if the cache is empty.
    ///
    /// # Example
    ///
    /// ```
    /// use vlru::LruCache;
    /// let mut cache = LruCache::new(2);
    ///
    /// cache.put(2, "a");
    /// cache.put(3, "b");
    /// cache.put(4, "c");
    /// cache.get(&3);
    ///
    /// assert_eq!(cache.pop_lru(), Some((4, "c")));
    /// assert_eq!(cache.pop_lru(), Some((3, "b")));
    /// assert_eq!(cache.pop_lru(), None);
    /// assert_eq!(cache.len(), 0);
    /// ```
    pub fn pop_lru(&mut self) -> Option<(K, V)> {
        let node = self.remove_last()?;
        // N.B.: Can't destructure directly because of https://github.com/rust-lang/rust/issues/28536
        let node = *node;
        let LruEntry { key, val, .. } = node;
        Some((key.unwrap(), val.unwrap()))
    }

    /// Removes and returns the key and value corresponding to the most recently
    /// used item or `None` if the cache is empty.
    ///
    /// # Example
    ///
    /// ```
    /// use vlru::LruCache;
    /// let mut cache = LruCache::new(2);
    ///
    /// cache.put(2, "a");
    /// cache.put(3, "b");
    /// cache.put(4, "c");
    /// cache.get(&3);
    ///
    /// assert_eq!(cache.pop_mru(), Some((3, "b")));
    /// assert_eq!(cache.pop_mru(), Some((4, "c")));
    /// assert_eq!(cache.pop_mru(), None);
    /// assert_eq!(cache.len(), 0);
    /// ```
    pub fn pop_mru(&mut self) -> Option<(K, V)> {
        let node = self.remove_first()?;
        // N.B.: Can't destructure directly because of https://github.com/rust-lang/rust/issues/28536
        let node = *node;
        let LruEntry { key, val, .. } = node;
        Some((key.unwrap(), val.unwrap()))
    }
}