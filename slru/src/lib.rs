// This implementation uses ownership semantics that assumes there's a hashmap 
// looking at the linked list nodes which will be implemented in the future.

#![feature(register_tool)]
#![register_tool(rr)]
#![feature(custom_inner_attributes)]

#![expect(incomplete_features)]
#![feature(explicit_tail_calls)]

#![no_std]

#![rr::package("slru")]
#![rr::coq_prefix("slru.verification")]
#![rr::include("stdlib")]

use core::{mem, ptr, marker};
extern crate alloc;

use alloc::{
    borrow::{self, Borrow},
    boxed::Box,
};

#[rr::refined_by("()": "(option ({rt_of K} * {rt_of V}) * loc * loc)")]
#[rr::exists("kv : option ({rt_of K} * {rt_of V})")]
#[rr::invariant("∃ kv, 
match kv with 
    Some (kb, vb) => kb == k && vb == v
    None => is_none k && is_none v
.")]

struct ListNode<K: Eq, V> {
    #[rr::field("k")] pub key: Option<K>,
    #[rr::field("v")] pub val: Option<V>,
    #[rr::field("n")] pub next: *mut ListNode<K, V>,
    #[rr::field("p")] pub prev: *mut ListNode<K, V>,
}
impl<K: Eq, V> ListNode<K, V> {
    #[rr::params("k", "v")]
    #[rr::args("k", "v")]
    #[rr::returns("(Some k, Some v, NULL_loc, NULL_loc)")]
    pub fn new(key: K, val: V) -> Self {
        ListNode {
            key: Some(key),
            val: Some(val),
            next: ptr::null_mut(),
            prev: ptr::null_mut(),
        }
    }

    #[rr::returns("(None, None, NULL_loc, NULL_loc)")]
    pub fn sigil() -> Self {
        ListNode {
            key: None,
            val: None,
            next: ptr::null_mut(),
            prev: ptr::null_mut(),
        }
    }

    pub fn attach(&mut self, node: &mut ListNode<K, V>) {
        unsafe { (*self.next).prev = node }
        node.next = self.next;
        node.prev = self;
        self.next = node;
    }

    pub fn detach(&mut self) {
        let prev = self.prev;
        let next = self.next;

        unsafe { (*prev).next = next };
        unsafe { (*next).prev = prev };
    }

    pub fn find<Q>(&self, target_key: &Q) -> Option<*mut ListNode<K, V>>
    where
        K: Borrow<Q>,
        Q: Eq + ?Sized,
    {
        let node = unsafe { &*self.next };
        let key: Option<&Q> = node.key.as_ref().map(Borrow::borrow);
        match key {
            Some(k) if k == target_key => Some(self.next),
            Some(_) => become node.find(target_key),
            None => None,
        }
    }
}

#[rr::only_spec(drop_glue)]
#[rr::refined_by("(l, cap)": "(list ({rt_of K} * {rt_of V}) * nat)")]
#[rr::exists("hd" : "loc", "tl" : "loc")]
#[rr::invariant("NoDup (fst <$> l)")]
#[rr::invariant("length l ≤ cap")]
#[rr::invariant(#iris "slru_dll π hd tl l")]
#[rr::context("EqDecision {xt_of K}")]
pub struct LruCache<K: Eq, V> {
    #[rr::field("cap")] cap: u32,
    #[rr::field("hd")] head: *mut ListNode<K, V>,
    #[rr::field("tl")] tail: *mut ListNode<K, V>,
    size: u32,
    _map: marker::PhantomData<*mut ListNode<K, V>>,
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
            cap,
            head,
            tail,
            size: 0,
            _map: marker::PhantomData,
        }
    }

    pub fn get<'a, Q>(&'a mut self, key: &Q) -> Option<&'a V>
    where
        K: borrow::Borrow<Q>,
        Q: Eq + ?Sized,
    {
        let head = unsafe {self.head.as_mut_unchecked()};
        let node = head.find(key)?;
        // we have the only mut ref since nobody self have mut self and we've only accessed through list
        let rnode = unsafe { node.as_mut().unwrap() };
        rnode.detach();
        head.attach(rnode);
        Some(rnode.val.as_ref().unwrap())
    }

    pub fn put(&mut self, key: K, mut val: V) -> Option<V> {
        let rhead = unsafe { self.head.as_mut_unchecked() };
        match rhead.find(&key).map(|n|unsafe{n.as_mut_unchecked()}) {
            Some(node) => {
                node.detach();

                mem::swap(&mut val, node.val.as_mut().unwrap());

                rhead.attach(node);
                Some(val)
            }
            None if self.size < self.cap => {
                self.size += 1;
                let mut node_ptr = Box::new(ListNode::new(key, val));
                rhead.attach(&mut node_ptr);
                None
            }
            None => {
                debug_assert!(self.cap >= 1);
                let rlast = unsafe { (*self.tail).prev.as_mut_unchecked() };
                rlast.detach();
                let _old_key = mem::replace(&mut rlast.key, Some(key)).unwrap();
                let old_val  = mem::replace(&mut rlast.val, Some(val)).unwrap();
                rhead.attach(rlast);
                Some(old_val)
            }
        }
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