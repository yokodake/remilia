#![cfg_attr(not(test), no_std)]
pub mod ansi_term;
pub mod mem;

pub mod units;
pub use units::{GiB, KiB, MiB, TiB};

pub mod addr;

use core::fmt;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Default)]
pub struct Range<T> {
    pub start: T,
    pub end: T,
}

impl<T> Range<T> {
    pub fn new(start: T, end: T) -> Self {
        Range { start, end }
    }
}

impl<T: PartialOrd<T>> Range<T> {
    pub fn contains(&self, i: T) -> bool {
        self.start <= i && i < self.end
    }
}

impl<T: fmt::Debug> fmt::Debug for Range<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "(");
        self.start.fmt(fmt)?;
        write!(fmt, " .. ")?;
        self.end.fmt(fmt)?;
        write!(fmt, ")")
    }
}

use x86_64::{PhysAddr, VirtAddr};
impl Range<PhysAddr> {
    pub fn len(&self) -> u64 {
        self.end.as_u64() - self.start.as_u64()
    }
}
impl Range<VirtAddr> {
    pub fn len(&self) -> u64 {
        self.end.as_u64() - self.start.as_u64()
    }
}

macro_rules! impl_num {
    { $($ty:ty)* } => {
        $(impl Range<$ty> {
            pub fn from(start: $ty) -> Self {
                Range { start, end: <$ty>::MAX }
            }
            pub fn to(end: $ty) -> Self {
                Range { end, start: <$ty>::MIN }
            }
            pub fn len(&self) -> $ty {
                self.end - self.start
            }
        })*
    }
}

impl_num! { u64 u32 }
