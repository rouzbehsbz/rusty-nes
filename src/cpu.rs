use bitflags::bitflags;

use crate::{
    bus::Bus,
    errors::{AppError, AppResult},
    instructions::{AddressingMode, Instruction, Opcode},
};

bitflags! {
    pub struct Status: u8 {
        const CARRY = 0b0000_0001;
        const ZERO = 0b0000_0010;
        const INTERRUPT = 0b0000_0100;
        const DECIMAL = 0b0000_1000;
        const BREAK = 0b0001_0000;
        const UNUSED = 0b0010_0000;
        const OVERFLOW = 0b0100_0000;
        const NEGATIVE = 0b1000_0000;
    }
}

impl Status {
    pub fn new() -> Self {
        Status::UNUSED | Status::INTERRUPT
    }
}

pub struct CPU {
    a: u8,
    x: u8,
    y: u8,
    pc: u16,
    sp: u8,
    status: Status,

    bus: Bus,

    cycles: u8,
    absolute_address: u16,
    relative_address: i16,
}

impl CPU {
    pub fn new(bus: Bus, pc: u16) -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            pc: pc,
            sp: 0,
            status: Status::new(),
            bus: bus,
            cycles: 0,
            absolute_address: 0,
            relative_address: 0,
        }
    }

    pub fn increment_pc(&mut self) {
        self.pc = self.pc.wrapping_add(1);
    }

    pub fn set_status_flag(&mut self, flag: Status, condition: bool) {
        if condition {
            self.status.insert(flag);
        } else {
            self.status.remove(flag);
        }
    }

    pub fn get_status_flag(&self, flag: Status) -> bool {
        self.status.contains(flag)
    }

    pub fn clock(&mut self) -> AppResult<()> {
        if self.cycles == 0 {
            let byte = self.bus.read(self.pc);
            self.increment_pc();

            match Opcode::decode(byte) {
                Some(opcode) => {
                    self.cycles = opcode.cycles;

                    self.execute_addressing_mode(opcode.addressing_mode);
                    self.execute_instruction(opcode.instruction);
                }
                None => return Err(AppError::InvalidOpcode),
            }
        }

        self.cycles -= 1;
        Ok(())
    }

    pub fn execute_addressing_mode(&mut self, addressing_mode: AddressingMode) {
        match addressing_mode {
            AddressingMode::Implied => {}
            AddressingMode::Accumulator => {}
            AddressingMode::Immediate => {
                self.absolute_address = self.pc;
                self.increment_pc();
            }
            AddressingMode::Relative => {
                let offset = self.bus.read(self.pc) as i8;
                self.increment_pc();

                self.relative_address = offset as i16;
            }
            AddressingMode::ZeroPage => {
                self.absolute_address = self.bus.read(self.pc) as u16;
                self.increment_pc();
            }
            AddressingMode::ZeroPageX => {
                self.absolute_address = self.bus.read(self.pc).wrapping_add(self.x) as u16;
                self.increment_pc();
            }
            AddressingMode::ZeroPageY => {
                self.absolute_address = self.bus.read(self.pc).wrapping_add(self.y) as u16;
                self.increment_pc();
            }
            AddressingMode::Absolute => {
                let lo = self.bus.read(self.pc) as u16;
                self.increment_pc();
                let hi = self.bus.read(self.pc) as u16;
                self.increment_pc();

                self.absolute_address = (hi << 8) | lo;
            }
            AddressingMode::AbsoluteX => {
                let lo = self.bus.read(self.pc) as u16;
                self.increment_pc();
                let hi = self.bus.read(self.pc) as u16;
                self.increment_pc();

                //TODO: need additional clock cycle

                self.absolute_address = ((hi << 8) | lo).wrapping_add(self.x as u16)
            }
            AddressingMode::AbsoluteY => {
                let lo = self.bus.read(self.pc) as u16;
                self.increment_pc();
                let hi = self.bus.read(self.pc) as u16;
                self.increment_pc();

                //TODO: need additional clock cycle

                self.absolute_address = ((hi << 8) | lo).wrapping_add(self.y as u16)
            }
            AddressingMode::Indirect => {
                let ptr_lo = self.bus.read(self.pc) as u16;
                self.increment_pc();
                let ptr_hi = self.bus.read(self.pc) as u16;
                self.increment_pc();

                let ptr = (ptr_hi << 8) | ptr_lo;

                let lo = self.bus.read(ptr) as u16;
                let hi = self.bus.read(ptr.wrapping_add(1)) as u16;

                self.absolute_address = (hi << 8) | lo
            }
            AddressingMode::IndirectX => {
                let base = self.bus.read(self.pc);
                self.increment_pc();

                let ptr = base.wrapping_add(self.x) as u16;

                let lo = self.bus.read(ptr & 0x00FF) as u16;
                let hi = self.bus.read((ptr.wrapping_add(1)) & 0x00FF) as u16;

                self.absolute_address = (hi << 8) | lo;
            }
            AddressingMode::IndirectY => {
                let base = self.bus.read(self.pc);
                self.increment_pc();

                let lo = self.bus.read(base as u16) as u16;
                let hi = self.bus.read((base.wrapping_add(1)) as u16 & 0x00FF) as u16;

                let ptr = (hi << 8) | lo;

                self.absolute_address = ptr.wrapping_add(self.y as u16);
            }
        }
    }

    pub fn execute_instruction(&self, instruction: Instruction) {
        match instruction {
            Instruction::AND => {
                let value = self.bus.read(self.absolute_address);

                self.a &= value;

                self.set_status_flag(Status::ZERO, self.is_zero(self.a));
                self.set_status_flag(Status::NEGATIVE, self.is_negative(self.a));

                //TODO: could potentially need additional clock cycle
            }
            Instruction::BCS => {
                //TODO: could potentially need additional clock cycle

                if self.get_status_flag(Status::CARRY) {
                    self.absolute_address = self.pc.wrapping_add(self.relative_address as u16);
                    self.pc = self.absolute_address;
                }
            }
            Instruction::BCC => {
                //TODO: could potentially need additional clock cycle

                if !self.get_status_flag(Status::CARRY) {
                    self.absolute_address = self.pc.wrapping_add(self.relative_address as u16);
                    self.pc = self.absolute_address;
                }
            }
            Instruction::BEQ => {
                //TODO: could potentially need additional clock cycle

                if self.get_status_flag(Status::ZERO) {
                    self.absolute_address = self.pc.wrapping_add(self.relative_address as u16);
                    self.pc = self.absolute_address;
                }
            }
            Instruction::BMI => {
                //TODO: could potentially need additional clock cycle

                if self.get_status_flag(Status::NEGATIVE) {
                    self.absolute_address = self.pc.wrapping_add(self.relative_address as u16);
                    self.pc = self.absolute_address;
                }
            }
            Instruction::BNE => {
                //TODO: could potentially need additional clock cycle

                if !self.get_status_flag(Status::ZERO) {
                    self.absolute_address = self.pc.wrapping_add(self.relative_address as u16);
                    self.pc = self.absolute_address;
                }
            }
            Instruction::BPL => {
                //TODO: could potentially need additional clock cycle

                if !self.get_status_flag(Status::NEGATIVE) {
                    self.absolute_address = self.pc.wrapping_add(self.relative_address as u16);
                    self.pc = self.absolute_address;
                }
            }
            Instruction::BVC => {
                //TODO: could potentially need additional clock cycle

                if !self.get_status_flag(Status::OVERFLOW) {
                    self.absolute_address = self.pc.wrapping_add(self.relative_address as u16);
                    self.pc = self.absolute_address;
                }
            }
            Instruction::BVS => {
                //TODO: could potentially need additional clock cycle

                if self.get_status_flag(Status::OVERFLOW) {
                    self.absolute_address = self.pc.wrapping_add(self.relative_address as u16);
                    self.pc = self.absolute_address;
                }
            }
            Instruction::CLC => {
                self.set_status_flag(Status::CARRY, false);
            }
            Instruction::CLD => {
                self.set_status_flag(Status::DECIMAL, false);
            }
        }
    }

    fn is_zero(&self, value: u8) -> bool {
        value == 0x00
    }

    fn is_negative(&self, value: u8) -> bool {
        value & 0x80 != 0
    }
}
