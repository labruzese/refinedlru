use core::fmt;
use std::hash::{Hash, BuildHasher};
use crate::LruCache;

#[rr::skip]
impl<K: Hash + Eq, V, S: BuildHasher> fmt::Debug for LruCache<K, V, S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("LruCache")
            .field("len", &self.len())
            .field("cap", &self.cap())
            .finish()
    }
}