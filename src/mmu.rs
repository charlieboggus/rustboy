use crate::gpu::GPU;
use crate::timer::Timer;

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

pub struct MMU
{
    /// Interrupt Enable register. Bits 0-4 are toggled depending on the type of
    /// interrupt that should be enabled.
    ie: u8,

    /// Working RAM
    wram: [u8; 0x2000],

    /// High RAM (Zero page RAM)
    hram: [u8; 0x7F],

    /// GPU instance - contains VRAM and OAM
    gpu: GPU,

    /// Gameboy Timer instance
    timer: Timer,
}

impl MMU
{
    /// Creates and returns a new instance of the GameBoy MMU
    pub fn new() -> Self
    {
        MMU 
        {
            ie: 0x0,

            wram: [0x0; 0x2000],
            
            hram: [0x0; 0x7F],
            
            gpu: GPU::new(),

            timer: Timer::new(),
        }
    }

    pub fn run_cycle(&mut self, cycles: u8)
    {
        // TODO: figure out wtf vram cycles is
        let vram_cycles = 0;
        let gpu_cycles = cycles / 1 + vram_cycles;
        let cpu_cycles = cycles + vram_cycles * 1;

        // Run timer cycle
        self.timer.run_cycle(cpu_cycles);
        if self.timer.timer_interrupt() 
        { 
            self.ie |= 0x4; 
        }
        self.timer.set_timer_interrupt(false);

        // Keypad interrupt
        // TODO: this

        // Run GPU cycle
        self.gpu.run_cycle(gpu_cycles);
        // TODO: update IE register based on GPU interrupt status
        // TODO: then reset the GPU interrupt status flags

        // Run SPU cycle
        // TODO: this

        // Serial Interrupt
        // TODO: this
    }

    /// Get the next interrupt to handle based on the value stored in the IE
    /// register
    pub fn next_interrupt(&self) -> Option< Interrupt >
    {
        if self.ie & 0x1 != 0
        {
            Some(Interrupt::VBlank)
        }
        else if self.ie & 0x2 != 0
        {
            Some(Interrupt::LCDC)
        }
        else
        {
            None
        }
    }

    /// Function to read a byte from memory at the given address
    pub fn read_byte(&mut self, addr: u16) -> u8
    {
        match addr
        {
            // ROM
            // TODO: implement cartridges or whatever
            0x0000...0x7FFF => 0x0,

            // VRAM
            0x8000...0x9FFF => self.gpu.read_byte(addr),

            // EXT RAM
            // TODO: implement cartridges or whatever
            0xA000...0xBFFF => 0x0,

            // WRAM Bank 0
            0xC000...0xCFFF => self.wram[(addr - 0xC000) as usize],

            // WRAM Bank 1
            0xD000...0xDFFF => self.wram[(addr - 0xD000) as usize],

            // WRAM ECHO
            0xE000...0xFDFF => self.wram[(addr - 0xE000) as usize],

            // OAM
            0xFE00...0xFE9F => self.gpu.read_byte(addr),

            // Unused
            0xFEA0...0xFEFF => 0x0,

            // IO Registers
            0xFF00...0xFF7F => self.read_io(addr),

            // HRAM
            0xFF80...0xFFFE => self.hram[(addr - 0xFF80) as usize],

            // IE Register
            0xFFFF          => self.ie
        }
    }

    /// Function to write a byte to memory at the given address
    pub fn write_byte(&mut self, addr: u16, val: u8)
    {
        match addr
        {
            // ROM
            0x0000...0x7FFF => { },

            // VRAM
            0x8000...0x9FFF => { self.gpu.write_byte(addr, val); },

            // EXT RAM
            0xA000...0xBFFF => { },

            // WRAM Bank 0
            0xC000...0xCFFF => { self.wram[(addr - 0xC000) as usize] = val; },

            // WRAM Bank 1
            0xD000...0xDFFF => { self.wram[(addr - 0xD000) as usize] = val; },

            // WRAM ECHO
            0xE000...0xFDFF => { self.wram[(addr - 0xE000) as usize] = val; },

            // OAM
            0xFE00...0xFE9F => { self.gpu.write_byte(addr, val); },

            // Unused
            0xFEA0...0xFEFF => { },

            // IO registers
            0xFF00...0xFF7F => { self.write_io(addr, val); },

            // HRAM
            0xFF80...0xFFFE => { self.hram[(addr - 0xFF80) as usize] = val; },

            // IE Register
            0xFFFF          => { self.ie = val; }
        }
    }

    /// Function to read a word from memory at the given address
    pub fn read_word(&mut self, addr: u16) -> u16
    {
        (self.read_byte(addr) as u16)  | ((self.read_byte(addr + 1) as u16) << 8)
    }

    /// Function to write a word to memory at the given address
    pub fn write_word(&mut self, addr: u16, val: u16)
    {
        self.write_byte(addr, (val & 0xFF) as u8);
        self.write_byte(addr + 1, (val >> 8) as u8);
    }

    /// Read a byte from an IO register
    fn read_io(&self, addr: u16) -> u8
    {
        match addr
        {
            // Input
            0xFF00 => {  },

            // Serial Data
            0xFF01 => {  },

            // Serial Control
            0xFF02 => {  },

            // Timer
            0xFF04...0xFF07 => { return self.timer.read_byte(addr); }

            // Interrupt Flag Register
            0xFF0F => {  },

            // SPU
            // TODO: implement sound later
            0xFF10...0xFF3F => {  },

            // GPU
            0xFF40...0xFF4B => { return self.gpu.read_byte(addr); }

            _ => {  }
        }

        0
    }

    /// Write a byte to an IO register
    fn write_io(&mut self, addr: u16, val: u8)
    {
        match addr
        {
            // Input
            0xFF00 => {  },

            // Serial Data
            0xFF01 => {  },

            // Serial Control
            0xFF02 => {  },

            // Timer
            0xFF04...0xFF07 => { self.timer.write_byte(addr, val); }

            // Interrupt Flag Register
            0xFF0F => {  },

            // SPU... implement sound later
            // TODO: this
            0xFF10...0xFF3F => {  },

            // GPU
            0xFF40...0xFF4B => { self.gpu.write_byte(addr, val); },

            _ => {  }
        }
    }
}

/// Gameboy Interrupts ordered by priority
#[derive(Debug, Clone, Copy)]
pub enum Interrupt
{
    VBlank,

    LCDC,

    Timer,
}