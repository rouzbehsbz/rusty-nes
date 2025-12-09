use crate::cartridge::cartridge::Cartridge;
use std::rc::Rc;

pub const CARTRIDGE_CHR_ADDRESS_LO: u16 = 0x0000;
pub const CARTRIDGE_CHR_ADDRESS_HI: u16 = 0x1FFF;

pub struct PpuBus {
    cartridge: Rc<Cartridge>,
}

impl PpuBus {
    pub fn new(cartridge: Rc<Cartridge>) -> Self {
        Self { cartridge }
    }

    fn read(&self, address: u16) -> u8 {
        match address {
            CARTRIDGE_CHR_ADDRESS_LO..=CARTRIDGE_CHR_ADDRESS_HI => self.cartridge.chr_read(address),
            _ => 0,
        }
    }

    fn write(&self, address: u16, value: u8) {}
}
