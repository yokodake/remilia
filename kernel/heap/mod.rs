//! TODO: expand the heap when we run out.
//! TODO: second allocator
//! TODO: framepage allocation

pub mod ffallocator;

use crate::locked::Locked;
use crate::vmem::paging::{LPAGE_SIZE, PAGE_SIZE};
use crate::{error, info, warn};
use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use ffallocator::FFAlloc;
use pache::addr::Addr;
use pache::Range;
use pache::{KiB, MiB};
use x86_64::structures::paging::{
    mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, PhysFrame, Size2MiB, Size4KiB,
};
use x86_64::{PhysAddr, VirtAddr};

/// addresses starting with 0x69f are in the kernel heap
pub const HEAP_START: u64 = 0x0069_f000_0000;
pub const HEAP_SIZE: u64 = 2 * MiB;
/// addresses starting with 0x69e are in the eternal heap
pub const HEAP_ETERNAL_START: u64 = HEAP_START - HEAP_ETERNAL_SIZE;
pub const HEAP_ETERNAL_SIZE: u64 = 512 * KiB;
pub const HEAP_TOTAL_SIZE: u64 = HEAP_SIZE + HEAP_ETERNAL_SIZE;

/// initializes the heap by mapping pages.
pub fn init(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut BootstrapFramesAlloc,
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

// FIXME: variable page size
#[derive(Clone, Copy, Debug)]
pub struct BootstrapFramesAlloc {
    pub srange: [Range<u64>; 2],
    pub snext: u64,
    pub lrange: Range<u64>,
    pub lnext: u64,
    pub back: u64, // if we're out of small pages, grow from the back
}
impl BootstrapFramesAlloc {
    // FIXME: align_down end_addresses ?
    // FIXME: this is ugly, please rewrite
    pub fn new(memory_map: &'static MemoryMap) -> Option<Self> {
        let mut small = false;
        let mut large = false;
        let mut allocator = BootstrapFramesAlloc {
            srange: [Range::new(u64::MAX, u64::MAX); 2],
            snext: u64::MAX,
            lrange: Range::new(u64::MAX, u64::MAX),
            lnext: u64::MAX,
            back: u64::MAX,
        };
        for r in memory_map.iter() {
            if small && large {
                break;
            }
            if r.region_type == MemoryRegionType::Usable {
                if !large && (r.range.end_addr() - r.range.start_addr() > HEAP_TOTAL_SIZE as u64) {
                    info!(
                        "found suitable large region @ 0x{:08x}-0x{:08x}",
                        r.range.start_addr(),
                        r.range.end_addr()
                    );
                    large = true;
                    // FIXME: align_to
                    let size = r.range.start_addr() - r.range.end_addr();
                    let (small, big, end) = r
                        .range
                        .start_addr()
                        .align_to(r.range.start_addr(), LPAGE_SIZE as u64);

                    allocator.srange[1] = Range::new(small, big);
                    allocator.lrange = Range::new(big, r.range.end_addr());
                    allocator.lnext = big;

                    // FIXME is this correct even if end_addr() is not aligned?
                    allocator.back =
                        (r.range.end_addr() - PAGE_SIZE as u64).align_down(PAGE_SIZE as u64);
                } else if !small {
                    small = true;
                    info!(
                        "found a small region @ 0x{:08x}-0x{:08x}",
                        r.range.start_addr(),
                        r.range.end_addr()
                    );
                    allocator.srange[0] = Range::new(r.range.start_addr(), r.range.end_addr());
                    allocator.snext = r.range.start_addr();
                }
            }
        }
        if large {
            if !small {
                warn!("no small region found for bootstrapping the heap.");
            }
            allocator.srange.sort_unstable_by_key(|r| r.start);
            Some(allocator)
        } else {
            error!("no suitable region found for bootstrapping the heap.");
            None
        }
    }

    pub fn pop_region(&mut self) -> Option<PhysFrame<Size4KiB>> {
        // FIXME: what if end is unaligned but smaller than cap
        if !self.srange.iter().any(|r| r.contains(self.snext)) {
            // allocate from the back;
            let addr = self.back;
            if addr <= self.lnext {
                error!("Out of physical memory. Time to download more RAM.");
            }
            self.back -= PAGE_SIZE as u64;
            return Some(PhysFrame::containing_address(PhysAddr::new(addr)));
        }

        let addr = PhysAddr::new(self.snext);
        if self.srange[0].contains(self.snext) {
            self.snext += PAGE_SIZE as u64;
            if !self.srange[0].contains(self.snext) {
                self.snext = self.srange[1].start;
            }
        }
        Some(PhysFrame::containing_address(addr))
    }
    pub fn pop_large(&mut self) -> Option<PhysFrame<Size2MiB>> {
        if self.lnext >= self.back {
            return None;
        }

        let addr = PhysAddr::new(self.lnext);
        self.lnext += LPAGE_SIZE as u64;
        Some(PhysFrame::containing_address(addr))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootstrapFramesAlloc {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        self.pop_region()
    }
}
unsafe impl FrameAllocator<Size2MiB> for BootstrapFramesAlloc {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size2MiB>> {
        self.pop_large()
    }
}
