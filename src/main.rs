mod cpu;
mod rom;
mod bus;
mod palette;
mod ppu;
mod frame;
mod render;
mod joypad;
mod trace;
mod mapper;
mod mapping;
use std::collections::HashMap;
use std::env;

use crate::cpu::CPU;
use crate::bus::Bus;
use crate::joypad::Joypad;
use crate::rom::Rom;
use crate::frame::Frame;
use crate::ppu::NesPPU;
use crate::trace::trace;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;



fn main() {
    let args: Vec<String> = env::args().collect();

    let mut rom_path: String = "".to_string();
    let mut debug: bool = false;

    // Process arguments
    let mut i = 1;
    while i < args.len() {
        if args[i] == "-rom" {
            if rom_path == "".to_string() {
                i += 1;
                rom_path = args[i].clone();
                i += 1;
            } else {
                panic!("Cannot set rom path multiple when providing arguments")
            }
        } else if args[i] == "-debug" {
            debug = true;
            i += 1;
        } else {
            panic!("Invalid argument passed: {}", args[i])
        }
    }

    if rom_path == "".to_string() {
        panic!("
            No rom path given!!! Please provide a rom path like so:\n
            \"cargo run rom PATH_TO_ROM\"
        ")
    }

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
    println!("Attempting to load rom at path: {}", rom_path);
    let bytes: Vec<u8> = std::fs::read(rom_path).unwrap();
    println!("Rom found successfully, preparing for emulation...");
    let rom = Rom::new(&bytes).unwrap();

    let mut frame = Frame::new(); // The current frame to be drawn by sdl2

    // create map for controller inputs
    let mut key_map = HashMap::new();
    key_map.insert(Keycode::Down, joypad::JoypadButton::DOWN);
    key_map.insert(Keycode::Up, joypad::JoypadButton::UP);
    key_map.insert(Keycode::Right, joypad::JoypadButton::RIGHT);
    key_map.insert(Keycode::Left, joypad::JoypadButton::LEFT);
    key_map.insert(Keycode::Space, joypad::JoypadButton::SELECT);
    key_map.insert(Keycode::Return, joypad::JoypadButton::START);
    key_map.insert(Keycode::A, joypad::JoypadButton::BUTTON_A);
    key_map.insert(Keycode::S, joypad::JoypadButton::BUTTON_B);

    // begin game cycle
    let bus = Bus::new(rom, move |ppu: &NesPPU, joypad1: &mut Joypad| {
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

                Event::KeyDown { keycode, .. } => {
                    if let Some(key) = key_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
                        joypad1.set_button_pressed_status(*key, true);
                    }
                }
                Event::KeyUp { keycode, .. } => {
                    if let Some(key) = key_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
                        joypad1.set_button_pressed_status(*key, false);
                    }
                }

                _ => { /* do nothing */ },
            }
        }

    });

    let mut cpu = CPU::new(bus);

    let mut callback_fn: Box<dyn FnMut(&mut CPU)> = if debug {
        Box::new(|cpu: &mut CPU| {
            println!("{}", trace(cpu));
            println!("MORE PPU DATA: VBLANK: {} CTRL: {:08b}, STATUS: {:08b}", cpu.bus.ppu.trigger_nmi, cpu.bus.ppu.ctrl.bits(), cpu.bus.ppu.peek_status());
        })
    } else {
        Box::new(|_: &mut CPU| {

        })
    };

    cpu.reset();
    cpu.run_with_callback(move |cpu| callback_fn.as_mut()(cpu));


}