
use crate::{joypad, ppu::NesPPU, rom::{Mirroring, Rom}};
use crate::joypad::Joypad;

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

pub struct Bus<'call> {
    cpu_vram: [u8; 2048],
    joypad1: Joypad,
    pub prg_rom: Vec<u8>,
    ppu: NesPPU,
    cycles: usize,
    gameloop_callback: Box<dyn FnMut(&NesPPU, &mut Joypad) + 'call>,
}

impl<'a> Bus<'a> {
    pub fn new<'call, F>(rom: Rom, gameloop_callback: F) -> Bus<'call>
    where
        F: FnMut(&NesPPU, &mut Joypad) + 'call,
    {
        let ppu = NesPPU::new(rom.chr_rom.clone(), rom.screen_mirroring);

        Bus {
            cpu_vram: [0; 2048],
            joypad1: Joypad::new(),
            prg_rom: rom.prg_rom.clone(),
            ppu: ppu,
            cycles: 0,
            gameloop_callback: Box::from(gameloop_callback),
        }
    }

    pub fn tick(&mut self, cycles: usize) {
        // println!("bus cycles: {}", self.cycles);
        self.cycles += cycles;


        // Read NMI status before and after a ppu clock cycle to see
        // if we just entered VBlank -> meaning a screen is ready to be rendered
        let nmi_before = self.ppu.trigger_nmi;
        for _ in 0..3 {
            self.ppu.tick(cycles); // ppu ticks 3 times faster than CPU
        }
        let nmi_after = self.ppu.trigger_nmi;

        // Call the gameloop function which will handle rendering other possible inputs
        if !nmi_before && nmi_after {
            (self.gameloop_callback)(&self.ppu, &mut self.joypad1);
        }
    }

    // Call instead of new if you don't need to use a ROM
    // pub fn new_fake_rom() -> Self {
    //     let temp_rom = test_rom_gen();

    //     Bus {
    //         cpu_vram: [0; 2048],
    //         prg_rom: temp_rom.prg_rom.clone(),
    //         ppu: NesPPU::new(temp_rom.chr_rom.clone(), temp_rom.screen_mirroring),
    //         cycles: 0,
    //     }
    // }

    pub fn new_fake_rom<'call, F>(gameloop_callback: F) -> Bus<'call>
    where
        F: FnMut(&NesPPU, &mut Joypad) + 'call,
    {
        let temp_rom = test_rom_gen();
        let ppu = NesPPU::new(temp_rom.chr_rom.clone(), temp_rom.screen_mirroring);

        Bus {
            cpu_vram: [0; 2048],
            joypad1: Joypad::new(),
            prg_rom: temp_rom.prg_rom.clone(),
            ppu: ppu,
            cycles: 0,
            gameloop_callback: Box::from(gameloop_callback),
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

    pub fn poll_nmi_status(&mut self) -> bool {
        let output = self.ppu.get_nmi_status();
        if output { 
            // println!("bus nmi poll gets true");
        }
        output
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

impl Mem for Bus<'_> {
    fn mem_read(&mut self, addr: u16) -> u8 {
        match addr {
            RAM ..= RAM_MIRRORS_END => {
                let mirrored_addr = addr & 0b00000111_11111111;
                self.cpu_vram[mirrored_addr as usize]
            }
            0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 | 0x4014 => {
                panic!("Attempt to read from write-only PPU address 0x{:04X}", addr);
            }
            0x2002 => self.ppu.read_status(),
            0x2004 => self.ppu.oam_data_read(),
            0x2007 => self.ppu.read_data(),
            0x2008 ..= PPU_REGISTERS_MIRRORS_END => {
                // Recall function with address properly mirrored
                let mirrored_addr = addr &0b0010000_00000111;
                self.mem_read(mirrored_addr)
            }
            ROM_MEM_START ..= ROM_MEM_END => {
                self.read_prg_rom(addr)
            }
            0x4016 => {
                self.joypad1.read()
            }
            0x4017 => {
                // this is controller 2 which is not implemented yet
                0
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
            0x2000 => self.ppu.write_to_ctrl(data),
            0x2001 => self.ppu.write_mask(data),
            0x2002 => {
                panic!("Attempt to write to read only PPU address 0x{:04X}", addr);
            }
            0x2003 => self.ppu.oam_addr_write(data),
            0x2004 => self.ppu.oam_data_write(data),
            0x2005 => self.ppu.write_scroll(data),
            0x2006 => self.ppu.write_to_ppu_addr(data),
            0x2007 => self.ppu.write_to_data(data),
            0x2008 ..= PPU_REGISTERS_MIRRORS_END => {
                let mirrored_addr = addr &0b0010000_00000111;
                self.mem_write(mirrored_addr, data);
            }
            ROM_MEM_START ..= ROM_MEM_END => {
                panic!("Attempted to write to Cartridge ROM space address: 0x{:04X}", addr)
            }
            0x4000 | 0x4001 | 0x4002 | 0x4003 | 0x4006 | 0x4005 | 0x4007 | 0x4004 => {
                // APU IGNORE
            }
            0x4014 => {
                let cpu_addr = (data as u16) << 8;
                let mut data = [0u8; 256];

                for i in 0..256u16 {
                    data[i as usize] = self.mem_read(cpu_addr + i);
                }
                self.ppu.oam_dma_write(&data);

                // to do: handle added cycles due to this action as seen on nesdev wiki for 0x4014
            }
            0x4016 => {
                self.joypad1.write(data);
            }
            0x4017 => {
                // this is controller 2 which is not implemented yet
            }
            _ => {
                println!("Attempted to write memory at unknown address 0x{:04X}", addr);
                // println!("^^ Above message is likely due to the lack of APU")
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::rom::{Rom, test};

    #[test]
    fn test_mem_read_write_to_ram() {
        let mut bus = Bus::new(test::test_rom(), |ppu, joypad1| {});
        bus.mem_write(0x01, 0x55);
        assert_eq!(bus.mem_read(0x01), 0x55);
    }
}