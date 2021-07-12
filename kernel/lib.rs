#![no_std]
#![cfg_attr(test, no_main)]
#![feature(
    custom_test_frameworks,
    abi_x86_interrupt,
    format_args_nl,
    asm,
    allocator_api,
    nonnull_slice_from_raw_parts
)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_harness_main"]
#![cfg_attr(test, feature(default_alloc_error_handler))]

extern crate alloc;
pub mod debug;
pub mod devices;
pub mod gdt;
pub mod heap;
pub mod interrupts;
pub mod locked;
pub mod vmem;

use alloc::boxed::Box;
#[cfg(test)]
use bootloader::entry_point;
use bootloader::{bootinfo::MemoryMap, BootInfo};
use core::panic::PanicInfo;
use x86_64::VirtAddr;

pub trait GlobalResource {
    fn init();
    fn the() -> &'static Self;
}

pub fn debug_regions(memory_map: &'static MemoryMap) {
    for region in memory_map.iter() {
        info!(
            "Multiboot mmap: [0x{:012x} : 0x{:012x}] {:?}",
            region.range.start_addr(),
            region.range.end_addr(),
            region.region_type
        );
    }
}

pub fn init(boot_info: &'static BootInfo) {
    use heap::BootstrapFramesAlloc;
    gdt::Gdt::init();
    interrupts::init_idt();
    interrupts::init_pic();
    info!("enabling IRQ");
    x86_64::instructions::interrupts::enable();
    info!("CPU init done.");
    debug_regions(&boot_info.memory_map);
    let pmem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { vmem::init(pmem_offset) };
    let mut bootstrap = match unsafe { BootstrapFramesAlloc::new(&boot_info.memory_map) } {
        Some(x) => x,
        None => panic!("Could not find enough memory for initial heap"),
    };
    heap::init(&mut mapper, &mut bootstrap).expect("failed to init kernel heap");
    info!("memory enabled");

    dbg!(alloc::alloc::Layout::new::<u8>());
    dbg!(alloc::alloc::Layout::new::<u16>());
    dbg!(alloc::alloc::Layout::new::<u32>());
    dbg!(alloc::alloc::Layout::new::<u64>());
    dbg!(alloc::alloc::Layout::new::<alloc::vec::Vec<u8>>());
    dbg!(alloc::alloc::Layout::new::<alloc::vec::Vec<u16>>());
    dbg!(alloc::alloc::Layout::new::<alloc::vec::Vec<u32>>());
    dbg!(alloc::alloc::Layout::new::<alloc::vec::Vec<u64>>());
    dbg!(alloc::alloc::Layout::new::<(u8, u8)>());
    dbg!(alloc::alloc::Layout::new::<(u8, u64)>());
    dbg!(alloc::alloc::Layout::new::<(u64, u8)>());
}

pub fn main() -> ! {
    let world = Box::new("World");
    vgaprintln!("Hello, {}!", world);

    info!("entering halt...");
    halt()
}

pub fn halt() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

/** TESTING */
pub trait Testable {
    fn test_name(&self) -> &'static str;
    fn run(&self, align_to: usize) -> ();
}
impl<T: Fn()> Testable for T {
    fn test_name(&self) -> &'static str {
        core::any::type_name::<T>()
    }
    fn run(&self, align_to: usize) {
        let name = self.test_name();
        print!("{}... {: >2$}", name, "", align_to - name.len());
        self();
        println!("[ok]");
    }
}
pub fn test_runner(tests: &[&dyn Testable]) {
    use core::cmp::max;
    println!("Running {} tests\n", tests.len());
    let mut max_len = 0;
    for test in tests {
        max_len = max(test.test_name().len(), max_len);
    }
    for test in tests {
        test.run(max_len)
    }
    exit_qemu(QEMU_SUCCESS);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    error!("[failed]\n");
    error!("Error: {}", info);
    exit_qemu(QEMU_FAILURE);
    halt()
}

/* ENTRY POINTS */

#[cfg(test)]
entry_point!(test_main);
#[cfg(test)]
#[no_mangle]
/// `cargo test` entry point
pub fn test_main(boot_info: &'static BootInfo) -> ! {
    init(boot_info);
    test_harness_main();
    halt()
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

pub const QEMU_SUCCESS: u32 = 0x10;
pub const QEMU_FAILURE: u32 = 0x11;
pub fn exit_qemu(code: u32) {
    use x86_64::instructions::port::Port;
    unsafe {
        let mut port = Port::new(0xf4);
        port.write(code);
    }
}
