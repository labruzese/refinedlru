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
#![rr::import("slru.verification.theories", "dll")]
#![rr::import("slru.verification.theories", "model")]

use core::{mem, ptr, marker};
extern crate alloc;

use alloc::{
    borrow::{self, Borrow},
    boxed::Box,
};

struct ListNode<K, V> {
    pub key:  Option<K>,
    pub val:  Option<V>,
    pub next: *mut ListNode<K, V>,
    pub prev: *mut ListNode<K, V>,
}
 
impl<K:Eq, V> ListNode<K, V> {
    #[rr::params("k", "v")]
    #[rr::args("k", "v")]
    #[rr::returns("*[Some k; Some v; NULL_loc; NULL_loc]")]
    pub fn new(key: K, val: V) -> Self {
        ListNode {
            key: Some(key),
            val: Some(val),
            next: ptr::null_mut(),
            prev: ptr::null_mut(),
        }
    }
 
    #[rr::returns("*[None; None; NULL_loc; NULL_loc]")]
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
 
    #[rr::skip]
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
#[rr::refined_by("(l, cap)" : "directRT (list ({xt_of K} * {xt_of V}) * nat)")]
// [locs] is the list of payload node addresses, exposed here rather than
// hidden inside [slru_dll] so that the hash map to be added later can relate a
// key to [locs !! i].
#[rr::exists("hd" : "loc", "tl" : "loc", "locs" : "list loc")]
#[rr::invariant("length locs = length l")]
#[rr::invariant("Forall (λ p, p.(loc_a) ≠ 0) (hd :: locs ++ [tl])")]
#[rr::invariant("NoDup l.*1")]
#[rr::invariant("length l ≤ cap")]
#[rr::depends_on(ListNode)]
#[rr::invariant(#iris "slru_dll
    (λ π ln ko vo prev next,
        guarded true (ln ◁ₗ[π, Owned] #( *[ #ko; #vo; #next; #prev ])
        @ ◁(ListNode_ty {rt_of K} {rt_of V} <TY> {K} <TY> {V} <INST!>)))
    π hd tl locs l")]
#[rr::ty_lfts("ty_lfts {K}", "ty_lfts {V}")]
#[rr::ty_wf_E("ty_wf_E {K}", "ty_wf_E {V}")]
pub struct LruCache<K, V> {
    #[rr::field("Z.of_nat cap")] cap: u32,
    #[rr::field("hd")] head: *mut ListNode<K, V>,
    #[rr::field("tl")] tail: *mut ListNode<K, V>,
    #[rr::field("Z.of_nat (length l)")] size: u32,
    #[rr::field("tt")] _map: marker::PhantomData<*mut ListNode<K, V>>,

}

#[rr::context("EqDecision {xt_of K}")]
impl<K: Eq, V> LruCache<K, V> {
    #[rr::params("cap")]
    #[rr::args("cap")]
    #[rr::requires("cap > 0")]
    #[rr::returns("([], Z.to_nat cap)")]
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

    #[rr::skip]
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

    #[rr::skip]
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
impl<K, V> Drop for LruCache<K, V> {
    fn drop(&mut self) {
        let mut cur = self.head;
        while !cur.is_null() {
            let next = unsafe { (*cur).next };
            unsafe { drop(Box::from_raw(cur)) };
            cur = next;
        }
    }
}