//! TODO: expand the heap when we run out.
//! TODO: second allocator
//! TODO: framepage allocation

pub mod ffallocator;

use crate::info;
use crate::locked::Locked;
use ffallocator::FFAlloc;
use pache::{KiB, MiB};
use x86_64::structures::paging::{
    mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
};
use x86_64::VirtAddr;

/// addresses starting with 0x69f are in the kernel heap
pub const HEAP_START: usize = 0x0069_f000_0000;
pub const HEAP_SIZE: usize = 1 * MiB;
/// addresses starting with 0x69e are in the eternal heap
pub const HEAP_ETERNAL_START: usize = HEAP_START - HEAP_ETERNAL_SIZE;
pub const HEAP_ETERNAL_SIZE: usize = 4 * KiB;

/// initializes the heap by mapping pages.
pub fn init(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    info!(
        "initializing kernel heap @ {:012p}...",
        VirtAddr::new(HEAP_START as u64)
    );
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe { mapper.map_to(page, frame, flags, frame_allocator)?.flush() };
    }

    unsafe {
        init_global_heap();
    }

    Ok(())
}

unsafe fn init_global_heap() {
    info!(
        "initializing GLOBAL_HEAP @ 0x{:x}; size={}",
        HEAP_START, HEAP_SIZE
    );
    GLOBAL_HEAP.lock().init(HEAP_START, HEAP_SIZE);
}

#[global_allocator]
pub static GLOBAL_HEAP: Locked<FFAlloc> = Locked::new(FFAlloc::new());
