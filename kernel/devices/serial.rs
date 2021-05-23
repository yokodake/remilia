use core::fmt::Write;

use sakuya::ansi_term::TermStyle;
use uart_16550::SerialPort;
use spin::Mutex;
use lazy_static::lazy_static;
use x86_64::instructions::interrupts;

pub const SERIAL_PORT_ID: u16 = 0x3f8;

lazy_static! {
    pub static ref SERIAL1: Mutex<(SerialPort, TermStyle)> = {
        let mut serial_port = unsafe { SerialPort::new(SERIAL_PORT_ID) };
        serial_port.init();
        Mutex::new((serial_port, TermStyle::default()))
    };
}

#[doc(hidden)]
pub fn _serial_print(args: ::core::fmt::Arguments) {
    interrupts::without_interrupts(|| {
        SERIAL1.lock().0.write_fmt(args).unwrap();
    });
}

#[doc(hidden)]
#[cfg(not(test))]
pub fn _serial_print_with_style(style: &TermStyle, args: ::core::fmt::Arguments) {
    interrupts::without_interrupts(|| {
        let mut guard = SERIAL1.lock();
        let old_style = guard.1;
        _serial_print_style(&mut guard.0, style);
        guard.0.write_fmt(args).unwrap();
        _serial_print_style(&mut guard.0, &old_style);
    });
}
#[doc(hidden)]
#[cfg(test)]
pub fn _serial_print_with_style(_: &TermStyle, _: ::core::fmt::Arguments) {}

#[doc(hidden)]
pub fn _serial_serial_set_style(style: &TermStyle) {
    interrupts::without_interrupts(|| {
        let mut guard = SERIAL1.lock();
        _serial_print_style(&mut guard.0, style);
        guard.1 = *style;
    });
}

#[doc(hidden)]
pub fn _serial_reset_style() {
    interrupts::without_interrupts(|| {
        let mut guard = SERIAL1.lock();
        _serial_print_reset_style(&mut guard.0);
        guard.1 = TermStyle::RESET;
    });
}

#[doc(hidden)]
fn _serial_print_style(serial: &mut SerialPort, style: &TermStyle) {
    #![allow(unused_must_use)]
    serial.send(0x1b);
    serial.send(b'[');
    if let Some(mode) = style.mode {
        serial.write_fmt(format_args!("{}", mode.as_u8()));
        if style.fg.is_some() || style.bg.is_some() {
            serial.send(b';');
        }
    }
    if let Some(colour) = style.fg {
        serial.write_fmt(format_args!("{}", colour.as_fg()));
        if style.bg.is_some() {
            serial.send(b';');
        }
    }
    if let Some(colour) = style.bg {
        serial.write_fmt(format_args!("{}", colour.as_bg()));
    }
    serial.write_char('m');
}
#[doc(hidden)]
fn _serial_print_reset_style(serial: &mut SerialPort) {
    #![allow(unused_must_use)]
    serial.send(0x1b);
    serial.send(b'[');
    serial.send(b'0');
    serial.send(b'm');
}
