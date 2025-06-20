mod cpu;
mod rom;
mod bus;

use crate::cpu::CPU;
use crate::rom::Rom;

fn main() {
    println!("Attempt at first ROM load!");

    let cart = Rom::new_test(vec![
        0x38,             // SEC
        0x6C, 0x10, 0x00, // JMP $8005
        0x18,             // CLC (should be skipped)
        0xA9, 0x10,       // LDA $10
        0x00,             // BRK
    ]).unwrap();

    let mut my_cpu: CPU = CPU::new(cart);
    my_cpu.mem_write_u16(0x0010, 0x8005);
    my_cpu.run_rom();
    println!("CPU STATUS: {:08b}", my_cpu.status);
    println!("A REGISTER: {:02X}", my_cpu.reg_a);

}
