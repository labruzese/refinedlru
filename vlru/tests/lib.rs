use vlru::LruCache;
use core::fmt::Debug;
use scoped_threadpool::Pool;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Doctests for what should *not* compile
///
/// ```compile_fail
/// let mut cache = vlru::LruCache::<u32, u32>::unbounded();
/// let _: &'static u32 = cache.get_or_insert(0, || 92);
/// ```
///
/// ```compile_fail
/// let mut cache = vlru::LruCache::<u32, u32>::unbounded();
/// let _: Option<(&'static u32, _)> = cache.peek_lru();
/// let _: Option<(_, &'static u32)> = cache.peek_lru();
/// ```
fn _test_lifetimes() {}

extern crate std;
extern crate alloc;

fn assert_opt_eq<V: PartialEq + Debug>(opt: Option<&V>, v: V) {
    assert!(opt.is_some());
    assert_eq!(opt.unwrap(), &v);
}

fn assert_opt_eq_mut<V: PartialEq + Debug>(opt: Option<&mut V>, v: V) {
    assert!(opt.is_some());
    assert_eq!(opt.unwrap(), &v);
}

fn assert_opt_eq_tuple<K: PartialEq + Debug, V: PartialEq + Debug>(
    opt: Option<(&K, &V)>,
    kv: (K, V),
) {
    assert!(opt.is_some());
    let res = opt.unwrap();
    assert_eq!(res.0, &kv.0);
    assert_eq!(res.1, &kv.1);
}

fn assert_opt_eq_mut_tuple<K: PartialEq + Debug, V: PartialEq + Debug>(
    opt: Option<(&K, &mut V)>,
    kv: (K, V),
) {
    assert!(opt.is_some());
    let res = opt.unwrap();
    assert_eq!(res.0, &kv.0);
    assert_eq!(res.1, &kv.1);
}

#[test]
fn test_unbounded() {
    let mut cache = LruCache::unbounded();
    for i in 0..13370 {
        cache.put(i, ());
    }
    assert_eq!(cache.len(), 13370);
}

#[test]
fn test_put_and_get() {
    let mut cache = LruCache::new(2);
    assert!(cache.is_empty());

    assert_eq!(cache.put("apple", "red"), None);
    assert_eq!(cache.put("banana", "yellow"), None);

    assert_eq!(cache.cap(), 2);
    assert_eq!(cache.len(), 2);
    assert!(!cache.is_empty());
    assert_opt_eq(cache.get(&"apple"), "red");
    assert_opt_eq(cache.get(&"banana"), "yellow");
}

#[test]
fn test_put_and_get_or_insert() {
    let mut cache = LruCache::new(2);
    assert!(cache.is_empty());

    assert_eq!(cache.put("apple", "red"), None);
    assert_eq!(cache.put("banana", "yellow"), None);

    assert_eq!(cache.cap(), 2);
    assert_eq!(cache.len(), 2);
    assert!(!cache.is_empty());
    assert_eq!(cache.get_or_insert("apple", || "orange"), &"red");
    assert_eq!(cache.get_or_insert("banana", || "orange"), &"yellow");
    assert_eq!(cache.get_or_insert("lemon", || "orange"), &"orange");
    assert_eq!(cache.get_or_insert("lemon", || "red"), &"orange");
}

#[test]
fn test_put_and_get_or_insert_with_key() {
    let mut cache = LruCache::new(2);
    assert!(cache.is_empty());

    assert_eq!(cache.put("apple", 2), None);
    assert_eq!(cache.put("banana", 8), None);

    assert_eq!(cache.cap(), 2);
    assert_eq!(cache.len(), 2);
    assert!(!cache.is_empty());
    assert_eq!(cache.get_or_insert_with_key("apple", |k| k.len()), &2);
    assert_eq!(cache.get_or_insert_with_key("banana", |k| k.len()), &8);
    assert_eq!(cache.get_or_insert_with_key("lemon", |k| k.len()), &5);
    assert_eq!(cache.get_or_insert_with_key("lemon", |k| k.len() + 3), &5);
}

#[test]
fn test_get_or_insert_ref() {
    use alloc::borrow::ToOwned;
    use alloc::string::String;

    let key1 = Rc::new("1".to_owned());
    let key2 = Rc::new("2".to_owned());
    let mut cache = LruCache::<Rc<String>, String>::new(2);
    assert!(cache.is_empty());
    assert_eq!(cache.get_or_insert_ref(&key1, || "One".to_owned()), "One");
    assert_eq!(cache.get_or_insert_ref(&key2, || "Two".to_owned()), "Two");
    assert_eq!(cache.len(), 2);
    assert!(!cache.is_empty());
    assert_eq!(
        cache.get_or_insert_ref(&key2, || "Not two".to_owned()),
        "Two"
    );
    assert_eq!(
        cache.get_or_insert_ref(&key2, || "Again not two".to_owned()),
        "Two"
    );
    assert_eq!(Rc::strong_count(&key1), 2);
    assert_eq!(Rc::strong_count(&key2), 2);
}

#[test]
fn test_try_get_or_insert() {
    let mut cache = LruCache::new(2);

    assert_eq!(
        cache.try_get_or_insert::<_, &str>("apple", || Ok("red")),
        Ok(&"red")
    );
    assert_eq!(
        cache.try_get_or_insert::<_, &str>("apple", || Err("failed")),
        Ok(&"red")
    );
    assert_eq!(
        cache.try_get_or_insert::<_, &str>("banana", || Ok("orange")),
        Ok(&"orange")
    );
    assert_eq!(
        cache.try_get_or_insert::<_, &str>("lemon", || Err("failed")),
        Err("failed")
    );
    assert_eq!(
        cache.try_get_or_insert::<_, &str>("banana", || Err("failed")),
        Ok(&"orange")
    );
}

#[test]
fn test_try_get_or_insert_with_key() {
    let mut cache = LruCache::new(2);

    assert_eq!(
        cache.try_get_or_insert_with_key::<_, &str>("apple", |k| Ok(k.len())),
        Ok(&5)
    );
    assert_eq!(
        cache.try_get_or_insert_with_key::<_, &str>("apple", |_| Err("failed")),
        Ok(&5)
    );
    assert_eq!(
        cache.try_get_or_insert_with_key::<_, &str>("banana", |k| Ok(k.len())),
        Ok(&6)
    );
    assert_eq!(
        cache.try_get_or_insert_with_key::<_, &str>("lemon", |_| Err("failed")),
        Err("failed")
    );
    assert_eq!(
        cache.try_get_or_insert_with_key::<_, &str>("banana", |_| Err("failed")),
        Ok(&6)
    );
}

#[test]
fn test_try_get_or_insert_ref() {
    use alloc::borrow::ToOwned;
    use alloc::string::String;

    let key1 = Rc::new("1".to_owned());
    let key2 = Rc::new("2".to_owned());
    let mut cache = LruCache::<Rc<String>, String>::new(2);
    let f = || -> Result<String, ()> { Err(()) };
    let a = || -> Result<String, ()> { Ok("One".to_owned()) };
    let b = || -> Result<String, ()> { Ok("Two".to_owned()) };
    assert_eq!(cache.try_get_or_insert_ref(&key1, a), Ok(&"One".to_owned()));
    assert_eq!(cache.try_get_or_insert_ref(&key2, f), Err(()));
    assert_eq!(cache.try_get_or_insert_ref(&key2, b), Ok(&"Two".to_owned()));
    assert_eq!(cache.try_get_or_insert_ref(&key2, a), Ok(&"Two".to_owned()));
    assert_eq!(cache.len(), 2);
    assert_eq!(Rc::strong_count(&key1), 2);
    assert_eq!(Rc::strong_count(&key2), 2);
}

#[test]
fn test_put_and_get_or_insert_mut() {
    let mut cache = LruCache::new(2);
    assert!(cache.is_empty());

    assert_eq!(cache.put("apple", "red"), None);
    assert_eq!(cache.put("banana", "yellow"), None);

    assert_eq!(cache.cap(), 2);
    assert_eq!(cache.len(), 2);

    let v = cache.get_or_insert_mut("apple", || "orange");
    assert_eq!(v, &"red");
    *v = "blue";

    assert_eq!(cache.get_or_insert_mut("apple", || "orange"), &"blue");
    assert_eq!(cache.get_or_insert_mut("banana", || "orange"), &"yellow");
    assert_eq!(cache.get_or_insert_mut("lemon", || "orange"), &"orange");
    assert_eq!(cache.get_or_insert_mut("lemon", || "red"), &"orange");
}

#[test]
fn test_put_and_get_or_insert_mut_with_key() {
    let mut cache = LruCache::new(2);
    assert!(cache.is_empty());

    assert_eq!(cache.put("apple", 2), None);
    assert_eq!(cache.put("banana", 8), None);

    assert_eq!(cache.cap(), 2);
    assert_eq!(cache.len(), 2);

    let v = cache.get_or_insert_mut_with_key("apple", |k| k.len());
    assert_eq!(v, &2);
    *v = 4;

    assert_eq!(cache.get_or_insert_mut_with_key("apple", |k| k.len()), &4);
    assert_eq!(cache.get_or_insert_mut_with_key("banana", |k| k.len()), &8);
    assert_eq!(cache.get_or_insert_mut_with_key("lemon", |k| k.len()), &5);
    assert_eq!(cache.get_or_insert_mut_with_key("lemon", |_| 0), &5);
}

#[test]
fn test_get_or_insert_mut_ref() {
    use alloc::borrow::ToOwned;
    use alloc::string::String;

    let key1 = Rc::new("1".to_owned());
    let key2 = Rc::new("2".to_owned());
    let mut cache = LruCache::<Rc<String>, &'static str>::new(2);
    assert_eq!(cache.get_or_insert_mut_ref(&key1, || "One"), &mut "One");
    let v = cache.get_or_insert_mut_ref(&key2, || "Two");
    *v = "New two";
    assert_eq!(cache.get_or_insert_mut_ref(&key2, || "Two"), &mut "New two");
    assert_eq!(Rc::strong_count(&key1), 2);
    assert_eq!(Rc::strong_count(&key2), 2);
}

#[test]
fn test_try_get_or_insert_mut() {
    let mut cache = LruCache::new(2);

    cache.put(1, "a");
    cache.put(2, "b");
    cache.put(2, "c");

    let f = || -> Result<&str, &str> { Err("failed") };
    let a = || -> Result<&str, &str> { Ok("a") };
    let b = || -> Result<&str, &str> { Ok("b") };
    if let Ok(v) = cache.try_get_or_insert_mut(2, a) {
        *v = "d";
    }
    assert_eq!(cache.try_get_or_insert_mut(2, a), Ok(&mut "d"));
    assert_eq!(cache.try_get_or_insert_mut(3, f), Err("failed"));
    assert_eq!(cache.try_get_or_insert_mut(4, b), Ok(&mut "b"));
    assert_eq!(cache.try_get_or_insert_mut(4, a), Ok(&mut "b"));
}

#[test]
fn test_try_get_or_insert_mut_with_key() {
    let mut cache = LruCache::new(2);

    cache.put("One", 1);
    cache.put("Two", 2);
    cache.put("Two", 3);

    let f = |_: &&str| -> Result<usize, &str> { Err("failed") };
    let len = |k: &&str| -> Result<usize, &str> { Ok(k.len()) };
    let zero = |_: &&str| -> Result<usize, &str> { Ok(0) };
    if let Ok(v) = cache.try_get_or_insert_mut_with_key("Two", f) {
        *v = 6;
    }
    assert_eq!(cache.try_get_or_insert_mut_with_key("Two", len), Ok(&mut 6));
    assert_eq!(
        cache.try_get_or_insert_mut_with_key("Three", f),
        Err("failed")
    );
    assert_eq!(
        cache.try_get_or_insert_mut_with_key("Four", len),
        Ok(&mut 4)
    );
    assert_eq!(
        cache.try_get_or_insert_mut_with_key("Four", zero),
        Ok(&mut 4)
    );
}

#[test]
fn test_try_get_or_insert_mut_ref() {
    use alloc::borrow::ToOwned;
    use alloc::string::String;

    let key1 = Rc::new("1".to_owned());
    let key2 = Rc::new("2".to_owned());
    let mut cache = LruCache::<Rc<String>, String>::new(2);
    let f = || -> Result<String, ()> { Err(()) };
    let a = || -> Result<String, ()> { Ok("One".to_owned()) };
    let b = || -> Result<String, ()> { Ok("Two".to_owned()) };
    assert_eq!(
        cache.try_get_or_insert_mut_ref(&key1, a),
        Ok(&mut "One".to_owned())
    );
    assert_eq!(cache.try_get_or_insert_mut_ref(&key2, f), Err(()));
    if let Ok(v) = cache.try_get_or_insert_mut_ref(&key2, b) {
        assert_eq!(v, &mut "Two");
        *v = "New two".to_owned();
    }
    assert_eq!(
        cache.try_get_or_insert_mut_ref(&key2, a),
        Ok(&mut "New two".to_owned())
    );
    assert_eq!(Rc::strong_count(&key1), 2);
    assert_eq!(Rc::strong_count(&key2), 2);
}

#[test]
fn test_put_and_get_mut() {
    let mut cache = LruCache::new(2);

    cache.put("apple", "red");
    cache.put("banana", "yellow");

    assert_eq!(cache.cap(), 2);
    assert_eq!(cache.len(), 2);
    assert_opt_eq_mut(cache.get_mut(&"apple"), "red");
    assert_opt_eq_mut(cache.get_mut(&"banana"), "yellow");
}

#[test]
fn test_get_mut_and_update() {
    let mut cache = LruCache::new(2);

    cache.put("apple", 1);
    cache.put("banana", 3);

    {
        let v = cache.get_mut(&"apple").unwrap();
        *v = 4;
    }

    assert_eq!(cache.cap(), 2);
    assert_eq!(cache.len(), 2);
    assert_opt_eq_mut(cache.get_mut(&"apple"), 4);
    assert_opt_eq_mut(cache.get_mut(&"banana"), 3);
}

#[test]
fn test_put_update() {
    let mut cache = LruCache::new(2);

    assert_eq!(cache.put("apple", "red"), None);
    assert_eq!(cache.put("apple", "green"), Some("red"));

    assert_eq!(cache.len(), 1);
    assert_opt_eq(cache.get(&"apple"), "green");
}

#[test]
fn test_put_removes_oldest() {
    let mut cache = LruCache::new(2);

    assert_eq!(cache.put("apple", "red"), None);
    assert_eq!(cache.put("banana", "yellow"), None);
    assert_eq!(cache.put("pear", "green"), None);

    assert!(cache.get(&"apple").is_none());
    assert_opt_eq(cache.get(&"banana"), "yellow");
    assert_opt_eq(cache.get(&"pear"), "green");

    // Even though we inserted "apple" into the cache earlier it has since been removed from
    // the cache so there is no current value for `put` to return.
    assert_eq!(cache.put("apple", "green"), None);
    assert_eq!(cache.put("tomato", "red"), None);

    assert!(cache.get(&"pear").is_none());
    assert_opt_eq(cache.get(&"apple"), "green");
    assert_opt_eq(cache.get(&"tomato"), "red");
}

#[test]
fn test_peek() {
    let mut cache = LruCache::new(2);

    cache.put("apple", "red");
    cache.put("banana", "yellow");

    assert_opt_eq(cache.peek(&"banana"), "yellow");
    assert_opt_eq(cache.peek(&"apple"), "red");

    cache.put("pear", "green");

    assert!(cache.peek(&"apple").is_none());
    assert_opt_eq(cache.peek(&"banana"), "yellow");
    assert_opt_eq(cache.peek(&"pear"), "green");
}

#[test]
fn test_peek_mut() {
    let mut cache = LruCache::new(2);

    cache.put("apple", "red");
    cache.put("banana", "yellow");

    assert_opt_eq_mut(cache.peek_mut(&"banana"), "yellow");
    assert_opt_eq_mut(cache.peek_mut(&"apple"), "red");
    assert!(cache.peek_mut(&"pear").is_none());

    cache.put("pear", "green");

    assert!(cache.peek_mut(&"apple").is_none());
    assert_opt_eq_mut(cache.peek_mut(&"banana"), "yellow");
    assert_opt_eq_mut(cache.peek_mut(&"pear"), "green");

    {
        let v = cache.peek_mut(&"banana").unwrap();
        *v = "green";
    }

    assert_opt_eq_mut(cache.peek_mut(&"banana"), "green");
}

#[test]
fn test_peek_lru() {
    let mut cache = LruCache::new(2);

    assert!(cache.peek_lru().is_none());

    cache.put("apple", "red");
    cache.put("banana", "yellow");
    assert_opt_eq_tuple(cache.peek_lru(), ("apple", "red"));

    cache.get(&"apple");
    assert_opt_eq_tuple(cache.peek_lru(), ("banana", "yellow"));

    cache.clear();
    assert!(cache.peek_lru().is_none());
}

#[test]
fn test_peek_mru() {
    let mut cache = LruCache::new(2);

    assert!(cache.peek_mru().is_none());

    cache.put("apple", "red");
    cache.put("banana", "yellow");
    assert_opt_eq_tuple(cache.peek_mru(), ("banana", "yellow"));

    cache.get(&"apple");
    assert_opt_eq_tuple(cache.peek_mru(), ("apple", "red"));

    cache.clear();
    assert!(cache.peek_mru().is_none());
}

#[test]
fn test_contains() {
    let mut cache = LruCache::new(2);

    cache.put("apple", "red");
    cache.put("banana", "yellow");
    cache.put("pear", "green");

    assert!(!cache.contains(&"apple"));
    assert!(cache.contains(&"banana"));
    assert!(cache.contains(&"pear"));
}

#[test]
fn test_pop() {
    let mut cache = LruCache::new(2);

    cache.put("apple", "red");
    cache.put("banana", "yellow");

    assert_eq!(cache.len(), 2);
    assert_opt_eq(cache.get(&"apple"), "red");
    assert_opt_eq(cache.get(&"banana"), "yellow");

    let popped = cache.pop(&"apple");
    assert!(popped.is_some());
    assert_eq!(popped.unwrap(), "red");
    assert_eq!(cache.len(), 1);
    assert!(cache.get(&"apple").is_none());
    assert_opt_eq(cache.get(&"banana"), "yellow");
}

#[test]
fn test_pop_entry() {
    let mut cache = LruCache::new(2);
    cache.put("apple", "red");
    cache.put("banana", "yellow");

    assert_eq!(cache.len(), 2);
    assert_opt_eq(cache.get(&"apple"), "red");
    assert_opt_eq(cache.get(&"banana"), "yellow");

    let popped = cache.pop_entry(&"apple");
    assert!(popped.is_some());
    assert_eq!(popped.unwrap(), ("apple", "red"));
    assert_eq!(cache.len(), 1);
    assert!(cache.get(&"apple").is_none());
    assert_opt_eq(cache.get(&"banana"), "yellow");
}

#[test]
fn test_pop_lru() {
    let mut cache = LruCache::new(200);

    for i in 0..75 {
        cache.put(i, "A");
    }
    for i in 0..75 {
        cache.put(i + 100, "B");
    }
    for i in 0..75 {
        cache.put(i + 200, "C");
    }
    assert_eq!(cache.len(), 200);

    for i in 0..75 {
        assert_opt_eq(cache.get(&(74 - i + 100)), "B");
    }
    assert_opt_eq(cache.get(&25), "A");

    for i in 26..75 {
        assert_eq!(cache.pop_lru(), Some((i, "A")));
    }
    for i in 0..75 {
        assert_eq!(cache.pop_lru(), Some((i + 200, "C")));
    }
    for i in 0..75 {
        assert_eq!(cache.pop_lru(), Some((74 - i + 100, "B")));
    }
    assert_eq!(cache.pop_lru(), Some((25, "A")));
    for _ in 0..50 {
        assert_eq!(cache.pop_lru(), None);
    }
}

#[test]
fn test_pop_mru() {
    let mut cache = LruCache::new(200);

    for i in 0..75 {
        cache.put(i, "A");
    }
    for i in 0..75 {
        cache.put(i + 100, "B");
    }
    for i in 0..75 {
        cache.put(i + 200, "C");
    }
    assert_eq!(cache.len(), 200);

    for i in 0..75 {
        assert_opt_eq(cache.get(&(74 - i + 100)), "B");
    }
    assert_opt_eq(cache.get(&25), "A");

    assert_eq!(cache.pop_mru(), Some((25, "A")));
    for i in 0..75 {
        assert_eq!(cache.pop_mru(), Some((i + 100, "B")));
    }
    for i in 0..75 {
        assert_eq!(cache.pop_mru(), Some((74 - i + 200, "C")));
    }
    for i in (26..75).into_iter().rev() {
        assert_eq!(cache.pop_mru(), Some((i, "A")));
    }
    for _ in 0..50 {
        assert_eq!(cache.pop_mru(), None);
    }
}

#[test]
fn test_clear() {
    let mut cache = LruCache::new(2);

    cache.put("apple", "red");
    cache.put("banana", "yellow");

    assert_eq!(cache.len(), 2);
    assert_opt_eq(cache.get(&"apple"), "red");
    assert_opt_eq(cache.get(&"banana"), "yellow");

    cache.clear();
    assert_eq!(cache.len(), 0);
}

#[test]
fn test_resize_larger() {
    let mut cache = LruCache::new(2);

    cache.put(1, "a");
    cache.put(2, "b");
    cache.resize(4);
    cache.put(3, "c");
    cache.put(4, "d");

    assert_eq!(cache.len(), 4);
    assert_eq!(cache.get(&1), Some(&"a"));
    assert_eq!(cache.get(&2), Some(&"b"));
    assert_eq!(cache.get(&3), Some(&"c"));
    assert_eq!(cache.get(&4), Some(&"d"));
}

#[test]
fn test_resize_smaller() {
    let mut cache = LruCache::new(4);

    cache.put(1, "a");
    cache.put(2, "b");
    cache.put(3, "c");
    cache.put(4, "d");

    cache.resize(2);

    assert_eq!(cache.len(), 2);
    assert!(cache.get(&1).is_none());
    assert!(cache.get(&2).is_none());
    assert_eq!(cache.get(&3), Some(&"c"));
    assert_eq!(cache.get(&4), Some(&"d"));
}

#[test]
fn test_send() {
    use std::thread;

    let mut cache = LruCache::new(4);
    cache.put(1, "a");

    let handle = thread::spawn(move || {
        assert_eq!(cache.get(&1), Some(&"a"));
    });

    assert!(handle.join().is_ok());
}

#[test]
fn test_multiple_threads() {
    let mut pool = Pool::new(1);
    let mut cache = LruCache::new(4);
    cache.put(1, "a");

    let cache_ref = &cache;
    pool.scoped(|scoped| {
        scoped.execute(move || {
            assert_eq!(cache_ref.peek(&1), Some(&"a"));
        });
    });

    assert_eq!((cache_ref).peek(&1), Some(&"a"));
}

#[test]
fn test_iter_forwards() {
    let mut cache = LruCache::new(3);
    cache.put("a", 1);
    cache.put("b", 2);
    cache.put("c", 3);

    {
        // iter const
        let mut iter = cache.iter();
        assert_eq!(iter.len(), 3);
        assert_opt_eq_tuple(iter.next(), ("c", 3));

        assert_eq!(iter.len(), 2);
        assert_opt_eq_tuple(iter.next(), ("b", 2));

        assert_eq!(iter.len(), 1);
        assert_opt_eq_tuple(iter.next(), ("a", 1));

        assert_eq!(iter.len(), 0);
        assert_eq!(iter.next(), None);
    }
    {
        // iter mut
        let mut iter = cache.iter_mut();
        assert_eq!(iter.len(), 3);
        assert_opt_eq_mut_tuple(iter.next(), ("c", 3));

        assert_eq!(iter.len(), 2);
        assert_opt_eq_mut_tuple(iter.next(), ("b", 2));

        assert_eq!(iter.len(), 1);
        assert_opt_eq_mut_tuple(iter.next(), ("a", 1));

        assert_eq!(iter.len(), 0);
        assert_eq!(iter.next(), None);
    }
}

#[test]
fn test_iter_backwards() {
    let mut cache = LruCache::new(3);
    cache.put("a", 1);
    cache.put("b", 2);
    cache.put("c", 3);

    {
        // iter const
        let mut iter = cache.iter();
        assert_eq!(iter.len(), 3);
        assert_opt_eq_tuple(iter.next_back(), ("a", 1));

        assert_eq!(iter.len(), 2);
        assert_opt_eq_tuple(iter.next_back(), ("b", 2));

        assert_eq!(iter.len(), 1);
        assert_opt_eq_tuple(iter.next_back(), ("c", 3));

        assert_eq!(iter.len(), 0);
        assert_eq!(iter.next_back(), None);
    }

    {
        // iter mut
        let mut iter = cache.iter_mut();
        assert_eq!(iter.len(), 3);
        assert_opt_eq_mut_tuple(iter.next_back(), ("a", 1));

        assert_eq!(iter.len(), 2);
        assert_opt_eq_mut_tuple(iter.next_back(), ("b", 2));

        assert_eq!(iter.len(), 1);
        assert_opt_eq_mut_tuple(iter.next_back(), ("c", 3));

        assert_eq!(iter.len(), 0);
        assert_eq!(iter.next_back(), None);
    }
}

#[test]
fn test_iter_forwards_and_backwards() {
    let mut cache = LruCache::new(3);
    cache.put("a", 1);
    cache.put("b", 2);
    cache.put("c", 3);

    {
        // iter const
        let mut iter = cache.iter();
        assert_eq!(iter.len(), 3);
        assert_opt_eq_tuple(iter.next(), ("c", 3));

        assert_eq!(iter.len(), 2);
        assert_opt_eq_tuple(iter.next_back(), ("a", 1));

        assert_eq!(iter.len(), 1);
        assert_opt_eq_tuple(iter.next(), ("b", 2));

        assert_eq!(iter.len(), 0);
        assert_eq!(iter.next_back(), None);
    }
    {
        // iter mut
        let mut iter = cache.iter_mut();
        assert_eq!(iter.len(), 3);
        assert_opt_eq_mut_tuple(iter.next(), ("c", 3));

        assert_eq!(iter.len(), 2);
        assert_opt_eq_mut_tuple(iter.next_back(), ("a", 1));

        assert_eq!(iter.len(), 1);
        assert_opt_eq_mut_tuple(iter.next(), ("b", 2));

        assert_eq!(iter.len(), 0);
        assert_eq!(iter.next_back(), None);
    }
}

#[test]
fn test_iter_multiple_threads() {
    let mut pool = Pool::new(1);
    let mut cache = LruCache::new(3);
    cache.put("a", 1);
    cache.put("b", 2);
    cache.put("c", 3);

    let mut iter = cache.iter();
    assert_eq!(iter.len(), 3);
    assert_opt_eq_tuple(iter.next(), ("c", 3));

    {
        let iter_ref = &mut iter;
        pool.scoped(|scoped| {
            scoped.execute(move || {
                assert_eq!(iter_ref.len(), 2);
                assert_opt_eq_tuple(iter_ref.next(), ("b", 2));
            });
        });
    }

    assert_eq!(iter.len(), 1);
    assert_opt_eq_tuple(iter.next(), ("a", 1));

    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);
}

#[test]
fn test_iter_clone() {
    let mut cache = LruCache::new(3);
    cache.put("a", 1);
    cache.put("b", 2);

    let mut iter = cache.iter();
    let mut iter_clone = iter.clone();

    assert_eq!(iter.len(), 2);
    assert_opt_eq_tuple(iter.next(), ("b", 2));
    assert_eq!(iter_clone.len(), 2);
    assert_opt_eq_tuple(iter_clone.next(), ("b", 2));

    assert_eq!(iter.len(), 1);
    assert_opt_eq_tuple(iter.next(), ("a", 1));
    assert_eq!(iter_clone.len(), 1);
    assert_opt_eq_tuple(iter_clone.next(), ("a", 1));

    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);
    assert_eq!(iter_clone.len(), 0);
    assert_eq!(iter_clone.next(), None);
}

#[test]
fn test_into_iter() {
    let mut cache = LruCache::new(3);
    cache.put("a", 1);
    cache.put("b", 2);
    cache.put("c", 3);

    let mut iter = cache.into_iter();
    assert_eq!(iter.len(), 3);
    assert_eq!(iter.next(), Some(("a", 1)));

    assert_eq!(iter.len(), 2);
    assert_eq!(iter.next(), Some(("b", 2)));

    assert_eq!(iter.len(), 1);
    assert_eq!(iter.next(), Some(("c", 3)));

    assert_eq!(iter.len(), 0);
    assert_eq!(iter.next(), None);
}

#[test]
fn test_that_pop_actually_detaches_node() {
    let mut cache = LruCache::new(5);

    cache.put("a", 1);
    cache.put("b", 2);
    cache.put("c", 3);
    cache.put("d", 4);
    cache.put("e", 5);

    assert_eq!(cache.pop(&"c"), Some(3));

    cache.put("f", 6);

    let mut iter = cache.iter();
    assert_opt_eq_tuple(iter.next(), ("f", 6));
    assert_opt_eq_tuple(iter.next(), ("e", 5));
    assert_opt_eq_tuple(iter.next(), ("d", 4));
    assert_opt_eq_tuple(iter.next(), ("b", 2));
    assert_opt_eq_tuple(iter.next(), ("a", 1));
    assert!(iter.next().is_none());
}

#[test]
fn test_get_with_borrow() {
    use alloc::string::String;

    let mut cache = LruCache::new(2);

    let key = String::from("apple");
    cache.put(key, "red");

    assert_opt_eq(cache.get("apple"), "red");
}

#[test]
fn test_get_mut_with_borrow() {
    use alloc::string::String;

    let mut cache = LruCache::new(2);

    let key = String::from("apple");
    cache.put(key, "red");

    assert_opt_eq_mut(cache.get_mut("apple"), "red");
}

#[test]
fn test_no_memory_leaks() {
    static DROP_COUNT: AtomicUsize = AtomicUsize::new(0);

    struct DropCounter;

    impl Drop for DropCounter {
        fn drop(&mut self) {
            DROP_COUNT.fetch_add(1, Ordering::SeqCst);
        }
    }

    let n = 100;
    for _ in 0..n {
        let mut cache = LruCache::new(1);
        for i in 0..n {
            cache.put(i, DropCounter {});
        }
    }
    assert_eq!(DROP_COUNT.load(Ordering::SeqCst), n * n);
}

#[test]
fn test_no_memory_leaks_with_clear() {
    static DROP_COUNT: AtomicUsize = AtomicUsize::new(0);

    struct DropCounter;

    impl Drop for DropCounter {
        fn drop(&mut self) {
            DROP_COUNT.fetch_add(1, Ordering::SeqCst);
        }
    }

    let n = 100;
    for _ in 0..n {
        let mut cache = LruCache::new(1);
        for i in 0..n {
            cache.put(i, DropCounter {});
        }
        cache.clear();
    }
    assert_eq!(DROP_COUNT.load(Ordering::SeqCst), n * n);
}

#[test]
fn test_no_memory_leaks_with_resize() {
    static DROP_COUNT: AtomicUsize = AtomicUsize::new(0);

    struct DropCounter;

    impl Drop for DropCounter {
        fn drop(&mut self) {
            DROP_COUNT.fetch_add(1, Ordering::SeqCst);
        }
    }

    let n = 100;
    for _ in 0..n {
        let mut cache = LruCache::new(1);
        for i in 0..n {
            cache.put(i, DropCounter {});
        }
        cache.clear();
    }
    assert_eq!(DROP_COUNT.load(Ordering::SeqCst), n * n);
}

#[test]
fn test_no_memory_leaks_with_pop() {
    static DROP_COUNT: AtomicUsize = AtomicUsize::new(0);

    #[derive(Hash, Eq)]
    struct KeyDropCounter(usize);

    impl PartialEq for KeyDropCounter {
        fn eq(&self, other: &Self) -> bool {
            self.0.eq(&other.0)
        }
    }

    impl Drop for KeyDropCounter {
        fn drop(&mut self) {
            DROP_COUNT.fetch_add(1, Ordering::SeqCst);
        }
    }

    let n = 100;
    for _ in 0..n {
        let mut cache = LruCache::new(1);

        for i in 0..100 {
            cache.put(KeyDropCounter(i), i);
            cache.pop(&KeyDropCounter(i));
        }
    }

    assert_eq!(DROP_COUNT.load(Ordering::SeqCst), n * n * 2);
}

#[test]
fn test_promote_and_demote() {
    let mut cache = LruCache::new(5);
    for i in 0..5 {
        cache.push(i, i);
    }
    cache.promote(&1);
    cache.promote(&0);
    cache.demote(&3);
    cache.demote(&4);
    assert_eq!(cache.pop_lru(), Some((4, 4)));
    assert_eq!(cache.pop_lru(), Some((3, 3)));
    assert_eq!(cache.pop_lru(), Some((2, 2)));
    assert_eq!(cache.pop_lru(), Some((1, 1)));
    assert_eq!(cache.pop_lru(), Some((0, 0)));
    assert_eq!(cache.pop_lru(), None);
}

#[test]
fn test_get_key_value() {
    use alloc::string::String;

    let mut cache = LruCache::new(2);

    let key = String::from("apple");
    cache.put(key, "red");

    assert_eq!(
        cache.get_key_value("apple"),
        Some((&String::from("apple"), &"red"))
    );
    assert_eq!(cache.get_key_value("banana"), None);
}

#[test]
fn test_get_key_value_mut() {
    use alloc::string::String;

    let mut cache = LruCache::new(2);

    let key = String::from("apple");
    cache.put(key, "red");

    let (k, v) = cache.get_key_value_mut("apple").unwrap();
    assert_eq!(k, &String::from("apple"));
    assert_eq!(v, &mut "red");
    *v = "green";

    assert_eq!(
        cache.get_key_value("apple"),
        Some((&String::from("apple"), &"green"))
    );
    assert_eq!(cache.get_key_value("banana"), None);
}

#[test]
fn test_clone() {
    let mut cache = LruCache::new(3);
    cache.put("a", 1);
    cache.put("b", 2);
    cache.put("c", 3);

    let mut cloned = cache.clone();

    assert_eq!(cache.pop_lru(), Some(("a", 1)));
    assert_eq!(cloned.pop_lru(), Some(("a", 1)));

    assert_eq!(cache.pop_lru(), Some(("b", 2)));
    assert_eq!(cloned.pop_lru(), Some(("b", 2)));

    assert_eq!(cache.pop_lru(), Some(("c", 3)));
    assert_eq!(cloned.pop_lru(), Some(("c", 3)));

    assert_eq!(cache.pop_lru(), None);
    assert_eq!(cloned.pop_lru(), None);
}

#[test]
fn test_clone_unbounded() {
    let mut cache = LruCache::unbounded();
    cache.put("a", 1);
    cache.put("b", 2);
    cache.put("c", 3);

    let mut cloned = cache.clone();

    assert_eq!(cache.pop_lru(), Some(("a", 1)));
    assert_eq!(cloned.pop_lru(), Some(("a", 1)));

    assert_eq!(cache.pop_lru(), Some(("b", 2)));
    assert_eq!(cloned.pop_lru(), Some(("b", 2)));

    assert_eq!(cache.pop_lru(), Some(("c", 3)));
    assert_eq!(cloned.pop_lru(), Some(("c", 3)));

    assert_eq!(cache.pop_lru(), None);
    assert_eq!(cloned.pop_lru(), None);
}

#[test]
fn iter_mut_stacked_borrows_violation() {
    let mut cache: LruCache<i32, i32> = LruCache::new(3);
    cache.put(1, 10);
    cache.put(2, 20);
    cache.put(3, 30);

    for (_k, v) in cache.iter_mut() {
        *v *= 2;
    }

    assert_eq!(cache.get(&1), Some(&20));
    assert_eq!(cache.get(&2), Some(&40));
    assert_eq!(cache.get(&3), Some(&60));
}