use uart_16550::SerialPort;
use spin::Mutex;
use lazy_static::lazy_static;
use crate::vga::Colour;

lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(0x3f8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

#[doc(hidden)]
pub fn _serial_print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    // FIXME: panic! will print to SERIAL1
    SERIAL1.lock().write_fmt(args).expect("Printing to serial failed!");
}
