pub struct PPU {}

impl PPU {
    pub fn new() -> Self {
        Self {}
    }

    pub fn read(&self, address: u16) -> u8 {
        return 0;
    }

    pub fn write(&self, address: u16, value: u8) {}
}
