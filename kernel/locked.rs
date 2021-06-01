use spin::{Mutex, MutexGuard};

#[repr(transparent)]
pub struct Locked<T>(Mutex<T>);

impl<T> Locked<T> {
    pub const fn new(inner: T) -> Self {
        Locked(Mutex::new(inner))
    }
    pub fn lock(&self) -> MutexGuard<'_, T> {
        self.0.lock()
    }
}
