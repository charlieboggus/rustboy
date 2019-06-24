use crate::mem::Speed;
use crate::cpu::Interrupts;

struct InternalClock
{
    div: u32,
    tima: u32
}

pub struct Timer
{
    /// Timer Divider (DIV) register. Counts up at a fixed 16kHz and resets to 0
    /// the register is writen to. Located at 0xFF04.
    div: u8,

    /// Timer Counter (TIMA) register. Counts up at timer speed. Triggers 
    /// Interrupt INT 0x50 on overflow. Located at 0xFF05.
    tima: u8,

    /// Timer Modulo (TMA) register. When TIMA overflows to 0 it is reset to 
    /// start at the value stored in TMA. Located at 0xFF06.
    tma: u8,

    /// Timer Control (TAC) register. Stores timer control information. 
    /// Located at 0xFF07.
    tac: u8,

    clock: InternalClock,

    speed: u32
}

impl Timer
{
    pub fn new() -> Self
    {
        Timer {
            div: 0,
            tima: 0,
            tma: 0,
            tac: 0,
            clock: InternalClock { tima: 0, div: 0 },
            speed: 256
        }
    }

    /// Step the timer a given number of ticks forward
    pub fn step(&mut self, ticks: u32, intf: &mut u8, speed: Speed)
    {
        let ticks = match speed
        {
            Speed::Normal => ticks / 4,
            Speed::Double => ticks
        };
        self.clock.div = self.clock.div.overflowing_add(ticks).0;

        // Increment DIV as necessary
        while self.clock.div >= 64
        {
            self.div = self.div.overflowing_add(1).0;
            self.clock.div = self.clock.div.overflowing_sub(64).0;
        }

        // Increment TIMA as necessary
        if self.tac & 0x4 != 0
        {
            self.clock.tima = self.clock.tima.overflowing_add(ticks).0;
            while self.clock.tima >= self.speed
            {
                self.tima = self.tima.overflowing_add(1).0;
                if self.tima == 0
                {
                    self.tima = self.tma;
                    *intf |= Interrupts::Timer as u8;
                }
                self.clock.tima = self.clock.tima.overflowing_sub(self.speed).0;
            }
        }
    }

    fn update(&mut self)
    {
        match self.tac & 0x3
        {
            0x0 => self.speed = 256,
            0x1 => self.speed = 4,
            0x2 => self.speed = 16,
            0x3 => self.speed = 64,
            _ => {}
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8
    {
        match addr
        {
            0xFF04 => self.div,
            0xFF05 => self.tima,
            0xFF06 => self.tma,
            0xFF07 => self.tac,

            _ => 0xFF
        }
    }

    pub fn write_byte(&mut self, addr: u16, val: u8)
    {
        match addr
        {
            0xFF04 => self.div = 0,
            0xFF05 => self.tima = val,
            0xFF06 => self.tma = val,
            0xFF07 => { self.tac = val; self.update(); },
            _ => {}
        }
    }
}