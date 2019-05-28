
pub struct GPU
{
}

impl GPU
{
    pub fn new() -> Self
    {
        GPU {
        }
    }

    pub fn run_cycle(&mut self)
    {
    }

    pub fn read_byte(&self, addr: u16) -> u8
    {
        0
    }

    pub fn write_byte(&self, addr: u16, b: u8)
    {
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