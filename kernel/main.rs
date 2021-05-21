#![no_std]
#![no_main]
#![feature(custom_test_frameworks, abi_x86_interrupt)]
#![test_runner(kernel::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use kernel::{error, vgaprintln, vgaeprintln};


fn main() -> ! {
    kernel::init();

    vgaprintln!("Hello,{}!", "World");
    kernel::halt()
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    if cfg!(test) {
        #[cfg(test)]
        test_main();
        kernel::halt()
    } else {
        main();
    }
}


#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    vgaeprintln!("{}", info);
    error!("{}", info);
    kernel::halt()
}
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kernel::test_panic_handler(info)
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
#[test_case]
fn trivial_assertion_failure() {
    // assert_eq!(1, 0);
}
