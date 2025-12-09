use crate::{
    cartridge::mapper::Mapper,
    errors::{AppError, AppResult},
    memory::Memory,
};
use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy)]
    struct MapperFirstFlags: u8 {
        const MIRRORING_VERTICAL     = 0b0000_0001;
        const BATTERY_BACKED_RAM     = 0b0000_0010;
        const TRAINER_PRESENT        = 0b0000_0100;
        const FOUR_SCREEN_VRAM       = 0b0000_1000;
        const LOWER_MAPPER_BITS_MASK = 0b1111_0000;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    struct MapperSecondFlags: u8 {
        const UPPER_MAPPER_BITS_MASK = 0b1111_0000;
        const VS_UNISYSTEM           = 0b0000_0001;
        const PLAYCHOICE_10          = 0b0000_0010;
        const NES2_0_INDICATOR       = 0b0000_1100;
    }
}

struct Header {
    pub prg_banks: u8,
    pub chr_banks: u8,
    pub first_mapper_flags: MapperFirstFlags,
    pub second_mapper_flags: MapperSecondFlags,
}

impl Header {
    fn new(bytes: &[u8]) -> AppResult<Self> {
        if bytes.len() < 16 {
            return Err(AppError::InvalidCartridgeHeaderSize);
        }

        if &bytes[0..4] != b"NES\x1A" {
            return Err(AppError::InvalidNesFile);
        }

        let prg_banks = bytes[4];
        let chr_banks = bytes[5];

        let first_mapper_flags = MapperFirstFlags::from_bits_truncate(bytes[6]);
        let second_mapper_flags = MapperSecondFlags::from_bits_truncate(bytes[7]);

        Ok(Self {
            prg_banks,
            chr_banks,
            first_mapper_flags,
            second_mapper_flags,
        })
    }

    fn get_mapper_id(&self) -> u8 {
        let lower =
            (self.first_mapper_flags.bits() & MapperFirstFlags::LOWER_MAPPER_BITS_MASK.bits()) >> 4;
        let upper =
            self.second_mapper_flags.bits() & MapperSecondFlags::UPPER_MAPPER_BITS_MASK.bits();
        upper | lower
    }
}

pub struct Cartridge {
    header: Header,

    prg_ram: Memory,
    chr_rom: Memory,
    mapper: Mapper,
}

impl Cartridge {
    pub fn new(bytes: &[u8]) -> AppResult<Self> {
        let header = Header::new(bytes)?;

        if header.get_mapper_id() != 0 {
            return Err(AppError::InvalidCartridgeMapper);
        }

        let mut offset = 528;

        let prg_memory_size = header.prg_banks as usize * 16384;
        let chr_memory_size = header.chr_banks as usize * 8192;

        let prg_ram = Memory::new(prg_memory_size);
        let chr_rom = Memory::new(chr_memory_size);

        prg_ram.write_chunk(0, &bytes[offset..offset + prg_memory_size]);
        offset += prg_memory_size;
        chr_rom.write_chunk(0, &bytes[offset..offset + chr_memory_size]);

        let mapper = Mapper::new(header.prg_banks, header.chr_banks);

        Ok(Self {
            header,
            prg_ram,
            chr_rom,
            mapper,
        })
    }

    pub fn prg_read(&self, address: u16) -> u8 {
        let mapped_address = self.mapper.get_prg_address(address);

        self.prg_ram.read(mapped_address)
    }

    pub fn prg_write(&self, address: u16, value: u8) {
        let mapped_address = self.mapper.get_prg_address(address);

        self.prg_ram.write(mapped_address, value);
    }

    pub fn chr_read(&self, address: u16) -> u8 {
        let mapped_address = self.mapper.get_chr_address(address);

        self.chr_rom.read(mapped_address)
    }
}
