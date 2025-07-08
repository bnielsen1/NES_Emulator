
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
    oam_addr: u8, // OAM Address written by 0x2003 and used by 0x2004
    pub ppu_status: u8,

    pub mirroring: Mirroring,
    cycles: usize,
    scanline: u16,
    trigger_nmi: bool, // Variable cpu reads to see if it should be interrupted

    addr: AddrRegister,
    status: StatusRegister,
    scroll: ScrollRegister,
    mask: MaskRegister,
    ctrl: ControlRegister,
}

impl NesPPU {

    pub fn new_empty_rom() -> Self {
        NesPPU {
            chr_rom: vec![0; 2048],
            mirroring: Mirroring::HORIZONTAL,
            internal_data_buf: 0,
            oam_addr: 0,
            ppu_status: 0b0000_0000,
            vram: [0; 2048],
            oam_data: [0; 64 * 4],
            palette_table: [0; 32],
            cycles: 0,
            scanline: 0,
            trigger_nmi: false,
            addr: AddrRegister::new(),
            status: StatusRegister::new(),
            scroll: ScrollRegister::new(),
            mask: MaskRegister::new(),
            ctrl: ControlRegister::new(),
        }
    }

    pub fn new(chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        NesPPU {
            chr_rom: chr_rom,
            mirroring: mirroring,
            internal_data_buf: 0,
            oam_addr: 0,
            ppu_status: 0b0000_0000,
            vram: [0; 2048],
            oam_data: [0; 64 * 4],
            palette_table: [0; 32],
            cycles: 0,
            scanline: 0,
            trigger_nmi: false,
            addr: AddrRegister::new(),
            status: StatusRegister::new(),
            scroll: ScrollRegister::new(),
            mask: MaskRegister::new(),
            ctrl: ControlRegister::new(),
        }
    }

    pub fn tick(&mut self, cycles: usize) -> bool {
        self.cycles += cycles;
        if self.cycles >= 341 {
            self.cycles -= 341;

            self.scanline += 1;
            if self.scanline == 241 { // Trigger interupt at 241st scanline (offscreen)
                self.status.set_vblank_started(true);
                self.status.set_sprite_zero_hit(false);
                if self.ctrl.is_generate_nmi() {
                    self.trigger_nmi = true;
                }
            }

            if self.scanline >= 262 {
                // Reset out scanlines
                self.trigger_nmi = false;
                self.scanline = 0;
                self.status.set_vblank_started(false);
                self.status.set_sprite_overflow(false);
                self.status.set_sprite_zero_hit(false);
                return true;
            }
        }
        
        return false;
    }

    // Handles 0x2006 write (updates addr 0x2007 reads or writes from)
    pub fn write_to_ppu_addr(&mut self, value: u8) {
        self.addr.update(value);
    }

    // Handles 0x2000 writes
    pub fn write_to_ctrl(&mut self, value: u8) {
        let prev_ctrl_status = self.ctrl.is_generate_nmi();
        self.ctrl.update(value);
        if !prev_ctrl_status && self.ctrl.is_generate_nmi() && self.status.is_vblank_started() {
            self.trigger_nmi = true;
        }
    }

    pub fn get_nmi_status(&self) -> bool {
        self.trigger_nmi
    }

    // Called upon 0x2007 writes or reads
    pub fn increment_vram_addr(&mut self) {
        self.addr.increment(self.ctrl.vram_addr_increment());
    }

    // For read upon 0x2007
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

    // For write on 0x2007
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

    // Handles 0x2002 reads
    pub fn read_status(&mut self) -> u8 {

        // Reset 0x2005 0x2006 latches
        self.addr.reset_latch();
        self.scroll.reset_latch();
        
        // Return output
        self.status.read()
    }

    // Handles 0x2005 writes
    pub fn write_scroll(&mut self, data: u8) {
        self.scroll.write(data);
    }

    // Handles 0x2001 writes
    pub fn write_mask(&mut self, data: u8) {
        self.mask.update(data);
    }

    // Handles 0x2003 writes
    pub fn oam_addr_write(&mut self, data: u8) {
        println!("Writing to OAM ADDR: 0x{:02X}", data);
        self.oam_addr = data;
    }

    // Handles 0x2004 reads
    pub fn oam_data_read(&self) -> u8 {
        println!("Reading OAM DATA from 0x{:02X}", self.oam_addr);
        println!("Read OAM DATA 0x{:02X}", self.oam_data[self.oam_addr as usize]);
        self.oam_data[self.oam_addr as usize]
    }

    // Handles 0x2004 writes
    pub fn oam_data_write(&mut self, data: u8) {
        println!("Writing OAM DATA 0x{:02X} to 0x{:02X}", data, self.oam_addr);
        self.oam_data[self.oam_addr as usize] = data;
        self.oam_addr = self.oam_addr.wrapping_add(1);
    }

    pub fn oam_dma_write(&mut self, data: &[u8; 256]) {
        for byte in data.iter() {
            self.oam_data[self.oam_addr as usize] = *byte;
            self.oam_addr = self.oam_addr.wrapping_add(1);
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
        self.hi_ptr = !self.hi_ptr;
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
   pub struct StatusRegister: u8 {
       const UNUSED1                 = 0b0000_0001;
       const UNUSED2                 = 0b0000_0010;
       const UNUSED3                 = 0b0000_0100;
       const UNUSED4                 = 0b0000_1000;
       const UNUSED5                 = 0b0001_0000;
       const SPRITE_OVERFLOW         = 0b0010_0000;
       const SPRITE_ZERO_HIT         = 0b0100_0000;
       const VBLANK_STARTED          = 0b1000_0000;
   }
}

impl StatusRegister {
    pub fn new() -> Self {
        StatusRegister::from_bits_truncate(0b0000_0000)
    }

    // Called upon 0x2002 Read
    pub fn read(&mut self) -> u8 {
        let output = self.bits(); // Store orignal state

        // Clear Vblank flag on read
        self.remove(StatusRegister::VBLANK_STARTED);

        output
    }

    // Used to read current state for debugging purposes
    pub fn current_val(&self) -> u8 {
        self.bits()
    }

    pub fn is_sprite_overflow(&self) -> bool {
        self.contains(StatusRegister::SPRITE_OVERFLOW)
    }

    pub fn is_sprite_zero_hit(&self) -> bool {
        self.contains(StatusRegister::SPRITE_ZERO_HIT)
    }

    pub fn is_vblank_started(&self) -> bool {
        self.contains(StatusRegister::VBLANK_STARTED)
    }

    pub fn set_sprite_overflow(&mut self, value: bool) {
        if value {
            self.insert(StatusRegister::SPRITE_OVERFLOW);
        } else {
            self.remove(StatusRegister::SPRITE_OVERFLOW);
        }
    }

    pub fn set_sprite_zero_hit(&mut self, value: bool) {
        if value {
            self.insert(StatusRegister::SPRITE_ZERO_HIT);
        } else {
            self.remove(StatusRegister::SPRITE_ZERO_HIT);
        }
    }

    pub fn set_vblank_started(&mut self, value: bool) {
        if value {
            self.insert(StatusRegister::VBLANK_STARTED);
        } else {
            self.remove(StatusRegister::VBLANK_STARTED);
        }
    }

    pub fn update(&mut self, data: u8) {
        *self = StatusRegister::from_bits_truncate(data);
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

    pub fn is_nametable1(&self) -> bool {
        self.contains(ControlRegister::NAMETABLE1)
    }

    pub fn is_nametable2(&self) -> bool {
        self.contains(ControlRegister::NAMETABLE2)
    }

    pub fn is_sprite_pattern_addr(&self) -> bool {
        self.contains(ControlRegister::SPRITE_PATTERN_ADDR)
    }

    pub fn is_background_pattern_addr(&self) -> bool {
        self.contains(ControlRegister::BACKROUND_PATTERN_ADDR)
    }

    pub fn is_sprite_size(&self) -> bool {
        self.contains(ControlRegister::SPRITE_SIZE)
    }

    pub fn is_master_slave_select(&self) -> bool {
        self.contains(ControlRegister::MASTER_SLAVE_SELECT)
    }

    pub fn is_generate_nmi(&self) -> bool {
        self.contains(ControlRegister::GENERATE_NMI)
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

bitflags! {

    // 7  bit  0
    // ---- ----
    // BGRs bMmG
    // |||| ||||
    // |||| |||+- Greyscale (0: normal color, 1: greyscale)
    // |||| ||+-- 1: Show background in leftmost 8 pixels of screen, 0: Hide
    // |||| |+--- 1: Show sprites in leftmost 8 pixels of screen, 0: Hide
    // |||| +---- 1: Enable background rendering
    // |||+------ 1: Enable sprite rendering
    // ||+------- Emphasize red (green on PAL/Dendy)
    // |+-------- Emphasize green (red on PAL/Dendy)
    // +--------- Emphasize blue
   pub struct MaskRegister: u8 {
       const GREYSCALE               = 0b0000_0001;
       const SHOW_LEFT_BACKGROUND    = 0b0000_0010;
       const SHOW_LEFT_SPRITES       = 0b0000_0100;
       const BACKGROUND_RENDERING    = 0b0000_1000;
       const SPRITE_RENDERING        = 0b0001_0000;
       const EMPH_RED                = 0b0010_0000;
       const EMPH_GREEN              = 0b0100_0000;
       const EMPH_BLUE               = 0b1000_0000;
   }
}

impl MaskRegister {
    pub fn new() -> Self {
        MaskRegister::from_bits_truncate(0b0000_0000)
    }

    pub fn is_greyscale(&self) -> bool {
        self.contains(MaskRegister::GREYSCALE)
    }

    pub fn is_left_background(&self) -> bool {
        self.contains(MaskRegister::SHOW_LEFT_BACKGROUND)
    }

    pub fn is_left_sprites(&self) -> bool {
        self.contains(MaskRegister::SHOW_LEFT_SPRITES)
    }

    pub fn is_background_rendering(&self) -> bool {
        self.contains(MaskRegister::BACKGROUND_RENDERING)
    }

    pub fn is_sprite_rendering(&self) -> bool {
        self.contains(MaskRegister::SPRITE_RENDERING)
    }

    pub fn is_emphasizing_red(&self) -> bool {
        self.contains(MaskRegister::EMPH_RED)
    }

    pub fn is_emphasizing_green(&self) -> bool {
        self.contains(MaskRegister::EMPH_GREEN)
    }

    pub fn is_emphasizing_blue(&self) -> bool {
        self.contains(MaskRegister::EMPH_BLUE)
    }

    pub fn update(&mut self, data: u8) {
        *self = MaskRegister::from_bits_truncate(data);
    }
}

pub struct ScrollRegister { // hi ptr tracks if we've received 1 of 2 bytes yet
    x_val: u8,
    y_val: u8,
    latch: bool
}

impl ScrollRegister {
    pub fn new() -> Self {
        ScrollRegister {
            x_val: 0,
            y_val: 0,
            latch: true,
        }
    }

    pub fn write(&mut self, data: u8) {
        if self.latch {
            self.x_val = data;
        } else {
            self.y_val = data;
        }

        self.latch = !self.latch;
    }

    pub fn read(&self) -> (u8, u8) {
        (self.x_val, self.y_val)
    }

    pub fn reset_latch(&mut self) {
        self.latch = true;
    }

}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn test_ppu_vram_writes() {
        let mut ppu = NesPPU::new_empty_rom();
        ppu.write_to_ppu_addr(0x23);
        ppu.write_to_ppu_addr(0x05);
        ppu.write_to_data(0x66);

        assert_eq!(ppu.vram[0x0305], 0x66);
    }

    #[test]
    fn test_ppu_vram_reads() {
        let mut ppu = NesPPU::new_empty_rom();
        ppu.write_to_ctrl(0);
        ppu.vram[0x0305] = 0x66;

        ppu.write_to_ppu_addr(0x23);
        ppu.write_to_ppu_addr(0x05);

        ppu.read_data(); //load_into_buffer
        assert_eq!(ppu.addr.get(), 0x2306);
        assert_eq!(ppu.read_data(), 0x66);
    }

    #[test]
    fn test_ppu_vram_reads_cross_page() {
        let mut ppu = NesPPU::new_empty_rom();
        ppu.write_to_ctrl(0);
        ppu.vram[0x01ff] = 0x66;
        ppu.vram[0x0200] = 0x77;

        ppu.write_to_ppu_addr(0x21);
        ppu.write_to_ppu_addr(0xff);

        ppu.read_data(); //load_into_buffer
        assert_eq!(ppu.read_data(), 0x66);
        assert_eq!(ppu.read_data(), 0x77);
    }

    #[test]
    fn test_ppu_vram_reads_step_32() {
        let mut ppu = NesPPU::new_empty_rom();
        ppu.write_to_ctrl(0b100);
        ppu.vram[0x01ff] = 0x66;
        ppu.vram[0x01ff + 32] = 0x77;
        ppu.vram[0x01ff + 64] = 0x88;

        ppu.write_to_ppu_addr(0x21);
        ppu.write_to_ppu_addr(0xff);

        ppu.read_data(); //load_into_buffer
        assert_eq!(ppu.read_data(), 0x66);
        assert_eq!(ppu.read_data(), 0x77);
        assert_eq!(ppu.read_data(), 0x88);
    }

    // Horizontal: https://wiki.nesdev.com/w/index.php/Mirroring
    //   [0x2000 A ] [0x2400 a ]
    //   [0x2800 B ] [0x2C00 b ]
    #[test]
    fn test_vram_horizontal_mirror() {
        let mut ppu = NesPPU::new_empty_rom();
        ppu.write_to_ppu_addr(0x24);
        ppu.write_to_ppu_addr(0x05);

        ppu.write_to_data(0x66); //write to a

        ppu.write_to_ppu_addr(0x28);
        ppu.write_to_ppu_addr(0x05);

        ppu.write_to_data(0x77); //write to B

        ppu.write_to_ppu_addr(0x20);
        ppu.write_to_ppu_addr(0x05);

        ppu.read_data(); //load into buffer
        assert_eq!(ppu.read_data(), 0x66); //read from A

        ppu.write_to_ppu_addr(0x2C);
        ppu.write_to_ppu_addr(0x05);

        ppu.read_data(); //load into buffer
        assert_eq!(ppu.read_data(), 0x77); //read from b
    }

    // Vertical: https://wiki.nesdev.com/w/index.php/Mirroring
    //   [0x2000 A ] [0x2400 B ]
    //   [0x2800 a ] [0x2C00 b ]
    #[test]
    fn test_vram_vertical_mirror() {
        let mut ppu = NesPPU::new(vec![0; 2048], Mirroring::VERTICAL);

        ppu.write_to_ppu_addr(0x20);
        ppu.write_to_ppu_addr(0x05);

        ppu.write_to_data(0x66); //write to A

        ppu.write_to_ppu_addr(0x2C);
        ppu.write_to_ppu_addr(0x05);

        ppu.write_to_data(0x77); //write to b

        ppu.write_to_ppu_addr(0x28);
        ppu.write_to_ppu_addr(0x05);

        ppu.read_data(); //load into buffer
        assert_eq!(ppu.read_data(), 0x66); //read from a

        ppu.write_to_ppu_addr(0x24);
        ppu.write_to_ppu_addr(0x05);

        ppu.read_data(); //load into buffer
        assert_eq!(ppu.read_data(), 0x77); //read from B
    }

    #[test]
    fn test_read_status_resets_latch() {
        let mut ppu = NesPPU::new_empty_rom();
        ppu.vram[0x0305] = 0x66;

        ppu.write_to_ppu_addr(0x21);
        ppu.write_to_ppu_addr(0x23);
        ppu.write_to_ppu_addr(0x05);

        ppu.read_data(); //load_into_buffer
        assert_ne!(ppu.read_data(), 0x66);

        ppu.read_status();

        ppu.write_to_ppu_addr(0x23);
        ppu.write_to_ppu_addr(0x05);

        ppu.read_data(); //load_into_buffer
        assert_eq!(ppu.read_data(), 0x66);
    }

    #[test]
    fn test_ppu_vram_mirroring() {
        let mut ppu = NesPPU::new_empty_rom();
        ppu.write_to_ctrl(0);
        ppu.vram[0x0305] = 0x66;

        ppu.write_to_ppu_addr(0x63); //0x6305 -> 0x2305
        ppu.write_to_ppu_addr(0x05);

        ppu.read_data(); //load into_buffer
        assert_eq!(ppu.read_data(), 0x66);
        // assert_eq!(ppu.addr.read(), 0x0306)
    }

    #[test]
    fn test_read_status_resets_vblank() {
        let mut ppu = NesPPU::new_empty_rom();
        ppu.status.set_vblank_started(true);

        let status = ppu.read_status();

        assert_eq!(status >> 7, 1);
        assert_eq!(ppu.status.current_val() >> 7, 0);
    }

    #[test]
    fn test_oam_read_write() {
        let mut ppu = NesPPU::new_empty_rom();
        ppu.oam_addr_write(0x10);
        ppu.oam_data_write(0x66);
        ppu.oam_data_write(0x77);

        ppu.oam_addr_write(0x10);
        assert_eq!(ppu.oam_data_read(), 0x66);

        ppu.oam_addr_write(0x11);
        assert_eq!(ppu.oam_data_read(), 0x77);
    }

    #[test]
    fn test_oam_dma() {
        let mut ppu = NesPPU::new_empty_rom();

        let mut data = [0x66; 256];
        data[0] = 0x77;
        data[255] = 0x88;

        ppu.oam_addr_write(0x10);
        ppu.oam_dma_write(&data);

        ppu.oam_addr_write(0xf); //wrap around
        assert_eq!(ppu.oam_data_read(), 0x88);

        ppu.oam_addr_write(0x10);
        assert_eq!(ppu.oam_data_read(), 0x77);
  
        ppu.oam_addr_write(0x11);
        assert_eq!(ppu.oam_data_read(), 0x66);
    }
}