#![feature(register_tool)]
#![register_tool(rr)]
#![feature(custom_inner_attributes)]
#![allow(unused)]

#![rr::package("extra_libs")]
#![rr::coq_prefix("extralibs.hashmap")]
#![rr::include("alloc")]
#![rr::include("option")]
#![rr::include("cmp")]
#![rr::include("clone")]
#![rr::include("hash")]
#![rr::include("borrow")]

use std::borrow::Borrow;
use std::hash::{BuildHasher, Hash};

// std::collections::HashMap
//
// std's *public* HashMap is `HashMap<K, V, S = RandomState>` -- there is NO
// public allocator parameter (unlike alloc::collections::BTreeMap, which is
// `<K, V, A: Allocator + Clone = Global>`). Modelling it here with an
// export_as shim stops the frontend from structurally unfolding std's private
// hashbrown internals. 
//
// The hasher `S` is carried opaquely (it does not affect the map's functional
// behaviour as a key -> value map), exactly like BTreeMap carries its
// allocator `A` opaquely.
//
// Refinement: a HashMap is refined by a finite map `gmap` from the semantic
// key type to the semantic value type. Keys therefore need `EqDecision` and
// `Countable` on their refinement type.

#[rr::export_as(std::collections::HashMap)]
#[rr::context("EqDecision {xt_of K}")]
#[rr::context("Countable {xt_of K}")]
#[rr::refined_by("M" : "directRT (gmap {xt_of K} ({xt_of V}))")]
#[rr::exists("k", "v", "s")]
#[rr::only_spec(drop_glue)]
pub struct HashMap<K, V, S = std::collections::hash_map::RandomState> {
    #[rr::field("k")]
    _k: K,
    #[rr::field("v")]
    _v: V,
    #[rr::field("s")]
    _s: S,
}

// constructors

#[rr::export_as(std::collections::HashMap)]
#[rr::context("EqDecision {xt_of K}")]
#[rr::context("Countable {xt_of K}")]
#[rr::only_spec]
impl<K, V> HashMap<K, V> {
    #[rr::skip]
    #[rr::returns("∅")]
    pub fn new() -> HashMap<K, V> {
        unimplemented!();
    }

    #[rr::skip]
    #[rr::params("cap")]
    #[rr::args("cap")]
    #[rr::returns("∅")]
    pub fn with_capacity(capacity: usize) -> HashMap<K, V> {
        unimplemented!();
    }
}

#[rr::export_as(std::collections::HashMap)]
#[rr::context("EqDecision {xt_of K}")]
#[rr::context("Countable {xt_of K}")]
#[rr::only_spec]
impl<K, V, S> HashMap<K, V, S>
where
    S: BuildHasher,
{
    #[rr::skip]
    #[rr::params("s")]
    #[rr::args("s")]
    #[rr::returns("∅")]
    pub fn with_hasher(hash_builder: S) -> HashMap<K, V, S> {
        unimplemented!();
    }

    #[rr::skip]
    #[rr::params("cap", "s")]
    #[rr::args("cap", "s")]
    #[rr::returns("∅")]
    pub fn with_capacity_and_hasher(capacity: usize, hash_builder: S) -> HashMap<K, V, S> {
        unimplemented!();
    }
}

// operations

#[rr::export_as(std::collections::HashMap)]
#[rr::context("EqDecision {xt_of K}")]
#[rr::context("Countable {xt_of K}")]
impl<K, V, S> HashMap<K, V, S>
where
    K: Eq + Hash,
    S: BuildHasher,
{
    #[rr::only_spec]
    #[rr::params("m", "k", "v", "γ")]
    #[rr::args("(m, γ)", "k", "v")]
    #[rr::observe("γ": "(<[k := v]> m)")]
    #[rr::returns("m !! k")]
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        unimplemented!();
    }

    #[rr::skip]
    #[rr::params("m", "k")]
    #[rr::args("m", "k")]
    // NB: `{borrow_from}` is the pure conversion K <- Q supplied by the Borrow
    // trait. RefinedRust's BTreeMap template also leaves this abstract (see the
    // TODO there). For the LruCache the lookups always go through KeyRef/KeyWrapper
    // where Q borrows K, so an identity conversion is the common case.
    #[rr::returns("(m !! {K::borrow_from} k)")]
    pub fn get<Q>(&self, k: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        unimplemented!();
    }

    #[rr::skip]
    #[rr::params("m", "k", "γ")]
    #[rr::args("(m, γ)", "k")]
    #[rr::exists("γi")]
    #[rr::returns("if decide (is_Some (m !! {K::borrow_from} k)) then Some (m !!! {K::borrow_from} k, γi) else None")]
    #[rr::observe("γ": "if decide (is_Some (m !! {K::borrow_from} k)) then <[{K::borrow_from} k := PlaceGhost γi]> m else m")]
    pub fn get_mut<Q>(&mut self, k: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        unimplemented!();
    }

    #[rr::only_spec]
    #[rr::params("m", "k", "γ")]
    #[rr::args("(m, γ)", "k")]
    #[rr::observe("γ": "delete ({K::borrow_from} k) m")]
    #[rr::returns("m !! {K::borrow_from} k")]
    pub fn remove<Q>(&mut self, k: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        unimplemented!();
    }

    #[rr::skip]
    #[rr::params("m", "k")]
    #[rr::args("m", "k")]
    #[rr::returns("bool_decide (is_Some (m !! {K::borrow_from} k))")]
    pub fn contains_key<Q>(&self, k: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        unimplemented!();
    }
}

#[rr::export_as(std::collections::HashMap)]
#[rr::context("EqDecision {xt_of K}")]
#[rr::context("Countable {xt_of K}")]
#[rr::only_spec]
impl<K, V, S> HashMap<K, V, S> {
    #[rr::skip]
    #[rr::params("m")]
    #[rr::args("m")]
    #[rr::returns("Z.of_nat (size m)")]
    pub fn len(&self) -> usize {
        unimplemented!();
    }
}