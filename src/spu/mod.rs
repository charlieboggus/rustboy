

pub type Sample = u8;

pub const SAMPLES_PER_BUFFER: usize = 0x200;

pub const SAMPLER_DIVIDER: u32 = 95;

pub const SAMPLE_RATE: u32 = 0x400000 / SAMPLER_DIVIDER;

pub const CHANNEL_DEPTH: usize = 4;

pub const SOUND_MAX_VOL: u8 = 15;

pub const SAMPLE_MAX_VOL: u8 = SOUND_MAX_VOL * 4 * 2;

/// Represents the GameBoy Sound Processing Unit
pub struct SPU
{
}

impl SPU
{
    /// Create and return a new instance of the GameBoy SPU
    pub fn new() -> Self
    {
        SPU {
        }
    }

    /// Step the SPU a given number of ticks forward.
    pub fn step(&mut self, ticks: u32, intf: &mut u8)
    {
    }

    pub fn read_byte(&self, addr: u16) -> u8
    {
        0u8
    }

    pub fn write_byte(&mut self, addr: u16, val: u8)
    {
    }
}