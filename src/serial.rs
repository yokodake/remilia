use uart_16550::SerialPort;
use spin::Mutex;
use lazy_static::lazy_static;

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

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::serial::_serial_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($fmt:expr) => ($crate::print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) =>
        ($crate::print!(concat!($fmt, "\n"), $($arg)*));
}

#[macro_export]
macro_rules! eprint {
    ($($arg:tt)*) => {
        // TODO change color
        $crate::print!($($arg)*);
    };
}
#[macro_export]
macro_rules! eprintln {
    ($($arg:tt)*) => {
        // TODO change color
        $crate::println!($($arg)*);
    }
}
