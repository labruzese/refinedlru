use core::{
    mem,
    hash::{Hash, BuildHasher},
    ptr::NonNull
};

use alloc::boxed::Box;

use crate::{
    LruCache,
    entry::LruEntry,
    keys::KeyRef,
};

impl<K: Hash + Eq, V, S: BuildHasher> LruCache<K, V, S> {
    // Used internally to swap out a node if the cache is full or to create a new node if space
    // is available. Shared between `put`, `push`, `get_or_insert`, and `get_or_insert_mut`.
    #[allow(clippy::type_complexity)]
    pub(crate) fn replace_or_create_node(&mut self, k: K, v: V) -> (Option<(K, V)>, NonNull<LruEntry<K, V>>) {
        if self.len() == self.cap() {
            // if the cache is full, remove the last entry so we can use it for the new key
            let old_key = KeyRef {
                k: unsafe { (*(*self.tail).prev).key.as_ref().unwrap() },
            };
            let old_node = self.map.remove(&old_key).unwrap();
            let node_ptr: *mut LruEntry<K, V> = old_node.as_ptr();

            // read out the node's old key and value and then replace it
            let replaced = unsafe {
                (
                    mem::replace(&mut (*node_ptr).key, Some(k)).unwrap(),
                    mem::replace(&mut (*node_ptr).val, Some(v)).unwrap(),
                )
            };

            self.detach(node_ptr);

            (Some(replaced), old_node)
        } else {
            // if the cache is not full allocate a new LruEntry
            // Safety: We allocate, turn into raw, and get NonNull all in one step.
            (None, unsafe {
                NonNull::new_unchecked(Box::into_raw(Box::new(LruEntry::new(k, v))))
            })
        }
    }

    pub(crate) fn remove_first(&mut self) -> Option<Box<LruEntry<K, V>>> {
        let next;
        unsafe { next = (*self.head).next }
        if !core::ptr::eq(next, self.tail) {
            let old_key = KeyRef {
                k: unsafe { (*(*self.head).next).key.as_ref().unwrap() },
            };
            let old_node = self.map.remove(&old_key).unwrap();
            let node_ptr: *mut LruEntry<K, V> = old_node.as_ptr();
            self.detach(node_ptr);
            unsafe { Some(Box::from_raw(node_ptr)) }
        } else {
            None
        }
    }

    pub(crate) fn remove_last(&mut self) -> Option<Box<LruEntry<K, V>>> {
        let prev;
        unsafe { prev = (*self.tail).prev }
        if !core::ptr::eq(prev, self.head) {
            let old_key = KeyRef {
                k: unsafe { (*(*self.tail).prev).key.as_ref().unwrap() },
            };
            let old_node = self.map.remove(&old_key).unwrap();
            let node_ptr: *mut LruEntry<K, V> = old_node.as_ptr();
            self.detach(node_ptr);
            unsafe { Some(Box::from_raw(node_ptr)) }
        } else {
            None
        }
    }

    pub(crate) fn detach(&mut self, node: *mut LruEntry<K, V>) {
        unsafe {
            (*(*node).prev).next = (*node).next;
            (*(*node).next).prev = (*node).prev;
        }
    }

    // Attaches `node` after the sigil `self.head` node.
    pub(crate) fn attach(&mut self, node: *mut LruEntry<K, V>) {
        unsafe {
            (*node).next = (*self.head).next;
            (*node).prev = self.head;
            (*self.head).next = node;
            (*(*node).next).prev = node;
        }
    }

    // Attaches `node` before the sigil `self.tail` node.
    pub(crate) fn attach_last(&mut self, node: *mut LruEntry<K, V>) {
        unsafe {
            (*node).next = self.tail;
            (*node).prev = (*self.tail).prev;
            (*self.tail).prev = node;
            (*(*node).prev).next = node;
        }
    }
}
