use pache::ansi_term::{TermColour, TermGMode, TermStyle};

// see: https://doc.rust-lang.org/src/std/macros.rs.html#285-307
#[macro_export]
macro_rules! dbg {

    // NOTE: We cannot use `concat!` to make a static string as a format argument
    // of `eprintln!` because `file!` could contain a `{` or
    // `$val` expression could be a block (`{ .. }`), in which case the `eprintln!`
    // will be malformed.
    () => {
        $crate::println!("[{}:{}]", file!(), line!());
    };
    ($val:expr $(,)?) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                $crate::println!("[{}:{}] {} = {:#?}",
                    file!(), line!(), stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::dbg!($val)),+,)
    };
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::devices::serial::_serial_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($fmt:expr) => ($crate::print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) =>
        ($crate::print!(concat!($fmt, "\n"), $($arg)*));
}

pub const INFO_STYLE: TermStyle = TermStyle::fg(TermColour::Cyan);
pub const WARN_STYLE: TermStyle = TermStyle::fg(TermColour::Yellow);
pub const ERROR_STYLE: TermStyle = TermStyle::fg(TermColour::Red).with_mode(TermGMode::Bold);

#[macro_export]
macro_rules! _println_style {
    ($style:expr) => ($crate::devices::serial::_serial_print_with_style(& $style, format_args_nl!()));
    ($style:expr, $($args:tt)*) =>
        ($crate::devices::serial::_serial_print_with_style(& $style, format_args_nl!($($args)*)));
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        $crate::_println_style!($crate::debug::INFO_STYLE, $($arg)*);
    }
}
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        $crate::_println_style!($crate::debug::WARN_STYLE, $($arg)*);
    };
}
#[macro_export]
macro_rules! error {
    () => {
        $crate::_println_style!($crate::debug::ERROR_STYLE, "[ERROR @ {}:{}]", file!(), line!());
    };
    ($($arg:tt)*) => {
        ($crate::_println_style!($crate::debug::ERROR_STYLE, $($arg)*));
    };
}
