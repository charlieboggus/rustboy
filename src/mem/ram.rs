use std::iter;

pub struct RAM
{
    data: Vec< u8 >
}

impl RAM
{
    /// Create and return a new instance of RAM. The default values are filled
    /// with garbage since they are usually undetermined
    pub fn new(size: usize) -> Self
    {
        let data = iter::repeat(0xCA).take(size).collect();
        RAM { data: data }
    }

    /// Read a byte from RAM at the given address
    pub fn read_byte(&self, addr: u16) -> u8
    {
        self.data[addr as usize]
    }

    /// Write a byte to RAM at the given address
    pub fn write_byte(&mut self, addr: u16, val: u8)
    {
        self.data[addr as usize] = val;
    }
}