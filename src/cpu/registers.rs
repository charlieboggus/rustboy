use crate::mem::Memory;

/// Represents all of the GB CPU registers
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
            pc: 0x100,
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
        self.pc = self.pc.overflowing_add(1).0;    
        pc
    }

    /// Get the value in the 16-bit 'BC' register
    pub fn bc(&self) -> u16 { ((self.b as u16) << 8) | (self.c as u16) }

    /// Get the value in the 16-bit 'DE' register
    pub fn de(&self) -> u16 { ((self.d as u16) << 8) | (self.e as u16) }

    /// Get the value in the 16-bit 'HL' register
    pub fn hl(&self) -> u16 { ((self.h as u16) << 8) |(self.l as u16) }

    /// Decrement HL
    pub fn dec_hl(&mut self)
    {
        let val = self.l.overflowing_sub(1);
        self.l = val.0;
        if self.l == 0xFF
        {
            let val = self.h.overflowing_sub(1);
            self.h = val.0;
        }
    }

    /// Increment HL
    pub fn inc_hl(&mut self)
    {
        let val = self.l.overflowing_add(1);
        self.l = val.0;
        if self.l == 0x0
        {
            let val = self.h.overflowing_add(1);
            self.h = val.0;
        }
    }

    /// Return from the current subroutine
    pub fn ret(&mut self, mem: &mut Memory)
    {
        self.pc = mem.read_word(self.sp);
        self.sp = self.sp.overflowing_add(2).0;
    }

    /// Reset PC to given address, i
    pub fn rst(&mut self, i: u16, mem: &mut Memory)
    {
        self.sp = self.sp.overflowing_sub(2).0;
        mem.write_word(self.sp, self.pc);
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

    /// Schedule Disabling of interrupts
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