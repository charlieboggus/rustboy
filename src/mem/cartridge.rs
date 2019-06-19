use crate::gb::Target;

use std::fs::{ File, OpenOptions };
use std::io::{ SeekFrom, Read, Write, Seek };
use std::io::Result as IoResult;
use std::iter::repeat;
use std::path::{ Path, PathBuf };

/// ROM Banks are always 16KB
const ROM_BANK_SIZE: i32 = 16 * 1024;

/// Starting address of the game title in uppercase ASCII
/// Title is located at 0x0134...0x0142
const TITLE: usize = 0x0134;

/// 0x80 if this cartridge is for CGB
/// 0x00 or other if this cartridge is non-CGB
const TARGET_CGB: usize = 0x0143;

/// 0x00 if this cartridge is for regular GameBoy
/// 0x03 if this cartridge uses Super GameBoy functions
const TARGET_SGB: usize = 0x0146;

/// Address where information about cartridge type is stored
const TYPE: usize = 0x0147;

/// Address where information about cartridge ROM size is stored
const ROM_SIZE: usize = 0x0148;

/// Address where information about cartridge RAM size is stored
const RAM_SIZE: usize = 0x0149;

/// The different types of cartridge Memory Bank Controllers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MBC
{
    Unknown,
    ROM,
    MBC1,
    MBC2,
    MBC3,
    MBC5
}

pub struct Cartridge
{
    /// Cartridge ROM data
    rom: Vec< u8 >,

    /// Cartridge RAM data
    ram: Vec< u8 >,

    /// Total number of ROM banks
    rom_banks: u8,

    /// Current ROM bank swapped in
    rom_bank: u16,

    /// Does this cartridge use RAM?
    ram_enabled: bool,

    /// Current RAM bank swapped in
    ram_bank: u8,

    /// Type of MBC the cartridge uses
    mbc: MBC,

    /// Whether we're in ROM banking (false) or RAM banking (true)
    bank_mode: bool,

    /// Path to ROM image for this cartridge
    path: PathBuf,

    /// Optional save file used to store non-volatile RAM on emulator shutdown
    save_file: Option< File >,
}

impl Cartridge
{
    pub fn from_file(rom_path: &Path) -> IoResult< Self >
    {
        // Open ROM file and read its contents into Vec
        let mut src = File::open(rom_path)?;
        let mut rom = Vec::new();
        (&mut src).take(2 * ROM_BANK_SIZE as u64).read_to_end(&mut rom)?;

        // Create a new instance of Cartridge
        let mut cart = Cartridge {
            rom: rom,
            ram: Vec::new(),
            rom_banks: 2,
            rom_bank: 1,
            ram_enabled: true,
            ram_bank: 0,
            mbc: MBC::Unknown,
            bank_mode: false,
            path: PathBuf::from(rom_path),
            save_file: None
        };

        // Determine cartridge MBC type
        match cart.rom[TYPE]
        {
            0x00 | 0x08 | 0x09                          => cart.mbc = MBC::ROM,
            0x01 | 0x02 | 0x03                          => cart.mbc = MBC::MBC1,
            0x05 | 0x06                                 => cart.mbc = MBC::MBC2,
            0x11 | 0x12 | 0x0F | 0x10 | 0x13            => cart.mbc = MBC::MBC3,
            0x19 | 0x1A | 0x1C | 0x1D | 0x1B | 0x1E     => cart.mbc = MBC::MBC5,
            n => panic!("Unknown cartridge type inserted: {:#x}", n)
        }

        // Get the number of ROM banks & read remaining banks if necessary
        let rom_banks = if let Some(n) = cart.rom_banks() { 
            n 
        } else { 
            panic!("Cannot determine ROM size!") 
        };
        cart.rom_banks = rom_banks;
        if rom_banks > 2
        {
            let rem_b = (rom_banks - 2) as usize;
            let mut off = 2 * ROM_BANK_SIZE as usize;
            let mut rem_sz = rem_b * ROM_BANK_SIZE as usize;

            // Reserve space for remaining banks
            cart.rom.extend(repeat(0u8).take(rem_sz));

            // Read remaining ROM bank data
            while rem_sz > 0
            {
                let r = src.read(&mut cart.rom[off..])?;
                rem_sz -= r;
                off += r;
            }
        }

        // Initialize cartridge RAM
        let (ram_banks, bank_size) = if let Some(v) = cart.ram_banks() { 
            v
        } else { 
            panic!("Cannot determine RAM size!") 
        };
        let ram_size = ram_banks * bank_size;

        // If this cartridge doesn't have RAM there's nothing left to do
        if ram_size == 0
        {
            cart.ram_enabled = false;
            return Ok(cart)
        }

        let mut save_path = cart.path.clone();
        save_path.set_extension("sav");
        let mut save_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(save_path.clone())?;
        let save_size = save_file.metadata()?.len();

        if save_size == 0
        {
            cart.ram = repeat(0u8).take(ram_size).collect();
            save_file.write_all(&cart.ram)?;
        }
        else if save_size == ram_size as u64
        {
            (&mut save_file).take(ram_size as u64).read_to_end(&mut cart.ram)?;
        }
        else
        {
            panic!("Unexpected save file size for {}: expected {} got {}", 
                save_path.display(), ram_size, save_size);
        }

        cart.save_file = Some(save_file);

        Ok(cart)
    }

    /// Returns the target GB system that this cartridge is for
    pub fn get_target(&self) -> Target
    {
        if self.rom[TARGET_CGB] & 0x80 != 0x0 { return Target::GameBoyColor; }
        if self.rom[TARGET_SGB] & 0x03 != 0x0 { return Target::SuperGameBoy; }
        Target::GameBoy
    }

    /// Attempts to return the title of the game from the ROM header
    pub fn get_title(&self) -> String
    {
        let mut title = String::with_capacity(16);
        for i in 0..16
        {
            // Get the byte of the next character in the title
            let val = self.rom[TITLE + i];

            // Titles shorter than 16 characters are padded with 0x0
            if val == 0x0 { break; }

            // Convert the value to char & append to string
            let c = val as char;
            title.push(c);
        }
        
        title
    }
    
    /// Read a byte from cartridge ROM
    pub fn read_rom(&self, addr: u16) -> u8
    {
        match addr
        {
            0x0000...0x3FFF => self.rom[addr as usize],
            0x4000...0x7FFF => self.rom[(((self.rom_bank as u32) << 14) | 
                ((addr as u32) & 0x3FFF)) as usize],

            _ => panic!("(r) Unreachable ROM address: {:#x}", addr)
        }
    }

    /// Write a byte to cartridge ROM
    pub fn write_rom(&mut self, addr: u16, val: u8)
    {
        match addr
        {
            0x0000...0x1FFF =>
            {
                match self.mbc
                {
                    MBC::MBC1 | MBC::MBC3 | MBC::MBC5 => self.ram_enabled = val & 0xF == 0xA,
                    MBC::MBC2 => if addr & 0x100 == 0 { self.ram_enabled = !self.ram_enabled; },
                    MBC::Unknown | MBC::ROM => {}
                }
            },

            0x2000...0x3FFF =>
            {
                let val = val as u16;
                match self.mbc
                {
                    MBC::MBC1 => {
                        self.rom_bank = (self.rom_bank & 0x60) | (val & 0x1F);
                        if self.rom_bank == 0 { self.rom_bank = 1; }
                    },
                    MBC::MBC2 => if addr & 0x100 != 0 { self.rom_bank = val & 0xF; },
                    MBC::MBC3 => {
                        let val = val & 0x7F;
                        self.rom_bank = val + if val != 0 { 0 } else { 1 };
                    },
                    MBC::MBC5 => {
                        if addr >> 12 == 0x2 
                        {
                            self.rom_bank = (self.rom_bank & 0xFF00) | val;
                        }
                        else
                        {
                            let val = (val & 1) << 8;
                            self.rom_bank = (self.rom_bank & 0x00FF) | val;
                        }
                    },
                    MBC::Unknown | MBC::ROM => {}
                }
            },

            0x4000...0x5FFF =>
            {
                match self.mbc
                {
                    MBC::MBC1 => { 
                        if !self.bank_mode 
                        { 
                            self.rom_bank = (self.rom_bank & 0x1F) | 
                                (((val as u16) & 0x3) << 5); 
                        }
                        else
                        {
                            self.ram_bank = val & 0x3;
                        }
                    },
                    MBC::MBC3 => {
                        // RTC
                        self.ram_bank = val & 0x3;
                    },
                    MBC::MBC5 => self.ram_bank = val & 0xF,
                    MBC::Unknown | MBC::ROM | MBC::MBC2 => {}
                }
            },

            0x6000...0x7FFF =>
            {
                match self.mbc
                {
                    MBC::MBC1 => self.bank_mode = val & 0x1 != 0,
                    MBC::MBC3 => { /* RTC */ },
                    _ => {}
                }
            },

            _ => panic!("[w] Unreachable ROM address: {:#x}", addr)
        }
    }

    /// Read a byte from cartridge RAM
    pub fn read_ram(&self, addr: u16) -> u8
    {
        if self.ram_enabled
        {
            self.ram[(((self.ram_bank as u16) << 12) | 
                (addr & 0x1FFF)) as usize]
        }
        else
        {
            0xFF
        }
    }

    /// Write a byte to cartridge RAM
    pub fn write_ram(&mut self, addr: u16, val: u8)
    {
        if self.ram_enabled
        {
            self.ram[(((self.ram_bank as u16) << 12) | 
                (addr & 0x1FFF)) as usize] = val;
        }
    }

    /// Update the save file
    pub fn save(&mut self) -> IoResult< () >
    {
        if let Some(f) = self.save_file.as_mut()
        {
            f.seek(SeekFrom::Start(0))?;
            f.write_all(&self.ram)?;
        }

        Ok(())
    }

    /// Return the number of ROM banks declared in cartridge header
    fn rom_banks(&self) -> Option< u8 >
    {
        let val = self.rom[ROM_SIZE];
        let num_banks = match val {
            0x00 => 2,
            0x01 => 4,
            0x02 => 8,
            0x03 => 16,
            0x04 => 32,
            0x05 => 64,
            0x06 => 128,
            0x52 => 72,
            0x53 => 80,
            0x54 => 96,

            _ => return None
        };

        Some(num_banks)
    }

    /// Returns the number of RAM banks declared in cartridge header along with
    /// the size of each bank in bytes
    fn ram_banks(&self) -> Option< (usize, usize) >
    {
        // MBC2 contains 1 bank of 256 bytes
        if self.mbc == MBC::MBC2 { return Some((1, 256)); }

        let val = self.rom[RAM_SIZE];
        let (num_banks, bank_size) = match val {
            0x00 => (0, 0),
            0x01 => (1, 2),
            0x02 => (1, 8),
            0x03 => (4, 8),
            0x04 => (16, 8),

            _ => return None
        };

        Some((num_banks, bank_size * 1024))
    }
}