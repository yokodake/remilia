pub use x86_64::{PhysAddr,VirtAddr};
use core::ops::{Range, Add, Sub};


pub trait Addr {
    /// Returns smallest value <= addr, aligned on `align`
    fn align_down<T: Into<u64>>(self, align: T) -> Self where Self:Sized;

    /// Returns smallest value >= addr, aligned on `align`
    fn align_up<T: Into<u64>>(self, align: T) -> Self where Self:Sized;

    /// see `core::slice::align_to`, except that len isn't encoded in the Addr,
    /// therefore that should be computed by caller.
    /// size is therefore the size (in bytes) of the total range we want to split
    fn align_to<T: Into<u64>, U: Into<u64>>(self, size: T, align: U) -> (Self, Self, Self)
    where Self
        : Into<u64>
        + From<u64>
        {
        #[inline]
        fn into3<T : From<u64>>((x, y, z): (u64, u64, u64)) -> (T, T, T) {
            (T::from(x), T::from(y), T::from(z))
        }
        // FIXME: we can do this faster assuming power of 2.
        let size = size.into();
        let align = align.into();
        let start = self.into();

        if size == 0u64 {
            return into3((start, start, start))
        }

        let aligned = Self::from(start).align_up(align).into();
        let end = start + size;
        if aligned >= end {
            return into3((start, start+size, start+size))
        }

        let trail = (end - aligned) % align;

        into3((start, aligned, end - trail))
    }
}
pub trait AddrRange where Self : Sized {
    fn align_to<T: Into<u64>>(self, align: T) -> (Self, Self, Self);
}

impl<T> AddrRange for Range<T>
where T : Into<u64> + From<u64> + Add<T, Output = T > + Sub<T, Output = T> + Addr + Copy
    + Add<u64, Output = T> + Sub<u64, Output = T> {
    fn align_to<U: Into<u64>>(self, align: U) -> (Self, Self, Self) {
        let size = self.start - self.end;
        let (s, a, e) = self.start.align_to(size, align);
        (s .. (a-s), a .. (e-a), e .. self.end)
    }
}

impl Addr for u64 {
    /// Returns smallest value <= addr, aligned on `align`
    fn align_down<T: Into<u64>>(self, align: T) -> Self {
        let align = align.into();
        self & !(align - 1)
    }

    /// Returns smallest value >= addr, aligned on `align`
    fn align_up<T: Into<u64>>(self, align: T) -> Self {
        let align = align.into();
        (self + align - 1) & !(align - 1)
    }
}

impl<T> Addr for *const T  {
    fn align_down<U: Into<u64>>(self, align: U) -> Self {
        let align = align.into();
        ((self as u64) & !(align - 1)) as *const T
    }
    fn align_up<U: Into<u64>>(self , align: U) -> *const T {
        let align = align.into();
        ((self as u64 + align - 1) & !(align - 1)) as *const T
    }
}



#[cfg(test)]
mod test {
    use super::*;
    use rand::prelude::*;
    const NUM_TEST : usize = 100_000;

    fn get_random_align(rng : &mut impl RngCore) -> u64 {
        2u64.pow(rng.gen_range(1..8))
    }

    #[test]
    fn start_zero_multiple() {
        let addr = 0;
        let size : u64 = 32;
        let align : u64 = 2;
        assert_eq!((addr, addr, addr+size), addr.align_to(size, align));
        let size : u64 = 48;
        let align : u64 = 8;
        assert_eq!((addr, addr, addr+size), addr.align_to(size, align));
        for _ in 0..NUM_TEST {
            let align = get_random_align(&mut thread_rng());
            let size = (random::<u32>() as u64) * align;
            assert_eq!((addr, addr, addr+size), addr.align_to(size, align));
        }
    }
    #[test]
    fn start_offset_multiple() {
        let addr = 2;
        let size : u64 = 24;
        let align : u64 = 2;
        assert_eq!((addr, addr, addr+size), addr.align_to(size, align));
        let addr = 64;
        let size : u64 = 3131 * 16;
        let align : u64 = 16;
        assert_eq!((addr, addr, addr+size), addr.align_to(size, align));
        for _ in 0..NUM_TEST {
            let align = get_random_align(&mut thread_rng());
            let size = (random::<u32>() as u64) * align;
            let addr = (random::<u32>() as u64) * align;
            dbg!(addr);
            assert_eq!((addr, addr, addr+size), addr.align_to(size, align));
        }
    }
    #[test]
    fn smaller_size_aligned() {
        let addr = 8;
        let size : u64 = 4;
        let align: u64 = 8;
        assert_eq!((addr, addr, addr), addr.align_to(size, align));
        let addr = 128;
        let size : u64 = 16;
        let align: u64 = 32;
        assert_eq!((addr, addr, addr), addr.align_to(size, align));
        for _ in 0..NUM_TEST {
            let mut rng = thread_rng();
            let align = get_random_align(&mut rng);
            let size =  rng.gen_range(0 .. align);
            let addr = (random::<u32>() as u64) * align;
            dbg!(addr);
            assert_eq!((addr, addr, addr), addr.align_to(size, align));
        }
    }

    #[test]
    fn smaller_size_unaligned() {
        let addr = 2;
        let size : u64 = 4;
        let align: u64 = 8;
        assert_eq!((addr, addr+size, addr+size), addr.align_to(size, align));
        let addr = 110;
        let size : u64 = 16;
        let align: u64 = 32;
        assert_eq!((addr, addr+size, addr+size), addr.align_to(size, align));
        for _ in 0..NUM_TEST {
            let mut rng = thread_rng();
            let align = get_random_align(&mut rng);
            // size smaller than align
            let size =  rng.gen_range(0 .. align);
            // get an aligned address
            let addr = (rng.gen::<u32>() as u64) * align;
            // unalign it, but such that it doesn't straddle an aligned address
            // e.g. smaller_size_straddle()
            let r = if align - size < 2 { 1 } else { rng.gen_range(1 .. align - size) };
            let addr =  addr + r;
            dbg!(addr);
            assert_eq!((addr, addr+size, addr+size), addr.align_to(size, align));
        }
    }
    #[test]
    fn size_zero() {
        for _ in 0..NUM_TEST {
            let mut rng = thread_rng();
            let align: u64 = get_random_align(&mut rng);
            let size : u64 = 0;
            let addr = (random::<u32>() as u64) * align;
            dbg!(addr);
            assert_eq!((addr, addr, addr), addr.align_to(size, align));
        }
    }
    #[test]
    fn smaller_size_straddle() {
        let addr = 6;
        let size : u64 = 4;
        let align: u64 = 8;
        assert_eq!((addr, addr+2, addr+2), addr.align_to(size, align));
        let addr = 110; // 96 + 14
        let size : u64 = 25;
        let align: u64 = 32;
        let trail = 32 - 14;
        assert_eq!((addr , addr + trail, addr + trail), addr.align_to(size, align));
    }
    #[test]
    fn normal() {

    }
}
