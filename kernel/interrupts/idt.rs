use crate::{println, vgaprint};
use crate::gdt;
use crate::interrupts::pic::{InterruptIndex};
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt[InterruptIndex::Timer.as_usize()]
            .set_handler_fn(timer_handler);
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(sf: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", sf);
}
extern "x86-interrupt" fn double_fault_handler(sf: InterruptStackFrame, _error_code: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}\n", sf);
}
extern "x86-interrupt" fn timer_handler(sf: InterruptStackFrame) {
    vgaprint!(".");
    unsafe {
        super::PICS.lock().notify_eoi(InterruptIndex::Timer.as_u8());
    }
}


#[test_case]
fn test_breakpoint_exception() {
    x86_64::instructions::interrupts::int3();
}
