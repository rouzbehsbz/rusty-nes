use std::cell::RefCell;

/*
 * Represents a memory device, which can be used as RAM,
 * ROM, or any other physical device requiring linear
 * memory allocation.
 */
pub struct Memory {
    cells: RefCell<Vec<u8>>,
}

impl Memory {
    /* Initialize new Memory */
    pub fn new(capacity: usize) -> Self {
        Self {
            cells: RefCell::new(vec![0; capacity]),
        }
    }

    /* Reading from specific address */
    pub fn read(&self, address: u16) -> u8 {
        self.cells.borrow()[address as usize]
    }

    /* Writing to a specific address */
    pub fn write(&self, address: u16, value: u8) {
        self.cells.borrow_mut()[address as usize] = value;
    }

    /* Writing vector of data starting from a specific address */
    pub fn write_chunk(&self, address: u16, value: &[u8]) {
        let start = address as usize;
        let end = start + value.len();

        self.cells.borrow_mut()[start..end].copy_from_slice(value);
    }
}
