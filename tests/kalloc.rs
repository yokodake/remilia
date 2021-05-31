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
    kernel::init(boot_info);
    main();
}

fn main() -> !{
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
fn big_allocations() {
    const EMPTY : Vec<u8> = Vec::new();
    let mut x : [Vec<u8>; 10] = [EMPTY; 10];
    for i in 0.. 20 {
        for i in 0 .. x.len() {
            x[i] = Vec::with_capacity(4*pache::KiB);
        }
    }
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
