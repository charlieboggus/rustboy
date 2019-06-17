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
}

impl Cartridge
{
}