use crate::{println, vgaprint};
use crate::gdt;
use crate::interrupts::pic::{IRQ};
use x86_64::structures::idt::{HandlerFunc, InterruptDescriptorTable, InterruptStackFrame};
use lazy_static::lazy_static;
use spin::Mutex;

const IRQ_HANDLERS: [(IRQ, HandlerFunc); 2] =
    [ (IRQ::Timer, timer_handler)
    , (IRQ::Keyboard, keyboard_handler)
    ];

pub fn init_idt() {
    lazy_static! { pub static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        for (irq, handler) in IRQ_HANDLERS {
            idt[irq.as_usize()].set_handler_fn(handler);
        }
        idt
    };}
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(sf: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", sf);
}
extern "x86-interrupt" fn double_fault_handler(sf: InterruptStackFrame, _error_code: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}\n", sf);
}
extern "x86-interrupt" fn timer_handler(_: InterruptStackFrame) {
    unsafe {
        super::PICS.lock().notify_eoi(IRQ::Timer.as_u8());
    }
}
extern "x86-interrupt" fn keyboard_handler(_: InterruptStackFrame) {
    crate::devices::KEYBOARD_DEVICE.handle_irq();
    unsafe {
        super::PICS.lock().notify_eoi(IRQ::Keyboard.as_u8());
    }
}

#[test_case]
fn test_breakpoint_exception() {
    x86_64::instructions::interrupts::int3();
}
