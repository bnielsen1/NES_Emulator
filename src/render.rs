

use std::vec;

use crate::ppu::NesPPU;
use crate::frame::Frame;
use crate::palette::{self, SYSTEM_PALLETE};
use crate::rom::Mirroring;

struct Rect {
    x1: usize,
    y1: usize,
    x2: usize,
    y2: usize,
}

impl Rect {
    pub fn new(x1: usize, y1: usize, x2: usize, y2: usize) -> Self {
        Rect {
            x1: x1,
            x2: x2,
            y1: y1,
            y2: y2
        }
    }
}

fn render_name_table(ppu: &NesPPU, frame: &mut Frame, name_table: &[u8], view_port: Rect, shift_x: isize, shift_y: isize) {
    let bank = ppu.ctrl.get_background_bank_val();

    let attribute_table = &name_table[0x3C0..0x400]; // Stores palette table information from the name table/screen ram

    for i in 0..0x3C0 { // For every tile in the current screen
        let tile_id = name_table[i] as u16; // what tile to grab out of chrom based on whats loaded on screen in vram

        // offsets to render individual tiles on to build the screen
        let x_offset: usize = i % 32;
        let y_offset: usize = i / 32;

        let palette = bg_pallette(ppu, attribute_table, x_offset, y_offset);
        
        let mut tile: Vec<u8> = vec![];
        let index_range = (bank + (tile_id * 16)) as usize..=(bank + (tile_id * 16) + 15) as usize;
        for i in index_range {
            tile.push(ppu.mapper.borrow().read_chr_rom(i));
        }

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

                let trans = if pal_id == 0 {
                    true
                } else {
                    false
                };
                
                let pixel_x = (x_offset * 8) + x;
                let pixel_y = (y_offset * 8) + y;

                if (pixel_x >= view_port.x1) && (pixel_x < view_port.x2) && (pixel_y >= view_port.y1) && (pixel_y < view_port.y2) {
                    frame.set_pixel(trans, ((pixel_x as isize) + shift_x) as usize, ((pixel_y as isize) + shift_y) as usize, color);
                }
            }
        }
    }
}

pub fn render(ppu: &NesPPU, frame: &mut Frame) {
    let scroll = ppu.scroll.read();

    let (main_nametable, other_nametable) = match (&ppu.mapper.borrow().get_mirroring(), ppu.ctrl.read_nametable()) {
        (Mirroring::VERTICAL, 0x2000) | (Mirroring::VERTICAL, 0x2800) | (Mirroring::HORIZONTAL, 0x2000) | (Mirroring::HORIZONTAL, 0x2400) => {
            (&ppu.vram[0..0x400], &ppu.vram[0x400..0x800])
        }
        (Mirroring::VERTICAL, 0x2400) | (Mirroring::VERTICAL, 0x2C00) | (Mirroring::HORIZONTAL, 0x2800) | (Mirroring::HORIZONTAL, 0x2C00) => {
            ( &ppu.vram[0x400..0x800], &ppu.vram[0..0x400])
        }
        (_,_) => panic!("Unsupported mirroring type?")
    };

    // Render main screen
    render_name_table(ppu, frame,
        main_nametable,
        Rect::new(scroll.0 as usize, scroll.1 as usize, 256, 240),
        -(scroll.0 as isize),
        -(scroll.1 as isize)
    );

    // Render other screen
    if scroll.0 > 0 {
        render_name_table(ppu, frame,
            other_nametable,
            Rect::new(0, 0, scroll.0 as usize, 240),
            256 - (scroll.0 as isize),
            0
        );
    } else if scroll.1 > 0 {
        render_name_table(ppu, frame,
            other_nametable,
            Rect::new(0, 0, 256, scroll.1 as usize),
            0,
            240 - (scroll.1 as isize)
        );
    } 
    // If we aren't scrolling in a direction we don't need to do any extra screen rendering!

    // Render sprites
    for i in (0..ppu.oam_data.len()).step_by(4).rev() {
        let tile_y: usize = ppu.oam_data[i] as usize;
        let tile_index: u16 = ppu.oam_data[i+1] as u16;
        let tile_attr = ppu.oam_data[i+2];
        let tile_x: usize = ppu.oam_data[i+3] as usize;

        let flip_vertical = if (tile_attr >> 7) & 1 == 1 {
            true
        } else {
            false
        };

        let flip_horizontal = if (tile_attr >> 6) & 1 == 1 {
            true
        } else {
            false
        };

        // true = draw above bkground
        let tile_prio = if (tile_attr >> 5) & 1 == 1 {
            false
        } else {
            true
        };

        let palette_index = tile_attr & 0b11;
        let sprite_palette = sprite_palette(ppu, palette_index);

        // Select bank based off ctrl register
        let bank = if ppu.ctrl.is_sprite_pattern_addr() {
            0x1000
        } else {
            0x0000
        };

        // load 

        
        let mut tile: Vec<u8> = vec![];
        let index_range = (bank + (tile_index * 16)) as usize..=(bank + (tile_index * 16) + 15) as usize;
        for i in index_range {
            tile.push(ppu.mapper.borrow().read_chr_rom(i));
        }
        for y in 0..=7usize {
            let mut lower = tile[y];
            let mut upper = tile[y+8];

            'outer: for x in (0..=7usize).rev() {
                let pal_id = (1 & upper) << 1 | (1 & lower);
                lower = lower >> 1;
                upper = upper >> 1;
                let color = match pal_id {
                    0 => continue 'outer,
                    1 => SYSTEM_PALLETE[sprite_palette[1] as usize],
                    2 => SYSTEM_PALLETE[sprite_palette[2] as usize],
                    3 => SYSTEM_PALLETE[sprite_palette[3] as usize],
                    _ => panic!("Somehow got invalid sprite color id???")
                };

                let trans = if pal_id == 0 {
                    true
                } else {
                    false
                };

                match (flip_horizontal, flip_vertical) {
                    (false, false) => frame.check_and_set(trans, tile_prio, tile_x + x,tile_y + y, color),
                    (true, false) => frame.check_and_set(trans, tile_prio, tile_x + 7 -x,tile_y + y, color),
                    (false, true) => frame.check_and_set(trans, tile_prio, tile_x + x,tile_y + 7 - y, color),
                    (true, true) => frame.check_and_set(trans, tile_prio, tile_x + 7 - x,tile_y + 7 - y, color),
                }
            }
        }
    }
}

fn bg_pallette(ppu: &NesPPU, attribute_table: &[u8], tile_column: usize, tile_row: usize) -> [u8;4] {

    // Start at attr table of name table 1 and shift to the 4x4 meta tile
    // corresponding to the calculation made in attr_table_index

    let attr_table_index = tile_row / 4 * 8 + tile_column / 4;
    let attr_byte = attribute_table[attr_table_index]; 

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
    [
        ppu.palette_table[0],
        ppu.palette_table[palette_start_index],
        ppu.palette_table[palette_start_index+1],
        ppu.palette_table[palette_start_index+2]
    ]
}


fn sprite_palette(ppu: &NesPPU, palette_index: u8) -> [u8;4] {
    let start = 0x11 + (palette_index * 4) as usize;
    [
        0,
        ppu.palette_table[start as usize],
        ppu.palette_table[start+1 as usize],
        ppu.palette_table[start+2 as usize]
    ]
}