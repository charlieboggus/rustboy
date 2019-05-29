
pub struct GPU
{
    /// Video RAM
    vram: [u8; 0x2000],

    /// Object Attribute Memory
    oam: [u8; 0xA0],

    /// VBlank Interrupt Status
    vblank_itr: bool,

    /// LCDC Interrupt Status
    lcdc_itr: bool,

    /// LCD Control Register. 
    /// See http://gbdev.gg8.se/wiki/articles/LCDC#Using_LCDC for details
    lcdc: u8,

    /// LCD Status Register
    lcdc_stat: u8,

    // Background Y position
    scy: u8,

    // Background X position
    scx: u8,

    // LCDC Y coordinate
    ly: u8,

    // LY compare
    lyc: u8,

    // Window top-left x position + 7
    wx: u8,

    // Window top-left y position
    wy: u8,

    /// BG Palette Data
    bgp: u8,

    /// Object Palette 0 Data
    obp0: u8,

    /// Object Palette 1 Data
    obp1: u8
}

impl GPU
{
    /// Create and return a new instance of the GPU
    pub fn new() -> Self
    {
        GPU 
        {
            vram: [0u8; 0x2000],

            oam: [0u8; 0xA0],

            vblank_itr: false,

            lcdc_itr: false,

            lcdc: 0u8,

            lcdc_stat: 0u8,

            scy: 0u8,

            scx: 0u8,

            ly: 0u8,

            lyc: 0u8,

            wx: 0u8,

            wy: 0u8,

            bgp: 0u8,

            obp0: 0u8,

            obp1: 0u8,
        }
    }

    pub fn run_cycle(&mut self, cycles: u8)
    {
    }
    
    /// Function to read a byte value from GPU memory
    pub fn read_byte(&self, addr: u16) -> u8
    {
        match addr
        {
            // Read byte from VRAM
            0x8000...0x9FFF => self.vram[(addr - 0x8000) as usize],

            // Read byte from OAM
            0xFE00...0xFE9F => self.oam[(addr - 0xFE00) as usize],

            // LCDC
            0xFF40 => self.lcdc,

            // LCDC Stat
            0xFF41 => self.lcdc_stat,

            // LCDC BG Y pos
            0xFF42 => self.scy,

            // LCDC BG X pos
            0xFF43 => self.scx,

            // Currently displayed line
            0xFF44 => self.ly,

            // Current line compare
            0xFF45 => self.lyc,

            // DMA transfer from ROM/RAM to OAM
            0xFF46 => 0,

            // Background pallette
            0xFF47 => self.bgp,

            // OBJ pallette 0
            0xFF48 => self.obp0,

            // OBJ pallette 1
            0xFF49 => self.obp1,

            // Window Y position
            0xFF4A => self.wy,

            // Window X position
            0xFF4B => self.wx,

            _ => panic!("GPU cannot read from address {:#X}!", addr)
        }
    }

    /// Function to write a byte value to the given address in GPU memory
    pub fn write_byte(&mut self, addr: u16, b: u8)
    {
        match addr
        {
            // Write byte to VRAM
            0x8000...0x9FFF => self.vram[(addr - 0x8000) as usize] = b,

            // Write byte to OAM
            0xFE00...0xFE9F => self.oam[(addr - 0xFE00) as usize] = b,

            // LCDC
            0xFF40 => self.lcdc = b,
            
            // LCDC Stat
            0xFF41 => self.lcdc_stat = b,

            // LCDC BG Y pos
            0xFF42 => self.scy = b,

            // LCDC BG X pos
            0xFF43 => self.scx = b,

            // Currently displayed line
            0xFF44 => self.ly = b,

            // Current line compare
            0xFF45 => self.lyc = b,

            // DMA transfer from ROM/RAM to OAM
            0xFF46 => {  },

            // Background pallette
            0xFF47 => self.bgp = b,

            // OBJ pallette 0
            0xFF48 => self.obp0 = b,

            // OBJ pallette 1
            0xFF49 => self.obp1 = b,

            // Window Y position
            0xFF4A => self.wy = b,

            // Window X position
            0xFF4B => self.wx = b,

            _ => panic!("GPU cannot write to address {:#X}!", addr)
        }
    }

    /// Return status of vblank interrupt
    pub fn vblank_interrupt(&self) -> bool
    {
        self.vblank_itr
    }

    /// Return status of LCDC interrupt
    pub fn lcdc_interrupt(&self) -> bool
    {
        self.lcdc_itr
    }
}