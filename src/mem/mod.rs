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

use crate::Target;
use crate::gpu::GPU;
use crate::timer::Timer;
use crate::keypad::Keypad;
use ram::RAM;
use std::iter::repeat;

/// GB has 8K of WRAM, CGB has 32K of WRAM
const WRAM_SIZE: usize = 32 << 10;

/// HRAM is from 0xFF80 to 0xFFFE
const HRAM_SIZE: usize = 0x7F;

/// The speed at which the GameBoy is running
#[derive(Debug, Clone, Copy)]
pub enum Speed
{
    Normal,
    Double
}

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

pub struct Memory
{
    /// Target system this memory is for
    target: Target,

    /// Interrupt flags, the master IEM register is on CPU
    pub intf: u8,
    pub inte: u8,

    /// The speed that the gameboy is operating at
    pub speed: Speed,

    /// Has a speed switch been requested?
    pub speed_switch: bool,

    /// Cartridge ROM memory
    rom: Vec< u8 >,

    /// Cartridge RAM memory
    ram: Vec< u8 >,

    /// Working RAM
    wram: Box< RAM >,

    /// High Speed RAM (Zeropage)
    hram: Box< RAM >,

    /// Current ROM bank swapped in
    rom_bank: u16,

    /// Current RAM bank swapped in
    ram_bank: u8,

    /// The current WRAM bank currently swapped in
    wram_bank: u8,

    /// Is cartridge RAM enabled?
    ram_enabled: bool,

    /// False for ROM banking mode, true for RAM banking mode
    bank_mode: bool,

    /// Does the cartridge use a battery?
    battery: bool,

    /// MBC type of current cartridge
    mbc: MBC,

    /// Should Super GameBoy functionality be used?
    sgb: bool,

    // Should GameBoy Color functionality be used?
    cgb: bool,

    /// GameBoy Timer
    timer: Box< Timer >,

    /// Gameboy GPU
    pub gpu: Box< GPU >,

    /// GameBoy Keypad
    pub keypad: Box< Keypad >,
}

impl Memory
{
    /// Create and return a new instance of the GameBoy memory
    pub fn new(target: Target) -> Self
    {
        Memory {
            target: target,
            intf: 0,
            inte: 0,
            speed: Speed::Normal,
            speed_switch: false,
            rom: Vec::new(),
            ram: Vec::new(),
            wram: Box::new(RAM::new(WRAM_SIZE)),
            hram: Box::new(RAM::new(HRAM_SIZE)),
            rom_bank: 1,
            ram_bank: 0,
            wram_bank: 1,
            ram_enabled: false,
            bank_mode: false,
            battery: false,
            mbc: MBC::Unknown,
            sgb: false,
            cgb: false,
            timer: Box::new(Timer::new()),
            gpu: Box::new(GPU::new(target)),
            keypad: Box::new(Keypad::new()),
        }
    }

    pub fn load_cartridge(&mut self, rom: Vec< u8 >)
    {
        use MBC::*;

        self.rom = rom;
        self.battery = true;
        self.mbc = Unknown;

        // 0x0147 gives info about cartridge type
        match self.rom[0x0147]
        {
            // 0x00 - ROM Only
            // 0x08 - ROM + RAM
            0x00 | 0x08 => { self.battery = false; self.mbc = ROM; },
            
            // 0x09 - ROM + RAM + Battery
            0x09 => { self.mbc = ROM },

            // 0x01 - ROM + MBC1
            // 0x02 - ROM + MBC1 + RAM
            0x01 | 0x02 => { self.battery = false; self.mbc = MBC1; },

            // 0x03 - ROM + MBC1 + RAM + Battery
            0x03 => { self.mbc = MBC1; },

            // 0x05 - ROM + MBC2
            0x05 => { self.battery = false; self.mbc = MBC2; },

            // 0x06 - ROM + MBC2 + Battery
            0x06 => { self.mbc = MBC2; },

            // 0x11 - ROM + MBC3
            // 0x12 - ROM + MBC3 + RAM
            0x11 | 0x12 => { self.battery = false; self.mbc = MBC3; },

            // 0x0F - ROM + MBC3 + Timer + Battery
            // 0x10 - ROM + MBC3 + Timer + Battery + RAM
            // 0x13 - ROM + MBC3 + RAM + Battery
            0x0F | 0x10 | 0x13 => { self.mbc = MBC3; },

            // 0x19 - ROM + MBC5
            // 0x1A - ROM + MBC5 + RAM
            // 0x1C - ROM + MBC5 + Rumble
            // 0x1D - ROM + MBC5 + Rumble + RAM
            0x19 | 0x1A | 0x1C | 0x1D => { self.battery = false; self.mbc = MBC5; },

            // 0x1B - ROM + MBC5 + RAM + Battery
            // 0x1E - ROM + MBC5 + Rumble + SRAM + Battery
            0x1B | 0x1E => { self.mbc = MBC5; },

            n => panic!("Unknown cartridge type inserted: {:#X}", n)
        }

        // Determine RAM size & initialize RAM with 0's
        let ram_size = self.ram_size();
        self.ram = repeat(0u8).take(ram_size).collect();

        // Determine functionality needed by cartridge
        if self.target == Target::GameBoyColor
        {
            self.cgb = self.rom[0x0143] & 0x80 != 0;
            self.gpu.is_cgb = self.cgb;
        }

        if self.target == Target::SuperGameBoy || self.target == Target::GameBoyColor
        {
            self.sgb = self.rom[0x0146] == 0x03;
            if self.sgb
            {
                self.gpu.is_sgb = self.sgb;
            }
        }
    }

    fn ram_size(&self) -> usize
    {
        match self.rom[0x0149]
        {
            0x00 => 0,
            0x01 => 2 << 10,    // 2kB
            0x02 => 8 << 10,    // 8kB
            0x03 => 32 << 10,   // 32kB
            0x04 => 125 << 10,  // 128kB
            _ => panic!("Unknown RAM size: {:#X}", self.rom[0x0149])
        }
    }

    /// Step the Timer and GPU a given number of ticks forward
    pub fn step(&mut self, time: u32)
    {
        self.timer.step(time, &mut self.intf, self.speed);
        self.gpu.step(time, &mut self.intf);
    }

    /// Read a byte from the given address in memory
    pub fn read_byte(&self, addr: u16) -> u8
    {
        match addr
        {
            // ROM Bank 0
            0x0000...0x3FFF => self.rom[addr as usize],

            // ROM Bank 1
            0x4000...0x7FFF => self.rom[(((self.rom_bank as u32) << 14) | 
                ((addr as u32) & 0x3FFF)) as usize],

            // VRAM
            0x8000...0x9FFF => self.gpu.read_byte(addr),

            // EXT RAM
            0xA000...0xBFFF => 
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
            },

            // WRAM 0 and WRAM 0 mirror
            0xC000...0xCFFF | 0xE000...0xEFFF => 
                self.wram.read_byte(addr & 0xFFF),

            // WRAM 1 and WRAM 1 mirror
            0xD000...0xDFFF | 0xF000...0xFDFF => 
                self.wram.read_byte((self.wram_bank as u16) << 12 | 
                (addr & 0xFFF)),

            // OAM
            0xFE00...0xFE9F => self.gpu.read_byte(addr),

            // Unused
            0xFEA0...0xFEFF => 0xFF,

            // IO Registers
            0xFF00...0xFF7F => self.read_byte_io(addr),

            // HRAM
            0xFF80...0xFFFE => self.hram.read_byte(addr & 0x7F),

            // IE Register
            0xFFFF => self.inte
        }
    }

    /// Read a byte from an IO Register address (0xFF00 thru 0xFF7F)
    fn read_byte_io(&self, addr: u16) -> u8
    {
        match addr
        {
            // Keypad
            0xFF00 => self.keypad.read_byte(addr),

            // Serial
            // TODO: serial interface registers

            // Timer
            0xFF04...0xFF07 => self.timer.read_byte(addr),

            // Interrupt Flag
            0xFF0F => self.intf,

            // Sound
            // TODO: sound controller registers
            0xFF10...0xFF3F => 0xFF,

            // GPU
            0xFF40...0xFF4F => {
                if self.cgb && addr == 0xFF4D
                {
                    let b = match self.speed {
                        Speed::Normal => 0x00,
                        Speed::Double => 0x80
                    };
                    b | (self.speed_switch as u8)
                }
                else
                {
                    self.gpu.read_byte(addr)
                }
            },

            // GPU DMA Transfer
            0xFF50...0xFF6F => self.gpu.read_byte(addr),

            0xFF70 =>
            {
                if self.target == Target::GameBoyColor
                {
                    self.wram_bank as u8
                }
                else
                {
                    0xFF
                }
            }

            _ => 0xFF
        }
    }

    /// Write a byte to the given address in memory
    pub fn write_byte(&mut self, addr: u16, val: u8)
    {
        use MBC::*;
        match addr
        {
            // ROM Banks
            0x0000...0x1FFF => 
            {
                match self.mbc
                {
                    MBC1 | MBC3 | MBC5 => self.ram_enabled = val & 0xF == 0xA,
                    MBC2 => {
                        if addr & 0x100 == 0
                        {
                            self.ram_enabled = !self.ram_enabled;
                        }
                    },
                    Unknown | ROM => {}
                }
            },
            0x2000...0x3FFF => 
            {
                let val = val as u16;
                match self.mbc
                {
                    MBC1 => {
                        self.rom_bank = (self.rom_bank & 0x60) | (val & 0x1F);
                        if self.rom_bank == 0
                        {
                            self.rom_bank = 1;
                        }
                    },
                    MBC2 => {
                        if addr & 0x100 != 0
                        {
                            self.rom_bank = val & 0xF;
                        }
                    },
                    MBC3 => {
                        let val = val & 0x7F;
                        self.rom_bank = val + if val != 0 { 0 } else { 1 };
                    },
                    MBC5 => {
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
                    Unknown | ROM => {}
                }
            },
            0x4000...0x5FFF => 
            {
                match self.mbc
                {
                    MBC1 => {
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
                    MBC3 => {
                        // RTC?
                        self.ram_bank = val & 0x3;
                    },
                    MBC5 => {
                        self.ram_bank = val & 0xF;
                    },
                    Unknown | ROM | MBC2 => {}
                }
            },
            0x6000...0x7FFF => 
            {
                match self.mbc
                {
                    MBC1 => self.bank_mode = val & 0x1 != 0,
                    MBC3 => { /* RTC ? */ },
                    _ => {}
                }
            },

            // VRAM
            0x8000...0x9FFF => self.gpu.write_byte(addr, val),

            // EXT RAM
            0xA000...0xBFFF => 
            {
                if self.ram_enabled
                {
                    let val = if self.mbc == MBC::MBC2 { val & 0xF } else { val };
                    self.ram[(((self.ram_bank as u16) << 12) | 
                        (addr & 0x1FFF)) as usize] = val;
                }
            },

            // WRAM 0 and WRAM 0 mirror
            0xC000...0xCFFF | 0xE000...0xEFFF => 
                self.wram.write_byte(addr & 0xFFF, val),

            // WRAM 1 and WRAM 1 mirror
            0xD000...0xDFFF | 0xF000...0xFDFF => 
                self.wram.write_byte((self.wram_bank as u16) << 12 | 
                (addr & 0xFFF), val),

            // OAM
            0xFE00...0xFE9F => self.gpu.write_byte(addr, val),

            // Unused
            0xFEA0...0xFEFF => {},

            // IO Registers
            0xFF00...0xFF7F => self.write_byte_io(addr, val),

            // HRAM
            0xFF80...0xFFFE => self.hram.write_byte(addr & 0x7F, val),

            // IE Register
            0xFFFF => self.inte = val
        }
    }

    /// Write a byte to an IO register address (0xFF00 thru 0xFF7F)
    fn write_byte_io(&mut self, addr: u16, val: u8)
    {
        match addr
        {
            // Keypad
            0xFF00 => self.keypad.write_byte(addr, val),
            
            // Serial
            // TODO: serial interface registers

            // Timer
            0xFF04...0xFF07 => self.timer.write_byte(addr, val),

            // Interrupt flag
            0xFF0F => self.intf = val,

            // Sound
            // TODO: sound controller registers

            // GPU
            0xFF40...0xFF6F => 
            {
                match addr
                {
                    0xFF46 => GPU::oam_dma_transfer(self, val),
                    0xFF55 => GPU::hdma_dma_transfer(self, val),
                    0xFF4D if self.cgb => 
                    {
                        if val & 0x01 != 0 {
                            self.speed_switch = true;
                        }
                        else {
                            self.speed_switch = false;
                        }
                    },
                    _ => self.gpu.write_byte(addr, val)
                }
            },

            // WRAM bank for CGB mode
            0xFF70 => 
            {
                if self.cgb
                {
                    let val = val & 0x7; 
                    self.wram_bank = if val != 0 { val } else { 1 }; 
                }
            }

            _ => {}
        }
    }

    /// Read a 16-bit word from the given address in memory
    pub fn read_word(&self, addr: u16) -> u16
    {
        (self.read_byte(addr) as u16) | 
            ((self.read_byte(addr + 1) as u16) << 8)
    }

    /// Write a 16-bit word to the given address in memory
    pub fn write_word(&mut self, addr: u16, val: u16)
    {
        self.write_byte(addr, val as u8);
        self.write_byte(addr + 1, (val >> 8) as u8);
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
}