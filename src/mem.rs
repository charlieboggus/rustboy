const UNMAP_BOOTROM: u16 = 0xFF50;

pub struct MMU
{
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
            memory: [0u8; 65536],
        }
    }

    /// Function to read a byte from memory at the given address
    pub fn read_byte(&mut self, addr: u16) -> u8
    {
        if let Some(offset) = map::in_range(addr, map::ROM_BANK)
        {
        }

        if let Some(offset) = map::in_range(addr, map::VRAM)
        {
        }

        if let Some(offset) = map::in_range(addr, map::EXT_RAM)
        {
        }

        if let Some(offset) = map::in_range(addr, map::WRAM)
        {
        }

        if let Some(offset) = map::in_range(addr, map::WRAM_ECHO)
        {
        }

        if let Some(offset) = map::in_range(addr, map::OAM)
        {
        }

        if let Some(offset) = map::in_range(addr, map::IO)
        {
        }

        if let Some(offset) = map::in_range(addr, map::HRAM)
        {
        }

        if addr == map::IE
        {
        }

        // TODO: this is only temporary
        self.memory[addr as usize]
    }

    /// Function to write a byte to memory at the given address
    pub fn write_byte(&mut self, addr: u16, val: u8)
    {
        if let Some(offset) = map::in_range(addr, map::ROM_BANK)
        {
        }

        if let Some(offset) = map::in_range(addr, map::VRAM)
        {
        }

        if let Some(offset) = map::in_range(addr, map::EXT_RAM)
        {
        }

        if let Some(offset) = map::in_range(addr, map::WRAM)
        {
        }

        if let Some(offset) = map::in_range(addr, map::WRAM_ECHO)
        {
        }

        if let Some(offset) = map::in_range(addr, map::OAM)
        {
        }

        if let Some(offset) = map::in_range(addr, map::IO)
        {
        }

        if let Some(offset) = map::in_range(addr, map::HRAM)
        {
        }

        if addr == map::IE
        {
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
}

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
    $FF80-$FFFE: High RAM (HRAM)
    $FFFF-$FFFF: Interrupts Enable Register (IE)
    -----------------------------------------------
    http://gbdev.gg8.se/wiki/articles/Memory_Map
*/
mod map
{
    pub const ROM_BANK: (u16, u16) = (0x0000, 0x7FFF);

    pub const VRAM: (u16, u16) = (0x8000, 0x9FFF);

    pub const EXT_RAM: (u16, u16) = (0xA000, 0xBFFF);

    pub const WRAM: (u16, u16) = (0xC000, 0xDFFF);

    pub const WRAM_ECHO: (u16, u16) = (0xE000, 0xFDFF);

    pub const OAM: (u16, u16) = (0xFE00, 0xFE9F);

    pub const IO: (u16, u16) = (0xFF00, 0xFF7F);

    pub const HRAM: (u16, u16) = (0xFF80, 0xFFFE);

    pub const IE: u16 = 0xFFFF;

    pub fn in_range(addr: u16, range: (u16, u16)) -> Option< u16 >
    {
        let (a, b) = range;
        if addr >= a && addr <= b
        {
            Some(addr - a)
        }
        else
        {
            None
        }
    }

    pub fn range_size(range: (u16, u16)) -> u16
    {
        let (a, b) = range;
        a - b + 1
    }
}