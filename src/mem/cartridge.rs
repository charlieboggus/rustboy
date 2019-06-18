use super::ram::RAM;

/// ROM Banks are always 16kb each
const ROM_BANK_SIZE: i32 = 16 * 1024;

pub struct Cartridge
{
    /// Cartridge ROM data
    rom: Box< RAM >,

    /// Cartridge RAM data
    ram: Box< RAM >,

    /// Total number of ROM banks for this cartridge
    rom_banks: u8,

    /// Current ROM bank mapped at 0x4000...0x7FFF
    rom_bank: u8,

    /// True if RAM if write protected
    ram_wp: bool,

    /// Does this cartridge use a battery?
    battery: bool,
}

impl Cartridge
{
    pub fn new() -> Self
    {
        Cartridge {
            rom: Box::new(RAM::new(100)),
            ram: Box::new(RAM::new(100)),
            rom_banks: 0,
            rom_bank: 0,
            ram_wp: false,
            battery: true
        }
    }
}