use crate::rom::Mirroring;


pub trait Mapper {
    fn cpu_read(&self, addr: u16) -> u8;
    fn cpu_write(&mut self, addr: u16, data: u8);
    fn ppu_read(&self, addr: u16) -> u8;
    fn ppu_write(&mut self, addr: u16, data: u8);
    fn read_chr_rom(&self, index: usize) -> u8;
    fn read_prg_rom(&self, index: usize) -> u8;
    fn get_chr_rom(&self) -> Vec<u8>;
    fn get_prg_rom(&self) -> Vec<u8>;
    fn get_mirroring(&self) -> Mirroring;
    fn get_mapping(&self) -> u8;
}