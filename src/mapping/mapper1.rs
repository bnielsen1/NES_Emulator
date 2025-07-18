use crate::mapper::Mapper;
use crate::rom::Mirroring;

pub struct Mapper1 {
    pub prg_rom: Vec<u8>,
    pub prg_ram: Vec<u8>,
    pub chr_rom: Vec<u8>,

    shift_register: u8, // Use only 5 bits of this register
    shift_count: usize, // Counter to track how many writes done to shift

    // Mapper registers
    control: u8, //        Control (internal, $8000-$9FFF)
    chr_bank_0: u8, //     CHR bank 0 (internal, $A000-$BFFF)
    chr_bank_1: u8, //     CHR bank 1 (internal, $C000-$DFFF)
    prg_bank: u8, //       PRG bank (internal, $E000-$FFFF)

    // Modes extracted from control bits upon control register update
    prg_rom_bank_mode: u8,
    chr_rom_bank_mode: u8,

    // Rom offsets extracted from bank registers upon update
    // Used as index offsets when reading from rom memory
    prg_bank_offset_first: usize,
    prg_bank_offset_second: usize,
    chr_bank_0_offset: usize,
    chr_bank_1_offset: usize,

    mirroring: Mirroring,
    chr_is_ram: bool,
}

impl Mapper1 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, mirroring: Mirroring, chr_is_ram: bool) -> Self {
        Mapper1 {
            prg_rom: prg_rom,
            prg_ram: vec![0; 0x2000],
            chr_rom: chr_rom,

            shift_register: 0,
            shift_count: 0,

            control: 0x0C,
            chr_bank_0: 0,
            chr_bank_1: 0,
            prg_bank: 0b0001_0000,

            prg_rom_bank_mode: 3,
            chr_rom_bank_mode: 0,

            prg_bank_offset_first: 0,
            prg_bank_offset_second: 0,
            chr_bank_0_offset: 0,
            chr_bank_1_offset: 0,


            mirroring: mirroring,
            chr_is_ram: chr_is_ram
        }
    }
}

/*
Control (internal, $8000-$9FFF)

4bit0
-----
CPPMM
|||||
|||++- Nametable arrangement: (0: one-screen, lower bank; 1: one-screen, upper bank;
|||               2: horizontal arrangement ("vertical mirroring", PPU A10); 
|||               3: vertical arrangement ("horizontal mirroring", PPU A11) )
|++--- PRG-ROM bank mode (0, 1: switch 32 KB at $8000, ignoring low bit of bank number;
|                         2: fix first bank at $8000 and switch 16 KB bank at $C000;
|                         3: fix last bank at $C000 and switch 16 KB bank at $8000)
+----- CHR-ROM bank mode (0: switch 8 KB at a time; 1: switch two separate 4 KB banks)
*/

impl Mapper1 {
    fn prg_ram_read(&self, addr: u16) -> u8 {
        self.prg_ram[addr as usize]
    }

    fn prg_ram_write(&mut self, addr: u16, data: u8) {
        self.prg_ram[addr as usize] = data;
    }

    fn update_banks(&mut self) {
        self.prg_rom_bank_mode = (self.control >> 2) & 0b11;
        self.chr_rom_bank_mode = (self.control >> 4) & 0b1;

        // Decide mirroring might have issues
        // Best fix could be to try is adding 2 modes to Mirroring enum for 0 and 1 cases
        let nametable_bits = self.control & 0b11;
        self.mirroring = match nametable_bits {
            0 => {
                // single screen first bank
                Mirroring::VERTICAL
            }
            1 => {
                // single screen second bank
                panic!("I dont think we can handle single screen second bank");
            }
            2 => {
                Mirroring::VERTICAL
            }
            3 => {
                Mirroring::HORIZONTAL
            }
            _ => panic!("Invalid mirroring value when updating banks in mapping mode 1")
        };

        let bank = (self.prg_bank & 0b0000_1111) as usize;
        let single_prg_bank_size = 0x4000; // 16 Kb

        match self.prg_rom_bank_mode {
            0 | 1 => {
                self.prg_bank_offset_first = (bank & 0b1110) * single_prg_bank_size;
                self.prg_bank_offset_second = self.prg_bank_offset_first + 0x4000;
            },
            2 => {
                // Fix first offset to beginning of prg
                self.prg_bank_offset_first = 0;
                // Set second to custom offset
                self.prg_bank_offset_second = bank * single_prg_bank_size;
            },
            3 => {
                // Switch first
                self.prg_bank_offset_first = bank * single_prg_bank_size;
                // Fix second to last bank of prg
                self.prg_bank_offset_second = ((self.prg_rom.len() / 0x4000) - 1) * single_prg_bank_size;
            },
            _ => panic!("Invalid prg rom bank setting in mapping mode 1 control bit")
        }

        let single_chr_bank_size = 0x1000;

        match self.chr_rom_bank_mode {
            0 => {
                // Set first bank and second bank based off first (8KB at once)
                self.chr_bank_0_offset = ((self.chr_bank_0 as usize) & 0b0001_1111) * single_chr_bank_size;
                self.chr_bank_1_offset = self.chr_bank_0_offset + 0x1000;
            },
            1 => {
                // Set each bank based off their own offset value
                self.chr_bank_0_offset = ((self.chr_bank_0 as usize) & 0b0001_1111) * single_chr_bank_size;
                self.chr_bank_1_offset = ((self.chr_bank_1 as usize) & 0b0001_1111) * single_chr_bank_size;
            },
            _ => panic!("Invalid chr rom bank setting in mapping mode 1 control bit")
        }

    }
}

// https://www.nesdev.org/wiki/NROM for details on mapping mode 0
impl Mapper for Mapper1 {
    // Default implementations mostly for test cases
    fn get_prg_rom(&self) -> Vec<u8> {
        self.prg_rom.clone()
    }

    fn get_chr_rom(&self) -> Vec<u8> {
        self.chr_rom.clone()
    }

    fn get_mapping(&self) -> u8 {
        1
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
            0x8000..=0xBFFF => {
                addr -= 0x8000;
                self.prg_rom[self.prg_bank_offset_first + addr as usize]
            }
            0xC000..=0xFFFF => {
                // println!("addr before offsetting: 0x{:04X}", addr);
                addr -= 0xC000; // Decrease more due to larger size of prg_bank_offset_second being larger
                // println!("CPU READ: prg_bank_offset: 0x{:04X}, offset from bank: 0x{:04X}.", self.prg_bank_offset_second, addr);
                self.prg_rom[self.prg_bank_offset_second + addr as usize]
            }
            _ => panic!("CPU READ to invalid address MAPPER 1")
        }
    }

    fn cpu_write(&mut self, mut addr: u16, data: u8) {
        // Check if we're completing a prg ram read before continuing
        match addr {
            0x6000..=0x7FFF => {
                addr = addr % 0x2000;
                self.prg_ram_write(addr, data);
                return
            },
            0x8000..=0xFFFF => {
                // println!("Performing CPU write on addr: 0x{:04X}", addr);
                // Reset shift when bit 7 is on
                if data & 0x80 != 0 {
                    self.shift_register = 0;
                    self.shift_count = 0;
                    self.control = 0x0C;
                    
                    self.update_banks();
                    return
                }

                // Otherwise insert lowest bit into shift register and shift
                self.shift_register = self.shift_register >> 1;
                self.shift_register |= (data & 0b0000_0001) << 4;
                self.shift_count += 1;

                // handle 5 shift (shift register filled)
                if self.shift_count == 5 {
                    let register_index = (addr - 0x8000) / 0x2000;
                    match register_index {
                        0 => {
                            self.control = self.shift_register & 0b0001_1111;
                        }
                        1 => {
                            self.chr_bank_0 = self.shift_register & 0b0001_1111;
                        }
                        2 => {
                            self.chr_bank_1 = self.shift_register & 0b0001_1111;
                        }
                        3 => {
                            self.prg_bank = self.shift_register & 0b0001_1111;
                        }
                        _ => panic!("Invalid register index value for cpu write in mapper 1")
                    }

                    // reset everything based on above changes
                    self.shift_count = 0;
                    self.shift_register = 0x0;
                    self.update_banks();
                }
            },
            _ => panic!("Invalid address 0x{:04X} passed to CPU write", addr)
        }
    }

    fn ppu_read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x0FFF => {
                return self.chr_rom[self.chr_bank_0_offset + addr as usize]
            }
            0x1000..=0x1FFF => {
                return self.chr_rom[self.chr_bank_1_offset + addr as usize]
            }
            _ => panic!("attempted to read from a ppu addr >= 0x2000 in mapper 1")
        }
    }

    fn ppu_write(&mut self, mut addr: u16, data: u8) {
        if self.chr_is_ram && addr < 0x2000 {
            match addr {
                0x0000..=0x0FFF => {
                    self.chr_rom[self.chr_bank_0_offset + addr as usize] = data;
                }
                0x1000..=0x1FFF => {
                    addr -= 0x1000;
                    self.chr_rom[self.chr_bank_1_offset + addr as usize] = data;
                }
                _ => panic!("attempted to read from a ppu addr >= 0x2000 in mapper 1")
            }
        } else {
            panic!("Invalid ppu write address for mapper0")
        }
    }
}