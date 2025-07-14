use crate::mapper::Mapper;
use crate::rom::Mirroring;

pub struct Mapper0 {
    pub prg_rom: Vec<u8>,
    pub prg_ram: Vec<u8>,
    pub chr_rom: Vec<u8>,
    mirroring: Mirroring,
    chr_is_ram: bool,
}

impl Mapper0 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, mirroring: Mirroring, chr_is_ram: bool) -> Self {
        Mapper0 {
            prg_rom: prg_rom,
            prg_ram: vec![0; 0x2000],
            chr_rom: chr_rom,
            mirroring: mirroring,
            chr_is_ram: chr_is_ram
        }
    }
}

impl Mapper0 {
    fn prg_ram_read(&self, mut addr: u16) -> u8 {
        addr = addr & 0x0FFF;
        self.prg_ram[addr as usize]
    }
}

// https://www.nesdev.org/wiki/NROM
impl Mapper for Mapper0 {
    // Default implementations mostly for test cases
    fn get_prg_rom(&self) -> Vec<u8> {
        self.prg_rom.clone()
    }

    fn get_chr_rom(&self) -> Vec<u8> {
        self.chr_rom.clone()
    }

    fn get_mapping(&self) -> u8 {
        0
    }

    fn get_mirroring(&self) -> Mirroring {
        self.mirroring.clone()
    }

    fn read_chr_rom(&self, index: usize) -> u8 {
        self.chr_rom[index]
    }

    fn read_prg_rom(&self, index: usize) -> u8 {
        self.prg_rom[index]
    }

    // Mapper specific
    fn cpu_read(&self, mut addr: u16) -> u8 {
        match addr {
            0x6000..=0x7FFF => {
                addr = addr % 0x2000;
                self.prg_ram_read(addr)
            }
            0x8000..=0xFFFF => {
                addr -= 0x8000; // Index from 0
                // Remember 0x4000 == 16kB (a standard size for prg)
                if self.prg_rom.len() == 0x4000 && addr >= 0x4000 {
                    addr = addr % 0x4000;
                }
                self.prg_rom[addr as usize]
            }
            _ => panic!("CPU READ to invalid address MAPPER 0")
        }
    }

    fn cpu_write(&mut self, _addr: u16, _data: u8) {
        // NROM PRG ROM is read-only
        panic!("CPU WRITE TO PRG ROM IN MAPPER 0 NOT ALLOWED (might not want to panic this)")
    }

    fn ppu_read(&self, addr: u16) -> u8 {
        if addr < 0x2000 {
            self.chr_rom[addr as usize]
        } else {
            panic!("Invalid ppu read address for mapper0")
        }
    }

    fn ppu_write(&mut self, addr: u16, data: u8) {
        if self.chr_is_ram && addr < 0x2000 {
            self.chr_rom[addr as usize] = data;
        } else {
            panic!("Invalid ppu write address for mapper0")
        }
    }
}