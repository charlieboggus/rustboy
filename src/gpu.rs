
/// Width of the GameBoy screen in pixels
pub const WIDTH: usize = 160;

/// Height of the GameBoy screen in pixels
pub const HEIGHT: usize = 144;

const NUM_TILES: usize = 384;

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

    /// Compiled palettes
    pal: Palette,

    /// Compiled tiles
    tiles: Tiles,

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
            pal: Palette { 
                bg: [[0; 4]; 4], 
                obp0: [[0; 4]; 4], 
                obp1: [[0; 4]; 4] 
            },
            tiles: Tiles { 
                data: [[[0; 8]; 8]; NUM_TILES * 2], 
                dirty: false, 
                to_update: [false; NUM_TILES * 2] 
            },
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

    /// Clears the screen to blank white
    pub fn clear(&mut self)
    {
        for pixel in self.image_data.iter_mut()
        {
            *pixel = 0xFF;
        }
    }

    /// Run a cycle of the GPU
    pub fn run_cycle(&mut self, ticks: u32, intf: &mut u8)
    {
        self.clock += ticks;
        
        // If clock >= 456 then an entire line has been completed
        if self.clock >= 456
        {
            self.clock -= 456;
            self.ly = (self.ly + 1) % 154;

            if self.ly >= 144 && self.stat.mode != Mode::VBlank
            {
                self.switch_mode(Mode::VBlank, intf);
            }

            if self.ly == self.lyc && self.stat.lycly
            {
                *intf |= 0x02;
            }
        }

        // Switch modes if we're not in VBlank
        if self.ly < 144
        {
            if self.clock <= 80
            {
                if self.stat.mode != Mode::RdOAM { self.switch_mode(Mode::RdOAM, intf); }
            }
            else if self.clock <= 252
            {
                if self.stat.mode != Mode::RdVRAM { self.switch_mode(Mode::RdVRAM, intf); }
            }
            else
            {
                if  self.stat.mode != Mode::HBlank { self.switch_mode(Mode::HBlank, intf); }
            }
        }
    }

    fn switch_mode(&mut self, mode: Mode, intf: &mut u8)
    {
        self.stat.mode = mode;
        match mode
        {
            Mode::HBlank => 
            {
                self.render_line();
                if self.stat.mode0_hblank
                {
                    *intf |= 0x02;
                }
            },

            Mode::VBlank => 
            {
                *intf |= 0x01;
                if self.stat.mode1_vblank
                {
                    *intf |= 0x02;
                }
            },

            Mode::RdOAM => 
            {
                if self.stat.mode2_oam
                {
                    *intf |= 0x02;
                }
            },

            Mode::RdVRAM => { }
        }
    }

    fn render_line(&mut self)
    {
        if !self.lcdc.lcd_enable { return }

        let mut scanline = [0u8; WIDTH];

        if self.tiles.dirty
        {
            self.update_tileset();
            self.tiles.dirty = false;
        }

        if self.lcdc.bg_enable
        {
            self.render_background(&mut scanline);
        }

        if self.lcdc.win_enable
        {
            self.render_window(&mut scanline);
        }

        if self.lcdc.obj_enable
        {
            self.render_sprites(&mut scanline);
        }
    }

    fn update_tileset(&mut self)
    {
        let tiles = &mut self.tiles;
        let iter = tiles.to_update.iter_mut();
        for (i, slot) in iter.enumerate().filter(|&(_, &mut i)| i)
        {
            *slot = false;
            for j in 0..8
            {
                let addr = ((i % NUM_TILES) * 16) + j * 2;
                let (mut lsb, mut msb) = (self.vram[addr], self.vram[addr + 1]);

                for k in (0..8).rev()
                {
                    tiles.data[i][j][k] = ((msb & 1) << 1) | (lsb & 1);
                    lsb >>= 1;
                    msb >>= 1;
                }
            }
        }
    }

    fn render_background(&mut self, scanline: &mut [u8; WIDTH])
    {
    }

    fn render_window(&mut self, scanline: &mut [u8; WIDTH])
    {
    }

    fn render_sprites(&mut self, scanline: &mut [u8; WIDTH])
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
            0xFF40 => {
                let mut b = 0x0;
                if self.lcdc.lcd_enable     { b |= 0x80; }
                if self.lcdc.win_tmap       { b |= 0x40; }
                if self.lcdc.win_enable     { b |= 0x20; }
                if self.lcdc.tile_data      { b |= 0x10; }
                if self.lcdc.bg_tmap        { b |= 0x8; }
                if self.lcdc.obj_size       { b |= 0x4; }
                if self.lcdc.obj_enable     { b |= 0x2; }
                if self.lcdc.bg_enable      { b |= 0x1; }

                b
            },

            // LCDC Stat
            0xFF41 => {
                let mut b = 0x0;
                if self.stat.lycly              { b |= 0x40; }
                if self.stat.mode2_oam          { b |= 0x20; }
                if self.stat.mode1_vblank       { b |= 0x10; }
                if self.stat.mode0_hblank       { b |= 0x8; }
                if self.stat.coincidence_flag   { b |= 0x4; }
                b |= self.stat.mode as u8;

                b
            },

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

            // Write a byte to LCDC Register
            0xFF40 => {
                self.lcdc.lcd_enable = if b & 0x80 != 0 { true } else { false };
                self.lcdc.win_tmap = if b & 0x40 != 0 { true } else { false };
                self.lcdc.win_enable = if b & 0x20 != 0 { true } else { false };
                self.lcdc.tile_data = if b & 0x10 != 0 { true } else { false };
                self.lcdc.bg_tmap = if b & 0x8 != 0 { true } else { false };
                self.lcdc.obj_size = if b & 0x4 != 0 { true } else { false };
                self.lcdc.obj_enable = if b & 0x2 != 0 { true } else { false };
                self.lcdc.bg_enable = if b & 0x1 != 0 { true } else { false };
            },
            
            // Write a byte to LCDC Status Register
            0xFF41 => {
                self.stat.lycly = if b & 0x40 != 0 { true } else { false };
                self.stat.mode2_oam = if b & 0x20 != 0 { true } else { false };
                self.stat.mode1_vblank = if b & 0x10 != 0 { true } else { false };
                self.stat.mode0_hblank = if b & 0x8 != 0 { true } else { false };
                self.stat.coincidence_flag = if b & 0x4 != 0 { true } else { false };
                self.stat.mode = 
                if b & 0x3 != 0 {
                    Mode::RdVRAM 
                } 
                else if b & 0x2 != 0 { 
                    Mode::RdOAM 
                } 
                else if b & 0x1 != 0 { 
                    Mode::VBlank 
                } 
                else { 
                    Mode::HBlank 
                };
            },

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

struct Tiles
{
    data: [[[u8; 8]; 8]; NUM_TILES * 2],
    dirty: bool,
    to_update: [bool; NUM_TILES * 2]
}