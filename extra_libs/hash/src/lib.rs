#![feature(register_tool)]
#![register_tool(rr)]
#![feature(custom_inner_attributes)]

#![rr::package("extra_libs")]
#![rr::coq_prefix("extralibs.hash")]

#[rr::export_as(core::hash::Hasher)]
pub trait Hasher {
    // Required methods.
    #[rr::only_spec]
    fn finish(&self) -> u64;

    #[rr::only_spec]
    fn write(&mut self, bytes: &[u8]);
}

#[rr::export_as(core::hash::Hash)]
pub trait Hash {
    // Required method.
    #[rr::only_spec]
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher;

    // Provided method.
    #[rr::only_spec]
    fn hash_slice<H>(data: &[Self], state: &mut H)
    where
        H: Hasher,
        Self: Sized,
    {
        for piece in data {
            piece.hash(state);
        }
    }
}

#[rr::export_as(core::hash::BuildHasher)]
pub trait BuildHasher {
    // BuildHasher::Hasher: Hasher
    type Hasher: Hasher;

    #[rr::only_spec]
    fn build_hasher(&self) -> Self::Hasher;
}