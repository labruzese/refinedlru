use core::{
    hash::BuildHasher, 
    ptr::NonNull
};
use alloc::boxed::Box;
use std::{
    collections::HashMap, 
    hash::Hash
};
use crate::{
    LruCache,
    entry::LruEntry,
    keys::KeyRef,
};


impl<K: Hash + Eq, V> LruCache<K, V> {
    /// Creates a new LRU Cache that holds at most `cap` items.
    ///
    /// # Example
    ///
    /// ```
    /// use vlru::LruCache;
    /// let mut cache: LruCache<isize, &str> = LruCache::new(10);
    /// ```
    pub fn new(cap: usize) -> LruCache<K, V> {
        LruCache::construct(cap, HashMap::with_capacity(cap))
    }

    /// Creates a new LRU Cache that never automatically evicts items.
    ///
    /// # Example
    ///
    /// ```
    /// use vlru::LruCache;
    /// let mut cache: LruCache<isize, &str> = LruCache::unbounded();
    /// ```
    pub fn unbounded() -> LruCache<K, V> {
        LruCache::construct(usize::MAX, HashMap::default())
    }
}

impl<K: Hash + Eq, V, S: BuildHasher> LruCache<K, V, S> {
    /// Creates a new LRU Cache that holds at most `cap` items and
    /// uses the provided hash builder to hash keys.
    ///
    /// # Example
    ///
    /// ```
    /// use vlru::{LruCache, DefaultHasher};
    ///
    /// let s = DefaultHasher::default();
    /// let mut cache: LruCache<isize, &str> = LruCache::with_hasher(10, s);
    /// ```
    pub fn with_hasher(cap: usize, hash_builder: S) -> LruCache<K, V, S> {
        LruCache::construct(
            cap,
            HashMap::with_capacity_and_hasher(cap.into(), hash_builder),
        )
    }

    /// Creates a new LRU Cache that never automatically evicts items and
    /// uses the provided hash builder to hash keys.
    ///
    /// # Example
    ///
    /// ```
    /// use vlru::{LruCache, DefaultHasher};
    ///
    /// let s = DefaultHasher::default();
    /// let mut cache: LruCache<isize, &str> = LruCache::unbounded_with_hasher(s);
    /// ```
    pub fn unbounded_with_hasher(hash_builder: S) -> LruCache<K, V, S> {
        LruCache::construct(usize::MAX, HashMap::with_hasher(hash_builder))
    }

    /// Creates a new LRU Cache with the given capacity.
    pub(crate) fn construct(
        cap: usize,
        map: HashMap<KeyRef<K>, NonNull<LruEntry<K, V>>, S>,
    ) -> LruCache<K, V, S> {
        // NB: The compiler warns that cache does not need to be marked as mutable if we
        // declare it as such since we only mutate it inside the unsafe block.
        let cache = LruCache {
            map,
            cap,
            head: Box::into_raw(Box::new(LruEntry::new_sigil())),
            tail: Box::into_raw(Box::new(LruEntry::new_sigil())),
        };

        unsafe {
            (*cache.head).next = cache.tail;
            (*cache.tail).prev = cache.head;
        }

        cache
    }
}