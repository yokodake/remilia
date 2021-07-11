//! TODO: expand the heap when we run out.
pub mod ffallocator;

use core::fmt;

use crate::locked::Locked;
use crate::vmem::paging::{LPAGE_SIZE, PAGE_SIZE};
use crate::{error, info, warn, dbg};
use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use ffallocator::FFAlloc;
use pache::addr::Addr;
use pache::Range;
use pache::{KiB, MiB};
use x86_64::structures::paging::{
    mapper::MapToError, page::PageRange, FrameAllocator, Mapper, Page, PageSize, PageTableFlags,
    PhysFrame, Size2MiB, Size4KiB,
};
use x86_64::{PhysAddr, VirtAddr};

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
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
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

/// another reason why the x86_64 library isn't really super good
fn map_page_err(err: MapToError<Size2MiB>) -> MapToError<Size4KiB> {
    use x86_64::structures::paging::mapper::MapToError::*;
    match err {
        PageAlreadyMapped(frame) => PageAlreadyMapped(unsafe {
            PhysFrame::from_start_address_unchecked(frame.start_address())
        }),
        FrameAllocationFailed => FrameAllocationFailed,
        ParentEntryHugePage => ParentEntryHugePage,
    }
}

/// helper to build page ranges
fn page_range<S: PageSize>(start_addr: u64, end_addr: u64) -> PageRange<S> {
    let start = VirtAddr::new(start_addr);
    let end = VirtAddr::new(end_addr.align_down(S::SIZE));
    let start_page = Page::containing_address(start);
    let end_page = Page::containing_address(end);
    Page::range(start_page, end_page)
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

// rename to kernel heap
#[global_allocator]
pub static KERNEL_HEAP: Locked<FFAlloc> = Locked::new(FFAlloc::new());

/// the frame allocate to bootstrap the kernel heap
#[derive(Clone, Copy)]
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
    // TODO: since pop_region supports variable sized we could support more than 1 small region
    /// SAFETY: memory_map must be valid
    pub unsafe fn new(memory_map: &'static MemoryMap) -> Option<Self> {
        let mut small = false;
        let mut large = false;
        // we use two regions: one with small pages, another with large pages
        let mut allocator = BootstrapFramesAlloc {
            srange: [Range::new(u64::MAX, u64::MAX); 2],
            snext: u64::MAX,
            lrange: Range::new(u64::MAX, u64::MAX),
            lnext: u64::MAX,
            back: u64::MAX,
        };
        for r in memory_map.iter() {
            // we already found two regions
            if small && large {
                break;
            }
            if r.region_type == MemoryRegionType::Usable {
                if !large && (r.range.end_addr() - r.range.start_addr() > TOTAL_HEAP_SIZE as u64) {
                    // the region is big enough to contain the entire kernel heap
                    // which probably isn't needed, since we use small regions too
                    // but it's a pain to do that computation beforehand
                    info!(
                        "found suitable large region @ 0x{:08x}-0x{:08x}",
                        r.range.start_addr(),
                        r.range.end_addr()
                    );
                    large = true;
                    // FIXME we ignore the suffix, the region might be just big enough *with* the suffix.
                    let (small, big, _) = r
                        .range
                        .start_addr()
                        .align_to(r.range.start_addr(), LPAGE_SIZE as u64);

                    allocator.srange[1] = Range::new(small, big);
                    // FIXME the range is too big, since there might be a suffix.
                    allocator.lrange = Range::new(big, r.range.end_addr());
                    allocator.lnext = big;

                    // FIXME is this correct even if end_addr() is not aligned?
                    allocator.back =
                        (r.range.end_addr() - PAGE_SIZE as u64).align_down(PAGE_SIZE as u64);
                } else if !small {
                    // if it's usable, but not big enough = small region
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
                // small region is optional
                warn!("no small region found for bootstrapping the heap.");
            }
            allocator.srange.sort_unstable_by_key(|r| r.start);
            Some(allocator)
        } else {
            error!("no suitable region found for bootstrapping the heap.");
            None
        }
    }

    /// pop a small page region (the page is not mapped)
    pub fn pop_region(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let i = self.srange.iter().position(|r| r.contains(self.snext));
        if i.is_none() {
            // allocate from the back;
            let addr = self.back;
            if addr - PAGE_SIZE <= self.lnext {
                error!("Out of physical memory. Time to download more RAM.");
            }
            self.back -= PAGE_SIZE as u64;
            return Self::atof(PhysAddr::new(addr));
        }

        let frame = Self::atof(PhysAddr::new(self.snext));
        self.snext += PAGE_SIZE as u64;

        let i = i.unwrap();

        // if we exceeded the range && we were not in the last range
        if !self.srange[i].contains(self.snext) && i < self.srange.len() - 1 {
            self.snext = self.srange[i + 1].start;
        }
        // if we were in the last range, we don't have to do anything anymore
        // as the start of the function will take care of using frames from the back

        frame
    }
    /// pop a large page region (the page is not mapped)
    pub fn pop_large(&mut self) -> Option<PhysFrame<Size2MiB>> {
        assert!(<Size2MiB as PageSize>::SIZE == LPAGE_SIZE); // TODO move this to tests
        if self.lnext + LPAGE_SIZE >= self.back {
            error!("Out of physical memory for Large Pages.");
            return None;
        }

        let addr = PhysAddr::new(self.lnext);
        self.lnext += LPAGE_SIZE as u64;
        Self::atof(addr)
    }
    /// (phys) addr to frame
    fn atof<S: PageSize>(addr: PhysAddr) -> Option<PhysFrame<S>> {
        PhysFrame::from_start_address(addr).map_or_else(
            |_| {
                error!("{:p} not aligned to physframe", addr);
                None
            },
            Some,
        )
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

impl fmt::Debug for BootstrapFramesAlloc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BootstrapFramesAlloc")
            .field("srange", &format_args!("{:#x?}", self.srange))
            .field("snext", &format_args!("{:#x}", self.snext))
            .field("lrange", &format_args!("{:#x?}", self.lrange))
            .field("lnext", &format_args!("{:#x}", self.lnext))
            .field("back", &format_args!("{:#x}", self.back))
            .finish()
    }
}

/// the eternal kernel heap
pub static ETERNAL_HEAP: Locked<EternalAlloc> = Locked::new(EternalAlloc::new());

/// An allocator that does not deallocate
#[derive(Copy, Clone, PartialEq, Eq, Debug, Ord, PartialOrd)]
pub struct EternalAlloc(u64);

impl EternalAlloc {
    pub const fn new() -> EternalAlloc {
        EternalAlloc(ETERNAL_HEAP_START)
    }
    fn swap(&mut self, new: u64) -> u64 {
        let v = self.0;
        self.0 = new;
        v
    }
    fn get(&self) -> u64 {
        self.0
    }
}

pub fn eternal_alloc<T>(size: u64) -> *mut T {
    use core::ptr;
    {
        let mut allocator = ETERNAL_HEAP.lock();
        let ptr = allocator.get();
        if ptr + size < ETERNAL_HEAP_END {
            return allocator.swap(ptr + size) as *mut T;
        }
    }
    ptr::null_mut()
}
