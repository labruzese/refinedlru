use core::borrow::Borrow;
use std::hash::{Hash, BuildHasher};
use crate::{
    LruCache,
    entry::LruEntry,
    keys::KeyWrapper,
};

impl<K: Hash + Eq, V, S: BuildHasher> LruCache<K, V, S> {
    /// Returns a reference to the value of the key in the cache or `None` if it is not
    /// present in the cache. Moves the key to the head of the LRU list if it exists.
    ///
    /// # Example
    ///
    /// ```
    /// use vlru::LruCache;
    /// let mut cache = LruCache::new(2);
    ///
    /// cache.put(1, "a");
    /// cache.put(2, "b");
    /// cache.put(2, "c");
    /// cache.put(3, "d");
    ///
    /// assert_eq!(cache.get(&1), None);
    /// assert_eq!(cache.get(&2), Some(&"c"));
    /// assert_eq!(cache.get(&3), Some(&"d"));
    /// ```
    #[rr::params("l", "cap", "y")]
    #[rr::args("(#(l, cap), y)", "k")]
    #[rr::returns("(λ v : {rt_of V}, #v) <$> al_lookup l k")]
    #[rr::observe("y": "(al_move_to_front l k, cap)")]
    pub fn get<'a, Q>(&'a mut self, k: &Q) -> Option<&'a V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        if let Some(node) = self.map.get_mut(KeyWrapper::from_ref(k)) {
            let node_ptr: *mut LruEntry<K, V> = node.as_ptr();

            self.detach(node_ptr);
            self.attach(node_ptr);

            Some(unsafe { (*node_ptr).val.as_ref().unwrap() })
        } else {
            None
        }
    }

    /// Returns a mutable reference to the value of the key in the cache or `None` if it
    /// is not present in the cache. Moves the key to the head of the LRU list if it exists.
    ///
    /// # Example
    ///
    /// ```
    /// use vlru::LruCache;
    /// let mut cache = LruCache::new(2);
    ///
    /// cache.put("apple", 8);
    /// cache.put("banana", 4);
    /// cache.put("banana", 6);
    /// cache.put("pear", 2);
    ///
    /// assert_eq!(cache.get_mut(&"apple"), None);
    /// assert_eq!(cache.get_mut(&"banana"), Some(&mut 6));
    /// assert_eq!(cache.get_mut(&"pear"), Some(&mut 2));
    /// ```
    pub fn get_mut<'a, Q>(&'a mut self, k: &Q) -> Option<&'a mut V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        if let Some(node) = self.map.get_mut(KeyWrapper::from_ref(k)) {
            let node_ptr: *mut LruEntry<K, V> = node.as_ptr();

            self.detach(node_ptr);
            self.attach(node_ptr);

            Some(unsafe { (*node_ptr).val.as_mut().unwrap() })
        } else {
            None
        }
    }

    /// Returns a key-value references pair of the key in the cache or `None` if it is not
    /// present in the cache. Moves the key to the head of the LRU list if it exists.
    ///
    /// # Example
    ///
    /// ```
    /// use vlru::LruCache;
    /// let mut cache = LruCache::new(2);
    ///
    /// cache.put(String::from("1"), "a");
    /// cache.put(String::from("2"), "b");
    /// cache.put(String::from("2"), "c");
    /// cache.put(String::from("3"), "d");
    ///
    /// assert_eq!(cache.get_key_value("1"), None);
    /// assert_eq!(cache.get_key_value("2"), Some((&String::from("2"), &"c")));
    /// assert_eq!(cache.get_key_value("3"), Some((&String::from("3"), &"d")));
    /// ```
    pub fn get_key_value<'a, Q>(&'a mut self, k: &Q) -> Option<(&'a K, &'a V)>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        if let Some(node) = self.map.get_mut(KeyWrapper::from_ref(k)) {
            let node_ptr: *mut LruEntry<K, V> = node.as_ptr();

            self.detach(node_ptr);
            self.attach(node_ptr);

            Some(unsafe { ((*node_ptr).key.as_ref().unwrap(), (*node_ptr).val.as_ref().unwrap()) })
        } else {
            None
        }
    }

    /// Returns a key-value references pair of the key in the cache or `None` if it is not
    /// present in the cache. The reference to the value of the key is mutable. Moves the key to
    /// the head of the LRU list if it exists.
    ///
    /// # Example
    ///
    /// ```
    /// use vlru::LruCache;
    /// let mut cache = LruCache::new(2);
    ///
    /// cache.put(1, "a");
    /// cache.put(2, "b");
    /// let (k, v) = cache.get_key_value_mut(&1).unwrap();
    /// assert_eq!(k, &1);
    /// assert_eq!(v, &mut "a");
    /// *v = "aa";
    /// cache.put(3, "c");
    /// assert_eq!(cache.get_key_value(&2), None);
    /// assert_eq!(cache.get_key_value(&1), Some((&1, &"aa")));
    /// assert_eq!(cache.get_key_value(&3), Some((&3, &"c")));
    /// ```
    pub fn get_key_value_mut<'a, Q>(&'a mut self, k: &Q) -> Option<(&'a K, &'a mut V)>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        if let Some(node) = self.map.get_mut(KeyWrapper::from_ref(k)) {
            let node_ptr: *mut LruEntry<K, V> = node.as_ptr();

            self.detach(node_ptr);
            self.attach(node_ptr);

            Some(unsafe {
                (
                    (*node_ptr).key.as_ref().unwrap(),
                    (*node_ptr).val.as_mut().unwrap(),
                )
            })
        } else {
            None
        }
    }
}