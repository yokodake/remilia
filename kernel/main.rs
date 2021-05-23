#![no_std]
#![no_main]
#![feature( custom_test_frameworks
          , abi_x86_interrupt
          , format_args_nl
          )]
#![test_runner(kernel::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{BootInfo, entry_point};

entry_point!(start);

fn start(boot_info: &'static BootInfo) -> ! {
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
