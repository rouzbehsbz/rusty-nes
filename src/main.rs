use crate::{
    bus::Bus,
    cpu::{CPU},
    ram::RAM,
};

mod bus;
mod cpu;
mod ram;
mod errors;
mod instructions;

fn main() {
    let sram = RAM::<2048>::new();
    let bus = Bus::new(Box::new(sram));

    let mut cpu = CPU::new(bus);

    loop {
        if let Err(err) = cpu.clock() {
            panic!("{}", err);
        }
    }
}
