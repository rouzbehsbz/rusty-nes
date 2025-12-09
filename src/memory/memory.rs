use std::cell::RefCell;

pub struct Memory {
    cells: RefCell<Vec<u8>>,
}

impl Memory {
    pub fn new(capacity: usize) -> Self {
        Self {
            cells: RefCell::new(vec![0; capacity]),
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        self.cells.borrow()[address as usize]
    }

    pub fn write(&self, address: u16, value: u8) {
        self.cells.borrow_mut()[address as usize] = value;
    }

    pub fn write_chunk(&self, address: u16, value: &[u8]) {
        let start = address as usize;
        let end = start + value.len();

        self.cells.borrow_mut()[start..end].copy_from_slice(value);
    }
}
