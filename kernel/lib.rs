#![no_std]
#![cfg_attr(test, no_main)]
#![feature( custom_test_frameworks
          , abi_x86_interrupt
          , format_args_nl
          )]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_harness_main"]

extern crate alloc;
pub mod debug;
pub mod devices;
pub mod gdt;
pub mod interrupts;
pub mod vmem;

use core::panic::PanicInfo;
use bootloader::BootInfo;
#[cfg(test)]
use bootloader::entry_point;
use x86_64::VirtAddr;

trait GlobalResource {
    fn init();
    fn the() -> &'static Self;
}

pub fn init() {
    gdt::Gdt::init();
    interrupts::init_idt();
    interrupts::init_pic();
    info!("enabling IRQ");
    x86_64::instructions::interrupts::enable();
}

pub fn main(boot_info: &'static BootInfo) -> ! {
    init();
    info!("init done.");
    // vgaprintln!("Hello,{}!", "World");

    let pmem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut _mapper = unsafe { vmem::init(pmem_offset) };
    let mut _frame_allocator = unsafe { vmem::BootInfoFrameAllocator::init(&boot_info.memory_map) };

    info!("entering halt...");
    halt()
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

/* ENTRY POINTS */

#[cfg(test)]
entry_point!(test_main);
#[cfg(test)]
#[no_mangle]
/// `cargo test` entry point
pub fn test_main(_boot_info: &'static BootInfo) -> ! {
    init();
    test_harness_main();
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
