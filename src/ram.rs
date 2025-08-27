use crate::bus::BusDevice;

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

    pub fn load_program(&mut self, program: &[u8], start_address: u16) {
        let start_index = self.get_index(start_address);
        let end_index = start_index + program.len();
        self.memory[start_index..end_index].copy_from_slice(program);
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
