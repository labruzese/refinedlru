use core::{
    borrow::Borrow, 
    hash::{Hash, Hasher}
};


/// Struct used to hold a reference a key
#[rr::refined_by("k" : "{rt_of K}")]
#[rr::exists("l" : "loc", "a" : "lft")]
#[rr::invariant(#iris "l ◁ₗ[π, Shared a] #k @ (◁ ({ty_of K}))")]
#[rr::ty_lfts("ty_lfts {K}")]
pub(crate) struct KeyRef<K> {
    #[rr::field("l")]
    pub k: *const K,
}

impl<K: Hash> Hash for KeyRef<K> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        unsafe { (*self.k).hash(state) }
    }
}

#[rr::verify]
#[rr::instantiate("PEq" := "{K::PEq}")]
impl<K: PartialEq> PartialEq for KeyRef<K> {
    // NB: The unconditional_recursion lint was added in 1.76.0 and can be removed
    // once the current stable version of Rust is 1.76.0 or higher.
    #![allow(unknown_lints)]
    #[allow(clippy::unconditional_recursion)]
    fn eq(&self, other: &KeyRef<K>) -> bool {
        unsafe { (*self.k).eq(&*other.k) }
    }
}

#[rr::verify]
#[rr::instantiate("PEq_refl"    := "{K::PEq_refl}")]
#[rr::instantiate("PEq_sym"     := "{K::PEq_sym}")]
#[rr::instantiate("PEq_trans"   := "{K::PEq_trans}")]
#[rr::instantiate("PEq_leibniz" := "{K::PEq_leibniz}")]
impl<K: Eq> Eq for KeyRef<K> {}

// This type exists to allow a "blanket" Borrow impl for KeyRef without conflicting with the
//  stdlib blanket impl
#[repr(transparent)]
#[rr::refined_by("k" : "{rt_of K}")]
pub(crate) struct KeyWrapper<K: ?Sized>(
    #[rr::field("k")]
    pub K
);

impl<K: ?Sized> KeyWrapper<K> {
    pub fn from_ref(key: &K) -> &Self {
        // safety: KeyWrapper is transparent, so casting the ref like this is allowable
        unsafe { &*(key as *const K as *const KeyWrapper<K>) }
    }
}

impl<K: ?Sized + Hash> Hash for KeyWrapper<K> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

#[rr::instantiate("PEq" := "{K::PEq}")]
impl<K: ?Sized + PartialEq> PartialEq for KeyWrapper<K> {
    // NB: The unconditional_recursion lint was added in 1.76.0 and can be removed
    // once the current stable version of Rust is 1.76.0 or higher.
    #![allow(unknown_lints)]
    #[allow(clippy::unconditional_recursion)]
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

#[rr::instantiate("PEq_refl" := "{K::PEq_refl}")]
#[rr::instantiate("PEq_sym" := "{K::PEq_sym}")]
#[rr::instantiate("PEq_trans" := "{K::PEq_trans}")]
#[rr::instantiate("PEq_leibniz" := "{K::PEq_leibniz}")]
impl<K: ?Sized + Eq> Eq for KeyWrapper<K> {}

impl<K, Q> Borrow<KeyWrapper<Q>> for KeyRef<K>
where
    K: Borrow<Q>,
    Q: ?Sized,
{
    fn borrow(&self) -> &KeyWrapper<Q> {
        let key = unsafe { &*self.k }.borrow();
        KeyWrapper::from_ref(key)
    }
}