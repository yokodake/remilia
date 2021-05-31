#![no_std]
#![no_main]
#![feature( custom_test_frameworks
          , default_alloc_error_handler
          )]

use core::panic::PanicInfo;
use kernel::{QEMU_FAILURE, QEMU_SUCCESS, exit_qemu, println, print};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    should_fail();
    println!("[test did non panic]");
    exit_qemu(QEMU_FAILURE);

    kernel::halt()
}

fn should_fail() {
    print!("tests/should_panic::should_fail... ");
    assert_eq!(0, 1);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("[ok]");
    exit_qemu(QEMU_SUCCESS);
    kernel::halt()
}
