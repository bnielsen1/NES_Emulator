
use crate::rom::Mirroring;
use bitflags::bitflags;

// PPU Register -> Reg Title translation
// NOTE: These memory addresses are mapped to the CPU
/*
Controller == 0x2000 == Bit flags to store states/info
Addr == 0x2006 == Helps provide CPU access to PPU memory
Data == 0x2007 == ^^^^^
*/

pub struct NesPPU {
    pub chr_rom: Vec<u8>,
    pub palette_table: [u8; 32],
    pub vram: [u8; 2048],
    pub oam_data: [u8; 256],
    internal_data_buf: u8, // Storage for 0x2007 reads

    pub mirroring: Mirroring,

    addr: AddrRegister,
    pub ctrl: ControlRegister,
}

impl NesPPU {
    pub fn new(chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        NesPPU {
            chr_rom: chr_rom,
            mirroring: mirroring,
            internal_data_buf: 0,
            vram: [0; 2048],
            oam_data: [0; 64 * 4],
            palette_table: [0; 32],
            addr: AddrRegister::new(),
            ctrl: ControlRegister::new(),
        }
    }

    pub fn write_to_ppu_addr(&mut self, value: u8) {
        self.addr.update(value);
    }

    pub fn write_to_ctrl(&mut self, value: u8) {
        self.ctrl.update(value);
    }

    // Called upon 0x2007 writes or reads
    pub fn increment_vram_addr(&mut self) {
        self.addr.increment(self.ctrl.vram_addr_increment());
    }

    pub fn read_data(&mut self) -> u8 {
        let addr = self.addr.get();
        self.increment_vram_addr();

        match addr {
            0..=0x1FFF => {
                let result = self.internal_data_buf;
                self.internal_data_buf = self.chr_rom[addr as usize];
                result
            },
            0x2000..=0x2FFF => {
                let result = self.internal_data_buf;
                self.internal_data_buf = self.vram[self.mirror_vram_addr(addr) as usize];
                result  
            },
            0x3000..=0x3EFF => panic!("Addr space 0x3000..=0x3EFF is not expected to be used. Attempted to read 0x{:04X}", addr),
            0x3F00..=0x3FFF => {
                self.palette_table[(addr - 0x3F00) as usize]
            },
            _ => panic!("Unexpected read access to mirrored space {}", addr),
        }
    }

    pub fn write_to_data(&mut self, data: u8) {
        let addr = self.addr.get();
        self.increment_vram_addr();

        match addr {
            0..=0x1FFF => {
                panic!("Cannot write to allocated CHR ROM space addr: 0x{:04X}", addr)
            },
            0x2000..=0x2FFF => {
                self.vram[self.mirror_vram_addr(addr) as usize] = data
            },
            0x3000..=0x3EFF => panic!("Addr space 0x3000..=0x3EFF is not expected to be used. Attempted to read 0x{:04X}", addr),
            0x3F00..=0x3FFF => {
                self.palette_table[(addr - 0x3F00) as usize] = data
            },
            _ => panic!("Unexpected read access to mirrored space {}", addr),
        }
    }

    // See section 6.1 of textbook on screen-state mirroring
    fn mirror_vram_addr(&self, addr: u16) -> u16 {
        let mirrored_vram = addr & 0b10111111111111; // Mirrors down 3000-3EFF to regular ranges
        let vram_index = mirrored_vram - 0x2000; // Screens can start at 0x2000 so reduct to start from 0
        let name_table = vram_index / 0x400; // Create an index for each mirrored chunk
        match (&self.mirroring, name_table) {
            (Mirroring::VERTICAL, 2) | (Mirroring::VERTICAL, 3) => vram_index - 0x800,
            (Mirroring::HORIZONTAL, 2) | (Mirroring::HORIZONTAL, 1) => vram_index - 0x400,
            (Mirroring::HORIZONTAL, 3) => vram_index - 0x800,
            _ => vram_index,
        }

    }
}

pub struct AddrRegister { // hi ptr tracks if we've received 1 of 2 bytes yet
    value: (u8, u8),
    hi_ptr: bool
}

impl AddrRegister {
    pub fn new() -> Self {
        AddrRegister {
            value: (0, 0),
            hi_ptr: true,
        }
    }

    pub fn set(&mut self, data: u16) {
        self.value.0 = (data >> 8) as u8;
        self.value.1 = (data & 0xff) as u8;
    }

    // Grabs and returns the 2 byte address stored in value
    pub fn get(&self) -> u16 {
        ((self.value.0 as u16) << 8) | (self.value.1 as u16)
    }

    // Called when something gets loaded into the 0x2006 register
    pub fn update(&mut self, data: u8) {
        if self.hi_ptr {
            self.value.0 = data;
        } else {
            self.value.1 = data;
        }

        // Everything above 0x3FFF is mirrored so mirror down if ever above
        if self.get() > 0x3FFF {
            self.set(self.get() & 0b11111111111111);
        }
    }

    pub fn increment(&mut self, inc: u8) {
        let lo = self.value.1;
        self.value.1 = self.value.1.wrapping_add(inc);
        if lo > self.value.1 {
            self.value.0 = self.value.0.wrapping_add(1);
        }
        if self.get() > 0x3fff {
            self.set(self.get() & 0b11111111111111); //mirror down addr above 0x3fff
        }
    }

    pub fn reset_latch(&mut self) {
        self.hi_ptr = true;
    }
}

bitflags! {

   // 7  bit  0
   // ---- ----
   // VPHB SINN
   // |||| ||||
   // |||| ||++- Base nametable address
   // |||| ||    (0 = $2000; 1 = $2400; 2 = $2800; 3 = $2C00)
   // |||| |+--- VRAM address increment per CPU read/write of PPUDATA
   // |||| |     (0: add 1, going across; 1: add 32, going down)
   // |||| +---- Sprite pattern table address for 8x8 sprites
   // ||||       (0: $0000; 1: $1000; ignored in 8x16 mode)
   // |||+------ Background pattern table address (0: $0000; 1: $1000)
   // ||+------- Sprite size (0: 8x8 pixels; 1: 8x16 pixels)
   // |+-------- PPU master/slave select
   // |          (0: read backdrop from EXT pins; 1: output color on EXT pins)
   // +--------- Generate an NMI at the start of the
   //            vertical blanking interval (0: off; 1: on)
   pub struct ControlRegister: u8 {
       const NAMETABLE1              = 0b0000_0001;
       const NAMETABLE2              = 0b0000_0010;
       const VRAM_ADD_INCREMENT      = 0b0000_0100;
       const SPRITE_PATTERN_ADDR     = 0b0000_1000;
       const BACKROUND_PATTERN_ADDR  = 0b0001_0000;
       const SPRITE_SIZE             = 0b0010_0000;
       const MASTER_SLAVE_SELECT     = 0b0100_0000;
       const GENERATE_NMI            = 0b1000_0000;
   }
}

impl ControlRegister {
    pub fn new() -> Self {
        ControlRegister::from_bits_truncate(0b0000_0000)
    }

    pub fn vram_addr_increment(&self) -> u8 {
        if !self.contains(ControlRegister::VRAM_ADD_INCREMENT) {
            1
        } else {
            32
        }
    }

    pub fn update(&mut self, data: u8) {
        *self = ControlRegister::from_bits_truncate(data);
    }
}