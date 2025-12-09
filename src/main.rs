use crate::{
    bus::cpu_bus::CpuBus, cartridge::cartridge::Cartridge, cpu::cpu::CPU, memory::Memory,
    ppu::ppu::PPU,
};
use std::{rc::Rc, thread, time::Duration};

mod bus;
mod cartridge;
mod cpu;
mod errors;
mod memory;
mod ppu;

fn main() {
    let ram = Memory::new(65536);
    let ppu = PPU::new();
    let cartridge = Cartridge::new(&[]).unwrap();
    let bus = Rc::new(CpuBus::new(ram, ppu, cartridge));

    let mut cpu = CPU::new(bus);

    loop {
        thread::sleep(Duration::from_secs(1));
        if let Err(err) = cpu.clock() {
            panic!("{}", err);
        }
    }
}
