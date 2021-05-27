#![no_std]
#![no_main]
#![feature(custom_test_frameworks, default_alloc_error_handler)]
#![test_runner(kernel::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader::{BootInfo, entry_point};
use kernel::{heap, println, vmem};
use x86_64::VirtAddr;
use core::panic::PanicInfo;
use alloc::{boxed::Box, vec::Vec};

entry_point!(start);
fn start(boot_info: &'static BootInfo) -> ! {
    kernel::init();

    let pmem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { vmem::init(pmem_offset) };
    let mut frame_allocator = unsafe { vmem::BootInfoFrameAllocator::new(&boot_info.memory_map) };
    unsafe { 
        heap::init(&mut mapper, &mut frame_allocator)
            .expect("failed to init kernel heap");
    }

    test_main();
    kernel::halt()
}

#[test_case]
fn simple_box() {
    let x = Box::new(13);
    let y = Box::new(42);
    assert_eq!(*x, 13);
    assert_eq!(*y, 42);
}

#[test_case]
fn vector() {
    let mut v = Vec::new();
    for i in 0 .. 1000 {
        v.push(i);
    }
    assert_eq!(v.len(), 1000);
}

#[test_case]
fn many_boxes() {
    for i in 0..kernel::heap::HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
}


#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kernel::test_panic_handler(info)
}
