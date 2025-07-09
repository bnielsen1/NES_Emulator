

use crate::ppu::NesPPU;
use crate::frame::Frame;
use crate::palette::{self, SYSTEM_PALLETE};

pub fn render(ppu: &NesPPU, frame: &mut Frame) {
    let bank = ppu.ctrl.get_background_bank_val();

    for i in 0..0x03C0 { // For each tile in the screen
        let tile_id = ppu.vram[i] as u16; // what tile to grab out of chrom based on whats loaded on screen in vram

        // offsets to render individual tiles on to build the screen
        let x_offset: usize = i % 32;
        let y_offset: usize = i / 32;

        let palette = bg_palette(ppu, x_offset, y_offset);

        let tile =  &ppu.chr_rom[(bank + (tile_id * 16)) as usize..=(bank + (tile_id * 16) + 15) as usize];

        for y in 0..=7 {
            let mut lower = tile[y];
            let mut upper = tile[y+8];

            for x in (0..=7).rev() {
                let pal_id = (1 & upper) << 1 | (1 & lower);
                lower = lower >> 1;
                upper = upper >> 1;
                let color = match pal_id {
                    0 => SYSTEM_PALLETE[palette[0] as usize],
                    1 => SYSTEM_PALLETE[palette[1] as usize],
                    2 => SYSTEM_PALLETE[palette[2] as usize],
                    3 => SYSTEM_PALLETE[palette[3] as usize],
                    _ => panic!("Somehow got invalid sprite color id???")
                };
                frame.set_pixel(x + (x_offset * 8), y + (y_offset * 8), color);
            }
        }
    }
}

fn bg_palette(ppu: &NesPPU, tile_column: usize, tile_row: usize) -> [u8;4] {

    // Start at attr table of name table 1 and shift to the 4x4 meta tile
    // corresponding to the calculation made in attr_table_index

    let attr_table_index = tile_row / 4 * 8 + tile_column / 4;
    let attr_byte = ppu.vram[0x03C0 + attr_table_index]; 

    // palette index gets which entry of 4 from the background palette table to pick from
    let palette_index = match ((tile_column % 4) / 2, (tile_row % 4) / 2) {
        (0,0) => attr_byte & 0b11,
        (1,0) => (attr_byte >> 2) & 0b11,
        (0,1) => (attr_byte >> 4) & 0b11,
        (1,1) => (attr_byte >> 6) & 0b11,
        (_,_) => panic!("Invalid tile column/tile row pair  ({}, {}) when selecting a bg_palette", tile_column, tile_row),
    };

    // multiply by 4 since each palette table entry is 4 bytes wide
    // add 1 since first palette table entry is a single stable value for all palettes
    let palette_start_index = 1 + (palette_index as usize) * 4; 
    [ppu.palette_table[0], ppu.palette_table[palette_start_index], ppu.palette_table[palette_start_index+1], ppu.palette_table[palette_start_index+2]]
}