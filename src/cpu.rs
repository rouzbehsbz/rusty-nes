use bitflags::bitflags;

use crate::{
    bus::Bus,
    errors::{AppError, AppResult},
    instructions::{AddressingMode, Instruction, Opcode},
};

pub const STACK_POINTER_INITIAL_OFFSET: u8 = 0xFD;
pub const PROGRAM_ROM_ADDRESS: u16 = 0x8000;
pub const STACK_POINTER_ADDRESS: u16 = 0x0100;

bitflags! {
    #[derive(Clone, Copy)]
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
    sp: u8,
    pc: u16,
    status: Status,

    bus: Bus,

    cycles: u8,
    absolute_address: u16,
    relative_address: i16,
}

impl CPU {
    pub fn new(bus: Bus) -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            sp: STACK_POINTER_INITIAL_OFFSET,
            pc: PROGRAM_ROM_ADDRESS,
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
                    println!("{:?}", opcode.instruction);
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

                self.absolute_address = ((hi << 8) | lo).wrapping_add(self.x as u16)
            }
            AddressingMode::AbsoluteY => {
                let lo = self.bus.read(self.pc) as u16;
                self.increment_pc();
                let hi = self.bus.read(self.pc) as u16;
                self.increment_pc();

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

    pub fn execute_instruction(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::NOP => {}
            Instruction::CLC => self.set_status_flag(Status::CARRY, false),
            Instruction::CLD => self.set_status_flag(Status::DECIMAL, false),
            Instruction::CLI => self.set_status_flag(Status::INTERRUPT, false),
            Instruction::CLV => self.set_status_flag(Status::OVERFLOW, false),
            Instruction::SEC => self.set_status_flag(Status::CARRY, true),
            Instruction::SED => self.set_status_flag(Status::DECIMAL, true),
            Instruction::SEI => self.set_status_flag(Status::INTERRUPT, true),
            Instruction::TAX => {
                self.x = self.a;
                self.update_zero_negative_flags(self.x)
            },
            Instruction::TAY => {
                self.y = self.a;
                self.update_zero_negative_flags(self.y)
            },
            Instruction::TXA => {
                self.a = self.x;
                self.update_zero_negative_flags(self.a)
            },
            Instruction::TYA => {
                self.a = self.y;
                self.update_zero_negative_flags(self.a)
            },
            Instruction::TSX => {
                self.x = self.sp;
                self.update_zero_negative_flags(self.x)
            },
            Instruction::TXS => {
                self.sp = self.x;
            },
            Instruction::INX => {
                self.x = self.x.wrapping_add(1);
                self.update_zero_negative_flags(self.x)
            }
            Instruction::INY => {
                self.y = self.y.wrapping_add(1);
                self.update_zero_negative_flags(self.y)
            },
            Instruction::DEX => {
                self.x = self.x.wrapping_sub(1);
                self.update_zero_negative_flags(self.x)
            },
            Instruction::DEY => {
                self.y = self.y.wrapping_sub(1);
                self.update_zero_negative_flags(self.y)
            },
            Instruction::LDA => {
                self.a = self.bus.read(self.absolute_address);
                self.update_zero_negative_flags(self.a);
            },
            Instruction::LDX => {
                self.x = self.bus.read(self.absolute_address);
                self.update_zero_negative_flags(self.x);
            }
            Instruction::LDY => {
                self.y = self.bus.read(self.absolute_address);
                self.update_zero_negative_flags(self.y);
            }
            Instruction::STA => {
                self.bus.write(self.absolute_address, self.a);
            },
            Instruction::STX => {
                self.bus.write(self.absolute_address, self.x);
            }
            Instruction::STY => {
                self.bus.write(self.absolute_address, self.y);
            }
            Instruction::AND => {
                let value = self.bus.read(self.absolute_address);
                self.a &= value;
                self.update_zero_negative_flags(self.a)
            },
            Instruction::ORA => {
                let value = self.bus.read(self.absolute_address);
                self.a |= value;
                self.update_zero_negative_flags(self.a)
            },
            Instruction::EOR => {
                let value = self.bus.read(self.absolute_address);
                self.a ^= value;
                self.update_zero_negative_flags(self.a);
            }
            Instruction::CMP => {
                
            }
            Instruction::CPX => {

            }
            Instruction::CPY => {

            }
            Instruction::BCS => {
                if self.get_status_flag(Status::CARRY) {
                    self.absolute_address = self.pc.wrapping_add(self.relative_address as u16);
                    self.pc = self.absolute_address;
                }
            }
            Instruction::BCC => {
                if !self.get_status_flag(Status::CARRY) {
                    self.absolute_address = self.pc.wrapping_add(self.relative_address as u16);
                    self.pc = self.absolute_address;
                }
            }
            Instruction::BEQ => {
                if self.get_status_flag(Status::ZERO) {
                    self.absolute_address = self.pc.wrapping_add(self.relative_address as u16);
                    self.pc = self.absolute_address;
                }
            }
            Instruction::BMI => {
                if self.get_status_flag(Status::NEGATIVE) {
                    self.absolute_address = self.pc.wrapping_add(self.relative_address as u16);
                    self.pc = self.absolute_address;
                }
            }
            Instruction::BNE => {
                if !self.get_status_flag(Status::ZERO) {
                    self.absolute_address = self.pc.wrapping_add(self.relative_address as u16);
                    self.pc = self.absolute_address;
                }
            }
            Instruction::BPL => {
                if !self.get_status_flag(Status::NEGATIVE) {
                    self.absolute_address = self.pc.wrapping_add(self.relative_address as u16);
                    self.pc = self.absolute_address;
                }
            }
            Instruction::BVC => {
                if !self.get_status_flag(Status::OVERFLOW) {
                    self.absolute_address = self.pc.wrapping_add(self.relative_address as u16);
                    self.pc = self.absolute_address;
                }
            }
            Instruction::BVS => {
                if self.get_status_flag(Status::OVERFLOW) {
                    self.absolute_address = self.pc.wrapping_add(self.relative_address as u16);
                    self.pc = self.absolute_address;
                }
            }
            Instruction::ADC => {

            }
            Instruction::SBC => {

            }
            Instruction::ASL => {
                
            }
            Instruction::LSR => {
                
            }
            Instruction::ROL => {
                
            }
            Instruction::ROR => {
                
            }
            Instruction::INC => {

            }
            Instruction::DEC => {
                
            }
            Instruction::JMP => {
                self.pc = self.absolute_address;
            }
            Instruction::PHA => {
                self.bus.write(STACK_POINTER_ADDRESS | self.sp as u16, self.a);
                self.sp = self.sp.wrapping_sub(1);
            }
            Instruction::PHP => {
                let status = self.status |  Status::BREAK | Status::UNUSED;
                self.bus.write(STACK_POINTER_ADDRESS | self.sp as u16, status.bits());
                self.sp = self.sp.wrapping_sub(1);
            }
            Instruction::PLA => {
                self.sp = self.sp.wrapping_add(1);
                self.a = self.bus.read(STACK_POINTER_ADDRESS | self.sp as u16);
                self.update_zero_negative_flags(self.a);
            }
            Instruction::PLP => {
                self.sp = self.sp.wrapping_add(1);
                let status = self.bus.read(STACK_POINTER_ADDRESS | self.sp as u16);
                self.status = Status::from_bits_truncate(status);
                self.set_status_flag(Status::BREAK, false);
                self.set_status_flag(Status::UNUSED, true);

            }
            Instruction::JSR => {

            }
            Instruction::RTS => {

            }
            Instruction::RTI => {

            }
            Instruction::BRK => {

            }
            Instruction::BIT => {

            }
        }
    }

    fn is_zero(&self, value: u8) -> bool {
        value == 0x00
    }

    fn is_negative(&self, value: u8) -> bool {
        value & 0x80 != 0
    }

    fn update_zero_negative_flags(&mut self, value: u8) {
        self.set_status_flag(Status::ZERO, self.is_zero(value));
        self.set_status_flag(Status::NEGATIVE, self.is_negative(value));
    }
}
