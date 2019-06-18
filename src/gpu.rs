use crate::gb::Target;
use crate::cpu::Interrupts;

const VRAM_SIZE: usize = 8 << 10;
const OAM_SIZE: usize = 0xA0;
const NUM_TILES: usize = 384;

pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 144;

/// A color is simply 4 bytes that represent RGBA values
type Color = [u8; 4];

/// The default GameBoy color palette
const PALETTE: [Color; 4] = [
    [255, 255, 255, 255],   // WHITE
    [192, 192, 192, 255],   // LIGHT GRAY
    [96, 96, 96, 255],      // DARK GRAY
    [0, 0, 0, 255]          // BLACK
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode
{
    HBlank = 0x00,
    VBlank = 0x01,
    RdOAM  = 0x02,
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
    need_update: bool,
    to_update: [bool; NUM_TILES * 2]
}

pub struct GPU
{
    /// Image data to be drawn to the screen
    pub image_data: Box< [u8; WIDTH * HEIGHT * 4] >,

    /// Target GB system
    target: Target,

    /// Internal GPU clock
    internal_clock: u32,

    /// VRAM banks - CGB supports 2 banks of VRAM
    vram: Box< [[u8; VRAM_SIZE]; 2] >,

    /// Selected VRAM bank
    vram_bank: u8,

    /// OAM memory
    oam: [u8; OAM_SIZE],

    /// Current mode
    mode: Mode,

    /// Compiled Palettes. Updated when BGP/OBP0/OBP1 are written to. Meant for
    /// non-CGB use only.
    pal: Box< Palette >,

    /// Compiled tiles
    tiles: Box< Tiles >,

    /// CGB VRAM DMA transfer
    hdma_src: u16,
    hdma_dst: u16,
    hdma5: u8,

    // --------- 0xFF40 - LCD Control Register (LCDC) ---------

    /// LCD Display On/Off (0 = Off, 1 = On)
    pub lcd_enabled: bool,

    /// Window Tilemap Display Select (0 = 0x9800-9BFF, 1 = 0x9C00-9FFF)
        win_tmap: bool,

    /// Window display on/off (0 = Off, 1 = On)
        win_enabled: bool,
    
    /// BG & Window Tile Data Select (0 = 0x8800-97FF, 1 = 0x8000-8FFF)
    pub tile_data: bool,

    /// BG Tilemap Display Select (0 = 0x9800-9BFF, 1 = 0x9C00-9FFF)
        bg_tmap: bool,
    
    /// OBJ (sprite) size (0 = 8x8, 1 = 8x16)
        obj_size: bool,
    
    /// OBJ (sprite) display enabled (0 = Off, 1 = On)
        obj_enabled: bool,
    
    /// BG/Window display/priority (0 = Off, 1 = On)
        bg_enabled: bool,
    
    // --------- 0xFF41 - LCDC Status Register (STAT) ---------

    /// LYC = LY coincidence interrupt (1 = Enabled)
    lycly: bool,

    /// Mode 2 OAM Interrupt (1 = Enable)
    mode2_int: bool,

    /// Mode 1 VBlank Interrupt (1 = Enable)
    mode1_int: bool,

    /// Mode 0 HBlank Interrupt (1 = Enable)
    mode0_int: bool,

    // ------------------ Other Registers ---------------------

    /// 0xFF42 - Scroll Y Register (SCY)
    scy: u8,

    /// 0xFF43 - Scroll X Register (SCX)
    scx: u8,

    /// 0xFF44 - LCDC Y-Coordinate Register (LY)
    ly: u8,

    /// 0xFF45 - LY Compare Register (LYC)
    lyc: u8,

    /// 0xFF47 - BG Palette Data Register (BGP)
    bgp: u8,

    /// 0xFF48 - Object Palette 0 Data Register (OBP0)
    obp0: u8,

    /// 0xFF49 - Object Palette 1 Data Register (OBP1)
    obp1: u8,

    /// 0xFF4A - Window Y Position Register (WY)
    wy: u8,

    /// 0xFF4B - Window X Position (minus 7) Register (WX)
    wx: u8
}

impl GPU
{
    /// Create and return a new instance of the GameBoy GPU
    pub fn new(target: Target) -> Self
    {
        GPU {
            image_data: Box::new([0xFF; HEIGHT * WIDTH * 4]),

            target: target,

            internal_clock: 0,
            
            vram: Box::new([[0x0; VRAM_SIZE]; 2]),
            vram_bank: 0,
            oam: [0x0; OAM_SIZE],
            
            mode: Mode::RdOAM,
            
            pal: Box::new(Palette {
                bg: [[0x0; 4]; 4],
                obp0: [[0x0; 4]; 4],
                obp1: [[0x0; 4]; 4]
            }),
            
            tiles: Box::new(Tiles {
                data: [[[0x0; 8]; 8]; NUM_TILES * 2],
                to_update: [false; NUM_TILES * 2],
                need_update: false
            }),

            hdma_src: 0,
            hdma_dst: 0,
            hdma5: 0,

            lcd_enabled: false,
            win_tmap: false,
            win_enabled: false,
            tile_data: false,
            bg_tmap: false,
            obj_size: false,
            obj_enabled: false,
            bg_enabled: false,

            lycly: false,
            mode2_int: false,
            mode1_int: false,
            mode0_int: false,

            scy: 0x0,
            scx: 0x0,
            ly: 0x0,
            lyc: 0x0,
            bgp: 0x0,
            obp0: 0x0,
            obp1: 0x0,
            wy: 0x0,
            wx: 0x0
        }
    }

    /// Clears the screen to blank white
    pub fn clear(&mut self)
    {
        for i in self.image_data.iter_mut()
        {
            *i = 0xFF;
        }
    }

    /// Step the GPU a given number of ticks forward. The GPU screen is
    /// synchronized with the CPU clock.
    pub fn step(&mut self, ticks: u32, intf: &mut u8)
    {
        self.internal_clock += ticks;

        // If clock >= 456 an entire line has been completed
        if self.internal_clock >= 456
        {
            self.internal_clock -= 456;
            self.ly = (self.ly + 1) % 154;

            if self.ly >= 144 && self.mode != Mode::VBlank
            {
                self.switch_mode(Mode::VBlank, intf);
            }

            // Trigger an LCD Status Interrupt if necessary
            if self.ly == self.lyc && self.lycly
            {
                *intf |= Interrupts::LCDStat as u8;
            }
        }

        // Switch modes if we're not VBlanking
        if self.ly < 144
        {
            if self.internal_clock <= 80
            {
                if self.mode != Mode::RdOAM { self.switch_mode(Mode::RdOAM, intf); }
            }
            else if self.internal_clock <= 252
            {
                if self.mode != Mode::RdVRAM { self.switch_mode(Mode::RdVRAM, intf); }
            }
            else
            {
                if self.mode != Mode::HBlank { self.switch_mode(Mode::HBlank, intf); }
            }
        }
    }

    /// Read a byte from GPU memory
    pub fn read_byte(&self, addr: u16) -> u8
    {
        match addr
        {
            // VRAM
            0x8000...0x9FFF => self.vram[self.vram_bank as usize][(addr & 0x1FFF) as usize],

            // OAM
            0xFE00...0xFE9F => self.oam[(addr & 0xFF) as usize],

            // LCDC Register
            0xFF40 => { 
                ((self.lcd_enabled as u8)   << 7) |
                ((self.win_tmap as u8)      << 6) |
                ((self.win_enabled as u8)   << 5) |
                ((self.tile_data as u8)     << 4) |
                ((self.bg_tmap as u8)       << 3) |
                ((self.obj_size as u8)      << 2) |
                ((self.obj_enabled as u8)   << 1) |
                ((self.bg_enabled as u8)    << 0)
             },

            // LCD STAT Register
            0xFF41 => {
                ((self.lycly as u8) << 6) |
                ((self.mode2_int as u8) << 5) |
                ((self.mode1_int as u8) << 4) |
                ((self.mode0_int as u8) << 3) |
                ((if self.lycly as u8 == self.ly { 1 } else { 0 } as u8) << 2) |
                ((self.mode as u8) << 0)
            },

            // SCY
            0xFF42 => self.scy,

            // SCX
            0xFF43 => self.scx,

            // LY
            0xFF44 => self.ly,

            // LYC
            0xFF45 => self.lyc,

            // BGP
            0xFF47 => self.bgp,

            // OBP0
            0xFF48 => self.obp0,

            // OBP1
            0xFF49 => self.obp1,

            // WY
            0xFF4A => self.wy,

            // WX
            0xFF4B => self.wx,

            // Selected VRAM bank
            0x4F => self.vram_bank,

            // TODO: CGB/SGB stuff ?

            _ => 0xFF
        }
    }

    /// Write a byte to GPU memory
    pub fn write_byte(&mut self, addr: u16, val: u8)
    {
        match addr
        {
            // VRAM
            0x8000...0x9FFF => {
                self.vram[self.vram_bank as usize][(addr & 0x1FFF) as usize] = val;
                if addr < 0x9800 { self.update_tile(addr); }
            },

            // OAM
            0xFE00...0xFE9F => self.oam[(addr & 0xFF) as usize] = val,

            // LCDC Register
            0xFF40 => {
                let b = self.lcd_enabled;

                self.lcd_enabled    = (val >> 7) & 1 != 0;
                self.win_tmap       = (val >> 6) & 1 != 0;
                self.win_enabled    = (val >> 5) & 1 != 0;
                self.tile_data      = (val >> 4) & 1 != 0;
                self.bg_tmap        = (val >> 3) & 1 != 0;
                self.obj_size       = (val >> 2) & 1 != 0;
                self.obj_enabled    = (val >> 1) & 1 != 0;
                self.bg_enabled     = (val >> 0) & 1 != 0;

                if !b && self.lcd_enabled
                {
                    self.internal_clock = 4;
                    self.ly = 0;
                }
            },

            // LCD STAT Register
            0xFF41 => {
                self.lycly          = (val >> 6) & 1 != 0;
                self.mode2_int      = (val >> 5) & 1 != 0;
                self.mode1_int      = (val >> 4) & 1 != 0;
                self.mode0_int      = (val >> 3) & 1 != 0;
                // Other bits are read-only
            },

            // SCY
            0xFF42 => self.scy = val,

            // SCX
            0xFF43 => self.scx = val,

            // 0xFF44 LY is Read Only

            // LYC
            0xFF45 => self.lyc = val,

            // BGP
            0xFF47 => { self.bgp = val; update_palette(&mut self.pal.bg, val); },

            // OBP0
            0xFF48 => { self.obp0 = val; update_palette(&mut self.pal.obp0, val); },

            // OBP1
            0xFF49 => { self.obp1 = val; update_palette(&mut self.pal.obp1, val); },

            // WY
            0xFF4A => self.wy = val,

            // WX
            0xFF4B => self.wx = val,

            // Selected VRAM bank
            0x4F => { 
                if self.target == Target::GameBoyColor 
                { 
                    self.vram_bank = val & 1; 
                } 
            },

            // TODO: CGB/SGB stuff ?

            _ => {}
        }
    }

    /// Register that a tile needs to be updated
    fn update_tile(&mut self, addr: u16)
    {
        let tile_i = (addr & 0x1FFF) / 16;
        let tile_i = tile_i + (self.vram_bank as u16) * (NUM_TILES as u16);
        self.tiles.need_update = true;
        self.tiles.to_update[tile_i as usize] = true;
    }

    /// Switch the current GPU mode
    fn switch_mode(&mut self, mode: Mode, intf: &mut u8)
    {
        self.mode = mode;
        match mode
        {
            Mode::HBlank => {
                self.render_line();
                if self.mode0_int { *intf |= Interrupts::LCDStat as u8; }
            },
            Mode::VBlank => {
                *intf |= Interrupts::VBlank as u8;
                if self.mode1_int { *intf |= Interrupts::LCDStat as u8; }
            },
            Mode::RdOAM => {
                if self.mode2_int { *intf |= Interrupts::LCDStat as u8; }
            },
            Mode::RdVRAM => {}
        }
    }

    /// Render a line to the screen. Performed when the GPU is HBlanking.
    fn render_line(&mut self)
    {
        // We can't render if the LCD isn't on
        if !self.lcd_enabled { return }

        // Line to draw
        let mut scanline = [0u8; WIDTH];

        // Update compiled tiles if necessary 
        if self.tiles.need_update
        {
            self.update_tileset();
            self.tiles.need_update = false;
        }

        // Render BG
        if self.bg_enabled  { self.render_background(&mut scanline); }

        // Render Window
        if self.win_enabled { self.render_window(&mut scanline); }

        // Render Sprites
        if self.obj_enabled { self.render_obj(&mut scanline); }
    }

    fn update_tileset(&mut self)
    {
        let tiles = &mut *self.tiles;
        let iter = tiles.to_update.iter_mut();
        for (i, t) in iter.enumerate().filter(|&(_, &mut i)| i)
        {
            *t = false;
            for j in 0..8
            {
                let addr = ((i % NUM_TILES) * 16) + j * 2;
                let (mut lsb, mut msb) = if i < NUM_TILES
                {
                    (self.vram[0][addr], self.vram[0][addr + 1])
                }
                else
                {
                    (self.vram[1][addr], self.vram[1][addr + 1])
                };

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
        let map_base = self.bg_base();
        let line = self.ly as usize + self.scy as usize;

        let map_base = map_base + ((line % 256) >> 3) * 32;

        // X and Y location inside tile to paint
        let y = (self.ly + self.scy) % 8;
        let mut x = self.scx % 8;

        // Canvas offset
        let mut canvas_offset = (self.ly as usize) * WIDTH * 4;

        let mut i = 0;
        let tile_base = if !self.tile_data { 256 } else { 0 };

        loop
        {
            let map_offset = ((i as usize + self.scx as usize) % 256) >> 3;
            let tile_i = self.vram[0][map_base + map_offset];

            let tile_base = self.add_tile_i(tile_base, tile_i);

            let row;
            let bgpri;
            let hflip;
            let bgp;
            if self.target == Target::GameBoyColor
            {
                let attrs = self.vram[1][map_base + map_offset as usize] as usize;
                let tile = self.tiles.data[tile_base + ((attrs >> 3) & 1) * NUM_TILES];

                bgpri = attrs & 0x80 != 0;
                hflip = attrs & 0x20 != 0;
                row = tile[if attrs & 0x40 != 0 { 7 - y } else { y } as usize];

                // TODO: change to CGB palette
                bgp = self.pal.bg;
            }
            else
            {
                row = self.tiles.data[tile_base as usize][y as usize];
                bgpri = false;
                hflip = false;
                bgp = self.pal.bg;
            }

            while x < 8 && i < WIDTH as u8
            {
                let color_i = row[if hflip { 7 - x } else { x } as usize];
                let color;
                if self.target == Target::SuperGameBoy && self.target != Target::GameBoyColor
                {
                    // TODO: this

                    // TODO: remove this
                    color = bgp[color_i as usize];
                }
                else
                {
                    color = bgp[color_i as usize];
                }

                scanline[i as usize] = if bgpri { 4 } else { color_i };

                self.image_data[canvas_offset]      = color[0];
                self.image_data[canvas_offset + 1]  = color[1];
                self.image_data[canvas_offset + 2]  = color[2];
                self.image_data[canvas_offset + 3]  = color[3];

                x += 1;
                i += 1;
                canvas_offset += 4;
            }

            x = 0;
            if i >= WIDTH as u8 { break }
        }
    }

    fn render_window(&mut self, scanline: &mut [u8; WIDTH])
    {
    }

    fn render_obj(&mut self, scanline: &mut [u8; WIDTH])
    {
    }

    fn add_tile_i(&self, base: usize, tile_i: u8) -> usize
    {
        if self.tile_data { base + tile_i as usize } else { (base as isize + (tile_i as i8 as isize)) as usize }
    }

    fn bg_base(&self) -> usize
    {
        if self.bg_tmap { 0x1C00 } else { 0x1800 }
    }
}

/// Update cached palettes for BG/OBP0/OBP1. Called whenever the registers
/// are written to or modified.
fn update_palette(pal: &mut [Color; 4], val: u8)
{
    pal[0] = PALETTE[((val >> 0) & 0x3) as usize];
    pal[1] = PALETTE[((val >> 2) & 0x3) as usize];
    pal[2] = PALETTE[((val >> 4) & 0x3) as usize];
    pal[3] = PALETTE[((val >> 6) & 0x3) as usize];
}