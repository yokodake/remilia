//! first fit allocator

use core::alloc::{GlobalAlloc, Layout};
use core::mem;
use core::ptr::{self, NonNull};
use pache::addr::Addr;

use crate::locked::Locked;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct Region {
    next: *mut Region,
    size: u64,
}
impl Region {
    const fn null() -> *mut Region {
        ptr::null_mut()
    }
    fn begin(&self) -> *const Region {
        self as *const Self
    }
    fn end(&self) -> *const Region {
        (self.begin() as u64 + self.size) as *const Region
    }
}
unsafe impl Sync for Region {}
unsafe impl Send for Region {}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct FFAlloc {
    free_list: *mut Region,
}
unsafe impl Sync for FFAlloc {}
unsafe impl Send for FFAlloc {}

impl FFAlloc {
    pub const fn new() -> Self {
        FFAlloc {
            free_list: ptr::null_mut(),
        }
    }

    pub unsafe fn init(&mut self, addr: u64, size: u64) {
        assert!(addr > 0);
        self.add_free_region(addr as *mut Region, size);
    }

    unsafe fn add_free_region(&mut self, addr: *mut Region, size: u64) {
        assert_eq!(addr.align_up(mem::align_of::<Region>() as u64), addr);
        assert!(size >= mem::size_of::<Region>() as u64);
        addr.write(Region {
            size,
            next: self.free_list,
        });
        self.free_list = addr;
    }
    unsafe fn free_region(&mut self, ptr: NonNull<Region>, size: u64) {
        ptr.as_ptr().write(Region {
            next: self.free_list,
            size,
        });
        self.free_list = ptr.as_ptr();
    }
    /// pops a suitable region from the free list
    ///
    /// Returns a pointer to the Region and
    fn pop_region(&mut self, size: u64, align: u64) -> Option<(NonNull<Region>, NonNull<Region>)> {
        // [hd] ---> [region] --> [region.next]
        let mut hd = &mut self.free_list;
        while *hd != Region::null() {
            let region = unsafe { hd.as_mut().unwrap() };
            if let Some(aligned) = Self::can_alloc(&region, size, align) {
                let mut hd = unsafe { NonNull::new_unchecked(*hd) };
                let next = region.next;
                let ret = unsafe {
                    Some((
                        NonNull::new_unchecked(region as *mut Region),
                        NonNull::new_unchecked(aligned),
                    ))
                };
                let hd_ = unsafe { hd.as_mut() };
                hd_.next = next;
                return ret;
            } else {
                hd = unsafe { hd.as_mut().map(|r| &mut r.next).unwrap() };
            }
        }
        None
    }
    /// Returns an aligned pointer inside the region
    fn can_alloc(region: &Region, size: u64, align: u64) -> Option<*mut Region> {
        let begin = region.begin().align_up(align);
        // avoid overflow => unaddressable space anyways
        let end = (begin as u64).checked_add(size)?;
        if end > region.end() as u64 {
            return None;
        }
        let excess = region.end() as u64 - end;
        if excess > 0 && excess < mem::size_of::<Region>() as u64 {
            // rest of region too small to hold a `Region`
            // since we trust layout to deallocate, we can only fail here
            // or we'll get some uber fragmentation
            return None;
        }
        Some(begin as *mut Region)
    }

    fn size_align(layout: Layout) -> Option<(u64, u64)> {
        let layout = layout
            .align_to(mem::align_of::<Region>())
            .ok()?
            .pad_to_align();
        let size = layout.size().max(mem::size_of::<Region>());
        Some((size as u64, layout.align() as u64))
    }
}

unsafe impl GlobalAlloc for Locked<FFAlloc> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let (size, align) = match FFAlloc::size_align(layout) {
            Some(x) => x,
            None => {
                // crate::error!("failed on requested alignment: {:?}", layout);
                return ptr::null_mut();
            }
        };

        let mut allocator = self.lock();
        if let Some((region, begin)) = allocator.pop_region(size, align) {
            let end = match (begin.as_ptr() as u64).checked_add(size) {
                Some(s) => s,
                None => return ptr::null_mut(),
            };
            let excess = region.as_ref().end() as u64 - end;
            if excess > 0 {
                allocator.add_free_region(end as *mut Region, excess);
            }
            begin.as_ptr() as *mut u8
        } else {
            // FIXME: try to get a bigger heap?
            crate::error!("Couldn't find a suitable region to allocate.");
            ptr::null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        let ptr = NonNull::new(ptr as *mut Region).expect("dealloc nullptr");
        let (size, _) = FFAlloc::size_align(layout).expect("requested alignment failed");
        self.lock().free_region(ptr, size);
    }
}
