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

use sakuya::TermColour;

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
macro_rules! info {
    ($($arg:tt)*) => {
        $crate::println!($($arg)*);
    }
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        $crate::println!($($arg)*);
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        // TODO change color
        $crate::println!($($arg)*);
    };
}
