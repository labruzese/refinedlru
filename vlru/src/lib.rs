// MIT License
// Copyright (c) 2016 Jerome Froelich
// Copyright (c) 2026 Skylar Abruzese
//
// This file is a modified version of the original work by Jerome Froelich.
// Modifications include RefinedRust annotations and small changes for the
// purposes of formal verification.
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

//! An implementation of a LRU cache. The cache supports `get`, `get_mut`, `put`,
//! and `pop` operations, all of which are O(1). This crate was heavily influenced
//! by the [LRU Cache implementation in an earlier version of Rust's std::collections crate](https://doc.rust-lang.org/0.12.0/std/collections/lru_cache/struct.LruCache.html).
//!
//! ## Example
//!
//! ```rust
//! extern crate vlru;
//!
//! use vlru::LruCache;
//!
//! fn main() {
//!         let mut cache = LruCache::new(2);
//!         cache.put("apple", 3);
//!         cache.put("banana", 2);
//!
//!         assert_eq!(*cache.get(&"apple").unwrap(), 3);
//!         assert_eq!(*cache.get(&"banana").unwrap(), 2);
//!         assert!(cache.get(&"pear").is_none());
//!
//!         assert_eq!(cache.put("banana", 4), Some(2));
//!         assert_eq!(cache.put("pear", 5), None);
//!
//!         assert_eq!(*cache.get(&"pear").unwrap(), 5);
//!         assert_eq!(*cache.get(&"banana").unwrap(), 4);
//!         assert!(cache.get(&"apple").is_none());
//!
//!         {
//!             let v = cache.get_mut(&"banana").unwrap();
//!             *v = 6;
//!         }
//!
//!         assert_eq!(*cache.get(&"banana").unwrap(), 6);
//! }
//! ```

#![feature(register_tool)]
#![register_tool(rr)]
#![feature(custom_inner_attributes)]

#![no_std]

#![rr::package("lru")]
#![rr::coq_prefix("lru.verification")]

#![rr::include("stdlib")]
#![rr::include("sized")]

#![rr::include("borrow")]
#![rr::include("hash")]
#![rr::include("hashmap")]

#![rr::import("vlru.verification.theories", "dll")]

use core::ptr::NonNull;

extern crate std;
extern crate alloc;

use std::collections::HashMap;

mod backend;

mod keys;
mod entry;
mod constructors;
mod pure; 
mod put; 
mod get; 
mod rmw; 
mod pop; 
mod realloc; 
mod meta; 
mod iter; 
mod clone; 
mod debug; 
mod drop;

pub type DefaultHasher = std::collections::hash_map::RandomState;

// cache ≡ list of (K*V) + capacity
#[rr::refined_by("l" : "list ({rt_of K} * {rt_of V})", "cap" : "nat")]
// ∃ head and tail pointers and our map of keys to list nodes
#[rr::exists("hd" : "loc", "tl" : "loc", "m" : "gmap ({rt_of K}) loc")]
// No duplicate keys in cache
#[rr::invariant("NoDup (fst <$> l)")]
// size of cache doesn't exceed capacity
#[rr::invariant("length l ≤ cap")]
// invariants about corrospondance of the map and list see `../verification/specs.v`
#[rr::depends_on(LruEntry)]
#[rr::invariant(#iris "lru_dll
    (λ π l ko vo prev next,
        l ◁ₗ[π, Owned] #((ko, vo, prev, next))
        @ ◁(LruEntry_ty {rt_of K} {rt_of V} <TY> {K} {V} <INST!>)
    )
    π hd tl m l")]
// keys are comparable for equality
#[rr::context("EqDecision {xt_of K}")]
// keys have injection to positive
#[rr::context("Countable {xt_of K}")]
// do not prove correctness of drop glue
#[rr::only_spec(drop_glue)]
pub struct LruCache<K, V, S = DefaultHasher> {
    #[rr::field("m")]
    map: HashMap<keys::KeyRef<K>, NonNull<entry::LruEntry<K, V>>, S>,
    #[rr::field("Z.of_nat cap")]
    cap: usize,

    // head and tail are sigil nodes to facilitate inserting entries
    #[rr::field("hd")] head: *mut entry::LruEntry<K, V>,
    #[rr::field("tl")] tail: *mut entry::LruEntry<K, V>,
}

// The compiler does not automatically derive Send and Sync for LruCache because it contains
// raw pointers. The raw pointers are safely encapsulated by LruCache though so we can
// implement Send and Sync for it below.
unsafe impl<K: Send, V: Send, S: Send> Send for LruCache<K, V, S> {}
unsafe impl<K: Sync, V: Sync, S: Sync> Sync for LruCache<K, V, S> {}