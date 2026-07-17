use core::ptr;

// Struct used to hold a key value pair. Also contains references to previous and next entries
// so we can maintain the entries in a linked list ordered by their use.
pub(crate) struct LruEntry<K, V> {
    pub key: Option<K>,
    pub val: Option<V>,
    pub prev: *mut LruEntry<K, V>,
    pub next: *mut LruEntry<K, V>,
}

impl<K, V> LruEntry<K, V> {
    pub(crate) fn new(key: K, val: V) -> Self {
        LruEntry {
            key: Some(key),
            val: Some(val),
            prev: ptr::null_mut(),
            next: ptr::null_mut(),
        }
    }

    pub(crate) fn new_sigil() -> Self {
        LruEntry {
            key: None,
            val: None,
            prev: ptr::null_mut(),
            next: ptr::null_mut(),
        }
    }
}
