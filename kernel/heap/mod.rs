//! TODO: expand the heap when we run out.
pub mod bootstrap_frames;
pub mod eternal;
pub mod ffallocator;

use alloc::alloc::Layout;
use core::ptr::NonNull;
use x86_64::structures::paging::{
    mapper::MapToError, FrameAllocator, Mapper, PageTableFlags, Size2MiB, Size4KiB,
};
use x86_64::VirtAddr;

use pache::addr::Addr;
use pache::{KiB, MiB};

use crate::info;
use crate::locked::Locked;
use crate::vmem::paging::LPAGE_SIZE;
use bootstrap_frames::{map_page_err, page_range};
use ffallocator::FFAlloc;

pub use bootstrap_frames::BootstrapFramesAlloc;
pub use eternal::EternalAlloc;

/// addresses starting with 0x69f are in the kernel heap
pub const KERNEL_HEAP_START: u64 = 0x0069_f000_0000;
pub const KERNEL_HEAP_SIZE: u64 = (2 * MiB) + (4 * KiB);
pub const KERNEL_HEAP_END: u64 = KERNEL_HEAP_START + KERNEL_HEAP_SIZE;
/// addresses starting with 0x69e are in the eternal heap, BOTH HEAPS MUST BE CONTIGUOUS
pub const ETERNAL_HEAP_START: u64 = KERNEL_HEAP_START - ETERNAL_HEAP_SIZE;
pub const ETERNAL_HEAP_SIZE: u64 = 512 * KiB;
pub const ETERNAL_HEAP_END: u64 = ETERNAL_HEAP_START + ETERNAL_HEAP_SIZE;

pub const TOTAL_HEAP_START: u64 = ETERNAL_HEAP_START;
pub const TOTAL_HEAP_SIZE: u64 = KERNEL_HEAP_SIZE + ETERNAL_HEAP_SIZE;
pub const TOTAL_HEAP_END: u64 = KERNEL_HEAP_END;

/// initializes the heap by mapping pages.
pub fn init<M: Mapper<Size4KiB> + Mapper<Size2MiB>>(
    mapper: &mut M,
    frame_allocator: &mut BootstrapFramesAlloc,
) -> Result<(), MapToError<Size4KiB>> {
    // TODO move these asserts in a better plac
    assert!(ETERNAL_HEAP_END == KERNEL_HEAP_START);
    assert!(TOTAL_HEAP_START + TOTAL_HEAP_SIZE == TOTAL_HEAP_END);

    // TODO print info! the ranges
    info!(
        "initializing kernel heap @ {:012p}...",
        VirtAddr::new(KERNEL_HEAP_START as u64)
    );
    let (prefix, big_start, suffix) = TOTAL_HEAP_START.align_to(TOTAL_HEAP_SIZE, LPAGE_SIZE);
    let small_ranges = [
        page_range::<Size4KiB>(prefix, big_start),
        page_range::<Size4KiB>(suffix, TOTAL_HEAP_END),
    ];
    for range in small_ranges {
        for page in range {
            let frame = frame_allocator
                .allocate_frame()
                .ok_or(MapToError::FrameAllocationFailed)?;
            let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
            unsafe {
                Mapper::<Size4KiB>::map_to(mapper, page, frame, flags, frame_allocator)?.flush()
            };
        }
    }
    for page in page_range::<Size2MiB>(big_start, suffix) {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::HUGE_PAGE;
        unsafe {
            Mapper::<Size2MiB>::map_to(mapper, page, frame, flags, frame_allocator)
                .map_err(map_page_err)?
                .flush()
        };
    }

    unsafe {
        init_kernel_heap();
    }
    init_eternal_heap();

    Ok(())
}

unsafe fn init_kernel_heap() {
    info!(
        "initializing KERNEL_HEAP @ 0x{:x}; size={}",
        KERNEL_HEAP_START, KERNEL_HEAP_SIZE
    );
    KERNEL_HEAP.lock().init(KERNEL_HEAP_START, KERNEL_HEAP_SIZE);
}
fn init_eternal_heap() {
    info!(
        "initializing ETERNAL_HEAP @ 0x{:x}; size={}",
        ETERNAL_HEAP_START, ETERNAL_HEAP_SIZE
    );
}

/// the heap for the kernel
#[global_allocator]
pub static KERNEL_HEAP: Locked<FFAlloc> = Locked::new(FFAlloc::new());

/// the eternal kernel heap
pub static ETERNAL_HEAP: Locked<EternalAlloc> =
    Locked::new(EternalAlloc::new(ETERNAL_HEAP_START, ETERNAL_HEAP_END));

pub fn eternal_alloc<T>(size: usize) -> Option<NonNull<T>> {
    let layout = match Layout::from_size_align(size, 1) {
        Ok(layout) => layout,
        Err(_) => return None,
    };
    ETERNAL_HEAP.lock().try_alloc(layout).map(NonNull::cast)
}
