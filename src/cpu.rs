pub struct CPU
{
    reg: Registers,
    halted: bool,
}

impl CPU
{
    pub fn new() -> Self
    {
        CPU
        {
            reg: Registers::new(),
            halted: false,
        }
    }

    pub fn run_cycle(&mut self)
    {
    }

    fn fetch_opcode(&self)
    {
    }

    fn execute_opcode(&mut self)
    {
    }
}

/// Represents all of the Gameboy registers
#[derive(Debug, Clone, Copy)]
pub struct Registers
{
    // GameBoy 8-bit registers
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,

    /// Flag register
    pub f: u8,

    /// Stack Pointer: points to the current stack position
    pub sp: u16,

    /// Program Counter: points to next instruction to be executed
    pub pc: u16
}

impl Registers
{
    /// Create and return a new instance of Registers
    pub fn new() -> Self
    {
        // TODO: figure out what to initialize values to
        Registers
        {
            a: 0x0,
            b: 0x0,
            c: 0x0,
            d: 0x0,
            e: 0x0,
            h: 0x0,
            l: 0x0,
            f: 0x0,
            sp: 0x0,
            pc: 0x0
        }
    }

    /// Set the value stored in the 16-bit paired register AF
    pub fn set_af(&mut self, val: u16)
    {
        self.a = (val >> 8) as u8;
        self.f = (val & 0x00F0) as u8;
    }

    /// Get the value stored in the 16-bit paired register AF
    pub fn af(&self) -> u16
    {
        ((self.a as u16) << 8) |  ((self.f & 0xF0) as u16)
    }

    /// Set the value stored in the 16-bit paired register BC
    pub fn set_bc(&mut self, val: u16)
    {
        self.b = (val >> 8) as u8;
        self.c = (val & 0x00FF) as u8;
    }

    /// Get the value stored in the 16-bit paired register BC
    pub fn bc(&self) -> u16
    {
        ((self.b as u16) << 8) |  (self.c as u16)
    }

    /// Set the value stored in the 16-bit paired register DE
    pub fn set_de(&mut self, val: u16)
    {
        self.d = (val >> 8) as u8;
        self.e = (val & 0x00FF) as u8;
    }

    /// Get the value stored in the 16-bit paired register DE
    pub fn de(&self) -> u16
    {
        ((self.d as u16) << 8) |  (self.e as u16)
    }

    /// Set the value stored in the 16-bit paired register HL
    pub fn set_hl(&mut self, val: u16)
    {
        self.h = (val >> 8) as u8;
        self.l = (val & 0x00FF) as u8;
    }

    /// Get the value stored in the 16-bit paired register HL
    pub fn hl(&self) -> u16
    {
        ((self.h as u16) << 8) |  (self.l as u16)
    }
}

bitflags!
{
    /// Bitflag representations of the flag register
    pub struct Flags: u8
    {
        /// Zero Flag: set when the result of a math operation is zero
        const Z = 0b10000000;

        /// Subtract Flag: Set if a subtraction was performed in the last math 
        /// instruction
        const N = 0b01000000;

        /// Half Carry Flag: Set if a carry occurred from the lower nibble in
        /// the last math operation
        const H = 0b00100000;

        /// Carry Flag: Set if a carry occurred from the last math operation or
        /// if register A is the smaller value when executing the CP instruction
        const C = 0b00010000;
    }
}