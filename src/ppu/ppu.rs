use crate::bus::ppu_bus::PpuBus;

pub struct PPU {
    bus: PpuBus,
}

impl PPU {
    pub fn new(bus: PpuBus) -> Self {
        Self { bus }
    }

    pub fn read(&self, address: u16) -> u8 {
        return 0;
    }

    pub fn write(&self, address: u16, value: u8) {}
}
