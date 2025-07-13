use std::collections::HashMap;

use crate::cpu::CPU;
use crate::cpu::{OpCode, AddressingMode, OPCODE_TABLE};

// CODE FOR TRACE MOSTLY TAKEN FROM https://bugzmanov.github.io/nes_ebook/chapter_5_1.html
// Specfically from the GitHub linked here

pub fn trace(cpu: &CPU) -> String {
    let ref opscodes: HashMap<u8, OpCode> = *OPCODE_TABLE;

    let code = cpu.mem_peek(cpu.pc);
    let ops = opscodes.get(&code).unwrap();

    let begin = cpu.pc;
    let mut hex_dump = vec![];
    hex_dump.push(code);

    let (mem_addr, stored_value) = match ops.addressing_mode {
        AddressingMode::Immediate | AddressingMode::NoneAddressing => (0, 0),
        _ => {
            let addr = cpu.debug_operand(begin+1, &ops.addressing_mode);
            (addr, cpu.mem_peek(addr))
        }
    };

    let tmp = match ops.bytes {
        1 => match ops.addr {
            0x0a | 0x4a | 0x2a | 0x6a => format!("A "),
            _ => String::from(""),
        },
        2 => {
            let address: u8 = cpu.mem_peek(begin + 1);
            // let value = cpu.mem_read(address));
            hex_dump.push(address);

            match ops.addressing_mode {
                AddressingMode::Immediate => format!("#${:02x}", address),
                AddressingMode::ZeroPage => format!("${:02x} = {:02x}", mem_addr, stored_value),
                AddressingMode::ZeroPage_X => format!(
                    "${:02x},X @ {:02x} = {:02x}",
                    address, mem_addr, stored_value
                ),
                AddressingMode::ZeroPage_Y => format!(
                    "${:02x},Y @ {:02x} = {:02x}",
                    address, mem_addr, stored_value
                ),
                AddressingMode::Indirect_X => format!(
                    "(${:02x},X) @ {:02x} = {:04x} = {:02x}",
                    address,
                    (address.wrapping_add(cpu.reg_x)),
                    mem_addr,
                    stored_value
                ),
                AddressingMode::Indirect_Y => format!(
                    "(${:02x}),Y = {:04x} @ {:04x} = {:02x}",
                    address,
                    (mem_addr.wrapping_sub(cpu.reg_y as u16)),
                    mem_addr,
                    stored_value
                ),
                AddressingMode::NoneAddressing => {
                    // assuming local jumps: BNE, BVS, etc....
                    let address: usize =
                        (begin as usize + 2).wrapping_add((address as i8) as usize);
                    format!("${:04x}", address)
                }

                _ => panic!(
                    "unexpected addressing mode {:?} has ops-len 2. code {:02x}",
                    ops.addressing_mode, ops.addr
                ),
            }
        }
        3 => {
            let address_lo = cpu.mem_peek(begin + 1);
            let address_hi = cpu.mem_peek(begin + 2);
            hex_dump.push(address_lo);
            hex_dump.push(address_hi);

            let address = cpu.mem_peek_u16(begin + 1);

            if ops.addr == 0x4C {
                return format!("#${:02x}", address);
            } else if ops.addr == 0x6C {
                return format!("${:04x} = {:02x}", mem_addr, stored_value);
            }

            match ops.addressing_mode {
                AddressingMode::NoneAddressing => {
                    if ops.addr == 0x6c {
                        //jmp indirect
                        let jmp_addr = if address & 0x00FF == 0x00FF {
                            let lo = cpu.mem_peek(address);
                            let hi = cpu.mem_peek(address & 0xFF00);
                            (hi as u16) << 8 | (lo as u16)
                        } else {
                            cpu.mem_peek_u16(address)
                        };

                        // let jmp_addr = cpu.mem_read_u16(address);
                        format!("(${:04x}) = {:04x}", address, jmp_addr)
                    } else {
                        format!("${:04x}", address)
                    }
                }
                AddressingMode::Absolute => format!("${:04x} = {:02x}", mem_addr, stored_value),
                AddressingMode::Absolute_X => format!(
                    "${:04x},X @ {:04x} = {:02x}",
                    address, mem_addr, stored_value
                ),
                AddressingMode::Absolute_Y => format!(
                    "${:04x},Y @ {:04x} = {:02x}",
                    address, mem_addr, stored_value
                ),
                _ => panic!(
                    "unexpected addressing mode {:?} has ops-len 3. code {:02x}",
                    ops.addressing_mode, ops.addr
                ),
            }
        }
        _ => String::from(""),
    };

    let hex_str = hex_dump
        .iter()
        .map(|z| format!("{:02x}", z))
        .collect::<Vec<String>>()
        .join(" ");
    let asm_str = format!("{:04x}  {:8} {: >4} {}", begin, hex_str, ops.code, tmp)
        .trim()
        .to_string();

    format!(
        "{:47} A:{:02x} X:{:02x} Y:{:02x} P:{:02x} SP:{:02x} | PPU: L: {} CYC: {}",
        asm_str, cpu.reg_a, cpu.reg_x, cpu.reg_y, cpu.status, cpu.sp, cpu.bus.ppu.scanline, cpu.bus.ppu.cycles
    )
    .to_ascii_uppercase()
}