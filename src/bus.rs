pub const RAM_ADDRESS_LO: u16 = 0x0000;
pub const RAM_ADDRESS_HI: u16 = 0x077F;

pub trait BusDevice {
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, value: u8);
}

pub struct Bus {
    ram: Box<dyn BusDevice>,
}

impl Bus {
    pub fn new(ram: Box<dyn BusDevice>) -> Bus {
        Self { 
            ram: ram
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            RAM_ADDRESS_LO..=RAM_ADDRESS_HI => {
                self.ram.read(address)
            }
            _ => 0
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        self.ram.write(address, value);
    }
}