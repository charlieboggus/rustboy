/*
    General Memory Map:
    -----------------------------------------------
    $0000-$3FFF: 16KB ROM bank 00
    $4000-$7FFF: 16KB ROM bank 01
    $8000-$9FFF: 8KB Video RAM (VRAM)
    $A000-$BFFF: 8KB External RAM
    $C000-$CFFF: 4KB Working RAM (WRAM) bank 0
    $D000-$DFFF: 4KB WRAM bank 1
    $E000-$FDFF: Mirror of $C000-$DDFF (ECHO RAM)
    $FE00-$FE9F: Sprite Attribute Table (OAM)
    $FEA0-$FEFF: Not usable
    $FF00-$FF7F: I/O registers
    $FF80-$FFFE: High RAM (HRAM) - (Zero Page RAM)
    $FFFF-$FFFF: Interrupts Enable Register (IE)
    -----------------------------------------------
    http://gbdev.gg8.se/wiki/articles/Memory_Map
*/

pub mod ram;
pub mod cartridge;

use crate::gb;

use ram::RAM;
use cartridge::Cartridge;

const WRAM_SIZE: usize = 32 << 10;

/// HRAM is from 0xFF80 to 0xFFFE
const HRAM_SIZE: usize = 0x7F;

#[derive(Debug, Clone, Copy)]
pub enum Speed
{
    Normal,
    Double
}

pub struct Memory
{
    /// Interrupt flags, the master IEM register is on CPU
    pub intf: u8,
    pub inte: u8,

    /// The speed that the gameboy is operating at
    pub speed: Speed,

    /// Has a speed switch been requested?
    pub speed_switch: bool,

    /// Loaded Cartridge
    //cart: Cartridge,

    /// Working RAM
    wram: Box< RAM >,

    /// High Speed RAM (Zeropage)
    hram: Box< RAM >,
}

impl Memory
{
    /// Create and return a new instance of the GameBoy memory
    pub fn new() -> Self
    {
        Memory {
            intf: 0,
            inte: 0,
            
            speed: Speed::Normal,
            speed_switch: false,

            //cart: cart,

            wram: Box::new(RAM::new(WRAM_SIZE)),
            hram: Box::new(RAM::new(HRAM_SIZE)),
        }
    }

    /// Read a byte from the given address in memory
    pub fn read_byte(&self, addr: u16) -> u8
    {
        match addr
        {
            // ROM
            0x0000...0x7FFF => 0xFF,

            // VRAM
            0x8000...0x9FFF => 0xFF,

            // EXT RAM
            0xA000...0xBFFF => 0xFF,

            // WRAM 0
            0xC000...0xCFFF => 0xFF,

            // WRAM 1
            0xD000...0xDFFF => 0xFF,

            // WRAM ECHO
            0xE000...0xFDFF => 0xFF,

            // OAM
            0xFE00...0xFE9F => 0xFF,

            // Unused
            0xFEA0...0xFEFF => 0xFF,

            // IO Registers
            0xFF00...0xFF7F => self.read_io_reg_byte(addr),

            // HRAM
            0xFF80...0xFFFE => 0xFF,

            // IE Register
            0xFFFF => self.inte
        }
    }

    /// Write a byte to the given address in memory
    pub fn write_byte(&mut self, addr: u16, val: u8)
    {
        match addr
        {
            // ROM
            0x0000...0x7FFF => { },

            // VRAM
            0x8000...0x9FFF => { },

            // EXT RAM
            0xA000...0xBFFF => { },

            // WRAM 0
            0xC000...0xCFFF => { },

            // WRAM 1
            0xD000...0xDFFF => { },

            // WRAM ECHO
            0xE000...0xFDFF => { },

            // OAM
            0xFE00...0xFE9F => { },

            // Unused
            0xFEA0...0xFEFF => { },

            // IO Registers
            0xFF00...0xFF7F => self.write_io_reg_byte(addr, val),

            // HRAM
            0xFF80...0xFFFE => { },

            // IE Register
            0xFFFF => self.inte = val
        }
    }

    /// Read a 16-bit word from the given address in memory
    pub fn read_word(&self, addr: u16) -> u16
    {
        (self.read_byte(addr) as u16) | ((self.read_byte(addr + 1) as u16) << 8)
    }

    /// Write a 16-bit word to the given address in memory
    pub fn write_word(&mut self, addr: u16, val: u16)
    {
        self.write_byte(addr, val as u8);
        self.write_byte(addr + 1, (val >> 8) as u8);
    }

    /// Read a byte from an IO Register address in memory (0xFF00 thru 0xFF7F)
    fn read_io_reg_byte(&self, addr: u16) -> u8
    {
        0
    }

    /// Write a byte to an IO register address in memory (0xFF00 thru 0xFF7F)
    fn write_io_reg_byte(&mut self, addr: u16, val: u8)
    {
    }

    /// Switches speed if a speed switch is requested by CPU
    pub fn switch_speed(&mut self)
    {
        self.speed_switch = false;
        self.speed = match self.speed 
        { 
            Speed::Normal => Speed::Double, 
            Speed::Double => Speed::Normal 
        };
    }

    pub fn select_target(rom: &[u8]) -> Option< gb::Target >
    {
        if rom.len() < 0x0146       { return None; }
        if rom[0x0143] & 0x80 != 0  { return Some(gb::Target::GameBoyColor); }
        if rom[0x0146] & 0x03 != 0  { return Some(gb::Target::SuperGameBoy); }
        None
    }
}