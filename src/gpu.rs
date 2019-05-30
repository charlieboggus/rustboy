
/// Width of the GameBoy screen in pixels
pub const WIDTH: usize = 160;

/// Height of the GameBoy screen in pixels
pub const HEIGHT: usize = 144;

/// The color palette for the Gameboy. Possible values:
/// 
/// 0 - White, 
/// 1 - Light Gray, 
/// 2 - Dark Gray, 
/// 3 - Black
const COLOR_PALETTE: [Color; 4] = [
    [255, 255, 255, 255],
    [192, 192, 192, 255],
    [96, 96, 96, 255],
    [0, 0, 0, 255]
];

/// Represents a color. Each value is an RGBA value
pub type Color = [u8; 4];

pub struct GPU
{
    /// Data about what is currently being displayed on screen
    image_data: [u8; WIDTH * HEIGHT * 4],

    /// Internal GPU clock
    clock: u32,

    /// Video RAM
    vram: [u8; 0x2000],

    /// Object Attribute Memory
    oam: [u8; 0xA0],

    // -------------------- GPU Registers --------------------

    /// 0xFF40 - LCD Control Register
    lcdc: LCDC,

    /// 0xFF41 - LCDC Status Register
    stat: STAT,

    /// 0xFF42 - Background Y position
    scy: u8,

    /// 0xFF43 - Background X position
    scx: u8,

    /// 0xFF44 - LCDC Y coordinate
    ly: u8,

    /// 0xFF45 - LY compare
    lyc: u8,

    /// 0xFF47 - BG Palette Data
    bgp: u8,

    /// 0xFF48 - Object Palette 0 Data
    obp0: u8,

    /// 0xFF49 - Object Palette 1 Data
    obp1: u8,

    /// 0xFF4A - Window top-left x position + 7
    wx: u8,

    /// 0xFF4B - Window top-left y position
    wy: u8,
}

impl GPU
{
    /// Create and return a new instance of the GPU
    pub fn new() -> Self
    {
        let lcdc = LCDC 
        {
            lcd_enable: false,
            win_tmap: false,
            win_enable: false,
            tile_data: false,
            bg_tmap: false,
            obj_size: false,
            obj_enable: false,
            bg_enable: false
        };

        let stat = STAT 
        {
            lycly: false,
            mode2_oam: false,
            mode1_vblank: false,
            mode0_hblank: false,
            coincidence_flag: false,
            mode: Mode::RdOAM
        };

        GPU 
        {
            image_data: [0u8; WIDTH * HEIGHT * 4],
            clock: 0u32,
            vram: [0u8; 0x2000],
            oam: [0u8; 0xA0],
            lcdc: lcdc,
            stat: stat,
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

    pub fn run_cycle(&mut self, ticks: u32, intf: &mut u8)
    {
    }

    fn set_mode(&mut self, mode: Mode)
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
            0xFF40 => 0,

            // LCDC Stat
            0xFF41 => 0,

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
            0xFF40 => {  },
            
            // LCDC Stat
            0xFF41 => {  },

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
}

/// LCD Control Register (LCDC). Fields are ordered from most to least
/// significant bit.
/// http://gbdev.gg8.se/wiki/articles/Video_Display#LCD_Control_Register
struct LCDC
{
    /// Bit 7: LCD Display Enable 
    /// (0 = Off, 1 = On)
    lcd_enable: bool,
    
    /// Bit 6: Window Tile Map Display Select 
    /// (0 = [0x9800 - 0x9BFF], 1 = [0x9C00 - 0x9FFF])
    win_tmap: bool,

    /// Bit 5: Window Display Enable
    /// (0 = Off, 1 = On)
    win_enable: bool,

    /// Bit 4: BG & Window Tile Data Select
    /// (0 = [0x8800 - 0x97FF], 1 = [0x8000 - 0x8FFF])
    tile_data: bool,

    /// Bit 3: BG Tile Map Display Select
    /// (0 = [0x9800 - 0x9BFF], 1 = [0x9C00 - 0x9FFF])
    bg_tmap: bool,

    /// Bit 2: OBJ (sprite) size
    /// (0 = 8x8, 1 = 8x16)
    obj_size: bool,

    /// Bit 1: OBJ (sprite) Display Enable
    /// (0 = Off, 1 = On)
    obj_enable: bool,

    /// Bit 0: BG Display Enable
    /// (0 = Off, 1 = On)
    bg_enable: bool
}

/// LCDC Status Register (STAT). Starts with bit 6.
/// http://gbdev.gg8.se/wiki/articles/Video_Display#LCD_Status_Register
struct STAT
{
    /// Bit 6: LYC = LY Coincidence Interrupt
    /// (1 = Enable)
    lycly: bool,

    /// Bit 5: Mode 2 OAM Interrupt
    /// (1 = Enable)
    mode2_oam: bool,

    /// Bit 4: Mode 1 VBlank Interrupt
    /// (1 = Enable)
    mode1_vblank: bool,

    /// Bit 3: Mode 0 HBlank Interrupt
    /// (1 = Enable)
    mode0_hblank: bool,

    /// Bit 2: Coincidence Flag
    /// (0 = [LYC != LY], 1 = [LYC == LY])
    coincidence_flag: bool,

    /// Bits 0-1: Mode Flag
    mode: Mode
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode
{
    /// In Horizontal Blanking Mode
    HBlank = 0x00,

    /// In Vertical Blanking Mode
    VBlank = 0x01,

    /// Searching OAM memory
    RdOAM  = 0x02,

    /// Accessing sprite and video memory
    RdVRAM = 0x03
}

struct Palette
{
    bg: [Color; 4],
    obp0: [Color; 4],
    obp1: [Color; 4]
}