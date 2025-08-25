use crate::bus::BusDevice;

pub const PROGRAM_ROM_ADDRESS: u16 = 0x8000;
pub const STACK_POINTER_ADDRESS: u16 = 0x0100;

pub struct RAM<const SIZE: usize> {
    memory: [u8; SIZE],
}

impl<const SIZE: usize> RAM<SIZE> {
    pub fn new() -> Self {
        Self { memory: [0; SIZE] }
    }

    pub fn get_index(&self, address: u16) -> usize {
        (address as usize) % SIZE
    }
}

impl<const SIZE: usize> BusDevice for RAM<SIZE> {
    fn read(&self, address: u16) -> u8 {
        self.memory[self.get_index(address)]
    }

    fn write(&mut self, address: u16, value: u8) {
        self.memory[self.get_index(address)] = value
    }
}
