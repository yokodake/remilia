use x86_64::VirtAddr;
use x86_64::structures::tss::TaskStateSegment;
use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor, SegmentSelector};
use lazy_static::lazy_static;
use crate::GlobalResource;

use crate::info;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
pub type Gdt = (GlobalDescriptorTable, Selectors);

lazy_static! {
    static ref GDT : Gdt = {
        let mut gdt = GlobalDescriptorTable::new();
        let code = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss = gdt.add_entry(Descriptor::tss_segment(TaskStateSegment::the()));
        (gdt, Selectors { code, tss })
    };
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_start = VirtAddr::from_ptr(unsafe {&STACK});
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss
    };
}

impl GlobalResource for (GlobalDescriptorTable, Selectors) {
    fn init() {
        use x86_64::instructions::{segmentation::set_cs, tables::load_tss};
        info!("initializing GDT");
        GDT.0.load();
        unsafe {
            set_cs(GDT.1.code);
            load_tss(GDT.1.tss);
        }
    }

    fn the() -> &'static Self {
        &GDT
    }
}
impl GlobalResource for TaskStateSegment {
    fn init() {}

    fn the() -> &'static Self {
        &TSS
    }
}

pub struct Selectors {
    code: SegmentSelector,
    tss: SegmentSelector
}