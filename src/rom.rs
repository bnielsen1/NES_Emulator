use crate::{mapper::Mapper, mapping::mapper1::Mapper1};
use crate::mapping::mapper0::Mapper0;

use std::{cell::RefCell, rc::Rc};

const NES_TAG: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A];
const PRG_ROM_PAGE_SIZE: usize = 16384;
const CHR_ROM_PAGE_SIZE: usize = 8192;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Mirroring {
    VERTICAL,
    HORIZONTAL,
    SINGLE_LOWER,
    SINGLE_UPPER,
    FOUR_SCREEN
}

pub struct Rom {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub mapper_id: u8,
    pub screen_mirroring: Mirroring,
    pub is_chr_ram: bool
}

impl Rom {
    pub fn new(raw: &Vec<u8>) -> Result<Rom, String> {
        if &raw[0..4] != NES_TAG {
            return Err("File is not in iNES file format".to_string());
        }

        let mapper_id = (raw[7] & 0b1111_0000) | (raw[6] >> 4);
        

        let ines_ver = (raw[7] >> 2) & 0b11;
        if ines_ver != 0 {
            return Err("NES2.0 format is not supported".to_string());
        }

        let four_screen = raw[6] & 0b1000 != 0;
        let vertical_mirroring = raw[6] & 0b1 != 0;
        let screen_mirroring = match (four_screen, vertical_mirroring) {
            (true, _) => Mirroring::FOUR_SCREEN,
            (false, true) => Mirroring::VERTICAL,
            (false, false) => Mirroring::HORIZONTAL,
        };

        let prg_rom_size = raw[4] as usize * PRG_ROM_PAGE_SIZE;
        let chr_rom_size = raw[5] as usize * CHR_ROM_PAGE_SIZE;



        let skip_trainer = raw[6] & 0b100 != 0;

        let mut prg_rom_start = 16;
        if skip_trainer {
            prg_rom_start += 512;
        }
        let chr_rom_start = prg_rom_start + prg_rom_size;

        println!("PRG ROM INFORMATION: start: {} size: {}", prg_rom_start, prg_rom_size);
        println!("CHR ROM INFORMATION: start: {} size: {}", chr_rom_start, chr_rom_size);

        let mut is_chr_ram: bool = false;

        let prg_rom = raw[prg_rom_start..(prg_rom_start+prg_rom_size)].to_vec();
        let chr_rom = if chr_rom_size == 0 {
            is_chr_ram = true;
            vec![0; 8192]
        } else {
            raw[chr_rom_start..(chr_rom_start+chr_rom_size)].to_vec()
        };

        Ok(Rom {
            prg_rom: prg_rom,
            chr_rom: chr_rom,
            is_chr_ram: is_chr_ram,
            mapper_id,
            screen_mirroring
        })
    }

    pub fn new_test(test: Vec<u8>) -> Result<Rom, String> {
        let mut output_raw = NES_TAG.to_vec(); // NES FILE RECOGNITION
        output_raw.push(0x01); // Rom has only 1 16kB ROM bank
        output_raw.push(0x00); // Rom has no CHR rom banks (ppu data)
        output_raw.push(0b1111_0000); // Byte 6 (bit 2 set to 0 for NO trainer)
        output_raw.push(0b1111_0000); // Byte 7 (last 4 bits tell EMU we're on iNES 1.0)
        output_raw.push(0x00);
        output_raw.push(0x00);
        output_raw.extend(std::iter::repeat(0).take(6)); // Add 6 0s for reserved

        // Insert PRG Rom data

        // First insert test case instructions
        output_raw.extend(test.iter().clone()); 

        // Fill rest of PRG with BRK instructions to complete the rom
        let num_brks = PRG_ROM_PAGE_SIZE - test.len();
        output_raw.extend(std::iter::repeat(0).take(num_brks));

        // Send our raw "test rom" to become an actual 'Rom' object and return
        Self::new(&output_raw)
    }

    pub fn generate_mapper(&self) -> Rc<RefCell<dyn Mapper>> {
        println!("Generating mapper with mode: {}", self.mapper_id);
        let mapper: Rc<RefCell<dyn Mapper>>  = match self.mapper_id {
            0 => Rc::new(RefCell::new(Mapper0::new(
                self.prg_rom.clone(),
                self.chr_rom.clone(),
                self.screen_mirroring,
                self.is_chr_ram,
            ))),
            1 => Rc::new(RefCell::new(Mapper1::new(
                self.prg_rom.clone(),
                self.chr_rom.clone(),
                self.screen_mirroring,
                self.is_chr_ram
            ))),
            _ => panic!("Unsupported mapper selected {}", self.mapper_id)
        };
        mapper
    }
}

pub mod test {

    use super::*;

    struct TestRom {
        header: Vec<u8>,
        trainer: Option<Vec<u8>>,
        pgp_rom: Vec<u8>,
        chr_rom: Vec<u8>,
    }

    fn create_rom(rom: TestRom) -> Vec<u8> {
        let mut result = Vec::with_capacity(
            rom.header.len()
                + rom.trainer.as_ref().map_or(0, |t| t.len())
                + rom.pgp_rom.len()
                + rom.chr_rom.len(),
        );

        result.extend(&rom.header);
        if let Some(t) = rom.trainer {
            result.extend(t);
        }
        result.extend(&rom.pgp_rom);
        result.extend(&rom.chr_rom);

        result
    }

    pub fn test_rom() -> Rom {
        test_rom_containing(vec![])
    }

    pub fn test_rom_containing(program: Vec<u8>) -> Rom {
        let mut pgp_rom_contents = program;
        pgp_rom_contents.resize(2 * PRG_ROM_PAGE_SIZE, 0);

        let test_rom = create_rom(TestRom {
            header: vec![
                0x4E, 0x45, 0x53, 0x1A, 0x02, 0x01, 0x31, 00, 00, 00, 00, 00, 00, 00, 00, 00,
            ],
            trainer: None,
            pgp_rom: pgp_rom_contents,
            chr_rom: vec![2; 1 * CHR_ROM_PAGE_SIZE],
        });

        Rom::new(&test_rom).unwrap()
    }

    #[test]
    fn test() {
        let test_rom = create_rom(TestRom {
            header: vec![
                0x4E, 0x45, 0x53, 0x1A, 0x02, 0x01, 0x31, 00, 00, 00, 00, 00, 00, 00, 00, 00,
            ],
            trainer: None,
            pgp_rom: vec![1; 2 * PRG_ROM_PAGE_SIZE],
            chr_rom: vec![2; 1 * CHR_ROM_PAGE_SIZE],
        });

        let rom: Rom = Rom::new(&test_rom).unwrap();

        assert_eq!(rom.chr_rom, vec!(2; 1 * CHR_ROM_PAGE_SIZE));
        assert_eq!(rom.prg_rom, vec!(1; 2 * PRG_ROM_PAGE_SIZE));
        assert_eq!(rom.mapper_id, 3);
        assert_eq!(rom.screen_mirroring, Mirroring::VERTICAL);
    }

    #[test]
    fn test_with_trainer() {
        let test_rom = create_rom(TestRom {
            header: vec![
                0x4E,
                0x45,
                0x53,
                0x1A,
                0x02,
                0x01,
                0x31 | 0b100,
                00,
                00,
                00,
                00,
                00,
                00,
                00,
                00,
                00,
            ],
            trainer: Some(vec![0; 512]),
            pgp_rom: vec![1; 2 * PRG_ROM_PAGE_SIZE],
            chr_rom: vec![2; 1 * CHR_ROM_PAGE_SIZE],
        });

        let rom: Rom = Rom::new(&test_rom).unwrap();

        assert_eq!(rom.chr_rom, vec!(2; 1 * CHR_ROM_PAGE_SIZE));
        assert_eq!(rom.prg_rom, vec!(1; 2 * PRG_ROM_PAGE_SIZE));
        assert_eq!(rom.mapper_id, 3);
        assert_eq!(rom.screen_mirroring, Mirroring::VERTICAL);
    }

    #[test]
    fn test_nes2_is_not_supported() {
        let test_rom = create_rom(TestRom {
            header: vec![
                0x4E, 0x45, 0x53, 0x1A, 0x01, 0x01, 0x31, 0x8, 00, 00, 00, 00, 00, 00, 00, 00,
            ],
            trainer: None,
            pgp_rom: vec![1; 1 * PRG_ROM_PAGE_SIZE],
            chr_rom: vec![2; 1 * CHR_ROM_PAGE_SIZE],
        });
        let rom = Rom::new(&test_rom);
        match rom {
            Result::Ok(_) => assert!(false, "should not load rom"),
            Result::Err(str) => assert_eq!(str, "NES2.0 format is not supported"),
        }
    }
}