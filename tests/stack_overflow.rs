#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(default_alloc_error_handler)]

use core::panic::PanicInfo;
use kernel::{self, exit_qemu, gdt, gdt::Gdt, print, println, GlobalResource, QEMU_SUCCESS};
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

lazy_static! {
    static ref TEST_IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            idt.double_fault
                .set_handler_fn(test_double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }

        idt
    };
}

pub fn init_test_idt() {
    TEST_IDT.load();
}

extern "x86-interrupt" fn test_double_fault_handler(_: InterruptStackFrame, _: u64) -> ! {
    println!("[ok]");
    exit_qemu(QEMU_SUCCESS);
    kernel::halt()
}

#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow();
    volatile::Volatile::new(0).read();
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    print!("stack_overflow::stack_overflow... ");

    Gdt::init();
    init_test_idt();

    stack_overflow();

    panic!("[failed to handle triple fault]");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kernel::test_panic_handler(info);
}
