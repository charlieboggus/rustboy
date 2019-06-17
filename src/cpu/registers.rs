use crate::mem::Memory;
use std::fmt;

#[derive(Debug, Clone, Copy)]
pub struct Registers
{
    /// 8-bit 'A' register
    pub a: u8,

    /// 8-bit 'B' register
    pub b: u8,

    /// 8-bit 'C' register
    pub c: u8,

    /// 8-bit 'D' register
    pub d: u8,

    /// 8-bit 'E' register
    pub e: u8,

    // 8-bit 'F' register used for CPU flags
    pub f: u8,

    /// 8-bit 'H' register
    pub h: u8,

    /// 8-bit 'L' register
    pub l: u8,

    /// Stack Pointer: points to the current stack position
    pub sp: u16,

    /// Program Counter: points to next instruction to be executed
    pub pc: u16,

    /// Interrupt Master Enable register. Flag for whether interrupts are
    /// enabled or not.
    pub ime: u32,

    /// Halt flag. Flag for whether a halt has happened or should happen
    pub halt: u32,

    /// Stop flag. Flag for whether a stop has happened or should happen
    pub stop: u32,

    pub delay: u32
}

impl Registers
{
    /// Create and return a new instance of the GameBoy registers. Values are
    /// initialized based on the GameBoy startup sequence.
    pub fn new() -> Self
    {
        Registers
        {
            a: 0x01,
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xD8,
            f: 0xB0,
            h: 0x01,
            l: 0x4D,
            sp: 0xFFFE,
            pc: 0x0100,
            ime: 0,
            halt: 0,
            stop: 0,
            delay: 0
        }
    }

    /// Advances the program counter
    pub fn adv(&mut self) -> u16
    {
        let pc = self.pc;
        self.pc += 1;
        pc
    }

    /// Get the value in the 16-bit 'AF' register
    pub fn af(&self) -> u16 { ((self.a as u16) << 8) | (self.f as u16) }

    /// Get the value in the 16-bit 'BC' register
    pub fn bc(&self) -> u16 { ((self.b as u16) << 8) | (self.c as u16) }

    /// Get the value in the 16-bit 'DE' register
    pub fn de(&self) -> u16 { ((self.d as u16) << 8) | (self.e as u16) }

    /// Get the value in the 16-bit 'HL' register
    pub fn hl(&self) -> u16 { ((self.h as u16) << 8) |(self.l as u16) }

    /// Decrement HL
    pub fn hlmm(&mut self)
    {
        self.l -= 1;
        if self.l == 0xFF
        {
            self.h -= 1;
        }
    }

    /// Increment HL
    pub fn hlpp(&mut self)
    {
        self.l += 1;
        if self.l == 0
        {
            self.h += 1;
        }
    }

    /// Return from the current subroutine
    pub fn ret(&mut self, m: &mut Memory)
    {
        self.pc = m.read_word(self.sp);
        self.sp += 2;
    }

    pub fn rst(&mut self, i: u16, m: &mut Memory)
    {
        self.sp -= 2;
        m.write_word(self.sp, self.pc);
        self.pc = i;
    }

    /// Schedule enabling of interrupts
    pub fn ei(&mut self, m: &mut Memory)
    {
        if self.delay == 2 || m.read_byte(self.pc) == 0x76
        {
            self.delay = 1;
        }
        else
        {
            self.delay = 2;
        }
    }

    /// Disable interrupts
    pub fn di(&mut self)
    {
        self.ime = 0;
        self.delay = 0;
    }

    pub fn interrupt_step(&mut self)
    {
        match self.delay
        {
            0 => {},
            1 => { self.delay = 0; self.ime = 1; },
            2 => { self.delay = 1; }
            _ => {}
        }
    }
}

impl fmt::Display for Registers
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        write!(f, "a: {:#X}, b: {:#X}, c: {:#X}, d: {:#X}, e: {:#X},
                   h: {:#X}, l: {:#X}, sp: {:#X}, pc: {:#X}", 
                   self.a, self.b, self.c, self.d, 
                   self.e, self.h, self.l, self.sp, 
                   self.pc)
    }
}