
pub struct GPU
{
    /// Video RAM
    vram: [u8; 0x2000],

    /// Object Attribute Memory
    oam: [u8; 0xA0],

    // Background Y position
    scy: u8,

    // Background X position
    scx: u8,

    // Window top-left x position + 7
    wx: u8,

    // Window top-left y position
    wy: u8,
}

impl GPU
{
    pub fn new() -> Self
    {
        GPU 
        {
            vram: [0u8; 0x2000],

            oam: [0u8; 0xA0],

            scy: 0u8,

            scx: 0u8,

            wx: 0u8,

            wy: 0u8
        }
    }

    pub fn run_cycle(&mut self)
    {
    }

    pub fn read_byte(&self, addr: u16) -> u8
    {
        match addr
        {
            // Read byte from VRAM
            0x8000...0x9FFF => self.vram[(addr - 0x8000) as usize],

            // Read byte from OAM
            0xFE00...0xFE9F => self.oam[(addr - 0xFE00) as usize],

            0xFF40 => 0,
            0xFF41 => 0,
            0xFF42 => 0,
            0xFF43 => 0,
            0xFF44 => 0,
            0xFF45 => 0,
            0xFF46 => 0,
            0xFF47 => 0,
            0xFF48 => 0,
            0xFF49 => 0,
            0xFF4A => 0,
            0xFF4B => 0,

            _ => 0
        }
    }

    pub fn write_byte(&mut self, addr: u16, b: u8)
    {
        match addr
        {
            // Write byte to VRAM
            0x8000...0x9FFF => { self.vram[(addr - 0x8000) as usize] = b; },

            // Write byte to OAM
            0xFE00...0xFE9F => { self.oam[(addr - 0xFE00) as usize] = b; },

            0xFF40 => {  },
            0xFF41 => {  },
            0xFF42 => {  },
            0xFF43 => {  },
            0xFF44 => {  },
            0xFF45 => {  },
            0xFF46 => {  },
            0xFF47 => {  },
            0xFF48 => {  },
            0xFF49 => {  },
            0xFF4A => {  },
            0xFF4B => {  },

            _ => {  }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color
{
    White       = 0,
    LightGrey   = 1,
    DarkGrey    = 2,
    Black       = 3
}

impl Color
{
    /// Create a color from a byte in the range 0x0..0x3
    fn from_byte(b: u8) -> Self
    {
        match b
        {
            0x0 => Color::White,
            0x1 => Color::LightGrey,
            0x2 => Color::DarkGrey,
            0x3 => Color::Black,
            _ => panic!("Invalid color: {:#X}!", b)
        }
    }
}