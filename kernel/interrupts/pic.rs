use core::u8;

use spin::Mutex;

pub static PICS: Mutex<DualPic> =
    Mutex::new(unsafe{ DualPic::new(MASTER_PIC_OFFSET, SLAVE_PIC_OFFSET)});

pub fn init_pic() {
    unsafe { PICS.lock().initialize() };
}
pub const MASTER_PIC_OFFSET : u8 = 32;
pub const SLAVE_PIC_OFFSET : u8 = MASTER_PIC_OFFSET + 8;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = MASTER_PIC_OFFSET,
}

impl InterruptIndex {
    #[inline]
    pub fn as_u8(self) -> u8 {
        self as u8
    }
    #[inline]
    pub fn as_usize(self) -> usize {
        self as usize
    }
}

use x86_64::instructions::port::Port;

const CMD_INIT : u8 = 0x11;
const CMD_EOI : u8 = 0x20;
const MODE_x86 : u8 = 0x01;
pub struct DualPic 
    { master : Pic
    , slave : Pic
    }
impl DualPic {
    pub const unsafe fn new(offset_master: u8, offset_slave :u8) -> DualPic {
        DualPic {
            master: Pic {
                command: Port::new(0x20),
                data: Port::new(0x21),
                offset: offset_master
            },
            slave: Pic {
                command: Port::new(0xa0),
                data: Port::new(0xa1),
                offset: offset_slave
            }
        }
    }
    pub unsafe fn initialize(&mut self) {
        let mut wait_port: Port<u8> = Port::new(0x80);
        let mut wait = || wait_port.write(0);

        let saved_masks = self.read_masks();
        // prepare pics for 3byte init sequence
        self.master.command.write(CMD_INIT);
        wait();
        self.slave.command.write(CMD_INIT);
        wait();

        // set up offsets
        self.master.data.write(self.master.offset);
        wait();
        self.slave.data.write(self.slave.offset);
        wait();

        // configure chaining between master and slave
        self.master.data.write(4);
        wait();
        self.slave.data.write(2);
        wait();

        // set x86 mode
        self.master.data.write(MODE_x86);
        wait();
        self.slave.data.write(MODE_x86);
        wait();

        // restore masks
        self.write_masks(saved_masks[0], saved_masks[1]);

    }

    pub unsafe fn read_masks(&mut self) -> [u8; 2] {
        [ self.master.read_mask()
        , self.slave.read_mask()
        ]
    }
    pub unsafe fn write_masks(&mut self, master_m: u8, slave_m: u8) {
        self.master.write_mask(master_m);
        self.slave.write_mask(slave_m);
    }
    pub fn handles_interrupt(&self, interrupt_id: u8) -> bool {
        self.master.handles_interrupt(interrupt_id)
        || self.slave.handles_interrupt(interrupt_id)
    }
    pub unsafe fn notify_eoi(&mut self, interrupt_id: u8) {
        if self.master.handles_interrupt(interrupt_id) {
            self.master.eoi();
        } else if self.slave.handles_interrupt(interrupt_id) {
            self.slave.eoi();
        }
    }
}
struct Pic 
    { command: Port<u8>
    , data: Port<u8>
    , /// offset for interrupts mapping
      offset: u8 
    }

impl Pic {
    // send end of interrupt
    unsafe fn eoi(&mut self) {
        self.command.write(CMD_EOI);
    }
    unsafe fn read_mask(&mut self) -> u8 {
        self.data.read()
    }
    unsafe fn write_mask(&mut self, mask: u8) {
        self.data.write(mask)
    }
    fn handles_interrupt(&self, interrupt_id: u8) -> bool {
        self.offset <= interrupt_id && interrupt_id < self.offset + 8
    }
}