pub struct Mapper {
    prg_banks: u8,
    chr_banks: u8,
}

impl Mapper {
    pub fn new(prg_banks: u8, chr_banks: u8) -> Self {
        Self {
            prg_banks,
            chr_banks,
        }
    }

    pub fn get_prg_address(&self, address: u16) -> u16 {
        address & if self.prg_banks > 1 { 0x7FFF } else { 0x3FFF }
    }

    pub fn get_chr_address(&self, address: u16) -> u16 {
        address
    }
}
