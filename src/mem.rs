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
        match addr
        {
            0x0000...0x7FFF => { /* ROM Bank */ },
            0x8000...0x9FFF => { /* VRAM */ },
            0xA000...0xBFFF => { /* External RAM */ },
            0xC000...0xCFFF | 0xE000...0xEFFF => { /* WRAM bank 0 */ },
            0xD000...0xDFFF | 0xF000...0xFDFF => { /* WRAM bank 1 */ },
            0xFE00...0xFE9F => { /* Sprite Attribute Table */ },
            0xFEA0...0xFEFF => { /* Not Usable */ },
            0xFF00...0xFF7F => { /* I/O Registers */ },
            0xFF80...0xFFFE => { /* HRAM */ },
            0xFFFF => { /* IE Register */ }
        }

        self.memory[addr as usize]
    }

    /// Function to write a byte to memory at the given address
    pub fn write_byte(&mut self, addr: u16, val: u8)
    {
        match addr
        {
            0x0000...0x7FFF => { /* ROM Bank */ },
            0x8000...0x9FFF => { /* VRAM */ },
            0xA000...0xBFFF => { /* External RAM */ },
            0xC000...0xCFFF | 0xE000...0xEFFF => { /* WRAM bank 0 */ },
            0xD000...0xDFFF | 0xF000...0xFDFF => { /* WRAM bank 1 */ },
            0xFE00...0xFE9F => { /* Sprite Attribute Table */ },
            0xFEA0...0xFEFF => { /* Not Usable */ },
            0xFF00...0xFF7F => { /* I/O Registers */ },
            0xFF80...0xFFFE => { /* HRAM */ },
            0xFFFF => { /* IE Register */ }
        }

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