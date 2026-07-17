use core::borrow::Borrow;
use std::hash::{Hash, BuildHasher};
use crate::{
    LruCache,
    entry::LruEntry,
    keys::KeyWrapper,
};

impl<K: Hash + Eq, V, S: BuildHasher> LruCache<K, V, S> {

    /// Marks the key as the most recently used one. Returns true if the key
    /// was promoted because it exists in the cache, false otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use vlru::LruCache;
    /// let mut cache = LruCache::new(3);
    ///
    /// cache.put(1, "a");
    /// cache.put(2, "b");
    /// cache.put(3, "c");
    /// cache.get(&1);
    /// cache.get(&2);
    ///
    /// // If we do `pop_lru` now, we would pop 3.
    /// // assert_eq!(cache.pop_lru(), Some((3, "c")));
    ///
    /// // By promoting 3, we make sure it isn't popped.
    /// assert!(cache.promote(&3));
    /// assert_eq!(cache.pop_lru(), Some((1, "a")));
    ///
    /// // Promoting an entry that doesn't exist doesn't do anything.
    /// assert!(!cache.promote(&4));
    /// ```
    pub fn promote<Q>(&mut self, k: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        if let Some(node) = self.map.get_mut(KeyWrapper::from_ref(k)) {
            let node_ptr: *mut LruEntry<K, V> = node.as_ptr();
            self.detach(node_ptr);
            self.attach(node_ptr);
            true
        } else {
            false
        }
    }

    /// Marks the key as the least recently used one. Returns true if the key was demoted
    /// because it exists in the cache, false otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use vlru::LruCache;
    /// let mut cache = LruCache::new(3);
    ///
    /// cache.put(1, "a");
    /// cache.put(2, "b");
    /// cache.put(3, "c");
    /// cache.get(&1);
    /// cache.get(&2);
    ///
    /// // If we do `pop_lru` now, we would pop 3.
    /// // assert_eq!(cache.pop_lru(), Some((3, "c")));
    ///
    /// // By demoting 1 and 2, we make sure those are popped first.
    /// assert!(cache.demote(&2));
    /// assert!(cache.demote(&1));
    /// assert_eq!(cache.pop_lru(), Some((1, "a")));
    /// assert_eq!(cache.pop_lru(), Some((2, "b")));
    ///
    /// // Demoting a key that doesn't exist does nothing.
    /// assert!(!cache.demote(&4));
    /// ```
    pub fn demote<Q>(&mut self, k: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        if let Some(node) = self.map.get_mut(KeyWrapper::from_ref(k)) {
            let node_ptr: *mut LruEntry<K, V> = node.as_ptr();
            self.detach(node_ptr);
            self.attach_last(node_ptr);
            true
        } else {
            false
        }
    }
}