use std::rc::Rc;

use crate::{cartridge::cartridge::Cartridge, memory::memory::Memory, ppu::ppu::PPU};

/* Hard-wired memory address boundaries for all physical
 * devices accessible by the CPU.
 */
pub const RAM_ADDRESS_LO: u16 = 0x0000;
pub const RAM_ADDRESS_HI: u16 = 0x1FFF;
pub const PPU_REGISTERS_ADDRESS_LO: u16 = 0x2000;
pub const PPU_REGISTERS_ADDRESS_HI: u16 = 0x3FFF;
pub const CARTRIDGE_PRG_ADDRESS_LO: u16 = 0x8000;
pub const CARTRIDGE_PRG_ADDRESS_HI: u16 = 0xFFFF;

/* Memory regions located in the cartridge CHR ROM,
 * used mainly for booting the game, resetting,
 * or servicing interrupt requests.
 */
pub const IRQ_VECTOR_ADDRESS_LO: u16 = 0xFFFE;
pub const IRQ_VECTOR_ADDRESS_HI: u16 = 0xFFFF;
pub const NMI_VECTOR_ADDRESS_LO: u16 = 0xFFFA;
pub const NMI_VECTOR_ADDRESS_HI: u16 = 0xFFFB;
pub const RESET_VECTOR_ADDRESS_LO: u16 = 0xFFFC;
pub const RESET_VECTOR_ADDRESS_HI: u16 = 0xFFFD;

/*
 * Represents the main communication component that allows
 * the CPU to interact with other hardware devices such as
 * RAM, PPU, or the Cartridge.
 *
 * The BUS simply routes a requested memory address to the
 * corresponding device, if the address falls within
 * that device's address boundaries.
 */
pub struct CpuBus {
    ram: Memory,
    ppu: PPU,
    cartridge: Rc<Cartridge>,
}

impl CpuBus {
    /* Initializing a new CPU BUS */
    pub fn new(ram: Memory, ppu: PPU, cartridge: Rc<Cartridge>) -> Self {
        Self {
            ram,
            ppu,
            cartridge,
        }
    }

    /* Reading from specific address */
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

    /* Writing to a specific address */
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

    /*
     * The NES uses only 2KB of its total 8KB RAM, so all memory locations
     * must be mirrored within first 2KB
     */
    fn get_mirrored_ram_address(&self, address: u16) -> u16 {
        address & 0x07FF
    }

    /*
     * These addresses correspond to the PPU I/O ports.
     * the PPU only uses 8 registers, so all addresses are mirrored
     * to the first 8 bytes.
     */
    fn get_mirrored_ppu_address(&self, address: u16) -> u16 {
        address & 0x0007
    }
}
