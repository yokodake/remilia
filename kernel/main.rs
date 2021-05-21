#![no_std]
#![no_main]
#![feature( custom_test_frameworks
          , abi_x86_interrupt
          , format_args_nl
          )]
#![test_runner(kernel::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use kernel::{error, vgaprintln, vgaeprintln};


fn main() -> ! {
    kernel::init();
    kernel::info!("init done.");
    vgaprintln!("Hello,{}!", "World");

    use x86_64::registers::control::Cr3;
    let (l4_pt, _) = Cr3::read();
    vgaprintln!("Level 4 page table at: 0x{:x}", l4_pt.start_address());


    kernel::info!("entering halt...");
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
