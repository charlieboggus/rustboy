use crate::cpu::CPU;
use crate::mem::Memory;

/// The width of the GameBoy screen in pixels
pub const DISPLAY_WIDTH: usize = 160;

/// The height of the GameBoy screen in pixels
pub const DISPLAY_HEIGHT: usize = 144;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target
{
    GameBoy,
    GameBoyColor,
    SuperGameBoy
}

pub struct Gameboy
{
    /// GameBoy CPU
    cpu: CPU,

    /// GameBoy Memory Unit
    mem: Memory,

    /// FPS GameBoy is running at
    fps: u32,

    /// Timing
    cycles: u32
}

impl Gameboy
{
    /// Create and return a new instance of a GameBoy running as the target system
    pub fn new(target: Target) -> Self
    {
        let mut gb = Gameboy { 
            cpu: CPU::new(target),
            mem: Memory::new(target),
            fps: 0, 
            cycles: 0 
        };

        gb
    }

    /// Execute the GameBoy power up sequence
    pub fn power_on(&mut self)
    {
    }

    /// Run a single cycle of the GameBoy
    pub fn run(&mut self)
    {
        self.cycles += 70224;
        while self.cycles <= 70224
        {
            let time = self.cpu.exec(&mut self.mem);
            self.mem.step(time);
            self.cycles -= time;
        }
    }

    /// Get the color of the pixel at the given x, y location
    pub fn get_pixel_color(&mut self, x: usize, y: usize)
    {
    }

    /// Register that a key has been pressed down
    pub fn key_down(&mut self)
    {
    }

    /// Register that a key has been released
    pub fn key_up(&mut self)
    {
    }

    /// Get the current FPS the GameBoy is running at
    pub fn fps(&mut self) -> u32
    {
        ::std::mem::replace(&mut self.fps, 0)
    }
}