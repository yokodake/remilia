use pc_keyboard::{layouts::Us104Key, ScancodeSet1, DecodedKey};
use spin::Mutex;
use x86_64::instructions::port::Port;
use lazy_static::lazy_static;

use crate::vgaprint;

const KB_PORT : u16 = 0x60;

lazy_static! {
    pub static ref KEYBOARD_DEVICE : KeyboardDevice = 
        KeyboardDevice::new(KB_PORT);
}

pub struct KeyboardDevice {
    handler: Mutex<pc_keyboard::Keyboard<Us104Key, ScancodeSet1>>,
    port: Mutex<Port<u8>>
}

impl KeyboardDevice {
    pub fn new(port_id: u16) -> KeyboardDevice {
        use pc_keyboard::{Keyboard, HandleControl};
        KeyboardDevice {
            handler: Mutex::new(Keyboard::new(Us104Key, ScancodeSet1, HandleControl::Ignore)),
            port : Mutex::new(Port::new(port_id))
        }
    }
    pub fn handle_irq(&self) {
        let mut handler = KEYBOARD_DEVICE.handler.lock();
        let mut port = KEYBOARD_DEVICE.port.lock();

        let scancode: u8 = unsafe { port.read() };
        if let Ok(Some(k_ev)) = handler.add_byte(scancode) {
            if let Some(key) = handler.process_keyevent(k_ev) {
                match key {
                    DecodedKey::RawKey(_) => {},
                    DecodedKey::Unicode(ch) => vgaprint!("{}", ch)
                }
            }

        }
    }
}
