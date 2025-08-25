use bitflags::bitflags;

use crate::{bus::Bus, errors::{AppError, AppResult}, instructions::{AddressingMode, Instruction, Opcode, OpcodeOperand, RawOpcode}};

const ENTRY_POINT_ADDRESS: u16 = 0x8000;

pub enum OpcodeInput {
    Empty,
    Value(u8),
    Address(u16)
}

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
    status: Status,

    bus: Bus,

    cycles: u8
}

impl CPU {
    pub fn new(bus: Bus) -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            pc: 0,
            status: Status::new(),
            bus: bus,
            cycles: 0,
        }
    }

    pub fn clock(&mut self) -> AppResult<()> {
        if self.cycles == 0 {
            let byte = self.bus.read(self.pc);
            self.pc = self.pc.wrapping_add(1);

            match Opcode::decode(byte) {
                Some(opcode) => {
                    self.cycles = opcode.cycles;
                },
                None => return Err(AppError::InvalidOpcode)
            }
        }

        self.cycles -= 1;
        Ok(())
    }

    pub fn get_next_opcode(&mut self) -> AppResult<(Instruction, OpcodeOperand)> {
        let byte = self.bus.read(self.pc);

        match RawOpcode::decode(byte) {
            Some(raw_opcode, ) => {
                let extra_bytes_needed = raw_opcode.bytes - 1;
                let mut operand_bytes = Vec::with_capacity(extra_bytes_needed as usize);

                for i in 1..=extra_bytes_needed {
                    let byte = self.bus.read(self.pc.wrapping_add(i as u16));
                    operand_bytes.push(byte)
                }

                let first_operand = operand_bytes[0];
                let second_operand = operand_bytes[1];

                let opcode_operand = match raw_opcode.addressing_mode {
                    AddressingMode::Accumulator | AddressingMode::Implied => OpcodeOperand::Implied(),
                    AddressingMode::Immediate => OpcodeOperand::Immediate(first_operand),
                    AddressingMode::ZeroPage => OpcodeOperand::Address(first_operand as u16),
                    AddressingMode::ZeroPageX => OpcodeOperand::Address(first_operand.wrapping_add(self.x) as u16),
                    AddressingMode::ZeroPageY => OpcodeOperand::Address(first_operand.wrapping_add(self.y) as u16),
                    AddressingMode::Relative => {
                        let sign = if first_operand & 0x80 == 0 {0x00u8} else {0xffu8};
                        let relative_address = u16::from_le_bytes([first_operand, sign]);

                        OpcodeOperand::Relative(relative_address)
                    },
                    AddressingMode::Absolute => {
                        let address = (second_operand as u16) << 8 | first_operand as u16;

                        OpcodeOperand::Address(address)
                    },
                    AddressingMode::AbsoluteX => {
                        let address =  (second_operand as u16) << 8 | first_operand as u16;

                        OpcodeOperand::Address(address.wrapping_add(self.x as u16))
                    },
                    AddressingMode::AbsoluteY => {
                        let address =  (second_operand as u16) << 8 | first_operand as u16;

                        OpcodeOperand::Address(address.wrapping_add(self.y as u16))
                    },
                    AddressingMode::Indirect => {
                        let ptr = (second_operand as u16) << 8 | first_operand as u16;
                        let lo = self.bus.read(ptr) as u16;
                        let hi = self.bus.read(ptr & 0xFF00 | (ptr + 1) & 0x00FF) as u16;

                        OpcodeOperand::Address((hi << 8) | lo)
                    },
                    AddressingMode::IndirectX => {
                        let ptr = first_operand.wrapping_add(self.x) as u16;
                        let lo = self.bus.read(ptr) as u16;
                        let hi = self.bus.read(ptr.wrapping_add(1)) as u16;

                        OpcodeOperand::Address((hi << 8) | lo)
                    },
                    AddressingMode::IndirectY => {
                        let lo = self.bus.read(first_operand as u16) as u16;
                        let hi = self.bus.read(first_operand.wrapping_add(1) as u16) as u16;
                        let base = (hi << 8) | lo;

                        OpcodeOperand::Address(base.wrapping_add(self.y as u16))
                    }
                };

                self.pc = self.pc.wrapping_add(raw_opcode.bytes as u16);

                Ok((raw_opcode.instruction, opcode_operand))
            },
            None => Err(AppError::InvalidOpcode)
        }
    }

    pub fn execute_opcode(&mut self, instruction: Instruction, opcode_operand: OpcodeOperand) {
        todo!()
    }

    pub fn run(&mut self) -> AppResult<()> {
        loop {
            let (instruction, opcode_operand) = self.get_next_opcode()?;

            self.execute_opcode(instruction, opcode_operand);
        }
    }
}