use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};

use crate::rom::Rom;
use crate::bus::{Bus, Mem};

pub struct CPU<'a> {
    // Registers
    pub reg_a: u8, // Stores results of arithmetic, logic, and memory access operations
    pub reg_x: u8,
    pub reg_y: u8,
    pub status: u8, // Each bit stores the 7 status flags (ex. Z = zero flag)
    pub pc: u16, // stores mem address of next byte of code (16 bits cause ram size)
    pub sp: u8,
    pub bus: Bus<'a>,
    pub extra_cycles: usize,
    pub test: bool,
}

// status register bit values
/*
| Bit | Flag                       | Mask (binary) | Purpose                      |
| --- | -------------------------- | ------------- | ---------------------------- |
| 7   | N (Negative)               | `0b1000_0000` | Set if result is negative    |
| 6   | V (Overflow)               | `0b0100_0000` | Set on signed overflow       |
| 5   | Unused (always 1 on stack) | `0b0010_0000` | Typically ignored            |
| 4   | B (Break)                  | `0b0001_0000` | Set by `BRK` instruction     |
| 3   | D (Decimal)                | `0b0000_1000` | Decimal mode (unused in NES) |
| 2   | I (Interrupt Disable)      | `0b0000_0100` | Disable interrupts           |
| 1   | Z (Zero)                   | `0b0000_0010` | Set if result is zero        |
| 0   | C (Carry)                  | `0b0000_0001` | Carry from math ops          |
*/

pub struct OpCode {
    pub addr: u8,
    pub code: &'static str,
    pub bytes: usize,
    pub cycles: usize,
    pub addressing_mode: AddressingMode
}

impl OpCode {
    pub fn new(addr: u8, code: &'static str, bytes: usize, cycles: usize, addressing_mode: AddressingMode) -> Self {
        OpCode { addr, code, bytes, cycles, addressing_mode }
    }
}

pub static PAGE_CROSSERS: Lazy<HashSet<u8>> = Lazy::new(|| {
    let mut set = HashSet::new();

    // ADC
    set.insert(0x7D);
    set.insert(0x79);
    set.insert(0x71);

    // AND
    set.insert(0x3D);
    set.insert(0x39);
    set.insert(0x31);

    // CMP
    set.insert(0xDD);
    set.insert(0xD9);
    set.insert(0xD1);

    // EOR
    set.insert(0x5D);
    set.insert(0x59);
    set.insert(0x51);

    // LDA
    set.insert(0xBD);
    set.insert(0xB9);
    set.insert(0xB1);

    // LDX
    set.insert(0xBE);

    // LDY
    set.insert(0xBC);

    // ORA
    set.insert(0x1D);
    set.insert(0x19);
    set.insert(0x11);

    // SBC
    set.insert(0xFD);
    set.insert(0xF9);
    set.insert(0xF1);

    set
});

pub static OPCODE_TABLE: Lazy<HashMap<u8, OpCode>> = Lazy::new(|| {
    let mut map = HashMap::new();

    //BRK
    map.insert(0x00, OpCode::new(0x00, "BRK", 1, 7, AddressingMode::NoneAddressing));

    //TAX
    map.insert(0xAA, OpCode::new(0xAA, "TAX", 1, 2, AddressingMode::NoneAddressing));

    //INX
    map.insert(0xE8, OpCode::new(0xE8, "INX", 1, 2, AddressingMode::NoneAddressing));

    // LDA
    map.insert(0xA9, OpCode::new(0xA9, "LDA", 2, 2, AddressingMode::Immediate));
    map.insert(0xA5, OpCode::new(0xA5, "LDA", 2, 3, AddressingMode::ZeroPage));
    map.insert(0xB5, OpCode::new(0xB5, "LDA", 2, 4, AddressingMode::ZeroPage_X));
    map.insert(0xAd, OpCode::new(0xAD, "LDA", 3, 4, AddressingMode::Absolute));
    map.insert(0xBD, OpCode::new(0xBD, "LDA", 3, 4, AddressingMode::Absolute_X));
    map.insert(0xB9, OpCode::new(0xB9, "LDA", 3, 4, AddressingMode::Absolute_Y));
    map.insert(0xA1, OpCode::new(0xA1, "LDA", 2, 6, AddressingMode::Indirect_X));
    map.insert(0xB1, OpCode::new(0xB1, "LDA", 2, 5, AddressingMode::Indirect_Y));

    // Unguided instructions

    // ADC
    map.insert(0x69, OpCode::new(0x69, "ADC", 2, 2, AddressingMode::Immediate));
    map.insert(0x65, OpCode::new(0x65, "ADC", 2, 3, AddressingMode::ZeroPage));
    map.insert(0x75, OpCode::new(0x75, "ADC", 2, 4, AddressingMode::ZeroPage_X));
    map.insert(0x6D, OpCode::new(0x6D, "ADC", 3, 4, AddressingMode::Absolute));
    map.insert(0x7D, OpCode::new(0x7D, "ADC", 3, 4, AddressingMode::Absolute_X));
    map.insert(0x79, OpCode::new(0x79, "ADC", 3, 4, AddressingMode::Absolute_Y));
    map.insert(0x61, OpCode::new(0x61, "ADC", 2, 6, AddressingMode::Indirect_X));
    map.insert(0x71, OpCode::new(0x71, "ADC", 2, 5, AddressingMode::Indirect_Y));

    // CLC
    map.insert(0x18, OpCode::new(0x18, "CLC", 1, 2, AddressingMode::NoneAddressing));

    // SEC
    map.insert(0x38, OpCode::new(0x38, "SEC", 1, 2, AddressingMode::NoneAddressing));

    // AND
    map.insert(0x29, OpCode::new(0x29, "AND", 2, 2, AddressingMode::Immediate));
    map.insert(0x25, OpCode::new(0x25, "AND", 2, 3, AddressingMode::ZeroPage));
    map.insert(0x35, OpCode::new(0x35, "AND", 2, 4, AddressingMode::ZeroPage_X));
    map.insert(0x2D, OpCode::new(0x2D, "AND", 3, 4, AddressingMode::Absolute));
    map.insert(0x3D, OpCode::new(0x3D, "AND", 3, 4, AddressingMode::Absolute_X));
    map.insert(0x39, OpCode::new(0x39, "AND", 3, 4, AddressingMode::Absolute_Y));
    map.insert(0x21, OpCode::new(0x21, "AND", 2, 6, AddressingMode::Indirect_X));
    map.insert(0x31, OpCode::new(0x31, "AND", 2, 5, AddressingMode::Indirect_Y));

    // ASL
    map.insert(0x0A, OpCode::new(0x0A, "ASL", 1, 2, AddressingMode::NoneAddressing));
    map.insert(0x06, OpCode::new(0x06, "ASL", 2, 5, AddressingMode::ZeroPage));
    map.insert(0x16, OpCode::new(0x16, "ASL", 2, 6, AddressingMode::ZeroPage_X));
    map.insert(0x0E, OpCode::new(0x0E, "ASL", 3, 6, AddressingMode::Absolute));
    map.insert(0x1E, OpCode::new(0x1E, "ASL", 3, 7, AddressingMode::Absolute_X));

    // BCC
    map.insert(0x90, OpCode::new(0x90, "BCC", 2, 2, AddressingMode::NoneAddressing));

    // BCS
    map.insert(0xB0, OpCode::new(0xB0, "BCS", 2, 2, AddressingMode::NoneAddressing));

    // BEQ
    map.insert(0xF0, OpCode::new(0xF0, "BEQ", 2, 2, AddressingMode::NoneAddressing));

    // BIT
    map.insert(0x24, OpCode::new(0x24, "BIT", 2, 3, AddressingMode::ZeroPage));
    map.insert(0x2C, OpCode::new(0x2C, "BIT", 3, 4, AddressingMode::Absolute));

    // BMI
    map.insert(0x30, OpCode::new(0x30, "BMI", 2, 2, AddressingMode::NoneAddressing));

    // BNE
    map.insert(0xD0, OpCode::new(0xD0, "BNE", 2, 2, AddressingMode::NoneAddressing));

    // BPL
    map.insert(0x10, OpCode::new(0x10, "BPL", 2, 2, AddressingMode::NoneAddressing));

    // BVC
    map.insert(0x50, OpCode::new(0x50, "BVC", 2, 2, AddressingMode::NoneAddressing));

    // BVS
    map.insert(0x70, OpCode::new(0x70, "BVS", 2, 2, AddressingMode::NoneAddressing));
    
    // CLD
    map.insert(0xD8, OpCode::new(0xD8, "CLD", 1, 2, AddressingMode::NoneAddressing));

    // CLV
    map.insert(0xB8, OpCode::new(0xB8, "CLV", 1, 2, AddressingMode::NoneAddressing));

    // CLI
    map.insert(0x58, OpCode::new(0x58, "CLI", 1, 2, AddressingMode::NoneAddressing));

    // CMP
    map.insert(0xC9, OpCode::new(0xC9, "CMP", 2, 2, AddressingMode::Immediate));
    map.insert(0xC5, OpCode::new(0xC5, "CMP", 2, 3, AddressingMode::ZeroPage));
    map.insert(0xD5, OpCode::new(0xD5, "CMP", 2, 4, AddressingMode::ZeroPage_X));
    map.insert(0xCD, OpCode::new(0xCD, "CMP", 3, 4, AddressingMode::Absolute));
    map.insert(0xDD, OpCode::new(0xDD, "CMP", 3, 4, AddressingMode::Absolute_X));
    map.insert(0xD9, OpCode::new(0xD9, "CMP", 3, 4, AddressingMode::Absolute_Y));
    map.insert(0xC1, OpCode::new(0xC1, "CMP", 2, 6, AddressingMode::Indirect_X));
    map.insert(0xD1, OpCode::new(0xD1, "CMP", 2, 5, AddressingMode::Indirect_Y));

    // CPX
    map.insert(0xE0, OpCode::new(0xE0, "CPX", 2, 2, AddressingMode::Immediate));
    map.insert(0xE4, OpCode::new(0xE4, "CPX", 2, 3, AddressingMode::ZeroPage));
    map.insert(0xEC, OpCode::new(0xEC, "CPX", 3, 4, AddressingMode::Absolute));

    // CPY
    map.insert(0xC0, OpCode::new(0xC0, "CPY", 2, 2, AddressingMode::Immediate));
    map.insert(0xC4, OpCode::new(0xC4, "CPY", 2, 3, AddressingMode::ZeroPage));
    map.insert(0xCC, OpCode::new(0xCC, "CPY", 3, 4, AddressingMode::Absolute));

    // LDX
    map.insert(0xA2, OpCode::new(0xA2, "LDX", 2, 2, AddressingMode::Immediate));
    map.insert(0xA6, OpCode::new(0xA6, "LDX", 2, 3, AddressingMode::ZeroPage));
    map.insert(0xB6, OpCode::new(0xB6, "LDX", 2, 4, AddressingMode::ZeroPage_Y));
    map.insert(0xAE, OpCode::new(0xAE, "LDX", 3, 4, AddressingMode::Absolute));
    map.insert(0xBE, OpCode::new(0xBE, "LDX", 3, 4, AddressingMode::Absolute_Y));

    // LDY
    map.insert(0xA0, OpCode::new(0xA0, "LDY", 2, 2, AddressingMode::Immediate));
    map.insert(0xA4, OpCode::new(0xA4, "LDY", 2, 3, AddressingMode::ZeroPage));
    map.insert(0xB4, OpCode::new(0xB4, "LDY", 2, 4, AddressingMode::ZeroPage_X));
    map.insert(0xAC, OpCode::new(0xAC, "LDY", 3, 4, AddressingMode::Absolute));
    map.insert(0xBC, OpCode::new(0xBC, "LDY", 3, 4, AddressingMode::Absolute_X));

    // DEC
    map.insert(0xC6, OpCode::new(0xC6, "DEC", 2, 5, AddressingMode::ZeroPage));
    map.insert(0xD6, OpCode::new(0xD6, "DEC", 2, 6, AddressingMode::ZeroPage_X));
    map.insert(0xCE, OpCode::new(0xCE, "DEC", 3, 6, AddressingMode::Absolute));
    map.insert(0xDE, OpCode::new(0xDE, "DEC", 3, 7, AddressingMode::Absolute_X));

    // DEX
    map.insert(0xCA, OpCode::new(0xCA, "DEX", 1, 2, AddressingMode::NoneAddressing));

    // DEY
    map.insert(0x88, OpCode::new(0x88, "DEY", 1, 2, AddressingMode::NoneAddressing));

    // EOR
    map.insert(0x49, OpCode::new(0x49, "EOR", 2, 2, AddressingMode::Immediate));
    map.insert(0x45, OpCode::new(0x45, "EOR", 2, 3, AddressingMode::ZeroPage));
    map.insert(0x55, OpCode::new(0x55, "EOR", 2, 4, AddressingMode::ZeroPage_X));
    map.insert(0x4D, OpCode::new(0x4D, "EOR", 3, 4, AddressingMode::Absolute));
    map.insert(0x5D, OpCode::new(0x5D, "EOR", 3, 4, AddressingMode::Absolute_X));
    map.insert(0x59, OpCode::new(0x59, "EOR", 3, 4, AddressingMode::Absolute_Y));
    map.insert(0x41, OpCode::new(0x41, "EOR", 2, 6, AddressingMode::Indirect_X));
    map.insert(0x51, OpCode::new(0x51, "EOR", 2, 5, AddressingMode::Indirect_Y));

    // INC
    map.insert(0xE6, OpCode::new(0xE6, "INC", 2, 5, AddressingMode::ZeroPage));
    map.insert(0xF6, OpCode::new(0xF6, "INC", 2, 6, AddressingMode::ZeroPage_X));
    map.insert(0xEE, OpCode::new(0xEE, "INC", 3, 6, AddressingMode::Absolute));
    map.insert(0xFE, OpCode::new(0xFE, "INC", 3, 7, AddressingMode::Absolute_X));

    // INY
    map.insert(0xC8, OpCode::new(0xC8, "INY", 1, 2, AddressingMode::NoneAddressing));

    // JMP
    map.insert(0x4C, OpCode::new(0x4C, "JMP", 3, 3, AddressingMode::Immediate));
    map.insert(0x6C, OpCode::new(0x6C, "JMP", 3, 5, AddressingMode::Absolute));

    // JSR
    map.insert(0x20, OpCode::new(0x20, "JSR", 3, 6, AddressingMode::Absolute));

    // RTS
    map.insert(0x60, OpCode::new(0x60, "RTS", 1, 6, AddressingMode::NoneAddressing));

    // LSR
    map.insert(0x4A, OpCode::new(0x4A, "LSR", 1, 2, AddressingMode::NoneAddressing));
    map.insert(0x46, OpCode::new(0x46, "LSR", 2, 5, AddressingMode::ZeroPage));
    map.insert(0x56, OpCode::new(0x56, "LSR", 2, 6, AddressingMode::ZeroPage_X));
    map.insert(0x4E, OpCode::new(0x4E, "LSR", 3, 6, AddressingMode::Absolute));
    map.insert(0x5E, OpCode::new(0x5E, "LSR", 3, 7, AddressingMode::Absolute_X));

    // NOP
    map.insert(0xEA, OpCode::new(0xEA, "NOP", 1, 2, AddressingMode::NoneAddressing));

    // ORA
    map.insert(0x09, OpCode::new(0x09, "ORA", 2, 2, AddressingMode::Immediate));
    map.insert(0x05, OpCode::new(0x05, "ORA", 2, 3, AddressingMode::ZeroPage));
    map.insert(0x15, OpCode::new(0x15, "ORA", 2, 4, AddressingMode::ZeroPage_X));
    map.insert(0x0D, OpCode::new(0x0D, "ORA", 3, 4, AddressingMode::Absolute));
    map.insert(0x1D, OpCode::new(0x1D, "ORA", 3, 4, AddressingMode::Absolute_X));
    map.insert(0x19, OpCode::new(0x19, "ORA", 3, 4, AddressingMode::Absolute_Y));
    map.insert(0x01, OpCode::new(0x01, "ORA", 2, 6, AddressingMode::Indirect_X));
    map.insert(0x11, OpCode::new(0x11, "ORA", 2, 5, AddressingMode::Indirect_Y));

    // PHA
    map.insert(0x48, OpCode::new(0x48, "PHA", 1, 3, AddressingMode::NoneAddressing));

    // PHP
    map.insert(0x08, OpCode::new(0x08, "PHP", 1, 3, AddressingMode::NoneAddressing));

    // PLA
    map.insert(0x68, OpCode::new(0x68, "PLA", 1, 4, AddressingMode::NoneAddressing));

    // PLP
    map.insert(0x28, OpCode::new(0x28, "PLP", 1, 4, AddressingMode::NoneAddressing));

    // ROL
    map.insert(0x2A, OpCode::new(0x2A, "ROL", 1, 2, AddressingMode::NoneAddressing));
    map.insert(0x26, OpCode::new(0x26, "ROL", 2, 5, AddressingMode::ZeroPage));
    map.insert(0x36, OpCode::new(0x36, "ROL", 2, 6, AddressingMode::ZeroPage_X));
    map.insert(0x2E, OpCode::new(0x2E, "ROL", 3, 6, AddressingMode::Absolute));
    map.insert(0x3E, OpCode::new(0x3E, "ROL", 3, 7, AddressingMode::Absolute_X));

    // ROR
    map.insert(0x6A, OpCode::new(0x6A, "ROR", 1, 2, AddressingMode::NoneAddressing));
    map.insert(0x66, OpCode::new(0x66, "ROR", 2, 5, AddressingMode::ZeroPage));
    map.insert(0x76, OpCode::new(0x76, "ROR", 2, 6, AddressingMode::ZeroPage_X));
    map.insert(0x6E, OpCode::new(0x6E, "ROR", 3, 6, AddressingMode::Absolute));
    map.insert(0x7E, OpCode::new(0x7E, "ROR", 3, 7, AddressingMode::Absolute_X));

    // RTI
    map.insert(0x40, OpCode::new(0x40, "RTI", 1, 6, AddressingMode::NoneAddressing));

    // SBC
    map.insert(0xE9, OpCode::new(0xE9, "SBC", 2, 2, AddressingMode::Immediate));
    map.insert(0xE5, OpCode::new(0xE5, "SBC", 2, 3, AddressingMode::ZeroPage));
    map.insert(0xF5, OpCode::new(0xF5, "SBC", 2, 4, AddressingMode::ZeroPage_X));
    map.insert(0xED, OpCode::new(0xED, "SBC", 3, 4, AddressingMode::Absolute));
    map.insert(0xFD, OpCode::new(0xFD, "SBC", 3, 4, AddressingMode::Absolute_X));
    map.insert(0xF9, OpCode::new(0xF9, "SBC", 3, 4, AddressingMode::Absolute_Y));
    map.insert(0xE1, OpCode::new(0xE1, "SBC", 2, 6, AddressingMode::Indirect_X));
    map.insert(0xF1, OpCode::new(0xF1, "SBC", 2, 5, AddressingMode::Indirect_Y));

    // SED
    map.insert(0xF8, OpCode::new(0xF8, "SED", 1, 2, AddressingMode::NoneAddressing));

    // SEI
    map.insert(0x78, OpCode::new(0x78, "SEI", 1, 2, AddressingMode::NoneAddressing));

    // STA
    map.insert(0x85, OpCode::new(0x85, "STA", 2, 3, AddressingMode::ZeroPage));
    map.insert(0x95, OpCode::new(0x95, "STA", 2, 4, AddressingMode::ZeroPage_X));
    map.insert(0x8D, OpCode::new(0x8D, "STA", 3, 4, AddressingMode::Absolute));
    map.insert(0x9D, OpCode::new(0x9D, "STA", 3, 4, AddressingMode::Absolute_X));
    map.insert(0x99, OpCode::new(0x99, "STA", 3, 4, AddressingMode::Absolute_Y));
    map.insert(0x81, OpCode::new(0x81, "STA", 2, 6, AddressingMode::Indirect_X));
    map.insert(0x91, OpCode::new(0x91, "STA", 2, 5, AddressingMode::Indirect_Y));

    // STX
    map.insert(0x86, OpCode::new(0x86, "STX", 2, 3, AddressingMode::ZeroPage));
    map.insert(0x96, OpCode::new(0x96, "STX", 2, 4, AddressingMode::ZeroPage_Y));
    map.insert(0x8E, OpCode::new(0x8E, "STX", 3, 4, AddressingMode::Absolute));

    // STY
    map.insert(0x84, OpCode::new(0x84, "STY", 2, 3, AddressingMode::ZeroPage));
    map.insert(0x94, OpCode::new(0x94, "STY", 2, 4, AddressingMode::ZeroPage_X));
    map.insert(0x8C, OpCode::new(0x8C, "STY", 3, 4, AddressingMode::Absolute));

    // TAX
    map.insert(0xA8, OpCode::new(0xA8, "TAY", 1, 2, AddressingMode::NoneAddressing));

    // TSX
    map.insert(0xBA, OpCode::new(0xBA, "TSX", 1, 2, AddressingMode::NoneAddressing));

    // TXA
    map.insert(0x8A, OpCode::new(0x8A, "TXA", 1, 2, AddressingMode::NoneAddressing));

    // TXS
    map.insert(0x9A, OpCode::new(0x9A, "TXS", 1, 2, AddressingMode::NoneAddressing));

    // TYA
    map.insert(0x98, OpCode::new(0x98, "TYA", 1, 2, AddressingMode::NoneAddressing));

    map
});

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
   Immediate,
   ZeroPage,
   ZeroPage_X,
   ZeroPage_Y,
   Absolute,
   Absolute_X,
   Absolute_Y,
   Indirect,
   Indirect_X,
   Indirect_Y,
   NoneAddressing,
}

impl<'a> CPU<'a> {
    pub fn new<'b>(bus: Bus<'b>) -> CPU<'b> {
        CPU {
            reg_a: 0,
            status: 0,
            pc: 0,
            sp: 0xFF,
            reg_x: 0,
            reg_y: 0,
            extra_cycles: 0,
            bus: bus,
            test: false,
        }
    }

    // Getting operand information
    pub fn get_opperand_address(&mut self, mode: &AddressingMode) -> u16 {
        // Do standard mode matching
        match mode {
            AddressingMode::Immediate => self.pc, // raw value at the address already
            AddressingMode::ZeroPage => self.mem_read(self.pc) as u16, // pc stores 1 byte addr
            AddressingMode::ZeroPage_X => {
                let addr = self.mem_read(self.pc);
                let output = addr.wrapping_add(self.reg_x) as u16;
                output
            },
            AddressingMode::ZeroPage_Y => {
                let addr = self.mem_read(self.pc);
                let output = addr.wrapping_add(self.reg_y) as u16;
                output
            },
            AddressingMode::Absolute => self.mem_read_u16(self.pc),
            AddressingMode::Absolute_X => {
                let addr = self.mem_read_u16(self.pc);
                let output = addr.wrapping_add(self.reg_x as u16);
                output
            },
            AddressingMode::Absolute_Y => {
                let addr = self.mem_read_u16(self.pc);
                let output = addr.wrapping_add(self.reg_y as u16);
                output
            },
            AddressingMode::Indirect => {
                let output = self.mem_read_u16(self.pc);
                output
            }
            AddressingMode::Indirect_X => {
                let addr = self.mem_read(self.pc);
                let ptr = addr.wrapping_add(self.reg_x);

                let low = self.mem_read(ptr as u16);
                let high = self.mem_read(ptr.wrapping_add(1) as u16);
                let output = (high as u16) << 8 | (low as u16);
                output
            },
            AddressingMode::Indirect_Y => {
                let addr = self.mem_read(self.pc);

                let low = self.mem_read(addr as u16);
                let high = self.mem_read((addr as u8).wrapping_add(1) as u16);
                let ptr = (high as u16) << 8 | (low as u16);
                let output = ptr.wrapping_add(self.reg_y as u16);
                output
            }
            AddressingMode::NoneAddressing => {
                panic!("Mode {:?} is not supported", mode);
            }
        }
    }

    pub fn debug_operand(&self, old_pc: u16, mode: &AddressingMode) -> u16 {
        // Caller prints the output of mem reading this calls return value
        match mode {
            AddressingMode::Immediate => old_pc, // no print cause upper function already prints
            AddressingMode::ZeroPage => {
                let output = self.mem_peek(old_pc) as u16;
                // print!("({:02X}) @ ", output);
                output
            }, // pc stores 1 byte addr
            AddressingMode::ZeroPage_X => {
                let addr = self.mem_peek(old_pc);
                // print!("(${:02X}),X) ", addr);
                let output = addr.wrapping_add(self.reg_x) as u16;
                // print!("@ {:02X} = ", output); // sum
                output
            },
            AddressingMode::ZeroPage_Y => {
                let addr = self.mem_peek(old_pc);
                // print!("(${:02X}),Y) ", addr);
                let output = addr.wrapping_add(self.reg_y) as u16;
                // print!("@ {:04X} = ", output); // sum
                output
            },
            AddressingMode::Absolute => {
                let output = self.mem_peek_u16(old_pc);
                // print!("(${:04X}) @ ", output);
                output
            },
            AddressingMode::Absolute_X => {
                let addr = self.mem_peek_u16(old_pc);
                // print!("(${:04X},X) @ ", addr);
                let output = addr.wrapping_add(self.reg_x as u16);
                // print!("{:04X} = ", output);
                output
            },
            AddressingMode::Absolute_Y => {
                let addr = self.mem_peek_u16(old_pc);
                // print!("(${:04X},Y) @ ", addr);
                let output = addr.wrapping_add(self.reg_y as u16);
                // print!("{:04X} = ", output);
                output
            },
            AddressingMode::Indirect => {
                let output = self.mem_peek_u16(old_pc);
                // print!("({:04X}) @ ", output);
                output
            }
            AddressingMode::Indirect_X => {
                let addr = self.mem_peek(old_pc);
                // print!("({:04X},X) @ ", addr);
                let ptr = addr.wrapping_add(self.reg_x);
                print!("{:04X} = ", ptr);

                let low = self.mem_peek(ptr as u16);
                let high = self.mem_peek(ptr.wrapping_add(1) as u16);
                let output = (high as u16) << 8 | (low as u16);
                // print!("{:04X} = ", ptr);
                output
            },
            AddressingMode::Indirect_Y => {
                let addr = self.mem_peek(old_pc);
                // print!("({:04X},Y) @ ", addr);

                let low = self.mem_peek(addr as u16);
                let high = self.mem_peek((addr as u8).wrapping_add(1) as u16);
                let ptr = (high as u16) << 8 | (low as u16);
                // print!("{:04X} = ", ptr);
                let output = ptr.wrapping_add(self.reg_y as u16);
                // print!("{:04X} = ", ptr);
                output
            }
            AddressingMode::NoneAddressing => {
                panic!("Mode {:?} is not supported", mode);
            }
        }
    }

    // Memory related functions

    pub fn mem_read(&mut self, addr: u16) -> u8 {
        self.bus.mem_read(addr)
    }

    pub fn mem_write(&mut self, addr: u16, data: u8) {
        self.bus.mem_write(addr, data);
    }

    pub fn mem_read_u16(&mut self, addr: u16) -> u16 {
        self.bus.mem_read_u16(addr)
    }

    pub fn mem_write_u16(&mut self, addr: u16, data: u16) {
        self.bus.mem_write_u16(addr, data);
    }

    // TESTING READS
    pub fn mem_peek(&self, addr: u16) -> u8 {
        self.bus.mem_peek(addr)
    }

    pub fn mem_peek_u16(&self, addr: u16) -> u16 {
        self.bus.mem_peek_u16(addr)
    }

    // Push one byte to the stack and adjust SP
    fn stack_push(&mut self, data: u8) {
        let full_sp: u16 = 0x0100 + (self.sp as u16);

        self.mem_write(full_sp , data);

        self.sp = self.sp.wrapping_sub(1);
    }

    // Move SP 1 to make room for 2 byes, push, move SP 1 more to free byte
    fn stack_push_u16(&mut self, data: u16) {
        self.sp = self.sp.wrapping_sub(1);
        let full_sp = 0x0100 + (self.sp as u16);

        self.mem_write_u16(full_sp , data);

        self.sp = self.sp.wrapping_sub(1);
    }

    fn stack_read(&mut self) -> u8 {
        let full_sp: u16 = 0x0100 + (self.sp as u16);
        let output = self.mem_read(full_sp);
        output
    }

    fn stack_read_u16(&mut self) -> u16 {
        let full_sp: u16 = 0x0100 + (self.sp as u16);
        let output = self.mem_read_u16(full_sp);
        output
    }

    fn stack_pop(&mut self) {
        self.sp = self.sp.wrapping_add(1);
    }

    pub fn run_rom(&mut self) {
        self.reset();
        self.run();
    }

    pub fn reset(&mut self) {
        self.reg_a = 0;
        self.reg_x = 0;
        self.status = 0b0010_0000;
        self.sp = 0xFF;

        self.pc = self.mem_read_u16(0xFFFC);
        // self.pc = 0x8000; // for testing
    }

    pub fn run(&mut self) {
        self.run_with_callback(|_: &mut CPU| {});
    }

    fn conditional_cycle_check(&mut self, addr: u16, offset: u8) {
        if self.is_page_cross(addr, offset) {
            self.extra_cycles += 2
        } else {
            self.extra_cycles += 1;
        }
    }

    fn is_page_cross(&self, addr: u16, offset: u8) -> bool {
        (addr & 0xFF00) != ((addr.wrapping_add(offset as u16)) & 0xFF00)
    }

    fn calc_page_cycles(&mut self, mode: &AddressingMode) -> usize {

        match mode {
            AddressingMode::Absolute_X => {
                let addr = self.mem_read_u16(self.pc);
                if self.is_page_cross(addr, self.reg_x) {
                    return 1;
                }
            },
            AddressingMode::Absolute_Y => {
                let addr = self.mem_read_u16(self.pc);
                if self.is_page_cross(addr, self.reg_y) {
                    return 1;
                }
            },
            AddressingMode::Indirect_Y => {
                let addr = self.mem_read(self.pc);

                let low = self.mem_read(addr as u16);
                let high = self.mem_read((addr as u8).wrapping_add(1) as u16);
                let ptr = (high as u16) << 8 | (low as u16);
                if self.is_page_cross(ptr, self.reg_y) {
                    return 1;
                }
            },
            _ => panic!("Passed an invalid addressing mode for a cycle calculation!")
        }

        // Return no extra cycles if we never found a reason to add some
        return 0;
    }

    fn interrupt_nmi(&mut self) {
        self.stack_push_u16(self.pc);
        let mut flag = self.status.clone();
        flag = flag & 0b1110_1111; // zero break bit for nmi interrupts
        flag = flag | 0b0010_0000; // always set unused break2 bit to 1 (idk why lol)

        self.stack_push(flag);
        self.status = self.status | 0b0000_0100; // Disable IRQ interrupts until cpu finishes

        self.bus.tick(2); // Standard tick time of processing an NMI interrupt
        self.pc = self.mem_read_u16(0xFFFA); // Set the pc to run whatever instruction our ROM runs on NMI interrupts
    }

    fn interrupt_irq(&mut self) {
        self.stack_push_u16(self.pc + 1); // +1 since BRK lies abouts its opcode size by 1
        let mut flag = self.status.clone();
        flag = flag| 0b0001_0000; // set break bit for irq interrupts
        flag = flag | 0b0010_0000; // always set unused break2 bit to 1 (idk why lol)

        self.stack_push(flag);
        self.status = self.status | 0b0000_0100; // Disable IRQ interrupts until cpu finishes

        self.bus.tick(2); // Standard tick time of processing an NMI interrupt
        self.pc = self.mem_read_u16(0xFFFE); // Set the pc to run whatever instruction our ROM runs on NMI interrupts
    }

    fn new_trace_status(&mut self, op_code: &OpCode, old_pc: u16) {

    }

    fn trace_status(&mut self, op_code: &OpCode, old_pc: u16) {
        // old_pc should be the PC pointing to the instruction

        // PC REGISTER
        print!("${:04X} ", old_pc);
        let mut cur_addr = old_pc; 

        // CPU opcode
        let mut num_instructions = op_code.bytes;
        for i in 0..3 {
            if num_instructions != 0 {
                num_instructions -= 1;
                print!("{:02X} ", self.mem_read(cur_addr));
                cur_addr = cur_addr.wrapping_add(1);
            } else {
                print!("   ");
            }
        }

        // ASSEMBLY CPU OPCODE

        // get the name of instruction
        print!("{} ", op_code.code);

        cur_addr = old_pc + 1;

        // Untranslated value of PC for arguments
        if op_code.bytes == 0 {
            print!("");
        } else {
            let ptr = self.debug_operand(cur_addr, &op_code.addressing_mode);
            let output = self.mem_read(ptr);
            print!("{} ", output);
        }

        // ALL CPU REGISTERS
        print!("A:{:02X} ", self.reg_a);
        print!("X:{:02X} ", self.reg_x);
        print!("Y:{:02X} ", self.reg_y);
        print!("SP:{:02X} ", self.sp);
        print!("S:{:08b} ", self.status);

        // PPU STATUS
        print!("PPU: ");
        print!("SL: {} ", self.bus.ppu.scanline);
        print!("CYC: {}", self.bus.ppu.cycles);

        println!("");
        /*
        FINISH IMPLEMENTING THIS BEFORE CONTINUING FURTHER
        SEE SECTION 5.1 of text book to see what else I should do.
        I'm currently trying to implement the third column  
         */

    }

    pub fn run_with_callback<F>(&mut self, mut callback: F) 
        where
            F: FnMut(&mut CPU),
        {
            loop {
                callback(self);

                let nmi_stat: bool = self.bus.poll_nmi_status();
                // println!("nmi stat from cpu {}", nmi_stat);
                if nmi_stat { // Check if there's an NMI interrupt and execute one
                    // println!("Interrupt triggered!!!");
                    self.interrupt_nmi();
                }

                // Read the current opcode in binary and convert using our table
                let opscode = self.mem_read(self.pc);
                if opscode != 0xEA {
                    // println!("Grabbing opscode 0x{:02X} at 0x{:04X} on the pc", self.mem_read(self.pc), self.pc);
                }
                let op_object: &OpCode = OPCODE_TABLE.get(&opscode).unwrap();

                // self.trace_status(op_object, self.pc);

                // Move the program counter to point to the next address after opscode
                self.pc += 1;

                // Calculate extra cycles due to page crossing
                if PAGE_CROSSERS.contains(&opscode) {
                    self.extra_cycles += self.calc_page_cycles(&op_object.addressing_mode);
                }

                // Match to the corresponding opscode and run that function
                if opscode != 0xEA {
                    // println!("Running instruction {}", op_object.code);
                }

                // Decides if the standard program counter increment should take place
                // We don't increment for stuff like JMP that manually set the PC
                let mut should_inc: bool = true;

                match op_object.code {
                    "LDA" => self.lda(&op_object.addressing_mode),
                    "BRK" => return, // should call brk() but fails to pass test cases w/o return
                    "TAX" => self.tax(),
                    "INX" => self.inx(),
                    "CLC" => self.clc(),
                    "SEC" => self.sec(),
                    "ASL" => self.asl(&op_object.addressing_mode),
                    "AND" => self.and(&op_object.addressing_mode),
                    "ADC" => self.adc(&op_object.addressing_mode),
                    "BCC" => self.bcc(),
                    "BCS" => self.bcs(),
                    "BEQ" => self.beq(),
                    "BMI" => self.bmi(),
                    "BNE" => self.bne(),
                    "BPL" => self.bpl(),
                    "BIT" => self.bit(&op_object.addressing_mode),
                    "BVC" => self.bvc(),
                    "BVS" => self.bvs(),
                    "CLD" => self.cld(),
                    "CLV" => self.clv(),
                    "CLI" => self.cli(),
                    "CPX" => self.cpx(&op_object.addressing_mode),
                    "CPY" => self.cpy(&op_object.addressing_mode),
                    "CMP" => self.cmp(&op_object.addressing_mode),
                    "LDX" => self.ldx(&op_object.addressing_mode),
                    "LDY" => self.ldy(&op_object.addressing_mode),
                    "DEC" => self.dec(&op_object.addressing_mode),
                    "DEX" => self.dex(),
                    "DEY" => self.dey(),
                    "EOR" => self.eor(&op_object.addressing_mode),
                    "INC" => self.inc(&op_object.addressing_mode),
                    "INY" => self.iny(),
                    "JMP" => {
                        should_inc = self.jmp(&op_object.addressing_mode);
                    },
                    "JSR" => {
                        should_inc = self.jsr(&op_object.addressing_mode);
                    },
                    "RTS" => {
                        should_inc = self.rts();
                    },
                    "LSR" => self.lsr(&op_object.addressing_mode),
                    "NOP" => {},
                    "ORA" => self.ora(&op_object.addressing_mode),
                    "PHA" => self.pha(),
                    "PHP" => self.php(),
                    "PLA" => self.pla(),
                    "PLP" => self.plp(),
                    "ROL" => self.rol(&op_object.addressing_mode),
                    "ROR" => self.ror(&op_object.addressing_mode),
                    "RTI" => {
                        should_inc = self.rti();
                    },
                    "SBC" => self.sbc(&op_object.addressing_mode),
                    "SED" => self.sed(),
                    "SEI" => self.sei(),
                    "STA" => self.sta(&op_object.addressing_mode),
                    "STX" => self.stx(&op_object.addressing_mode),
                    "STY" => self.sty(&op_object.addressing_mode),
                    "TAY" => self.tay(),
                    "TSX" => self.tsx(),
                    "TXA" => self.txa(),
                    "TXS" => self.txs(),
                    "TYA" => self.tya(),
                    _ => panic!("Returned op_code: \"{}\" is not yet implemented...", op_object.code)
                }

                // Handle number of ticks to move
                // println!("adding cycles base {} + extra {} to cpu cycles", op_object.cycles, self.extra_cycles);
                self.bus.tick(op_object.cycles + self.extra_cycles);

                // Reset extra cycles from last instruction
                if self.extra_cycles > 0 {
                    self.extra_cycles = 0;
                }

                // Increment the program counter depending on the addressing mode
                // println!("Performing a pc increment from {} to {}", self.pc, self.pc + (op_object.bytes - 1) as u16);
                // println!("What we add: {}", (op_object.bytes - 1) as u16);
                if should_inc {
                    self.pc = self.pc.wrapping_add((op_object.bytes - 1) as u16);
                }
            }
    }

    // Begin instruction set implementations

    fn brk(&mut self) {
        self.interrupt_irq();
    }

    fn lda(&mut self, mode: &AddressingMode) {
        let addr = self.get_opperand_address(mode);

        self.reg_a = self.mem_read(addr);
        self.update_z_and_n_flags(self.reg_a);
    }

    fn tax(&mut self) {
        self.reg_x = self.reg_a;
        self.update_z_and_n_flags(self.reg_x);
    }

    fn inx(&mut self) {
        self.reg_x = self.reg_x.wrapping_add(1);

        self.update_z_and_n_flags(self.reg_x);
    }

    fn iny(&mut self) {
        self.reg_y = self.reg_y.wrapping_add(1);

        self.update_z_and_n_flags(self.reg_y);
    }

    fn clc(&mut self) {
        self.status = self.status & 0b1111_1110;
    }

    fn sec(&mut self) {
        self.status = self.status | 0b0000_0001;
    }

    fn adc(&mut self, mode: &AddressingMode) {
        let addr = self.get_opperand_address(mode);
        let param = self.mem_read(addr);

        self.add_carry(param);
    }

    fn and(&mut self, mode: &AddressingMode) {
        let addr = self.get_opperand_address(mode);
        let param = self.mem_read(addr);

        self.reg_a = self.reg_a & param;

        self.update_z_and_n_flags(self.reg_a);
    }

    fn asl(&mut self, mode: &AddressingMode) {
        // Set default to working on accumulator
        let mut param = self.reg_a;

        // If we have a non A addressing mode handle it
        if !matches!(mode, AddressingMode::NoneAddressing) {
            let addr = self.get_opperand_address(mode);
            param = self.mem_read(addr);
        }

        // Shift our data
        let output = param << 1;

        // If we have a carry
        if (param & 0b1000_0000) == 0b1000_0000 {
            self.sec();
        } else {
            self.clc();
        }

        // Set status flags
        self.update_z_and_n_flags(output);

        // If we're modifying memory
        if !matches!(mode, AddressingMode::NoneAddressing) {
            let addr = self.get_opperand_address(mode);
            self.mem_write(addr, output);
        } else { // modifying accumultor
            self.reg_a = output;
        }
    } 

    fn bcc(&mut self) {
        // If carry flag is clear, branch pc
        if (self.status ^ 0b0000_0001) & 0b0000_0001 == 0b0000_0001 { 
            let offset: i8 = self.mem_read(self.pc) as i8;
            self.conditional_cycle_check(self.pc, offset as u8);
            self.pc = self.pc.wrapping_add(offset as u16);
        }
    }

    fn bcs(&mut self) {
        // If carry flag is set, branch pc
        if (self.status & 0b0000_0001) == 0b0000_0001 {
            let offset: i8 = self.mem_read(self.pc) as i8;
            self.conditional_cycle_check(self.pc, offset as u8);
            self.pc = self.pc.wrapping_add(offset as u16);
        }
    }

    fn beq(&mut self) {
        if (self.status & 0b0000_00010) == 0b0000_0010 {
            let offset: i8 = self.mem_read(self.pc) as i8;
            self.conditional_cycle_check(self.pc, offset as u8);
            self.pc = self.pc.wrapping_add(offset as u16);
        }
    }

    fn bit(&mut self, mode: &AddressingMode) {
        let addr = self.get_opperand_address(mode);
        let param = self.mem_read(addr);

        if param & 0b1000_0000 == 0b1000_0000 {
            // println!("BIT read an $2002 address with 0b1000_0000!!")
        }

        let output = param & self.reg_a;

        if output == 0 {
            self.update_z_flag(true);
        } else {
            // Update Z flag
            self.update_z_flag(false);
        }

        // Update N flag
        if (param & 0b1000_0000) == 0b1000_0000 {
            self.update_n_flag(true);
            // println!("NEGATIVE FLAG ON?? ======================================================");
            self.test = true;
        } else {
            self.update_n_flag(false);
            self.test = false;
        }

        if self.test {
            // println!("bit after n status flag: 0b{:08b}", self.status);
        }

        // Update O flag
        if (param & 0b0100_0000) == 0b0100_0000 {
            self.update_o_flag(true);
        } else {
            self.update_o_flag(false);
        }

        if self.test {
            // println!("bit end status flag: 0b{:08b}", self.status);
        }
    }

    fn bmi(&mut self) {
        if (self.status & 0b1000_0000) == 0b1000_0000 {
            let offset: i8 = self.mem_read(self.pc) as i8;
            self.conditional_cycle_check(self.pc, offset as u8);
            self.pc = self.pc.wrapping_add(offset as u16);
        }
    }

    fn bne(&mut self) {
        if (self.status ^ 0b0000_0010) & 0b0000_0010 == 0b0000_0010 { 
            let offset: i8 = self.mem_read(self.pc) as i8;
            self.conditional_cycle_check(self.pc, offset as u8);
            self.pc = self.pc.wrapping_add(offset as u16);
        }
    }

    fn bpl(&mut self) {
        if self.test {
            // println!("bpl run status flag: 0b{:08b}", self.status);
        }
        if (self.status ^ 0b1000_0000) & 0b1000_0000 == 0b1000_0000 {
            // println!("negative flag is clear in bpl!");
            
            let offset: i8 = self.mem_read(self.pc) as i8;
            self.conditional_cycle_check(self.pc, offset as u8);
            self.pc = self.pc.wrapping_add(offset as u16);
        } else {
            // println!("Branch should have happened due to negative bit PPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPP");
        }
    }

    fn bvc(&mut self) {
        if (self.status ^ 0b0100_0000) & 0b0100_0000 == 0b0100_0000 { 
            let offset: i8 = self.mem_read(self.pc) as i8;
            self.conditional_cycle_check(self.pc, offset as u8);
            self.pc = self.pc.wrapping_add(offset as u16);
        }
    }

    fn bvs(&mut self) {
        // If carry flag is set, branch pc
        if (self.status & 0b0100_0000) == 0b0100_0000 {
            let offset: i8 = self.mem_read(self.pc) as i8;
            self.conditional_cycle_check(self.pc, offset as u8);
            self.pc = self.pc.wrapping_add(offset as u16);
        }
    }

    fn cld(&mut self) {
        self.status = self.status & 0b1111_0111;
    }

    fn cli(&mut self) {
        self.status = self.status & 0b1111_1011;
    }

    fn clv(&mut self) {
        self.status = self.status & 0b1011_1111;
    }

    fn cmp(&mut self, mode: &AddressingMode) {
        let addr = self.get_opperand_address(mode);
        let param = self.mem_read(addr);
        self.compare(self.reg_a, param);
    }

    fn cpx(&mut self, mode: &AddressingMode) {
        let addr = self.get_opperand_address(mode);
        let param = self.mem_read(addr);
        self.compare(self.reg_x, param);
    }
    
    fn cpy(&mut self, mode: &AddressingMode) {
        let addr = self.get_opperand_address(mode);
        let param = self.mem_read(addr);
        self.compare(self.reg_y, param);
    }

    fn ldx(&mut self, mode: &AddressingMode) {
        let addr = self.get_opperand_address(mode);

        self.reg_x = self.mem_read(addr);
        self.update_z_and_n_flags(self.reg_x);
    }

    fn ldy(&mut self, mode: &AddressingMode) {
        let addr = self.get_opperand_address(mode);

        self.reg_y = self.mem_read(addr);
        self.update_z_and_n_flags(self.reg_y);
    }

    fn dec(&mut self, mode: &AddressingMode) {
        let addr = self.get_opperand_address(mode);
        let param = self.mem_read(addr);
        let output = self.decrement(param);

        self.update_z_and_n_flags(output);

        self.mem_write(addr, output);
    }

    fn dex(&mut self) {
        self.reg_x = self.decrement(self.reg_x);
        self.update_z_and_n_flags(self.reg_x);
    }

    fn dey(&mut self) {
        self.reg_y = self.decrement(self.reg_y);
        self.update_z_and_n_flags(self.reg_y);
    }

    fn eor(&mut self, mode: &AddressingMode) {
        let addr = self.get_opperand_address(mode);
        let param = self.mem_read(addr);
        let output = self.reg_a ^ param;
        self.reg_a = output;
        self.update_z_and_n_flags(self.reg_a);
    }

    fn inc(&mut self, mode: &AddressingMode) {
        let addr = self.get_opperand_address(mode);
        let param = self.mem_read(addr);

        let output = param.wrapping_add(1); 
        self.mem_write(addr, output);
        self.update_z_and_n_flags(output);
    }

    fn jmp(&mut self, mode: &AddressingMode) -> bool {
        let addr = self.get_opperand_address(mode);
        // println!("Address read by opperand: 0x{:04X}", addr);

        // Custom code for the 6502 error for indirect
        if matches!(mode, &AddressingMode::Absolute) {
            let next_addr = self.mem_read_u16(self.pc);

            // println!("next addr: 0x{:04X}", next_addr);

            // Only adjust if last byte is all ones of indirect address
            if next_addr & 0x00FF == 0x00FF {
                let bad_read_addr: u16 = next_addr & 0xFF00;
                // println!("bad_read_addr: 0x{:04X}", bad_read_addr);

                let hi: u8 = self.mem_read(bad_read_addr);
                let lo: u8 = self.mem_read(next_addr);

                let new_addr: u16 = ((hi as u16) << 8) + (lo as u16);
                // println!("new_addr: 0x{:04X}", new_addr);
                
                self.pc = new_addr;
            } else {
                self.pc = self.mem_read_u16(addr);
            }
        } else {
            self.pc = self.mem_read_u16(addr);
        }

        // Tell program not to auto increment
        false
    }

    fn jsr(&mut self, mode: &AddressingMode) -> bool {
        // println!("pc points to 0x{:04X} during jsr", self.pc);
        let addr = self.get_opperand_address(mode);
        // println!("JSR is attempting to jump to address: 0x{:04X}", addr);

        // Return address -1 is just next instruction -1
        let return_ptr = self.pc.wrapping_add(1);

        // Push return address to the stack
        self.stack_push_u16(return_ptr);

        // Update pc to given address
        self.pc = addr;

        false
    }

    fn rts(&mut self) -> bool {
        // Move the stack back to the next value and read it
        self.stack_pop();
        let ptr: u16 = self.stack_read_u16();
        self.stack_pop();
        

        let output = ptr.wrapping_add(1);
        self.pc = output;

        false
    }

    fn lsr(&mut self, mode: &AddressingMode) {
        // If we're modifying the accumulator or not
        if matches!(mode, AddressingMode::NoneAddressing) { // Accumulator
            if self.reg_a & 0b0000_0001 == 0b0000_0001 {
                self.update_c_bit(true);
            } else {
                self.update_c_bit(false);
            }
            self.reg_a = self.reg_a >> 1;
            self.update_z_and_n_flags(self.reg_a);
        } else {
            let addr = self.get_opperand_address(mode);
            let param = self.mem_read(addr);
            if param & 0b0000_0001 == 0b0000_0001 {
                self.update_c_bit(true);
            } else {
                self.update_c_bit(false);
            }

            let output = param >> 1;
            self.update_z_and_n_flags(output);
            self.mem_write(addr, output);
        }
    }

    fn ora(&mut self, mode: &AddressingMode) {
        let addr = self.get_opperand_address(mode);
        let param = self.mem_read(addr);
        let output = self.reg_a | param;
        self.reg_a = output;
        self.update_z_and_n_flags(self.reg_a);
    }

    fn pha(&mut self) {
        self.stack_push(self.reg_a);
    }

    fn php(&mut self) {
        self.update_b_flag(true);

        // Push all flags to the stack
        self.stack_push(self.status);
    }

    fn pla(&mut self) {
        self.stack_pop();
        self.reg_a = self.stack_read();
        self.update_z_and_n_flags(self.reg_a);
    }

    fn plp(&mut self) {
        self.stack_pop();
        self.status = self.stack_read();
    }

    fn rol(&mut self, mode: &AddressingMode) {
        // If we're modifying the accumulator or not
        if matches!(mode, AddressingMode::NoneAddressing) { // Accumulator
            let old_c: u8 = self.status & 0b0000_0001;
            if self.reg_a & 0b1000_0000 == 0b1000_0000 {
                self.update_c_bit(true);
            } else {
                self.update_c_bit(false);
            }

            self.reg_a = self.reg_a << 1;
            self.reg_a = self.reg_a | old_c;

            self.update_z_and_n_flags(self.reg_a);
        } else {
            let addr = self.get_opperand_address(mode);
            let param = self.mem_read(addr);

            let old_c: u8 = self.status & 0b0000_0001;
            if param & 0b1000_0000 == 0b1000_0000 {
                self.update_c_bit(true);
            } else {
                self.update_c_bit(false);
            }

            let mut output = param << 1;
            output = output | old_c;

            self.update_z_and_n_flags(output);
            self.mem_write(addr, output);
        }
    }

    fn ror(&mut self, mode: &AddressingMode) {
        // If we're modifying the accumulator or not
        if matches!(mode, AddressingMode::NoneAddressing) { // Accumulator
            let mut old_c: u8 = self.status & 0b0000_0001;
            old_c = old_c << 7;
            if self.reg_a & 0b0000_0001 == 0b0000_0001 {
                self.update_c_bit(true);
            } else {
                self.update_c_bit(false);
            }

            self.reg_a = self.reg_a >> 1;
            self.reg_a = self.reg_a | old_c;

            self.update_z_and_n_flags(self.reg_a);
        } else {
            let addr = self.get_opperand_address(mode);
            let param = self.mem_read(addr);
            
            let mut old_c: u8 = self.status & 0b0000_0001;
            old_c = old_c << 7;
            if param & 0b0000_0001 == 0b0000_0001 {
                self.update_c_bit(true);
            } else {
                self.update_c_bit(false);
            }

            let mut output = param >> 1;
            output = output | old_c;

            self.update_z_and_n_flags(output);
            self.mem_write(addr, output);
        }
    }

    fn rti(&mut self) -> bool {
        // Pull processor flags
        self.stack_pop();
        self.status = self.stack_read();

        // Pull program counter
        self.stack_pop();
        self.pc = self.stack_read_u16();
        self.stack_pop(); 

        // Tell loop not to increment 
        false
    }

    fn sbc(&mut self, mode: &AddressingMode) {
        let addr = self.get_opperand_address(mode);
        let param = self.mem_read(addr);

        /* Explanation for +2 in n_param
        
        SBC result should be like this
        A = A-M-(1-C)
        A = A-M-1+C

        but we want to use the orignal ADC code which does this
        A = A+M+C

        so we can convert M to get the right result
        -M = !M + 1
        -M-1 = !M

        substituting M for !M turns ADC code into perfect subtraction */
        let n_param = !param;
        self.add_carry(n_param);
    }

    fn add_carry(&mut self, param: u8) {
        // If carry bit is on already add it to sum
        let mut sum: u32 = (self.reg_a as u32) + (param as u32);
        if (self.status & 0b0000_0001) == 0b0000_0001 {
            sum += 1;
        }

        // Update carry bit
        if sum > 0xFF {
            self.update_c_bit(true);
        } else {
            self.update_c_bit(false);
        }

        // Shorten sum to u32
        let short_sum: u8 = sum as u8;

        // Check for overflow
        if (self.reg_a ^ short_sum) & (param ^ short_sum) & 0b1000_0000 == 0b1000_0000 {
            self.update_o_flag(true);
        } else {
            self.update_o_flag(false);
        }

        self.reg_a = short_sum;

        // Set other flags
        self.update_z_and_n_flags(self.reg_a);
    }

    fn sed(&mut self) {
        self.status = self.status | 0b0000_1000;
    }

    fn sei(&mut self) {
        self.status = self.status | 0b0000_0100;
    }

    fn sta(&mut self, mode: &AddressingMode) {
        let addr = self.get_opperand_address(mode);
        // println!("STA is Storing value 0b{:08b} at address 0x{:04X}", self.reg_a, addr);
        self.mem_write(addr, self.reg_a);
    }

    fn stx(&mut self, mode: &AddressingMode) {
        let addr = self.get_opperand_address(mode);
        self.mem_write(addr, self.reg_x);
    }

    fn sty(&mut self, mode: &AddressingMode) {
        let addr = self.get_opperand_address(mode);
        self.mem_write(addr, self.reg_y);
    }

    fn tay(&mut self) {
        self.reg_y = self.reg_a;
        self.update_z_and_n_flags(self.reg_y);
    }

    fn tsx(&mut self) {
        self.reg_x = self.sp;
        self.update_z_and_n_flags(self.reg_x);
    }

    fn txa(&mut self) {
        self.reg_a = self.reg_x;
        self.update_z_and_n_flags(self.reg_a);
    }

    fn txs(&mut self) {
        self.sp = self.reg_x;
    }

    fn tya(&mut self) {
        self.reg_a = self.reg_y;
        self.update_z_and_n_flags(self.reg_a);
    }

    fn decrement(&mut self, value: u8) -> u8 {
        let output: i8 = (value as i8).wrapping_sub(1);
        self.update_z_and_n_flags(output as u8);
        output as u8
    }

    fn compare(&mut self, val1: u8, val2: u8) {
        let result = val1.wrapping_sub(val2);
    
        // Carry: set if val1 >= val2
        if val1 >= val2 {
            self.update_c_bit(true);
        } else {
            self.update_c_bit(false);
        }
    
        self.update_z_and_n_flags(result);
    }

    fn update_z_and_n_flags(&mut self, value: u8) {
        // Set Z flag
        if value == 0 {
            self.status = self.status | 0b0000_0010;
        } else {
            self.status = self.status & 0b1111_1101;
        }

        // Set N flag
        if value & 0b1000_0000 != 0 {
            self.status = self.status | 0b1000_0000;
        } else {
            self.status = self.status & 0b0111_1111;
        }
    }

    fn update_n_flag(&mut self, status: bool) {
        if status {
            self.status = self.status | 0b1000_0000;
        } else {
            self.status = self.status & 0b0111_1111;
        }
    }

    fn update_z_flag(&mut self, status: bool) {
        if status {
            self.status = self.status | 0b0000_0010;
        } else {
            self.status = self.status & 0b1111_1101;
        }
    }

    fn update_o_flag(&mut self, status: bool) {
        if status {
            self.status = self.status | 0b0100_0000;
        } else {
            self.status = self.status & 0b1011_1111;
        }
    }

    fn update_b_flag(&mut self, status: bool) {
        if status {
            self.status = self.status | 0b0001_0000;
        } else {
            self.status = self.status & 0b1110_1111;
        }
    }

    fn update_c_bit(&mut self, status: bool) {
        if status {
            self.status = self.status | 0b0000_0001;
        } else {
            self.status = self.status & 0b1111_1110;
        }
    }

}













