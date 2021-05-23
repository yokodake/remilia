pub mod paging;

use core::usize;

use x86_64::structures::paging::{FrameAllocator, OffsetPageTable, PhysFrame, Size4KiB};
use x86_64::{PhysAddr, VirtAddr}; 
use paging::PAGE_SIZE;

/// init a new OffsetPageTable with the l4frame's physical addr and the offset.
///
/// SAFETY: caller must guarantee complete pmem. is mapped to vmem. at the passed `pmem_offset`.
/// Also, only call once because of `&mut` aliasing.
pub unsafe fn init(pmem_offset: VirtAddr) -> OffsetPageTable<'static> {
    let phys = l4frame().start_address();
    let virt : VirtAddr = pmem_offset + phys.as_u64();
    
    OffsetPageTable::new(&mut *(virt.as_mut_ptr()), pmem_offset)
}

fn l4frame() -> PhysFrame {
    use x86_64::registers::control::Cr3;
    Cr3::read().0
}

/*
pub fn create_example_mapping(
    page: Page,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>
) {
    use x86_64::structures::paging::PageTableFlags as Flags;
    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = Flags::PRESENT | Flags::WRITABLE;
    // FIXME: bad, please remove me after testing
    unsafe { mapper.map_to(page, frame, flags, frame_allocator) }
        .expect("map_to failed")
        .flush();
}

*/

pub struct EmptyFrameAllocator;
unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        None
    }
}

use bootloader::bootinfo::{MemoryMap, MemoryRegionType};

pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}
impl BootInfoFrameAllocator {
    /// SAFETY: This function is unsafe because the caller must guarantee that the passed
    /// memory map is valid. The main requirement is that all frames that are marked
    /// as `USABLE` in it are really unused.
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map, 
            next: 0
        }
    }
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        self.memory_map.iter() 
            // get usable regions
            .filter(|r| r.region_type == MemoryRegionType::Usable)
            // map each region to its address range
            .map(|r| r.range.start_addr() .. r.range.end_addr())
            // transform to an iterator of frame start addresses
            .flat_map(|r| r.step_by(PAGE_SIZE))
            // create `PhysFrame`s from the start addresses
            .map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}