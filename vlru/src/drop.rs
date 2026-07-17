use alloc::boxed::Box;

#[rr::only_spec]
impl<K, V, S> Drop for super::LruCache<K, V, S> {
    fn drop(&mut self) {
        self.map.drain().for_each(|(_, node)| unsafe {
            let mut _node = *Box::from_raw(node.as_ptr());
        });

        let _head = unsafe { *Box::from_raw(self.head) };
        let _tail = unsafe { *Box::from_raw(self.tail) };
    }
}