#![feature(register_tool)]
#![register_tool(rr)]
#![feature(custom_inner_attributes)]

#![no_std]

#![rr::package("slru")]
#![rr::coq_prefix("slru.verification")]
#![rr::include("stdlib")]

use core::{mem, ptr};
extern crate alloc;

use alloc::{
    borrow::{self, Borrow},
    boxed::Box,
};

struct ListNode<K: Eq, V> {
    pub key: Option<K>,
    pub val: Option<V>,
    pub next: *mut ListNode<K, V>,
    pub prev: *mut ListNode<K, V>,
}
impl<K: Eq, V> ListNode<K, V> {
    pub fn new(key: K, val: V) -> Self {
        ListNode {
            key: Some(key),
            val: Some(val),
            next: ptr::null_mut(),
            prev: ptr::null_mut(),
        }
    }
    pub fn sigil() -> Self {
        ListNode {
            key: None,
            val: None,
            next: ptr::null_mut(),
            prev: ptr::null_mut(),
        }
    }
}

#[rr::only_spec(drop_glue)]
pub struct LruCache<K: Eq, V> {
    size: u32,
    cap: u32,
    head: *mut ListNode<K, V>,
    tail: *mut ListNode<K, V>,
}

impl<K: Eq, V> LruCache<K, V> {
    pub fn new(cap: u32) -> Self {
        assert!(cap > 0);
        let head = Box::into_raw(Box::new(ListNode::sigil()));
        let tail = Box::into_raw(Box::new(ListNode::sigil()));
        unsafe {
            (*head).next = tail;
        }
        unsafe {
            (*tail).prev = head;
        }
        LruCache {
            head,
            tail,
            size: 0,
            cap,
        }
    }

    pub fn get<'a, Q>(&'a mut self, key: &Q) -> Option<&'a V>
    where
        K: borrow::Borrow<Q>,
        Q: Eq + ?Sized,
    {
        let vptr = self.lookup(key)?;
        self.detach(vptr);
        self.attach(vptr, self.head);
        let v = unsafe { (*vptr).val.as_ref().unwrap() };
        Some(v)
    }

    pub fn put(&mut self, key: K, mut val: V) -> Option<V> {
        match self.lookup(&key) {
            Some(node) => {
                self.detach(node);

                mem::swap(&mut val, unsafe { (*node).val.as_mut().unwrap() });

                self.attach(node, self.head);
                Some(val)
            }
            None if self.size < self.cap => {
                self.size += 1;
                let node_ptr = Box::into_raw(Box::new(ListNode::new(key, val)));
                self.attach(node_ptr, self.head);
                None
            }
            None => {
                let node_ptr = unsafe { (*self.tail).prev };
                self.detach(node_ptr);
                let replaced = unsafe {
                    (
                        mem::replace(&mut (*node_ptr).key, Some(key)).unwrap(),
                        mem::replace(&mut (*node_ptr).val, Some(val)).unwrap(),
                    )
                };
                self.attach(node_ptr, self.head);
                Some(replaced.1)
            }
        }
    }

    fn lookup<Q>(&self, target_key: &Q) -> Option<*mut ListNode<K, V>>
    where
        K: Borrow<Q>,
        Q: Eq + ?Sized,
    {
        let mut cur_ptr = self.head;
        loop {
            cur_ptr = unsafe { (*cur_ptr).next };
            if ptr::eq(self.tail, cur_ptr) {
                return None;
            }

            let key_ref = unsafe { (*(cur_ptr)).key.as_ref().unwrap() };
            if key_ref.borrow() == target_key {
                return Some(cur_ptr);
            }
        }
    }

    fn attach(&mut self, node: *mut ListNode<K, V>, after: *mut ListNode<K, V>) {
        unsafe { (*(*after).next).prev = node }
        unsafe { (*node).next = (*after).next }
        unsafe { (*node).prev = after }
        unsafe { (*after).next = node }
    }

    fn detach(&mut self, node: *mut ListNode<K, V>) {
        let prev = unsafe { (*node).prev };
        let next = unsafe { (*node).next };

        unsafe { (*prev).next = next };
        unsafe { (*next).prev = prev };
    }
}

#[rr::skip]
impl<K: Eq, V> Drop for LruCache<K, V> {
    fn drop(&mut self) {
        let mut cur = self.head;
        while !cur.is_null() {
            let next = unsafe { (*cur).next };
            unsafe { drop(Box::from_raw(cur)) };
            cur = next;
        }
    }
}
