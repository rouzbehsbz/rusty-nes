use crate::{
    bus::{cpu_bus::CpuBus, ppu_bus::PpuBus},
    cartridge::cartridge::Cartridge,
    cpu::cpu::CPU,
    memory::memory::Memory,
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
    let cartridge = Rc::new(Cartridge::new(&[]).unwrap());

    let ppu_bus = PpuBus::new(cartridge.clone());
    let ppu = PPU::new(ppu_bus);

    let cpu_bus = CpuBus::new(ram, ppu, cartridge.clone());
    let mut cpu = CPU::new(cpu_bus);

    loop {
        thread::sleep(Duration::from_secs(1));
        if let Err(err) = cpu.clock() {
            panic!("{}", err);
        }
    }
}
