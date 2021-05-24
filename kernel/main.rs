#![no_std]
#![no_main]
#![feature( custom_test_frameworks
          , abi_x86_interrupt
          , format_args_nl
          )]
#![reexport_test_harness_main = "test_main"]

fn hello() {
    let video = 0xb8000 as *mut u16;
    unsafe {
        video.offset(0).write(0x0240);
        video.offset(1).write(0x0268); // h
        video.offset(2).write(0x026e); // e
        video.offset(3).write(0x026c); // l
        video.offset(4).write(0x026c); // l
    }
}

#[no_mangle]
pub fn rust_main() -> ! {
    hello();
    hang()
}

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    hang()
}

fn hang() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

