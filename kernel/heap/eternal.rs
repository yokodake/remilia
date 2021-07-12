use alloc::alloc::{AllocError, Allocator, Layout};
use core::ptr::NonNull;

/// An allocator that does not deallocate
#[derive(Copy, Clone, PartialEq, Eq, Debug, Ord, PartialOrd)]
pub struct EternalAlloc(u64, u64);

impl EternalAlloc {
    pub const fn new(start: u64, end: u64) -> EternalAlloc {
        EternalAlloc(start, end)
    }
    /// SAFETY new must be in (start, end) range
    unsafe fn swap(&mut self, new: u64) -> u64 {
        let v = self.0;
        self.0 = new;
        v
    }
    const fn get(&self) -> u64 {
        self.0
    }
    const fn end(&self) -> u64 {
        self.1
    }
    pub fn try_alloc(&mut self, layout: Layout) -> Option<NonNull<u8>> {
        // TODO alignment
        if self.get() + (layout.size() as u64) < self.1 {
            let addr = unsafe { self.swap(self.end() + (layout.size() as u64)) };
            NonNull::new(addr as *mut u8)
        } else {
            None
        }
    }
    pub fn alloc(&mut self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        match self.try_alloc(layout) {
            Some(ptr) => Ok(NonNull::slice_from_raw_parts(ptr, layout.size())),
            None => Err(AllocError {}),
        }
    }
}

unsafe impl Allocator for EternalAlloc {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        super::ETERNAL_HEAP.lock().alloc(layout)
    }

    unsafe fn deallocate(&self, _ptr: NonNull<u8>, _layout: Layout) {}

    unsafe fn grow(
        &self,
        _ptr: NonNull<u8>,
        _old_layout: Layout,
        _new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        todo!()
    }
    unsafe fn grow_zeroed(
        &self,
        _ptr: NonNull<u8>,
        _old_layout: Layout,
        _new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        todo!()
    }
    unsafe fn shrink(
        &self,
        _ptr: NonNull<u8>,
        _old_layout: Layout,
        _new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        todo!("change alignment")
    }
}
