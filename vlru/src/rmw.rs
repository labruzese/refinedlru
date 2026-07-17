use core::borrow::Borrow;
use std::hash::{Hash, BuildHasher};
use crate::{
    LruCache,
    entry::LruEntry,
    keys::{
        KeyWrapper,
        KeyRef,
    }
};

impl<K: Hash + Eq, V, S: BuildHasher> LruCache<K, V, S> {
    /// Returns a reference to the value of the key in the cache if it is
    /// present in the cache and moves the key to the head of the LRU list.
    /// If the key does not exist the provided `FnOnce` is used to populate
    /// the list and a reference is returned.
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
    /// assert_eq!(cache.get_or_insert(2, ||"a"), &"c");
    /// assert_eq!(cache.get_or_insert(3, ||"a"), &"d");
    /// assert_eq!(cache.get_or_insert(1, ||"a"), &"a");
    /// assert_eq!(cache.get_or_insert(1, ||"b"), &"a");
    /// ```
    pub fn get_or_insert<F>(&mut self, k: K, f: F) -> &V
    where
        F: FnOnce() -> V,
    {
        self.get_or_insert_with_key(k, |_| f())
    }

    /// Returns a reference to the value of the key in the cache if it is
    /// present in the cache and moves the key to the head of the LRU list.
    /// If the key does not exist the provided `FnOnce` is used by passing
    /// a reference to the key to populate the list and a reference is returned.
    ///
    /// # Example
    ///
    /// ```
    /// use vlru::LruCache;
    /// let mut cache = LruCache::new(2);
    ///
    /// cache.put("One", 1);
    /// cache.put("Two", 2);
    /// cache.put("Two", 3);
    /// cache.put("Three", 4);
    ///
    /// assert_eq!(cache.get_or_insert_with_key("Two", |_|1), &3);
    /// assert_eq!(cache.get_or_insert_with_key("Three", |k|k.len()), &4);
    /// assert_eq!(cache.get_or_insert_with_key("One", |_|1), &1);
    /// assert_eq!(cache.get_or_insert_with_key("One", |k|k.len()), &1);
    /// ```
    pub fn get_or_insert_with_key<F>(&mut self, k: K, f: F) -> &V
    where
        F: FnOnce(&K) -> V,
    {
        if let Some(node) = self.map.get_mut(&KeyRef { k: &k }) {
            let node_ptr: *mut LruEntry<K, V> = node.as_ptr();

            self.detach(node_ptr);
            self.attach(node_ptr);

            unsafe { (*node_ptr).val.as_ref().unwrap() }
        } else {
            let v = f(&k);
            let (_, node) = self.replace_or_create_node(k, v);
            let node_ptr: *mut LruEntry<K, V> = node.as_ptr();

            self.attach(node_ptr);

            let keyref = unsafe { (*node_ptr).key.as_ref().unwrap() };
            self.map.insert(KeyRef { k: keyref }, node);
            unsafe { (*node_ptr).val.as_ref().unwrap() }
        }
    }

    /// Returns a reference to the value of the key in the cache if it is
    /// present in the cache and moves the key to the head of the LRU list.
    /// If the key does not exist the provided `FnOnce` is used to populate
    /// the list and a reference is returned. The value referenced by the
    /// key is only cloned (using `to_owned()`) if it doesn't exist in the
    /// cache.
    ///
    /// # Example
    ///
    /// ```
    /// use vlru::LruCache;
    /// use std::rc::Rc;
    ///
    /// let key1 = Rc::new("1".to_owned());
    /// let key2 = Rc::new("2".to_owned());
    /// let mut cache = LruCache::<Rc<String>, String>::new(2);
    /// assert_eq!(cache.get_or_insert_ref(&key1, ||"One".to_owned()), "One");
    /// assert_eq!(cache.get_or_insert_ref(&key2, ||"Two".to_owned()), "Two");
    /// assert_eq!(cache.get_or_insert_ref(&key2, ||"Not two".to_owned()), "Two");
    /// assert_eq!(cache.get_or_insert_ref(&key2, ||"Again not two".to_owned()), "Two");
    /// assert_eq!(Rc::strong_count(&key1), 2);
    /// assert_eq!(Rc::strong_count(&key2), 2); // key2 was only cloned once even though we
    ///                                         // queried it 3 times
    /// ```
    pub fn get_or_insert_ref<'a, Q, F>(&'a mut self, k: &'_ Q, f: F) -> &'a V
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized + alloc::borrow::ToOwned<Owned = K>,
        F: FnOnce() -> V,
    {
        if let Some(node) = self.map.get_mut(KeyWrapper::from_ref(k)) {
            let node_ptr: *mut LruEntry<K, V> = node.as_ptr();

            self.detach(node_ptr);
            self.attach(node_ptr);

            unsafe { (*node_ptr).val.as_ref().unwrap() }
        } else {
            let v = f();
            let (_, node) = self.replace_or_create_node(k.to_owned(), v);
            let node_ptr: *mut LruEntry<K, V> = node.as_ptr();

            self.attach(node_ptr);

            let keyref = unsafe { (*node_ptr).key.as_ref().unwrap() };
            self.map.insert(KeyRef { k: keyref }, node);
            unsafe { (*node_ptr).val.as_ref().unwrap() }
        }
    }

    /// Returns a reference to the value of the key in the cache if it is
    /// present in the cache and moves the key to the head of the LRU list.
    /// If the key does not exist the provided `FnOnce` is used to populate
    /// the list and a reference is returned. If `FnOnce` returns `Err`,
    /// returns the `Err`.
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
    /// let f = ||->Result<&str, String> {Err("failed".to_owned())};
    /// let a = ||->Result<&str, String> {Ok("a")};
    /// let b = ||->Result<&str, String> {Ok("b")};
    /// assert_eq!(cache.try_get_or_insert(2, a), Ok(&"c"));
    /// assert_eq!(cache.try_get_or_insert(3, a), Ok(&"d"));
    /// assert_eq!(cache.try_get_or_insert(4, f), Err("failed".to_owned()));
    /// assert_eq!(cache.try_get_or_insert(5, b), Ok(&"b"));
    /// assert_eq!(cache.try_get_or_insert(5, a), Ok(&"b"));
    /// ```
    pub fn try_get_or_insert<F, E>(&mut self, k: K, f: F) -> Result<&V, E>
    where
        F: FnOnce() -> Result<V, E>,
    {
        self.try_get_or_insert_with_key(k, |_| f())
    }

    /// Returns a reference to the value of the key in the cache if it is
    /// present in the cache and moves the key to the head of the LRU list.
    /// If the key does not exist the provided `FnOnce` is used by passing
    /// a reference to the key to populate the list and a reference is returned.
    /// If `FnOnce` returns `Err`, returns the `Err`.
    ///
    /// # Example
    ///
    /// ```
    /// use vlru::LruCache;
    /// let mut cache = LruCache::new(2);
    ///
    /// cache.put("One", 1);
    /// cache.put("Two", 2);
    /// cache.put("Two", 3);
    /// cache.put("Three", 4);
    ///
    /// let f = |_: &&str|->Result<usize, String> {Err("failed".to_owned())};
    /// let len = |k: &&str|->Result<usize, String> {Ok(k.len())};
    /// let zero = |_: &&str|->Result<usize, String> {Ok(0)};
    /// assert_eq!(cache.try_get_or_insert_with_key("Two", len), Ok(&3));
    /// assert_eq!(cache.try_get_or_insert_with_key("Three", len), Ok(&4));
    /// assert_eq!(cache.try_get_or_insert_with_key("Four", f), Err("failed".to_owned()));
    /// assert_eq!(cache.try_get_or_insert_with_key("Five", len), Ok(&4));
    /// assert_eq!(cache.try_get_or_insert_with_key("Five", zero), Ok(&4));
    /// ```
    pub fn try_get_or_insert_with_key<F, E>(&mut self, k: K, f: F) -> Result<&V, E>
    where
        F: FnOnce(&K) -> Result<V, E>,
    {
        if let Some(node) = self.map.get_mut(&KeyRef { k: &k }) {
            let node_ptr: *mut LruEntry<K, V> = node.as_ptr();

            self.detach(node_ptr);
            self.attach(node_ptr);

            unsafe { Ok((*node_ptr).val.as_ref().unwrap()) }
        } else {
            let v = f(&k)?;
            let (_, node) = self.replace_or_create_node(k, v);
            let node_ptr: *mut LruEntry<K, V> = node.as_ptr();

            self.attach(node_ptr);

            let keyref = unsafe { (*node_ptr).key.as_ref().unwrap() };
            self.map.insert(KeyRef { k: keyref }, node);
            Ok(unsafe { (*node_ptr).val.as_ref().unwrap() })
        }
    }

    /// Returns a reference to the value of the key in the cache if it is
    /// present in the cache and moves the key to the head of the LRU list.
    /// If the key does not exist the provided `FnOnce` is used to populate
    /// the list and a reference is returned. If `FnOnce` returns `Err`,
    /// returns the `Err`. The value referenced by the key is only cloned
    /// (using `to_owned()`) if it doesn't exist in the cache and `FnOnce`
    /// succeeds.
    ///
    /// # Example
    ///
    /// ```
    /// use vlru::LruCache;
    /// use std::rc::Rc;
    ///
    /// let key1 = Rc::new("1".to_owned());
    /// let key2 = Rc::new("2".to_owned());
    /// let mut cache = LruCache::<Rc<String>, String>::new(2);
    /// let f = ||->Result<String, ()> {Err(())};
    /// let a = ||->Result<String, ()> {Ok("One".to_owned())};
    /// let b = ||->Result<String, ()> {Ok("Two".to_owned())};
    /// assert_eq!(cache.try_get_or_insert_ref(&key1, a), Ok(&"One".to_owned()));
    /// assert_eq!(cache.try_get_or_insert_ref(&key2, f), Err(()));
    /// assert_eq!(cache.try_get_or_insert_ref(&key2, b), Ok(&"Two".to_owned()));
    /// assert_eq!(cache.try_get_or_insert_ref(&key2, a), Ok(&"Two".to_owned()));
    /// assert_eq!(Rc::strong_count(&key1), 2);
    /// assert_eq!(Rc::strong_count(&key2), 2); // key2 was only cloned once even though we
    ///                                         // queried it 3 times
    /// ```
    pub fn try_get_or_insert_ref<'a, Q, F, E>(&'a mut self, k: &'_ Q, f: F) -> Result<&'a V, E>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized + alloc::borrow::ToOwned<Owned = K>,
        F: FnOnce() -> Result<V, E>,
    {
        if let Some(node) = self.map.get_mut(KeyWrapper::from_ref(k)) {
            let node_ptr: *mut LruEntry<K, V> = node.as_ptr();

            self.detach(node_ptr);
            self.attach(node_ptr);

            unsafe { Ok((*node_ptr).val.as_ref().unwrap()) }
        } else {
            let v = f()?;
            let (_, node) = self.replace_or_create_node(k.to_owned(), v);
            let node_ptr: *mut LruEntry<K, V> = node.as_ptr();

            self.attach(node_ptr);

            let keyref = unsafe { (*node_ptr).key.as_ref().unwrap()};
            self.map.insert(KeyRef { k: keyref }, node);
            Ok(unsafe { (*node_ptr).val.as_ref().unwrap() })
        }
    }

    /// Returns a mutable reference to the value of the key in the cache if it is
    /// present in the cache and moves the key to the head of the LRU list.
    /// If the key does not exist the provided `FnOnce` is used to populate
    /// the list and a mutable reference is returned.
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
    /// let v = cache.get_or_insert_mut(2, ||"c");
    /// assert_eq!(v, &"b");
    /// *v = "d";
    /// assert_eq!(cache.get_or_insert_mut(2, ||"e"), &mut "d");
    /// assert_eq!(cache.get_or_insert_mut(3, ||"f"), &mut "f");
    /// assert_eq!(cache.get_or_insert_mut(3, ||"e"), &mut "f");
    /// ```
    pub fn get_or_insert_mut<F>(&mut self, k: K, f: F) -> &mut V
    where
        F: FnOnce() -> V,
    {
        self.get_or_insert_mut_with_key(k, |_| f())
    }

    /// Returns a mutable reference to the value of the key in the cache if it is
    /// present in the cache and moves the key to the head of the LRU list.
    /// If the key does not exist the provided `FnOnce` is used by passing
    /// a reference to the key to populate the list and a mutable reference
    /// is returned.
    ///
    /// # Example
    ///
    /// ```
    /// use vlru::LruCache;
    /// let mut cache = LruCache::new(2);
    ///
    /// cache.put("One", 1);
    /// cache.put("Two", 2);
    /// cache.put("Two", 3);
    /// cache.put("Three", 4);
    ///
    /// assert_eq!(cache.get_or_insert_mut_with_key("Two", |_|1), &mut 3);
    /// assert_eq!(cache.get_or_insert_mut_with_key("Three", |k|k.len()), &mut 4);
    /// assert_eq!(cache.get_or_insert_mut_with_key("One", |_|1), &mut 1);
    /// assert_eq!(cache.get_or_insert_mut_with_key("One", |k|k.len()), &mut 1);
    /// ```
    pub fn get_or_insert_mut_with_key<F>(&mut self, k: K, f: F) -> &mut V
    where
        F: FnOnce(&K) -> V,
    {
        if let Some(node) = self.map.get_mut(&KeyRef { k: &k }) {
            let node_ptr: *mut LruEntry<K, V> = node.as_ptr();

            self.detach(node_ptr);
            self.attach(node_ptr);

            unsafe { (*node_ptr).val.as_mut().unwrap() }
        } else {
            let v = f(&k);
            let (_, node) = self.replace_or_create_node(k, v);
            let node_ptr: *mut LruEntry<K, V> = node.as_ptr();

            self.attach(node_ptr);

            let keyref = unsafe { (*node_ptr).key.as_ref().unwrap()};
            self.map.insert(KeyRef { k: keyref }, node);
            unsafe { (*node_ptr).val.as_mut().unwrap() }
        }
    }

    /// Returns a mutable reference to the value of the key in the cache if it is
    /// present in the cache and moves the key to the head of the LRU list.
    /// If the key does not exist the provided `FnOnce` is used to populate
    /// the list and a mutable reference is returned. The value referenced by the
    /// key is only cloned (using `to_owned()`) if it doesn't exist in the cache.
    ///
    /// # Example
    ///
    /// ```
    /// use vlru::LruCache;
    /// use std::rc::Rc;
    ///
    /// let key1 = Rc::new("1".to_owned());
    /// let key2 = Rc::new("2".to_owned());
    /// let mut cache = LruCache::<Rc<String>, &'static str>::new(2);
    /// cache.get_or_insert_mut_ref(&key1, ||"One");
    /// let v = cache.get_or_insert_mut_ref(&key2, ||"Two");
    /// *v = "New two";
    /// assert_eq!(cache.get_or_insert_mut_ref(&key2, ||"Two"), &mut "New two");
    /// assert_eq!(Rc::strong_count(&key1), 2);
    /// assert_eq!(Rc::strong_count(&key2), 2); // key2 was only cloned once even though we
    ///                                         // queried it 2 times
    /// ```
    pub fn get_or_insert_mut_ref<'a, Q, F>(&'a mut self, k: &'_ Q, f: F) -> &'a mut V
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized + alloc::borrow::ToOwned<Owned = K>,
        F: FnOnce() -> V,
    {
        if let Some(node) = self.map.get_mut(KeyWrapper::from_ref(k)) {
            let node_ptr: *mut LruEntry<K, V> = node.as_ptr();

            self.detach(node_ptr);
            self.attach(node_ptr);

            unsafe { (*node_ptr).val.as_mut().unwrap() }
        } else {
            let v = f();
            let (_, node) = self.replace_or_create_node(k.to_owned(), v);
            let node_ptr: *mut LruEntry<K, V> = node.as_ptr();

            self.attach(node_ptr);

            let keyref = unsafe { (*node_ptr).key.as_ref().unwrap()};
            self.map.insert(KeyRef { k: keyref }, node);
            unsafe { (*node_ptr).val.as_mut().unwrap() }
        }
    }

    /// Returns a mutable reference to the value of the key in the cache if it is
    /// present in the cache and moves the key to the head of the LRU list.
    /// If the key does not exist the provided `FnOnce` is used to populate
    /// the list and a mutable reference is returned. If `FnOnce` returns `Err`,
    /// returns the `Err`.
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
    ///
    /// let f = ||->Result<&str, String> {Err("failed".to_owned())};
    /// let a = ||->Result<&str, String> {Ok("a")};
    /// let b = ||->Result<&str, String> {Ok("b")};
    /// if let Ok(v) = cache.try_get_or_insert_mut(2, a) {
    ///     *v = "d";
    /// }
    /// assert_eq!(cache.try_get_or_insert_mut(2, a), Ok(&mut "d"));
    /// assert_eq!(cache.try_get_or_insert_mut(3, f), Err("failed".to_owned()));
    /// assert_eq!(cache.try_get_or_insert_mut(4, b), Ok(&mut "b"));
    /// assert_eq!(cache.try_get_or_insert_mut(4, a), Ok(&mut "b"));
    /// ```
    pub fn try_get_or_insert_mut<F, E>(&mut self, k: K, f: F) -> Result<&mut V, E>
    where
        F: FnOnce() -> Result<V, E>,
    {
        self.try_get_or_insert_mut_with_key(k, |_| f())
    }

    /// Returns a mutable reference to the value of the key in the cache if it is
    /// present in the cache and moves the key to the head of the LRU list.
    /// If the key does not exist the provided `FnOnce` is used by passing
    /// a reference to the key to populate the list and a mutable reference
    /// is returned. If `FnOnce` returns `Err`, returns the `Err`.
    ///
    /// # Example
    ///
    /// ```
    /// use vlru::LruCache;
    /// let mut cache = LruCache::new(2);
    ///
    /// cache.put("One", 1);
    /// cache.put("Two", 2);
    /// cache.put("Two", 3);
    /// cache.put("Three", 4);
    ///
    /// let f = |_: &&str|->Result<usize, String> {Err("failed".to_owned())};
    /// let len = |k: &&str|->Result<usize, String> {Ok(k.len())};
    /// let zero = |_: &&str|->Result<usize, String> {Ok(0)};
    /// assert_eq!(cache.try_get_or_insert_mut_with_key("Two", len), Ok(&mut 3));
    /// assert_eq!(cache.try_get_or_insert_mut_with_key("Three", len), Ok(&mut 4));
    /// assert_eq!(cache.try_get_or_insert_mut_with_key("Four", f), Err("failed".to_owned()));
    /// assert_eq!(cache.try_get_or_insert_mut_with_key("Five", len), Ok(&mut 4));
    /// assert_eq!(cache.try_get_or_insert_mut_with_key("Five", zero), Ok(&mut 4));
    /// ```
    pub fn try_get_or_insert_mut_with_key<F, E>(&mut self, k: K, f: F) -> Result<&mut V, E>
    where
        F: FnOnce(&K) -> Result<V, E>,
    {
        if let Some(node) = self.map.get_mut(&KeyRef { k: &k }) {
            let node_ptr: *mut LruEntry<K, V> = node.as_ptr();

            self.detach(node_ptr);
            self.attach(node_ptr);

            unsafe { Ok((*node_ptr).val.as_mut().unwrap()) }
        } else {
            let v = f(&k)?;
            let (_, node) = self.replace_or_create_node(k, v);
            let node_ptr: *mut LruEntry<K, V> = node.as_ptr();

            self.attach(node_ptr);

            let keyref = unsafe { (*node_ptr).key.as_ref().unwrap()};
            self.map.insert(KeyRef { k: keyref }, node);
            unsafe { Ok((*node_ptr).val.as_mut().unwrap()) }
        }
    }

    /// Returns a mutable reference to the value of the key in the cache if it is
    /// present in the cache and moves the key to the head of the LRU list.
    /// If the key does not exist the provided `FnOnce` is used to populate
    /// the list and a mutable reference is returned. If `FnOnce` returns `Err`,
    /// returns the `Err`. The value referenced by the key is only cloned
    /// (using `to_owned()`) if it doesn't exist in the cache and `FnOnce`
    /// succeeds.
    ///
    /// # Example
    ///
    /// ```
    /// use vlru::LruCache;
    /// use std::rc::Rc;
    ///
    /// let key1 = Rc::new("1".to_owned());
    /// let key2 = Rc::new("2".to_owned());
    /// let mut cache = LruCache::<Rc<String>, String>::new(2);
    /// let f = ||->Result<String, ()> {Err(())};
    /// let a = ||->Result<String, ()> {Ok("One".to_owned())};
    /// let b = ||->Result<String, ()> {Ok("Two".to_owned())};
    /// assert_eq!(cache.try_get_or_insert_mut_ref(&key1, a), Ok(&mut "One".to_owned()));
    /// assert_eq!(cache.try_get_or_insert_mut_ref(&key2, f), Err(()));
    /// if let Ok(v) = cache.try_get_or_insert_mut_ref(&key2, b) {
    ///     *v = "New two".to_owned();
    /// }
    /// assert_eq!(cache.try_get_or_insert_mut_ref(&key2, a), Ok(&mut "New two".to_owned()));
    /// assert_eq!(Rc::strong_count(&key1), 2);
    /// assert_eq!(Rc::strong_count(&key2), 2); // key2 was only cloned once even though we
    ///                                         // queried it 3 times
    /// ```
    pub fn try_get_or_insert_mut_ref<'a, Q, F, E>(
        &'a mut self,
        k: &'_ Q,
        f: F,
    ) -> Result<&'a mut V, E>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized + alloc::borrow::ToOwned<Owned = K>,
        F: FnOnce() -> Result<V, E>,
    {
        if let Some(node) = self.map.get_mut(KeyWrapper::from_ref(k)) {
            let node_ptr: *mut LruEntry<K, V> = node.as_ptr();

            self.detach(node_ptr);
            self.attach(node_ptr);

            unsafe { Ok((*node_ptr).val.as_mut().unwrap()) }
        } else {
            let v = f()?;
            let (_, node) = self.replace_or_create_node(k.to_owned(), v);
            let node_ptr: *mut LruEntry<K, V> = node.as_ptr();

            self.attach(node_ptr);

            let keyref = unsafe { (*node_ptr).key.as_ref().unwrap()};
            self.map.insert(KeyRef { k: keyref }, node);
            unsafe { Ok((*node_ptr).val.as_mut().unwrap()) }
        }
    }
}