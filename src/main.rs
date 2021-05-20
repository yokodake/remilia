#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(osdev::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use osdev::{vgaeprintln, eprintln, vgaprintln};


#[no_mangle]
pub extern "C" fn _start() -> ! {
    if cfg!(test) {
        #[cfg(test)]
        test_main();
        loop {}
    } else {
        main();
    }
}

fn main() -> ! {
    vgaprintln!("Hello,{}!", "World");
    loop {}
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    vgaeprintln!("{}", info);
    eprintln!("{}", info);
    loop {}
}
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    osdev::test_panic_handler(info)
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
#[test_case]
fn trivial_assertion_failure() {
    // assert_eq!(1, 0);
}
