mod registers;
mod instructions;

use crate::Target;
use crate::mem::{ Memory, Speed };
use registers::Registers;

/// The different types of GB interrupts
pub enum Interrupts
{
    VBlank  = 0x01,
    LCDStat = 0x02,
    Timer   = 0x04,
    Serial  = 0x08,
    Joypad  = 0x10
}

/// Represents the GB CPU
pub struct CPU
{
    pub regs: Registers,
}

impl CPU
{
    /// Create and return a new instance of the Gameboy CPU
    pub fn new(_target: Target) -> Self
    {
        CPU { regs: Registers::new() }
    }

    /// Execute a CPU cycle
    pub fn exec(&mut self, mem: &mut Memory) -> u32
    {
        // Step the interrupts forward
        self.regs.interrupt_step();

        // Execute next instruction & get the number of ticks it took
        let mut ticks = if self.regs.halt == 0 && self.regs.stop == 0 
        {
            let pc = self.regs.adv();
            let opcode = mem.read_byte(pc);
            instructions::exec(opcode, &mut self.regs, mem)
        } 
        else 
        {
            if self.regs.stop != 0 && mem.speed_switch
            {
                mem.switch_speed();
                self.regs.stop = 0;
            }

            1
        };

        // Handle interrupts
        if self.regs.ime != 0 || self.regs.halt != 0
        {
            let ints = mem.intf & mem.inte;
            if ints != 0
            {
                let i = ints.trailing_zeros();
                if self.regs.ime != 0
                {
                    mem.intf &= !(1 << (i as u32));
                }

                self.regs.ime = 0;
                self.regs.halt = 0;
                self.regs.stop = 0;
                
                match i
                {
                    0 => { self.regs.rst(0x40, mem); },
                    1 => { self.regs.rst(0x48, mem); },
                    2 => { self.regs.rst(0x50, mem); },
                    3 => { self.regs.rst(0x58, mem); },
                    4 => { self.regs.rst(0x60, mem); },
                    _ => {},
                }

                ticks += 1;
            }
        }

        // Multiply ticks based on current speed
        match mem.speed
        {
            Speed::Normal => { ticks *= 4; },
            Speed::Double => { ticks *= 2; }
        }
        
        ticks
    }
}