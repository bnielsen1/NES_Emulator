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

fn main() {
    let bytes: Vec<u8> = std::fs::read("/home/briyoda/Projects/Rust/NES_Emulator/src/pacman.nes").unwrap();
    let rom = Rom::new(&bytes).unwrap();

    for (i, byte) in rom.prg_rom.iter().enumerate().take(80) {
        println!("${:04X}: {:02X}", 0x8000 + i, byte);
    }
}