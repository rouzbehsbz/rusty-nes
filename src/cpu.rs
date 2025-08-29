use bitflags::bitflags;

use crate::{
    bus::Bus,
    errors::{AppError, AppResult},
    instructions::{AddressingMode, Instruction, Opcode},
};

pub const STACK_POINTER_INITIAL_OFFSET: u8 = 0xFD;
pub const STACK_POINTER_ADDRESS: u16 = 0x0100;
pub const IRQ_VECTOR_ADDRESS_LO: u16 = 0xFFFE;
pub const IRQ_VECTOR_ADDRESS_HI: u16 = 0xFFFF;
pub const NMI_VECTOR_ADDRESS_LO: u16 = 0xFFFA;
pub const NMI_VECTOR_ADDRESS_HI: u16 = 0xFFFB;
pub const RESET_VECTOR_ADDRESS_LO: u16 = 0xFFFC;
pub const RESET_VECTOR_ADDRESS_HI: u16 = 0xFFFD;

bitflags! {
    #[derive(Debug, Clone, Copy)]
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
        let lo = bus.read(RESET_VECTOR_ADDRESS_LO) as u16;
        let hi = bus.read(RESET_VECTOR_ADDRESS_HI) as u16;

        Self {
            a: 0,
            x: 0,
            y: 0,
            sp: STACK_POINTER_INITIAL_OFFSET,
            pc: (hi << 8) | lo,
            status: Status::UNUSED | Status::INTERRUPT,
            bus: bus,
            cycles: 0,
            absolute_address: 0,
            relative_address: 0,
        }
    }

    pub fn clock(&mut self) -> AppResult<()> {
        if self.cycles == 0 {
            let byte = self.bus.read(self.pc);
            self.increment_pc();

            match Opcode::decode(byte) {
                Some(opcode) => {
                    self.cycles = opcode.cycles;

                    self.execute_addressing_mode(opcode.addressing_mode);
                    self.execute_instruction(opcode.instruction, opcode.addressing_mode);
                }
                None => return Err(AppError::InvalidOpcode),
            }
        }

        self.cycles -= 1;
        Ok(())
    }

    pub fn reset(&mut self) {
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.sp = STACK_POINTER_INITIAL_OFFSET;

        let lo = self.bus.read(RESET_VECTOR_ADDRESS_LO);
        let hi = self.bus.read(RESET_VECTOR_ADDRESS_HI);

        self.pc = self.get_bytes_to_address(hi, lo);
        self.absolute_address = 0x0000;
        self.relative_address = 0x0000;
        self.status = Status::UNUSED;
        self.cycles = 8;
    }

    pub fn irq(&mut self) {
        if !self.get_status_flag(Status::INTERRUPT) {
            return
        }

        let pc = self.pc;

        self.write_to_stack((pc >> 8) as u8);
        self.write_to_stack(pc as u8);

        let status = self.status;

        self.write_to_stack(status.bits());

        self.set_status_flag(Status::INTERRUPT, true);

        let lo = self.bus.read(IRQ_VECTOR_ADDRESS_LO);
        let hi = self.bus.read(IRQ_VECTOR_ADDRESS_HI);
        self.pc = self.get_bytes_to_address(hi, lo);

        self.cycles = 7;
    }

    pub fn nmi(&mut self) {
        let pc = self.pc;

        self.write_to_stack((pc >> 8) as u8);
        self.write_to_stack(pc as u8);

        let status = self.status | Status::UNUSED;

        self.write_to_stack(status.bits());

        self.set_status_flag(Status::INTERRUPT, true);

        let lo = self.bus.read(NMI_VECTOR_ADDRESS_LO) as u8;
        let hi = self.bus.read(NMI_VECTOR_ADDRESS_HI) as u8;
        self.pc = self.get_bytes_to_address(hi, lo);

        self.cycles = 7;
    }

    fn increment_pc(&mut self) {
        self.pc = self.pc.wrapping_add(1);
    }

    fn set_status_flag(&mut self, flag: Status, condition: bool) {
        if condition {
            self.status.insert(flag);
        } else {
            self.status.remove(flag);
        }
    }

    fn get_status_flag(&self, flag: Status) -> bool {
        self.status.contains(flag)
    }

    fn execute_addressing_mode(&mut self, addressing_mode: AddressingMode) {
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
                let lo = self.bus.read(self.pc);
                self.increment_pc();
                let hi = self.bus.read(self.pc);
                self.increment_pc();

                self.absolute_address = self.get_bytes_to_address(hi, lo);
            }
            AddressingMode::AbsoluteX => {
                let lo = self.bus.read(self.pc);
                self.increment_pc();
                let hi = self.bus.read(self.pc);
                self.increment_pc();

                self.absolute_address = (self.get_bytes_to_address(hi, lo)).wrapping_add(self.x as u16)
            }
            AddressingMode::AbsoluteY => {
                let lo = self.bus.read(self.pc);
                self.increment_pc();
                let hi = self.bus.read(self.pc);
                self.increment_pc();

                self.absolute_address = (self.get_bytes_to_address(hi, lo)).wrapping_add(self.y as u16)
            }
            AddressingMode::Indirect => {
                let ptr_lo = self.bus.read(self.pc);
                self.increment_pc();
                let ptr_hi = self.bus.read(self.pc);
                self.increment_pc();

                let ptr = self.get_bytes_to_address(ptr_hi, ptr_lo);

                let lo = self.bus.read(ptr);
                let hi = self.bus.read(ptr.wrapping_add(1));

                self.absolute_address = self.get_bytes_to_address(hi, lo)
            }
            AddressingMode::IndirectX => {
                let base = self.bus.read(self.pc);
                self.increment_pc();

                let ptr = base.wrapping_add(self.x) as u16;

                let lo = self.bus.read(ptr & 0x00FF);
                let hi = self.bus.read((ptr.wrapping_add(1)) & 0x00FF);

                self.absolute_address = self.get_bytes_to_address(hi, lo);
            }
            AddressingMode::IndirectY => {
                let base = self.bus.read(self.pc);
                self.increment_pc();

                let lo = self.bus.read(base as u16);
                let hi = self.bus.read((base.wrapping_add(1)) as u16 & 0x00FF);

                let ptr = self.get_bytes_to_address(hi, lo);

                self.absolute_address = ptr.wrapping_add(self.y as u16);
            }
        }
    }

    fn execute_instruction(&mut self, instruction: Instruction, addressing_mode: AddressingMode) {
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
                let value = self.bus.read(self.absolute_address);
                let result = self.a.wrapping_sub(value);

                self.set_status_flag(Status::CARRY, self.a >= value);
                self.update_zero_negative_flags(result);
            }
            Instruction::CPX => {
                let value = self.bus.read(self.absolute_address);
                let result = self.x.wrapping_sub(value);

                self.set_status_flag(Status::CARRY, self.x >= value);
                self.update_zero_negative_flags(result);
            }
            Instruction::CPY => {
                let value = self.bus.read(self.absolute_address);
                let result = self.y.wrapping_sub(value);

                self.set_status_flag(Status::CARRY, self.y >= value);
                self.update_zero_negative_flags(result);
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
                let value = self.bus.read(self.absolute_address);
                let carry = if self.get_status_flag(Status::CARRY) { 1 } else { 0 };
                let result = self.a as u16 + value as u16 + carry;

                self.set_status_flag(Status::CARRY, result > 0xFF);
                self.set_status_flag(Status::OVERFLOW, 
                    (self.a ^ value) & 0x80 == 0 && (self.a ^ result as u8) & 0x80 != 0);
                self.a = result as u8;
                self.update_zero_negative_flags(self.a);
            }
            Instruction::SBC => {
                let value = self.bus.read(self.absolute_address);
                let carry = if self.get_status_flag(Status::CARRY) { 1 } else { 0 };
                let result = self.a as i16 - value as i16 - (1 - carry) as i16;

                self.set_status_flag(Status::CARRY, result >= 0);
                self.set_status_flag(Status::OVERFLOW, 
                    (self.a ^ value) & 0x80 != 0 && (self.a ^ result as u8) & 0x80 != 0);
                self.a = result as u8;
                self.update_zero_negative_flags(self.a);
            }
            Instruction::ASL => {
                let value = self.read_a_or_absolute(addressing_mode);
                let result = value << 1;

                self.set_status_flag(Status::CARRY, self.is_negative(value));
                self.write_a_or_absolute(addressing_mode, result);
                self.update_zero_negative_flags(result);
            }
            Instruction::LSR => {
                let value = self.read_a_or_absolute(addressing_mode);
                let result = value >> 1;

                self.set_status_flag(Status::CARRY, self.is_bit0_set(value));
                self.write_a_or_absolute(addressing_mode, result);
                self.update_zero_negative_flags(result);
            }
            Instruction::ROL => {
                let mut value = self.read_a_or_absolute(addressing_mode);
                let old_carry = if self.get_status_flag(Status::CARRY) { 1 } else { 0 };
                self.set_status_flag(Status::CARRY, self.is_negative(value));

                value = (value << 1) | old_carry;

                self.update_zero_negative_flags(value);
                self.write_a_or_absolute(addressing_mode, value);
            }
            Instruction::ROR => {
                let mut value = self.read_a_or_absolute(addressing_mode);
                let old_carry = if self.get_status_flag(Status::CARRY) { 0x80 } else { 0 };
                self.set_status_flag(Status::CARRY, self.is_bit0_set(value));

                value = (value >> 1) | old_carry;

                self.update_zero_negative_flags(value);
                self.write_a_or_absolute(addressing_mode, value);
            }
            Instruction::INC => {
                let mut value = self.bus.read(self.absolute_address);
                value = value.wrapping_add(1);
                self.bus.write(self.absolute_address, value);
                self.update_zero_negative_flags(value);
            }
            Instruction::DEC => {
                let mut value = self.bus.read(self.absolute_address);
                value = value.wrapping_sub(1);
                self.bus.write(self.absolute_address, value);
                self.update_zero_negative_flags(value);
            }
            Instruction::JMP => {
                self.pc = self.absolute_address;
            }
            Instruction::PHA => {
                self.write_to_stack(self.a);
            }
            Instruction::PHP => {
                let status = self.status | Status::BREAK | Status::UNUSED;
                self.write_to_stack(status.bits());
            }
            Instruction::PLA => {
                self.a = self.read_from_stack();
                self.update_zero_negative_flags(self.a);
            }
            Instruction::PLP => {
                let status = self.read_from_stack();
                self.status = Status::from_bits_truncate(status);
                self.set_status_flag(Status::BREAK, false);
                self.set_status_flag(Status::UNUSED, true);

            }
            Instruction::JSR => {
                let return_address = self.pc.wrapping_sub(1);
                self.write_to_stack((return_address >> 8) as u8);
                self.write_to_stack(return_address as u8);
                self.pc = self.absolute_address;
            }
            Instruction::RTS => {
                let lo: u8 = self.read_from_stack();
                let hi = self.read_from_stack();
                self.pc = (self.get_bytes_to_address(hi, lo)).wrapping_add(1);
            }
            Instruction::RTI => {
                let status = self.read_from_stack();
                self.status = Status::from_bits_truncate(status);
                self.set_status_flag(Status::BREAK, false);
                self.set_status_flag(Status::UNUSED, true);
                let lo = self.read_from_stack();
                let hi = self.read_from_stack();
                self.pc = self.get_bytes_to_address(hi, lo);
            }
            Instruction::BRK => {
                let return_address = self.pc.wrapping_add(1);
                self.write_to_stack((return_address >> 8) as u8);
                self.write_to_stack(return_address as u8);
                let status = self.status | Status::BREAK | Status::UNUSED;
                self.write_to_stack(status.bits());
                self.set_status_flag(Status::INTERRUPT, true); 
                let lo = self.bus.read(IRQ_VECTOR_ADDRESS_LO);
                let hi = self.bus.read(IRQ_VECTOR_ADDRESS_HI);
                self.pc = self.get_bytes_to_address(hi, lo);
            }
            Instruction::BIT => {
                let value = self.bus.read(self.absolute_address);
                self.update_zero_negative_flags(self.a & value);
                self.set_status_flag(Status::NEGATIVE, self.is_negative(value));
                self.set_status_flag(Status::OVERFLOW, self.is_overflow(value));
            }
        }
    }

    fn write_to_stack(&mut self, value: u8) {
        self.bus.write(self.get_stack_address(), value);
        self.sp = self.sp.wrapping_sub(1);
    }

    fn read_from_stack(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        self.bus.read(self.get_stack_address())
    }

    fn is_zero(&self, value: u8) -> bool {
        value == 0x00
    }

    fn is_negative(&self, value: u8) -> bool {
        value & 0x80 != 0
    }

    fn is_overflow(&self, value: u8) -> bool {
        value & 0x40 != 0
    }

    fn is_bit0_set(&self, value: u8) -> bool {
        value & 0x01 != 0
    }

    fn update_zero_negative_flags(&mut self, value: u8) {
        self.set_status_flag(Status::ZERO, self.is_zero(value));
        self.set_status_flag(Status::NEGATIVE, self.is_negative(value));
    }

    fn get_bytes_to_address(&self, hi: u8, lo: u8) -> u16 {
        ((hi as u16) << 8) | (lo as u16)
    }

    fn get_stack_address(&self) -> u16 {
        STACK_POINTER_ADDRESS | self.sp as u16
    }

    fn read_a_or_absolute(&self, addressing_mode: AddressingMode) -> u8 {
        match addressing_mode {
            AddressingMode::Accumulator => self.a,
            _ => self.bus.read(self.absolute_address)
        }
    }

    fn write_a_or_absolute(&mut self, addressing_mode: AddressingMode, value: u8) {
        match addressing_mode {
            AddressingMode::Accumulator => self.a = value,
            _ => self.bus.write(self.absolute_address, value)
        }
    }
}
