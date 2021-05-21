#![no_std]
#![cfg_attr(test, no_main)]
#![feature( custom_test_frameworks
          , abi_x86_interrupt
          , format_args_nl
          )]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

pub mod debug;
pub mod devices;
pub mod gdt;
pub mod interrupts;

use core::panic::PanicInfo;

pub fn init() {
    gdt::init();
    interrupts::init_idt();
    interrupts::init_pic();
    info!("enabling IRQ");
    x86_64::instructions::interrupts::enable();
}

pub fn halt() -> ! {
    loop { x86_64::instructions::hlt(); }
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

/** ENTRY POINTS */
/// `cargo test` entry point
#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    init();
    test_main();
    halt()
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

pub const QEMU_SUCCESS : u32 = 0x10;
pub const QEMU_FAILURE : u32 = 0x11;
pub fn exit_qemu(code: u32) {
    use x86_64::instructions::port::Port;
    unsafe {
        let mut port = Port::new(0xf4);
        port.write(code);
    }
}
