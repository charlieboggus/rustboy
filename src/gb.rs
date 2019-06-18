use crate::cpu::CPU;
use crate::mem::Memory;
use crate::keypad::Button;

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
        Gameboy { 
            cpu: CPU::new(target),
            mem: Memory::new(target),
            fps: 0, 
            cycles: 0 
        }
    }

    /// Execute the GameBoy power up sequence
    pub fn power_on(&mut self)
    {
        // http://marc.rawer.de/Gameboy/Docs/GBCPUman.pdf - page 18
        self.mem.write_byte(0xFF05, 0x00);  // TIMA
        self.mem.write_byte(0xFF06, 0x00);  // TMA
        self.mem.write_byte(0xFF07, 0x00);  // TAC
        self.mem.write_byte(0xFF10, 0x80);  // NR10
        self.mem.write_byte(0xFF11, 0xBF);  // NR11
        self.mem.write_byte(0xFF12, 0xF3);  // NR12
        self.mem.write_byte(0xFF14, 0xBF);  // NR14
        self.mem.write_byte(0xFF16, 0x3F);  // NR21
        self.mem.write_byte(0xFF17, 0x00);  // NR22
        self.mem.write_byte(0xFF19, 0xBF);  // NR24
        self.mem.write_byte(0xFF1A, 0x7F);  // NR30
        self.mem.write_byte(0xFF1B, 0xFF);  // NR31
        self.mem.write_byte(0xFF1C, 0x9F);  // NR32
        self.mem.write_byte(0xFF1E, 0xBF);  // NR33
        self.mem.write_byte(0xFF20, 0xFF);  // NR41
        self.mem.write_byte(0xFF21, 0x00);  // NR42
        self.mem.write_byte(0xFF22, 0x00);  // NR43
        self.mem.write_byte(0xFF23, 0xBF);  // NR30
        self.mem.write_byte(0xFF24, 0x77);  // NR50
        self.mem.write_byte(0xFF25, 0xF3);  // NR51
        self.mem.write_byte(0xFF26, 0xF1);  // NR52
        self.mem.write_byte(0xFF40, 0x91);  // LCDC
        self.mem.write_byte(0xFF42, 0x00);  // SCY
        self.mem.write_byte(0xFF43, 0x00);  // SCX
        self.mem.write_byte(0xFF45, 0x00);  // LYC
        self.mem.write_byte(0xFF47, 0xFC);  // BGP
        self.mem.write_byte(0xFF48, 0xFF);  // OBP0
        self.mem.write_byte(0xFF49, 0xFF);  // OBP1
        self.mem.write_byte(0xFF4A, 0x00);  // WY
        self.mem.write_byte(0xFF4B, 0x00);  // WX
        self.mem.write_byte(0xFFFF, 0x00);  // IE
    }

    /// Load a GameBoy ROM
    pub fn load(&mut self)
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
    pub fn key_down(&mut self, key: Button)
    {
        self.mem.keypad.key_down(key, &mut self.mem.intf);
    }

    /// Register that a key has been released
    pub fn key_up(&mut self, key: Button)
    {
        self.mem.keypad.key_up(key);
    }

    /// Get the current FPS the GameBoy is running at
    pub fn fps(&mut self) -> u32
    {
        ::std::mem::replace(&mut self.fps, 0)
    }
}