use core::borrow::Borrow;
use std::hash::{Hash, BuildHasher};
use crate::{
    LruCache,
    entry::LruEntry,
    keys::KeyWrapper,
};

impl<K: Hash + Eq, V, S: BuildHasher> LruCache<K, V, S> {
    /// Whether this LRU cache is unbounded.
    pub(crate) fn is_unbounded(&self) -> bool {
        self.cap() == usize::MAX
    }

    /// Returns a reference to the value corresponding to the key in the cache or `None` if it is
    /// not present in the cache. Unlike `get`, `peek` does not update the LRU list so the key's
    /// position will be unchanged.
    ///
    /// # Example
    ///
    /// ```
    /// use vlru::LruCache;
    /// let mut cache = LruCache::new(2);
    ///
    /// cache.put(1, "a");
    /// cache.put(2, "b");
    ///
    /// assert_eq!(cache.peek(&1), Some(&"a"));
    /// assert_eq!(cache.peek(&2), Some(&"b"));
    /// ```
    pub fn peek<'a, Q>(&'a self, k: &Q) -> Option<&'a V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.map
            .get(KeyWrapper::from_ref(k))
            .map(|node| unsafe { node.as_ref().val.as_ref().unwrap() })
    }
    
    /// Returns a mutable reference to the value corresponding to the key in the cache or `None`
    /// if it is not present in the cache. Unlike `get_mut`, `peek_mut` does not update the LRU
    /// list so the key's position will be unchanged.
    ///
    /// # Example
    ///
    /// ```
    /// use vlru::LruCache;
    /// let mut cache = LruCache::new(2);
    ///
    /// cache.put(1, "a");
    /// cache.put(2, "b");
    ///
    /// assert_eq!(cache.peek_mut(&1), Some(&mut "a"));
    /// assert_eq!(cache.peek_mut(&2), Some(&mut "b"));
    /// ```
    pub fn peek_mut<'a, Q>(&'a mut self, k: &Q) -> Option<&'a mut V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        match self.map.get_mut(KeyWrapper::from_ref(k)) {
            None => None,
            Some(node) => Some(unsafe { (*node.as_ptr()).val.as_mut().unwrap() }),
        }
    }

    /// Returns the value corresponding to the least recently used item or `None` if the
    /// cache is empty. Like `peek`, `peek_lru` does not update the LRU list so the item's
    /// position will be unchanged.
    ///
    /// # Example
    ///
    /// ```
    /// use vlru::LruCache;
    /// let mut cache = LruCache::new(2);
    ///
    /// cache.put(1, "a");
    /// cache.put(2, "b");
    ///
    /// assert_eq!(cache.peek_lru(), Some((&1, &"a")));
    /// ```
    pub fn peek_lru(&self) -> Option<(&K, &V)> {
        if self.is_empty() {
            return None;
        }

        let (key, val);
        unsafe {
            let node = (*self.tail).prev;
            key = (*node).key.as_ref().unwrap();
            val = (*node).val.as_ref().unwrap();
        }

        Some((key, val))
    }

    /// Returns the value corresponding to the most recently used item or `None` if the
    /// cache is empty. Like `peek`, `peek_mru` does not update the LRU list so the item's
    /// position will be unchanged.
    ///
    /// # Example
    ///
    /// ```
    /// use vlru::LruCache;
    /// let mut cache = LruCache::new(2);
    ///
    /// cache.put(1, "a");
    /// cache.put(2, "b");
    ///
    /// assert_eq!(cache.peek_mru(), Some((&2, &"b")));
    /// ```
    pub fn peek_mru(&self) -> Option<(&K, &V)> {
        if self.is_empty() {
            return None;
        }

        let (key, val);
        unsafe {
            let node: *mut LruEntry<K, V> = (*self.head).next;
            key = (*node).key.as_ref().unwrap();
            val = (*node).val.as_ref().unwrap();
        }

        Some((key, val))
    }

    /// Returns a bool indicating whether the given key is in the cache. Does not update the
    /// LRU list.
    ///
    /// # Example
    ///
    /// ```
    /// use vlru::LruCache;
    /// let mut cache = LruCache::new(2);
    ///
    /// cache.put(1, "a");
    /// cache.put(2, "b");
    /// cache.put(3, "c");
    ///
    /// assert!(!cache.contains(&1));
    /// assert!(cache.contains(&2));
    /// assert!(cache.contains(&3));
    /// ```
    pub fn contains<Q>(&self, k: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.map.contains_key(KeyWrapper::from_ref(k))
    }

    /// Returns the number of key-value pairs that are currently in the the cache.
    ///
    /// # Example
    ///
    /// ```
    /// use vlru::LruCache;
    /// let mut cache = LruCache::new(2);
    /// assert_eq!(cache.len(), 0);
    ///
    /// cache.put(1, "a");
    /// assert_eq!(cache.len(), 1);
    ///
    /// cache.put(2, "b");
    /// assert_eq!(cache.len(), 2);
    ///
    /// cache.put(3, "c");
    /// assert_eq!(cache.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns a bool indicating whether the cache is empty or not.
    ///
    /// # Example
    ///
    /// ```
    /// use vlru::LruCache;
    /// let mut cache = LruCache::new(2);
    /// assert!(cache.is_empty());
    ///
    /// cache.put(1, "a");
    /// assert!(!cache.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.map.len() == 0
    }

    /// Returns the maximum number of key-value pairs the cache can hold.
    ///
    /// # Example
    ///
    /// ```
    /// use vlru::LruCache;
    /// let mut cache: LruCache<isize, &str> = LruCache::new(2);
    /// assert_eq!(cache.cap(), 2);
    /// ```
    pub fn cap(&self) -> usize{
        self.cap
    }
}