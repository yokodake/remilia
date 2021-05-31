#![cfg_attr(not(test), no_std)]
pub mod ansi_term;
pub mod mem;

pub mod units;
pub use units::{GiB, KiB, MiB, TiB};

pub mod addr;

#[derive(Copy, Clone, Debug)]
struct Range<T> {
    pub start: T,
    pub end: T
}

use core::num::Wrapping;
impl<T> Range<T> where T: Wrapping<T> {
    pub fn new(start: T, end: T) {
        Range { start, end }
    }
}
