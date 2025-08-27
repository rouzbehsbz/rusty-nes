use crate::{
    bus::Bus,
    cpu::{CPU, RESET_VECTOR_ADDRESS_HI, RESET_VECTOR_ADDRESS_LO},
    ram::RAM,
};

mod bus;
mod cpu;
mod ram;
mod errors;
mod instructions;

fn main() {
    let mut ram = RAM::<2048>::new();

    let program = [
        0xA2, 0x0A, 0x8E, 0x00, 0x00, 0xA2, 0x03, 0x8E,
        0x01, 0x00, 0xAC, 0x00, 0x00, 0xA9, 0x00, 0x18,
        0x6D, 0x01, 0x00, 0x88, 0xD0, 0xFA, 0x8D, 0x02,
        0x00, 0xEA, 0xEA, 0xEA
    ];

    let start_address: u16 = 0x8000;

    ram.load_program(&program, start_address);

    let mut bus = Bus::new(Box::new(ram));

    let lo = (start_address & 0xFF) as u8;
    let hi = (start_address >> 8) as u8;

    bus.write(RESET_VECTOR_ADDRESS_LO, lo);
    bus.write(RESET_VECTOR_ADDRESS_HI, hi);

    let mut cpu = CPU::new(bus);

    loop {
        if let Err(err) = cpu.clock() {
            panic!("{}", err);
        }
    }
}
