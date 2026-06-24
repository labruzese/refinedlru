// #![allow(internal_features)]

// #![feature(register_tool)]
// #![register_tool(rr)]
// #![feature(custom_inner_attributes)]
// #![feature(nonzero_internals)]

// #![rr::package("extra_libs")]
// #![rr::coq_prefix("extralibs.nonzero")]

// #[rr::export_as(core::num::nonzero::NonZero<usize>)]
// #[rr::refined_by("n" : "Z")]
// #[rr::invariant("(n > 0)%Z")]
// pub struct NonZeroUsizeShim(
//     #[rr::field("n")]
//     usize
// );

// #[rr::export_as(core::num::nonzero::ZeroablePrimitive)]
// pub unsafe trait ZeroablePrimitive {
//     type NonZeroInner;
// }

// #[rr::export_as(core::num::nonzero::ZeroablePrimitive for u8)]
// unsafe impl ZeroablePrimitive for u8 {
//     type NonZeroInner = u8;
// }

// #[rr::export_as(core::num::nonzero::ZeroablePrimitive for u16)]
// unsafe impl ZeroablePrimitive for u16 {
//     type NonZeroInner = u16;
// }

// #[rr::export_as(core::num::nonzero::ZeroablePrimitive for u32)]
// unsafe impl ZeroablePrimitive for u32 {
//     type NonZeroInner = u32;
// }

// #[rr::export_as(core::num::nonzero::ZeroablePrimitive for u64)]
// unsafe impl ZeroablePrimitive for u64 {
//     type NonZeroInner = u64;
// }

// #[rr::export_as(core::num::nonzero::ZeroablePrimitive for u128)]
// unsafe impl ZeroablePrimitive for u128 {
//     type NonZeroInner = u128;
// }

// #[rr::export_as(core::num::nonzero::ZeroablePrimitive for usize)]
// unsafe impl ZeroablePrimitive for usize {
//     type NonZeroInner = usize;
// }


// #[rr::export_as(core::num::nonzero::ZeroablePrimitive for i8)]
// unsafe impl ZeroablePrimitive for i8 {
//     type NonZeroInner = i8;
// }

// #[rr::export_as(core::num::nonzero::ZeroablePrimitive for i16)]
// unsafe impl ZeroablePrimitive for i16 {
//     type NonZeroInner = i16;
// }

// #[rr::export_as(core::num::nonzero::ZeroablePrimitive for i32)]
// unsafe impl ZeroablePrimitive for i32 {
//     type NonZeroInner = i32;
// }

// #[rr::export_as(core::num::nonzero::ZeroablePrimitive for i64)]
// unsafe impl ZeroablePrimitive for i64 {
//     type NonZeroInner = i64;
// }

// #[rr::export_as(core::num::nonzero::ZeroablePrimitive for i128)]
// unsafe impl ZeroablePrimitive for i128 {
//     type NonZeroInner = i128;
// }

// #[rr::export_as(core::num::nonzero::ZeroablePrimitive for isize)]
// unsafe impl ZeroablePrimitive for isize {
//     type NonZeroInner = isize;
// }

// #[rr::export_as(core::num::nonzero::ZeroablePrimitive for char)]
// unsafe impl ZeroablePrimitive for char {
//     type NonZeroInner = char;
// }