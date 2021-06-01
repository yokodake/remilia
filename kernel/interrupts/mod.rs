pub mod idt;
pub mod pic;

pub use idt::init_idt;
pub use pic::{init_pic, PICS};
