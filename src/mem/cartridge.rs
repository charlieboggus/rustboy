use super::ram::RAM;
use std::iter::repeat;

use std::fs::{ File, OpenOptions };
use std::io::{ SeekFrom, Read, Write, Seek };
use std::io::Result as IoResult;
use std::path::{ Path, PathBuf };

const ROM_BANK_SIZE: i32 = 16 * 1024;

pub struct Cartridge
{
    /// Cartridge ROM data
    rom: Vec< u8 >,

    /// Cartridge RAM data
    ram: Vec< u8 >,

    /// Total number of ROM banks
    rom_banks: u8,

    /// Current ROM bank swapped in
    rom_bank: u8,
}

impl Cartridge
{
    pub fn from_file(rom_path: &Path) -> IoResult< Self >
    {
        let mut src = File::open(rom_path)?;
        let mut rom = Vec::new();

        src.take(2 * ROM_BANK_SIZE as u64).read_to_end(&mut rom)?;

        let mut cart = Cartridge {
            rom: rom,
            ram: Vec::new(),
            rom_banks: 2,
            rom_bank: 1
        };

        cart.initialize_ram()?;

        Ok(cart)
    }

    fn initialize_ram(&mut self) -> IoResult< () >
    {
        Ok(())
    }

    pub fn ram_size(&self) -> usize
    {
        match self.rom[0x0149]
        {
            0x00 => 0,
            0x01 => 2 << 10,
            0x02 => 8 << 10,
            0x03 => 32 << 10,
            _ => 0
        }
    }
    
    pub fn read_rom(&self, addr: u16) -> u8
    {
        0
    }

    pub fn write_rom(&mut self, addr: u16, val: u8)
    {
    }

    pub fn read_ram(&self, addr: u16) -> u8
    {
        0
    }

    pub fn write_ram(&mut self, addr: u16, val: u8)
    {
    }
}