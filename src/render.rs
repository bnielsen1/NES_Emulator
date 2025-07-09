

use crate::ppu::NesPPU;
use crate::frame::Frame;
use crate::palette::SYSTEM_PALLETE;

pub fn render(ppu: &NesPPU, frame: &mut Frame) {
    let bank = ppu.ctrl.get_background_bank_val();

    for i in 0..0x03C0 { // For each tile in the screen
        let tile_id = ppu.vram[i] as u16; // 1 byte value that stores the tile id to render out of 255 (for our bank)

        // offsets to render individual tiles on to build the screen
        let x_offset: usize = (i % 32) * 8;
        let y_offset: usize = (i / 32) * 8; 

        let tile =  &ppu.chr_rom[(bank + (tile_id * 16)) as usize..=(bank + (tile_id * 16) + 15) as usize];

        for y in 0..=7 {
            let mut lower = tile[y];
            let mut upper = tile[y+8];

            for x in (0..=7).rev() {
                let pal_id = (1 & upper) << 1 | (1 & lower);
                lower = lower >> 1;
                upper = upper >> 1;
                let color = match pal_id {
                    0 => SYSTEM_PALLETE[0x01],
                    1 => SYSTEM_PALLETE[0x27],
                    2 => SYSTEM_PALLETE[0x23],
                    3 => SYSTEM_PALLETE[0x30],
                    _ => panic!("Somehow got invalid sprite color id???")
                };
                frame.set_pixel(x + x_offset, y + y_offset, color);
            }
        }
    }
}