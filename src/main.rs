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
    let mut ram = RAM::<65536>::new();

    /*
    
                LDX #$03        ; X = 3 (the multiplier)
                LDA #$00        ; A = 0 (clear accumulator, will hold result)

        Loop:   CLC             ; Clear Carry before addition
                ADC #$0A        ; A = A + 10 (the multiplicand)
                DEX             ; X = X - 1
                BNE Loop        ; Repeat until X = 0

                STA $50         ; Store result into zero-page location $50

                BRK             ; End of program
    
    */
    let program = [
        0xA2, 0x03, 0xA9, 0x00, 0x18, 0x69, 0x0A, 0xCA,
        0xD0, 0xFA, 0x85, 0x50, 0x00
    ];

    let start_address: u16 = 0x8000;

    ram.load_program(&program, start_address);

    let mut bus = Bus::new(Box::new(ram));

    let lo = (start_address) as u8;
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
