#![allow(dead_code)]
use core::fmt;
use core::mem;
use lazy_static::lazy_static;
use spin::mutex::Mutex;
use volatile::Volatile;
use x86_64::instructions::interrupts;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Colour {
    Black = 0,
    Blue,
    Green,
    Cyan,
    Red,
    Magenta,
    Brown,
    LightGray,
    DarkGray,
    LightBlue,
    LightGreen,
    LightCyan,
    LightRed,
    Pink,
    Yellow,
    White,
}
impl Colour {
    #[inline]
    pub fn as_u8(self) -> u8 {
        self as u8
    }
    #[inline]
    pub fn from_u8(num: u8) -> Colour {
        if num > Colour::White.as_u8() {
            Colour::White
        } else {
            unsafe { mem::transmute::<u8, Colour>(num) }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColourCode(u8);

impl ColourCode {
    pub fn new(fg: Colour, bg: Colour) -> ColourCode {
        ColourCode(bg.as_u8() << 4 | fg.as_u8())
    }

    pub fn bg(&self) -> Colour {
        Colour::from_u8((self.0 & 0xf0u8) >> 4)
    }
    pub fn fg(&self) -> Colour {
        Colour::from_u8(self.0 & 0x0fu8)
    }
    pub fn with_fg(&self, fg: Colour) -> ColourCode {
        ColourCode((self.0 & 0xf0) | fg.as_u8())
    }
    pub fn with_bg(&self, bg: Colour) -> ColourCode {
        ColourCode((self.0 & 0x0f) | (bg.as_u8() << 4))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScChar {
    ascii_char: u8,
    colour_code: ColourCode,
}
impl ScChar {
    pub fn blank(colour_code: ColourCode) -> ScChar {
        ScChar {
            ascii_char: b' ',
            colour_code,
        }
    }
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
pub struct Buffer([[Volatile<ScChar>; BUFFER_WIDTH]; BUFFER_HEIGHT]);

pub struct Writer {
    pub col: usize,
    pub row: usize,
    pub colour_code: ColourCode,
    pub buffer: &'static mut Buffer,
}
lazy_static! {
    pub static ref VGA_WRITER: Mutex<Writer> = {
        let addr = 0xb8000;
        crate::info!("loading VGA buffer @ 0x{:x}", addr);
        Mutex::new(Writer {
            col: 0,
            row: 0,
            colour_code: ColourCode::new(Colour::Yellow, Colour::Black),
            buffer: unsafe { &mut *(addr as *mut Buffer) },
        })
    };
}

impl Writer {
    pub fn get_colour(&self) -> ColourCode {
        self.colour_code
    }
    pub fn set_colour(&mut self, colour_code: ColourCode) {
        self.colour_code = colour_code;
    }
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.col >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = self.row;
                let col = self.col;

                let colour_code = self.colour_code;
                self.buffer.0[row][col].write(ScChar {
                    ascii_char: byte,
                    colour_code,
                });
                self.col += 1;
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }

    fn new_line(&mut self) {
        self.row += 1;
        self.col = 0;

        if self.row < BUFFER_HEIGHT {
            return;
        }
        self.row = BUFFER_HEIGHT - 1;

        for r in 1..BUFFER_HEIGHT {
            for c in 0..BUFFER_WIDTH {
                self.buffer.0[r - 1][c].write(self.buffer.0[r][c].read());
            }
        }
        self.clear_row(self.row);
    }

    fn clear_row(&mut self, r: usize) {
        let blank = ScChar::blank(self.colour_code);
        for i in 0..BUFFER_WIDTH {
            self.buffer.0[r][i].write(blank);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! vgaprint {
    ($($arg:tt)*) => ($crate::devices::vga::_vgaprint(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! vgaprintln {
    () => ($crate::devices::vgaprint!("{}\n"));
    ($($arg:tt)*) => ($crate::vgaprint!("{}\n", format_args!($($arg)*)));
}
#[macro_export]
macro_rules! vgaeprintln {
    ($($arg:tt)*) => ({
        use $crate::devices::vga::{VGA_WRITER, Colour};
        let cc = VGA_WRITER.lock().get_colour();
        VGA_WRITER.lock().set_colour(cc.with_fg(Colour::Red));
        $crate::vgaprintln!($($arg)*);
        VGA_WRITER.lock().set_colour(cc);
    });
}

#[doc(hidden)]
pub fn _vgaprint(args: fmt::Arguments) {
    use core::fmt::Write;
    interrupts::without_interrupts(|| {
        VGA_WRITER.lock().write_fmt(args).unwrap();
    });
}
