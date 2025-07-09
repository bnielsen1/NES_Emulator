mod cpu;
mod rom;
mod bus;
mod palette;
mod ppu;
mod frame;
mod render;

use std::path::Path;
use std::error::Error;

use crate::cpu::CPU;
use crate::bus::Bus;
use crate::rom::Rom;
use crate::frame::Frame;
use crate::ppu::NesPPU;
use crate::palette::SYSTEM_PALLETE;

use rand::Rng;
use sdl2::event::Event;
use sdl2::EventPump;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;


fn show_tile(chr_rom: &Vec<u8>, bank: usize, tile_n: usize) -> Frame {
    assert!(bank <= 1); // Ensure bank is a valid size of 0 or 1

    let mut frame: Frame = Frame::new();
    let bank = (bank * 0x1000) as usize;
    let tile = &chr_rom[(bank + (tile_n * 16))..=(bank + (tile_n * 16) + 15)];

    for y in 0..7 {
        let mut lower = tile[y];
        let mut upper = tile[y+8];

        for x in (0..7).rev() {
            let pal_id = (1 & upper) << 1 | (1 & lower);
            lower = lower >> 1;
            upper = upper >> 1;
            let color = match pal_id {
                0 => SYSTEM_PALLETE[0x01],
                1 => SYSTEM_PALLETE[0x23],
                2 => SYSTEM_PALLETE[0x27],
                3 => SYSTEM_PALLETE[0x30],
                _ => panic!("Somehow got invalid sprite color id???")
            };
            frame.set_pixel(x, y, color);
        }
    }

    frame
}

fn show_tile_bank(chr_rom: &Vec<u8>, bank: usize) -> Frame {
    assert!(bank <= 1); // Ensure bank is a valid size of 0 or 1

    let mut frame: Frame = Frame::new();
    let bank = (bank * 0x1000) as usize;

    let mut y_offset = 0;
    let mut x_offset = 0;

    for tile in 0..255 {
        x_offset = (tile % 16) * 9;
        y_offset = (tile / 16) * 9;

        let tile = &chr_rom[(bank + (tile * 16))..=(bank + (tile * 16) + 15)];

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

    frame
}

fn his_show_tile_bank(chr_rom: &Vec<u8>, bank: usize) ->Frame {
    assert!(bank <= 1);

    let mut frame = Frame::new();
    let mut tile_y = 0;
    let mut tile_x = 0;
    let bank = (bank * 0x1000) as usize;

    for tile_n in 0..255 {
        if tile_n != 0 && tile_n % 20 == 0 {
            tile_y += 10;
            tile_x = 0;
        }
        let tile = &chr_rom[(bank + tile_n * 16)..=(bank + tile_n * 16 + 15)];

        for y in 0..=7 {
            let mut upper = tile[y];
            let mut lower = tile[y + 8];

            for x in (0..=7).rev() {
                let value = (1 & upper) << 1 | (1 & lower);
                upper = upper >> 1;
                lower = lower >> 1;
                let rgb = match value {
                    0 => SYSTEM_PALLETE[0x01],
                    1 => SYSTEM_PALLETE[0x23],
                    2 => SYSTEM_PALLETE[0x27],
                    3 => SYSTEM_PALLETE[0x30],
                    _ => panic!("can't be"),
                };
                frame.set_pixel(tile_x + x, tile_y + y, rgb)
            }
        }

        tile_x += 10;
    }
    frame
}

fn main() {
    // init SDL2
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Texture viewer", (256.0 * 3.0) as u32, (240.0 * 3.0) as u32)
        .position_centered()
        .build().unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    canvas.set_scale(3.0, 3.0).unwrap();

    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_target(PixelFormatEnum::RGB24, 256, 240).unwrap();

    //load the game
    let bytes: Vec<u8> = std::fs::read("/home/briyoda/Projects/Rust/NES_Emulator/src/pacman.nes").unwrap();
    let rom = Rom::new(&bytes).unwrap();

    let mut frame = Frame::new(); // The current frame to be drawn by sdl2

    // begin game cycle
    let bus = Bus::new(rom, move |ppu: &NesPPU| {
        render::render(ppu, &mut frame); // Causes PPU to process a frame and insert that data into the passed frame object

        // Process the frame object via SDL2
        texture.update(None, &frame.data, 256 * 3).unwrap();

        canvas.copy(&texture, None, None).unwrap();

        canvas.present();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => std::process::exit(0),
                _ => { /* do nothing */ }
            }
        }

    });

    let mut cpu = CPU::new(bus);

    cpu.reset();
    cpu.run();


}