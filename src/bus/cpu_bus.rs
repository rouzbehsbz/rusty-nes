use crate::{cartridge::cartridge::Cartridge, memory::memory::Memory, ppu::ppu::PPU};

pub const RAM_ADDRESS_LO: u16 = 0x0000;
pub const RAM_ADDRESS_HI: u16 = 0x1FFF;
pub const PPU_REGISTERS_ADDRESS_LO: u16 = 0x2000;
pub const PPU_REGISTERS_ADDRESS_HI: u16 = 0x3FFF;
pub const CARTRIDGE_PRG_ADDRESS_LO: u16 = 0x8000;
pub const CARTRIDGE_PRG_ADDRESS_HI: u16 = 0xFFFF;

pub const IRQ_VECTOR_ADDRESS_LO: u16 = 0xFFFE;
pub const IRQ_VECTOR_ADDRESS_HI: u16 = 0xFFFF;
pub const NMI_VECTOR_ADDRESS_LO: u16 = 0xFFFA;
pub const NMI_VECTOR_ADDRESS_HI: u16 = 0xFFFB;
pub const RESET_VECTOR_ADDRESS_LO: u16 = 0xFFFC;
pub const RESET_VECTOR_ADDRESS_HI: u16 = 0xFFFD;

pub struct CpuBus {
    ram: Memory,
    ppu: PPU,
    cartridge: Cartridge,
}

impl CpuBus {
    pub fn new(ram: Memory, ppu: PPU, cartridge: Cartridge) -> Self {
        Self {
            ram,
            ppu,
            cartridge,
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            RAM_ADDRESS_LO..=RAM_ADDRESS_HI => {
                self.ram.read(self.get_mirrored_ram_address(address))
            }
            PPU_REGISTERS_ADDRESS_LO..=PPU_REGISTERS_ADDRESS_HI => {
                self.ppu.read(self.get_mirrored_ppu_address(address))
            }
            CARTRIDGE_PRG_ADDRESS_LO..=CARTRIDGE_PRG_ADDRESS_HI => self.cartridge.prg_read(address),
            _ => 0,
        }
    }

    pub fn write(&self, address: u16, value: u8) {
        match address {
            RAM_ADDRESS_LO..=RAM_ADDRESS_HI => self
                .ram
                .write(self.get_mirrored_ram_address(address), value),
            PPU_REGISTERS_ADDRESS_LO..=PPU_REGISTERS_ADDRESS_HI => {
                self.ppu
                    .write(self.get_mirrored_ppu_address(address), value);
            }
            CARTRIDGE_PRG_ADDRESS_LO..=CARTRIDGE_PRG_ADDRESS_HI => {
                self.cartridge.prg_write(address, value)
            }
            _ => {}
        }
    }

    fn get_mirrored_ram_address(&self, address: u16) -> u16 {
        address & 0x07FF
    }

    fn get_mirrored_ppu_address(&self, address: u16) -> u16 {
        address & 0x0007
    }
}
