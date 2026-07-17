use std::{
    mem,
    hash::{Hash, BuildHasher}
};
use crate::{
    LruCache,
    entry::LruEntry,
    keys::KeyRef,
};

impl<K: Hash + Eq, V, S: BuildHasher> LruCache<K, V, S> {
    /// Puts a key-value pair into cache. If the key already exists in the cache, then it updates
    /// the key's value and returns the old value. Otherwise, `None` is returned.
    ///
    /// # Example
    ///
    /// ```
    /// use vlru::LruCache;
    /// let mut cache = LruCache::new(2);
    ///
    /// assert_eq!(None, cache.put(1, "a"));
    /// assert_eq!(None, cache.put(2, "b"));
    /// assert_eq!(Some("b"), cache.put(2, "beta"));
    ///
    /// assert_eq!(cache.get(&1), Some(&"a"));
    /// assert_eq!(cache.get(&2), Some(&"beta"));
    /// ```
    #[rr::params("l", "cap", "y")]
    #[rr::args("(#(l, cap), y)", "k", "v")]
    #[rr::requires("cap > 0")]
    #[rr::returns("al_lookup l k")]
    #[rr::observe("y": "(al_put cap l k v, cap)")]
    pub fn put(&mut self, k: K, v: V) -> Option<V> {
        self.capturing_put(k, v, false).map(|(_, v)| v)
    }

    /// Pushes a key-value pair into the cache. If an entry with key `k` already exists in
    /// the cache or another cache entry is removed (due to the lru's capacity),
    /// then it returns the old entry's key-value pair. Otherwise, returns `None`.
    ///
    /// # Example
    ///
    /// ```
    /// use vlru::LruCache;
    /// let mut cache = LruCache::new(2);
    ///
    /// assert_eq!(None, cache.push(1, "a"));
    /// assert_eq!(None, cache.push(2, "b"));
    ///
    /// // This push call returns (2, "b") because that was previously 2's entry in the cache.
    /// assert_eq!(Some((2, "b")), cache.push(2, "beta"));
    ///
    /// // This push call returns (1, "a") because the cache is at capacity and 1's entry was the lru entry.
    /// assert_eq!(Some((1, "a")), cache.push(3, "alpha"));
    ///
    /// assert_eq!(cache.get(&1), None);
    /// assert_eq!(cache.get(&2), Some(&"beta"));
    /// assert_eq!(cache.get(&3), Some(&"alpha"));
    /// ```
    pub fn push(&mut self, k: K, v: V) -> Option<(K, V)> {
        self.capturing_put(k, v, true)
    }

    // Used internally by `put` and `push` to add a new entry to the lru.
    // Takes ownership of and returns entries replaced due to the cache's capacity
    // when `capture` is true.
    fn capturing_put(&mut self, k: K, mut v: V, capture: bool) -> Option<(K, V)> {
        let node_ref = self.map.get_mut(&KeyRef { k: &k });


        match node_ref {
            Some(node_ref) => {
                // if the key is already in the cache just update its value and move it to the
                // front of the list
                let node_ptr: *mut LruEntry<K, V> = node_ref.as_ptr();

                // gets a reference to the node to perform a swap and drops it right after
                let node_ref = unsafe { 
                    (*node_ptr).val.as_mut().unwrap()
                };

                mem::swap(&mut v, node_ref);
                let _ = node_ref;

                self.detach(node_ptr);
                self.attach(node_ptr);
                Some((k, v))
            }
            None => {
                let (replaced, node) = self.replace_or_create_node(k, v);
                let node_ptr: *mut LruEntry<K, V> = node.as_ptr();

                self.attach(node_ptr);

                let keyref = unsafe { (*node_ptr).key.as_ref().unwrap() };
                self.map.insert(KeyRef { k: keyref }, node);

                replaced.filter(|_| capture)
            }
        }
    }

}