#![feature(register_tool)]
#![register_tool(rr)]
#![feature(custom_inner_attributes)]

#![rr::package("extra_libs")]
#![rr::coq_prefix("extralibs.borrow")]

// No round-trip laws are stated: correctness of Borrow is out of scope
//
// borrow_to   : Self     -> Borrowed   (what `.borrow()` produces)
// borrow_from : Borrowed -> Self       (recover the key from a query; used by map specs)
#[rr::export_as(core::borrow::Borrow)]
#[rr::exists("borrow_to"   : "{xt_of Self} → {xt_of Borrowed}")]
#[rr::exists("borrow_from" : "{xt_of Borrowed} → {xt_of Self}")]
pub trait Borrow<Borrowed: ?Sized> {
    #[rr::returns("borrow_to self")]
    fn borrow(&self) -> &Borrowed;
}
