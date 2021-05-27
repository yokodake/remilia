#![no_std]
#![no_main]
#![feature( custom_test_frameworks
          , abi_x86_interrupt
          , format_args_nl
          , default_alloc_error_handler
          )]
#![test_runner(kernel::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{BootInfo, entry_point};
extern crate alloc;

entry_point!(start);

fn start(boot_info: &'static BootInfo) -> ! {
    kernel::info!("{:p}", &start);
    let p : *const u8 = 0 as *const u8;
    let y : &*const u8 = &p;
    let z : *const & *const u8 = (&y) as *const & *const u8;
    if cfg!(test) {
        #[cfg(test)]
        test_main();
        kernel::halt()
    } else {
        kernel::main(boot_info);
    }
}

use core::panic::PanicInfo;

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kernel::vgaeprintln!("{}", info);
    kernel::error!("{}", info);
    kernel::halt()
}
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kernel::test_panic_handler(info)
}
#[test_case]
fn valid() {
    assert_eq!(1, 1);
}
