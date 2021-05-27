
pub fn align_up<T>(addr: *const T, align: usize) -> *const T {
    ((addr as usize + align - 1) & !(align - 1)) as *const T
}