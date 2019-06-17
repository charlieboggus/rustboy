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
    cpu: CPU,
    mem: Memory,
    fps: u32,
    cycles: u32
}

impl Gameboy
{
    pub fn new(target: Target) -> Self
    {
        let mut gb = Gameboy { 
            cpu: CPU::new(),
            mem: Memory::new(),
            fps: 0, 
            cycles: 0 
        };

        gb
    }

    pub fn run(&mut self)
    {
        self.cycles += 70224;
        while self.cycles <= 70224
        {
            let time = self.cpu.exec(&mut self.mem);
            // Timer tick
            // GPU tick
            self.cycles -= time;
        }
    }

    pub fn get_pixel_color(&mut self, x: usize, y: usize)
    {
    }

    pub fn key_down(&mut self)
    {
    }

    pub fn key_up(&mut self)
    {
    }

    pub fn fps(&mut self) -> u32
    {
        ::std::mem::replace(&mut self.fps, 0)
    }
}