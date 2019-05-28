
pub struct Timer
{
    /// Timer divider. Counts up at a fixed 16kHz and resets to 0 whenever
    /// written to. Stored at address 0xFF04.
    divider: u8,

    /// Timer counter. Counts up at the specified rate. Triggers interrupt 
    /// INT 0x50 on overflow. Stored at address 0xFF05.
    counter: u8,

    /// Timer modulo. When Counter overflows to 0 it is reset to start at 
    /// this value. Stored at address 0xFF06.
    modulo: u8,

    /// True if the timer is enabled and is counting/generating interrupts
    enabled: bool,

    /// True if the timer has an interrupt pending
    interrupt: bool,

    /// Timer speed (TODO: better description)
    step: u32
}

impl Timer
{
    /// Create and return a new Timer instance
    pub fn new() -> Self
    {
        Timer {
            divider: 0u8,
            counter: 0u8,
            modulo: 0u8,
            enabled: false,
            interrupt: false,
            step: 0u32,
        }
    }

    pub fn run_cycle(&mut self)
    {
        // If the timer isn't enabled we don't wanna do anything
        if self.enabled
        {
        }
    }

    /// Read the value of a timer register at given address
    pub fn read_byte(&mut self, addr: u16) -> u8
    {
        match addr
        {
            // Reads the divider register
            0xFF04 => self.divider,

            // Reads the counter register
            0xFF05 => self.counter,

            // Reads the modulo register
            0xFF06 => self.modulo,

            // Reads the control register
            0xFF07 => {
                // Value to return for the control register
                let mut control = 0x0;

                // Turn on bits 0/1 depending on timer speed
                control |= match self.step 
                {
                    16 => 0x1,
                    64 => 0x2,
                    256 => 0x3,
                    1024 => 0x0,
                    _ => 0x0
                };

                // If the timer is enabled turn on bit 2
                control |= if self.enabled { 0x4 } else { 0x0 };

                control
            },

            // Timer cannot read any other addresses
            _ => panic!("Timer cannot read address {:#X}!", addr)
        }
    }

    /// Set the value of a timer register at given address
    pub fn write_byte(&mut self, addr: u16, b: u8)
    {
        match addr
        {
            // Write the divider register. Resets to 0 whenever written to.
            0xFF04 => { self.divider = 0; },

            // Write the counter register
            0xFF05 => { self.counter = b; },

            // Write the modulo register
            0xFF06 => { self.modulo = b; },

            // Timer control register
            0xFF07 => {
                // Bits 0-1: determines the speed of the timer
                match b & 0x3
                {
                    // 00: 4096Hz
                    0x0 => { self.step = 1024; },

                    // 01: 262.144 kHz
                    0x1 => { self.step = 16; },

                    // 10: 65.536 kHz
                    0x2 => { self.step = 64; },

                    // 11: 16.384 kHz
                    0x3 => { self.step = 256; },

                    // Anything else just set to default step of 1024
                    _ => { self.step = 1024; }
                }

                // Bit 2: Set to 1 to run timer, set to 0 to stop timer
                self.enabled = b & 0x4 != 0;

                // Bits 3-7: unused
            },

            // Timer cannot write to any other addresses
            _ => panic!("Timer cannot write to address {:#X}!", addr)
        }
    }

    /// Returns the timer's interrupt status
    pub fn interrupt_status(&self) -> bool
    {
        self.interrupt
    }

    /// Acknowledge an interrupt
    pub fn ok_interrupt(&mut self)
    {
        self.interrupt = false;
    }

    /// Force the interrupt state
    pub fn force_interrupt(&mut self, state: bool)
    {
        self.interrupt = state;
    }
}