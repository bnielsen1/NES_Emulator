
use crate::{ppu::NesPPU, rom::{Mirroring, Rom}};

const RAM: u16 = 0x0000;
const RAM_MIRRORS_END: u16 = 0x1FFF;
const PPU_REGISTERS: u16 = 0x2000;
const PPU_REGISTERS_MIRRORS_END: u16 = 0x3FFF;
const ROM_MEM_START: u16 = 0x8000;
const ROM_MEM_END: u16 = 0xFFFF;

// Generates a dummy rom for when a rom isn't needed
fn test_rom_gen() -> Rom {
    Rom {
        prg_rom: vec![0xEA; 0x4000], // NOPs
        chr_rom: vec![],
        mapper: 0,
        screen_mirroring: Mirroring::HORIZONTAL,
    }
}

pub struct Bus {
    cpu_vram: [u8; 2048],
    pub prg_rom: Vec<u8>,
    ppu: NesPPU
}

impl Bus {
    pub fn new(rom: Rom) -> Self {
        Bus {
            cpu_vram: [0; 2048],
            prg_rom: rom.prg_rom.clone(),
            ppu: NesPPU::new(rom.chr_rom.clone(), rom.screen_mirroring)
        }
    }

    // Call instead of new if you don't need to use a ROM
    pub fn new_fake_rom() -> Self {
        let temp_rom = test_rom_gen();

        Bus {
            cpu_vram: [0; 2048],
            prg_rom: temp_rom.prg_rom.clone(),
            ppu: NesPPU::new(temp_rom.chr_rom.clone(), temp_rom.screen_mirroring)
        }
    }

    pub fn read_prg_rom(&self, mut addr: u16) -> u8 {
        addr -= ROM_MEM_START;
        // Remember 0x4000 == 16kB (a standard size for prg)
        if self.prg_rom.len() == 0x4000 && addr >= 0x4000 {
            addr = addr % 0x4000;
        }
        self.prg_rom[addr as usize]
    }

    // Used only for test cases
    fn write_prg_rom(&mut self, mut addr: u16, data: u8) {
        addr -= ROM_MEM_START;
        // Remember 0x4000 == 16kB (a standard size for prg)
        if self.prg_rom.len() == 0x4000 && addr >= 0x4000 {
            addr = addr % 0x4000;
        }
        self.prg_rom[addr as usize] = data;
    }
}


pub trait Mem {
    fn mem_read(&mut self, addr: u16) -> u8;
    fn mem_write(&mut self, addr: u16, data: u8);
    fn mem_read_u16(&mut self, addr: u16) -> u16;
    fn mem_write_u16(&mut self, addr: u16, data: u16);
    fn mem_write_test(&mut self, addr: u16, data: u8);
}

// 

impl Mem for Bus {
    fn mem_read(&mut self, addr: u16) -> u8 {
        match addr {
            RAM ..= RAM_MIRRORS_END => {
                let mirrored_addr = addr & 0b00000111_11111111;
                self.cpu_vram[mirrored_addr as usize]
            }
            0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 | 0x4014 => {
                panic!("Attempt to read from write-only PPU address {:x}", addr);
            }
            0x2007 => self.ppu.read_data(),
            0x2008 ..= PPU_REGISTERS_MIRRORS_END => {
                // Recall function with address properly mirrored
                let mirrored_addr = addr &0b0010000_00000111;
                todo!("PPU IS NOT YET SUPPORTED")
            }
            ROM_MEM_START ..= ROM_MEM_END => {
                self.read_prg_rom(addr)
            }
            _ => {
                println!("Attempted to read memory at unknown address 0x{:04X}", addr);
                0
            }
        }
    }

    fn mem_read_u16(&mut self, addr: u16) -> u16 {
        let lo = self.mem_read(addr) as u16;
        let hi = self.mem_read(addr + 1) as u16;
        (hi << 8) | lo
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        match addr {
            RAM ..= RAM_MIRRORS_END => {
                let mirrored_addr = addr & 0b00000111_11111111;
                self.cpu_vram[mirrored_addr as usize] = data;
            }
            0x2000 => {
                self.ppu.write_to_ctrl(data);
            }
            0x2006 => {
                self.ppu.write_to_ppu_addr(data);
            }
            0x2007 => {
                self.ppu.write_to_data(data);
            }
            0x2008 ..= PPU_REGISTERS_MIRRORS_END => {
                let mirrored_addr = addr &0b0010000_00000111;
                todo!("PPU IS NOT YET SUPPORTED")
            }
            ROM_MEM_START ..= ROM_MEM_END => {
                panic!("Attempted to write to Cartridge ROM space!!!")
            }
            _ => {
                println!("Attempted to write memory at unknown address 0x{:04X}", addr);
            }
        }
    }

    // Allows writing to cartridge ROM space (ONLY USED FOR TEST CASES)
    fn mem_write_test(&mut self, addr: u16, data: u8) {
        self.write_prg_rom(addr, data);
    }

    fn mem_write_u16(&mut self, addr: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0x00ff) as u8;

        self.mem_write(addr, lo);
        self.mem_write(addr + 1, hi);
    }
}