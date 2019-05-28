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
    /// Working RAM
    wram: [u8; 0x2000],

    /// High RAM (Zero page RAM)
    hram: [u8; 0x7F],

    /// GPU instance - contains VRAM and OAM
    gpu: GPU,

    /// Gameboy Timer instance
    timer: Timer,

    // TODO: this is only temporary
    memory: [u8; 65536],
}

impl MMU
{
    /// Creates and returns a new instance of the GameBoy MMU
    pub fn new() -> Self
    {
        MMU 
        {
            wram: [0u8; 0x2000],
            
            hram: [0u8; 0x7F],
            
            gpu: GPU::new(),

            timer: Timer::new(),

            memory: [0u8; 65536],
        }
    }

    pub fn run_cycle(&mut self)
    {
    }

    /// Function to read a byte from memory at the given address
    pub fn read_byte(&mut self, addr: u16) -> u8
    {
        match addr
        {
            0x0000...0x7FFF => { /* ROM */ },
            0x8000...0x9FFF => { return self.gpu.read_byte(addr); },
            0xA000...0xBFFF => { /* EXT RAM */ },
            0xC000...0xCFFF => { return self.wram[(addr - 0xC000) as usize]; },
            0xD000...0xDFFF => { return self.wram[(addr - 0xD000) as usize]; },
            0xE000...0xFDFF => { return self.wram[(addr - 0xE000) as usize];},
            0xFE00...0xFE9F => { return self.gpu.read_byte(addr); },
            0xFEA0...0xFEFF => { return 0x0; }, // Not used
            0xFF00...0xFF7F => { return self.read_io(addr); },
            0xFF80...0xFFFE => { return self.hram[(addr - 0xFF80) as usize]; },
            0xFFFF          => { /* IE Register */ }
        }

        self.memory[addr as usize]
    }

    /// Function to write a byte to memory at the given address
    pub fn write_byte(&mut self, addr: u16, val: u8)
    {
        match addr
        {
            0x0000...0x7FFF => { /* ROM */ },
            0x8000...0x9FFF => { self.gpu.write_byte(addr, val); },
            0xA000...0xBFFF => { /* EXT RAM */ },
            0xC000...0xCFFF => { self.wram[(addr - 0xC000) as usize] = val; },
            0xD000...0xDFFF => { self.wram[(addr - 0xD000) as usize] = val; },
            0xE000...0xFDFF => { self.wram[(addr - 0xE000) as usize] = val; },
            0xFE00...0xFE9F => { self.gpu.write_byte(addr, val); },
            0xFEA0...0xFEFF => { /* Not Used */ },
            0xFF00...0xFF7F => { self.write_io(addr, val); },
            0xFF80...0xFFFE => { self.hram[(addr - 0xFF80) as usize] = val; },
            0xFFFF          => { /* IE Register */ }
        }

        // TODO: this is only temporary
        self.memory[addr as usize] = val;
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

    /// Creates and returns a memory dump String for debugging
    pub fn dump(&self) -> String
    {
        let mut dump = String::new();
        for i in 0..self.memory.len()
        {
            let s = format!("{:#X}: {:#X}\n", i as u16, self.memory[i]);
            dump.push_str(&s[..]);
        }
        
        dump
    }

    fn read_io(&self, addr: u16) -> u8
    {
        let offset = addr - 0xFF00;
        match offset
        {
            // Input
            0x00 => {  },

            // Serial Data
            0x01 => {  },

            // Serial Control
            0x02 => {  },

            // Timer DIV register
            0x04 => { return self.timer.read_byte(addr); },

            // Timer TIMA register
            0x05 => { return self.timer.read_byte(addr); },

            // Timer TMA register
            0x06 => { return self.timer.read_byte(addr); },

            // Timer TCA register
            0x07 => { return self.timer.read_byte(addr); },

            // Interrupt Flag Register
            0x0F => {  },

            // Sound stuff... implement later
            // TODO: this
            0x10...0x3F => {  },

            // LCD Control (LCDC) register
            0x40 => { return self.gpu.read_byte(addr); },

            // LCDC Status + IT selection
            0x41 => { return self.gpu.read_byte(addr); },

            // LCDC bg Y pos
            0x42 => { return self.gpu.read_byte(addr); },

            // LCDC bg X pos
            0x43 => { return self.gpu.read_byte(addr); },

            // Currently displayed line
            0x44 => { return self.gpu.read_byte(addr); },

            // Current line compare
            0x45 => { return self.gpu.read_byte(addr); },

            // DMA transfer from ROM/RAM to OAM
            0x46 => { return self.gpu.read_byte(addr); },

            // Background pallette
            0x47 => { return self.gpu.read_byte(addr); },

            // Sprite Pallette 0
            0x48 => { return self.gpu.read_byte(addr); },

            // Sprite Pallette 1
            0x49 => { return self.gpu.read_byte(addr); },

            // Window Y position
            0x4A => { return self.gpu.read_byte(addr); },

            // Window X position
            0x4B => { return self.gpu.read_byte(addr); },

            _ => {  }
        }
        0
    }

    fn write_io(&mut self, addr: u16, val: u8)
    {
        let offset = addr - 0xFF00;
        match offset
        {
            // Input
            0x00 => {  },

            // Serial Data
            0x01 => {  },

            // Serial Control
            0x02 => {  },

            // Timer DIV register
            0x04 => { self.timer.write_byte(addr, val); },

            // Timer TIMA register
            0x05 => { self.timer.write_byte(addr, val); },

            // Timer TMA register
            0x06 => { self.timer.write_byte(addr, val); },

            // Timer TCA register
            0x07 => { self.timer.write_byte(addr, val); },

            // Interrupt Flag Register
            0x0F => {  },

            // Sound stuff... implement later
            // TODO: this
            0x10...0x3F => {  },

            // LCD Control (LCDC) register
            0x40 => { self.gpu.write_byte(addr, val); },

            // LCDC Status + IT selection
            0x41 => { self.gpu.write_byte(addr, val); },

            // LCDC bg Y pos
            0x42 => { self.gpu.write_byte(addr, val); },

            // LCDC bg X pos
            0x43 => { self.gpu.write_byte(addr, val); },

            // Currently displayed line
            0x44 => { self.gpu.write_byte(addr, val); },

            // Current line compare
            0x45 => { self.gpu.write_byte(addr, val); },

            // DMA transfer from ROM/RAM to OAM
            0x46 => { self.gpu.write_byte(addr, val); },

            // Background pallette
            0x47 => { self.gpu.write_byte(addr, val); },

            // Sprite Pallette 0
            0x48 => { self.gpu.write_byte(addr, val); },

            // Sprite Pallette 1
            0x49 => { self.gpu.write_byte(addr, val); },

            // Window Y position
            0x4A => { self.gpu.write_byte(addr, val); },

            // Window X position
            0x4B => { self.gpu.write_byte(addr, val); },

            _ => {  }
        }
    }
}