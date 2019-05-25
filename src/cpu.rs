use crate::mem::MMU;
use std::collections::HashMap;

/// Represents all of the Gameboy registers
#[derive(Debug, Clone, Copy)]
struct Registers
{
    /// 8-bit 'A' register
    a: u8,

    /// 8-bit 'B' register
    b: u8,

    /// 8-bit 'C' register
    c: u8,

    /// 8-bit 'D' register
    d: u8,

    /// 8-bit 'E' register
    e: u8,

    /// 8-bit 'H' register
    h: u8,

    /// 8-bit 'L' register
    l: u8,

    /// Stack Pointer: points to the current stack position
    sp: u16,

    /// Program Counter: points to next instruction to be executed
    pc: u16
}

/// Represents the CPU Flags register (register 'F')
struct Flags
{
    /// Zero flag: set when the result of a math operation is zero or two
    /// values match when using the CP instruction
    z: bool,

    /// Subtract flag: set if a subtraction was performed in the last
    /// math operation
    n: bool,

    /// Half-Carry flag: set if a carry occurred from the lower nibble
    /// in the last math operation
    h: bool,

    /// Carry flag: set if a carry occurred from the last math operation
    /// or if register A is the smaller value when executing the CP instruction
    c: bool
}

impl Flags
{
    /// Retrieve the value of the 'F' register as a single byte
    fn f(&self) -> u8
    {
        let z_u8 = self.z as u8;
        let n_u8 = self.n as u8;
        let h_u8 = self.h as u8;
        let c_u8 = self.c as u8;

        (z_u8 << 7) | (n_u8 << 6) | (h_u8 << 5) | (c_u8 << 4)
    }

    /// Set the value of the 'F' register using a single byte
    fn set_f(&mut self, v: u8)
    {
        self.z = (v & (1 << 7)) != 0;
        self.n = (v & (1 << 6)) != 0;
        self.h = (v & (1 << 5)) != 0;
        self.c = (v & (1 << 4)) != 0;
    }
}

/// An Instruction is a function that returns the number of cycles it took to execute
type Instruction = (fn (&mut CPU) -> u8);

/// Represents the GameBoy CPU
pub struct CPU
{
    /// CPU Registers
    regs: Registers,

    /// CPU Flags ('F' Register)
    flags: Flags,
    
    /// CPU Memory Management Unit
    mem: MMU,

    /// Interrupt Master Enable flag: Reset by the DI and prohibits all interrupts
    ime: bool,

    /// CPU halted flag
    halted: bool,

    /// Number of cycles elapsed running the current instruction
    cycles: u8
}

impl CPU
{
    /// Creates and returns a new instance of the GameBoy CPU
    pub fn new() -> Self
    {
        let regs = Registers {
            a: 0x0,
            b: 0x0,
            c: 0x0,
            d: 0x0,
            e: 0x0,
            h: 0x0,
            l: 0x0,
            sp: 0x0,
            pc: 0x0
        };

        let flags = Flags {
            z: false,
            n: false,
            h: false,
            c: false
        };

        CPU {
            regs: regs,
            flags: flags,
            mem: MMU::new(),
            ime: true,
            halted: false,
            cycles: 0u8
        }
    }

    ///  Execute the next CPU cycle. Returns the number of cycles elapsed.
    pub fn run_cycle(&mut self) -> u8
    {
        // Reset the cycles counter
        self.cycles = 0;

        // Interrupts
        // TODO

        // Halted
        // TODO

        // Fetch and run the next instruction and return number of cycles 
        // it took to execute
        let instruction = self.get_next_instruction();
        self.cycles = (instruction)(self);
        self.cycles
    }

    /// Retrieve the next instruction to be executed
    fn get_next_instruction(&mut self) -> Instruction
    {
        // Get the next opcode from memory
        let op = self.fetch_byte(self.regs.pc);

        // Increment the program counter
        self.regs.pc += 1;
        
        // Return function pointer to instruction operation, or undefined if
        // opcode doesn't index any existing instructions
        if let Some((instruction, _)) = OPCODES.get(&op)
        {
            *instruction
        }
        else
        {
            crate::cpu::undefined
        }
    }

    /// Fetch a byte from the given address. Takes one cycle.
    fn fetch_byte(&mut self, addr: u16) -> u8
    {
        let b = self.mem.read_byte(addr);
        self.delay(1);
        b
    }

    /// Store a byte at the given address. Takes one cycle.
    fn store_byte(&mut self, addr: u16, byte: u8)
    {
        self.mem.write_byte(addr, byte);
        self.delay(1);
    }

    /// Push one byte onto the stack and decrement the stack pointer
    fn push_byte(&mut self, byte: u8)
    {
        self.regs.sp -= 1;
        self.store_byte(self.regs.sp, byte);
    }

    /// Push two bytes onto the stack and decrement the stack pointer twice
    fn push_word(&mut self, word: u16)
    {
        self.push_byte((word >> 8) as u8);
        self.push_byte(word as u8);
    }

    /// Pop a single byte from the stack and increment stack pointer
    fn pop_byte(&mut self) -> u8
    {
        let b = self.fetch_byte(self.regs.sp);
        self.regs.sp += 1;
        b
    }

    /// Pops two bytes from the stack and increments stack pointer twice
    fn pop_word(&mut self) -> u16
    {
        let lo = self.pop_byte() as u16;
        let hi = self.pop_byte() as u16;

        (hi << 8) | lo
    }

    /// Retrieve the value stored in the 'AF' register
    fn af(&self) -> u16
    {
        let mut v = self.flags.f() as u16;
        v |= (self.regs.a as u16) << 8;
        v
    }

    /// Set the value stored in the 'AF' register
    fn set_af(&mut self, af: u16)
    {
        self.regs.a = (af >> 8) as u8;
        self.flags.set_f(af as u8);
    }

    /// Retrieve the value stored in the 'BC' register
    fn bc(&self) -> u16
    {
        let mut v = self.regs.c as u16;
        v |= (self.regs.b as u16) << 8;
        v
    }

    /// Set the value stored in the 'BC' register
    fn set_bc(&mut self, bc: u16)
    {
        self.regs.b = (bc >> 8) as u8;
        self.regs.c = bc as u8;
    }

    /// Retrieve the value stored in the 'DE' register
    fn de(&self) -> u16
    {
        let mut v = self.regs.e as u16;
        v |= (self.regs.d as u16) << 8;
        v
    }

    /// Set the value stored in the 'DE' register
    fn set_de(&mut self, de: u16)
    {
        self.regs.d = (de >> 8) as u8;
        self.regs.e = de as u8;
    }

    /// Retrieve the value stored in the 'HL' register
    fn hl(&self) -> u16
    {
        let mut v = self.regs.l as u16;
        v |= (self.regs.h as u16) << 8;
        v
    }

    /// Set the value stored in the 'HL' register
    fn set_hl(&mut self, hl: u16)
    {
        self.regs.h = (hl >> 8) as u8;
        self.regs.l = hl as u8;
    }

    /// Disable interrupts
    fn disable_interrupts(&mut self)
    {
        self.ime = false;
    }

    /// Enable interrupts
    fn enable_interrupts(&mut self)
    {
        self.ime = true;
    }

    /// Halt and wait for interrupts
    fn halt(&mut self)
    {
        self.halted = true;
    }

    fn interrupt(&mut self)
    {
    }

    /// Delay the CPU by a given number of cycles
    fn delay(&mut self, cycles: u8)
    {
    }

    pub fn reset(&mut self)
    {
        self.regs.pc = 0x0;
    }
}

lazy_static!
{
    /// HashMap that indexes instruction function pointers and their names by opcode
    static ref OPCODES: HashMap< u8, (Instruction, &'static str) > =
    {
        use crate::cpu::*;
        let mut m = HashMap::new();
        
        m.insert(0x0, (nop as Instruction, "NOP"));

        m.insert(0x06, (ld_b_n as Instruction, "LD B, n"));
        m.insert(0x0E, (ld_c_n as Instruction, "LD C, n"));
        m.insert(0x16, (ld_d_n as Instruction, "LD D, n"));
        m.insert(0x1E, (ld_e_n as Instruction, "LD E, n"));
        m.insert(0x26, (ld_h_n as Instruction, "LD H, n"));
        m.insert(0x2E, (ld_l_n as Instruction, "LD L, n"));

        m.insert(0x7F, (ld_a_a as Instruction, "LD A, A"));
        m.insert(0x78, (ld_a_b as Instruction, "LD A, B"));
        m.insert(0x79, (ld_a_c as Instruction, "LD A, B"));
        m.insert(0x7A, (ld_a_d as Instruction, "LD A, B"));
        m.insert(0x7B, (ld_a_e as Instruction, "LD A, B"));
        m.insert(0x7C, (ld_a_h as Instruction, "LD A, B"));
        m.insert(0x7D, (ld_a_l as Instruction, "LD A, B"));
        m.insert(0x7E, (ld_a_hl as Instruction, "LD A, (HL)"));

        m.insert(0x40, (ld_b_b as Instruction, "LD B, B"));
        m.insert(0x41, (ld_b_c as Instruction, "LD B, C"));
        m.insert(0x42, (ld_b_d as Instruction, "LD B, D"));
        m.insert(0x43, (ld_b_e as Instruction, "LD B, E"));
        m.insert(0x44, (ld_b_h as Instruction, "LD B, H"));
        m.insert(0x45, (ld_b_l as Instruction, "LD B, L"));
        m.insert(0x46, (ld_b_hl as Instruction, "LD B, (HL)"));

        m
    };
}

/// No operation
pub fn nop(_: &mut CPU) -> u8
{
    4
}

/// Undefined opcode
fn undefined(cpu: &mut CPU) -> u8
{
    let pc = cpu.regs.pc.wrapping_sub(1);
    println!("Undefined instruction called at {:#X}. CPU stalled.", pc);
    cpu.regs.pc = pc;
    0
}

/// Load the immediate 8-bit value into register 'B'
fn ld_b_n(cpu: &mut CPU) -> u8
{
    let pc = cpu.regs.pc + 1;
    let n = cpu.fetch_byte(pc);
    cpu.regs.b = n;
    8
}

/// Load the immediate 8-bit value into register 'C'
fn ld_c_n(cpu: &mut CPU) -> u8
{
    let pc = cpu.regs.pc + 1;
    let n = cpu.fetch_byte(pc);
    cpu.regs.c = n;
    8
}

/// Load the immediate 8-bit value into register 'D'
fn ld_d_n(cpu: &mut CPU) -> u8
{
    let pc = cpu.regs.pc + 1;
    let n = cpu.fetch_byte(pc);
    cpu.regs.d = n;
    8
}

/// Load the immediate 8-bit value into register 'E'
fn ld_e_n(cpu: &mut CPU) -> u8
{
    let pc = cpu.regs.pc + 1;
    let n = cpu.fetch_byte(pc);
    cpu.regs.e = n;
    8
}

/// Load the immediate 8-bit value into register 'H'
fn ld_h_n(cpu: &mut CPU) -> u8
{
    let pc = cpu.regs.pc + 1;
    let n = cpu.fetch_byte(pc);
    cpu.regs.h = n;
    8
}

/// Load the immediate 8-bit value into register 'L'
fn ld_l_n(cpu: &mut CPU) -> u8
{
    let pc = cpu.regs.pc + 1;
    let n = cpu.fetch_byte(pc);
    cpu.regs.l = n;
    8
}

/// Load 'A' into 'A' (no operation)
fn ld_a_a(_: &mut CPU) -> u8
{
    4
}

/// Load 'B' into 'A'
fn ld_a_b(cpu: &mut CPU) -> u8
{
    let b = cpu.regs.b;
    cpu.regs.a = b;
    4
}

/// Load 'C' into 'A'
fn ld_a_c(cpu: &mut CPU) -> u8
{
    let c = cpu.regs.c;
    cpu.regs.a = c;
    4
}

/// Load 'D' into 'A'
fn ld_a_d(cpu: &mut CPU) -> u8
{
    let d = cpu.regs.d;
    cpu.regs.a = d;
    4
}

/// Load 'E' into 'A'
fn ld_a_e(cpu: &mut CPU) -> u8
{
    let e = cpu.regs.e;
    cpu.regs.a = e;
    4
}

/// Load 'H' into 'A'
fn ld_a_h(cpu: &mut CPU) -> u8
{
    let h = cpu.regs.h;
    cpu.regs.a = h;
    4
}

/// Load 'L' into 'A'
fn ld_a_l(cpu: &mut CPU) -> u8
{
    let l = cpu.regs.l;
    cpu.regs.a = l;
    4
}

/// Load 'HL' into 'A'
fn ld_a_hl(cpu: &mut CPU) -> u8
{
    let hl = cpu.hl();
    let v = cpu.fetch_byte(hl);
    cpu.regs.a = v;
    8
}

/// Load 'B' into 'B' (no operation)
fn ld_b_b(_: &mut CPU) -> u8
{
    4
}

/// Load 'C' into 'B'
fn ld_b_c(cpu: &mut CPU) -> u8
{
    let c = cpu.regs.c;
    cpu.regs.b = c;
    4
}

/// Load 'D' into 'B'
fn ld_b_d(cpu: &mut CPU) -> u8
{
    let d = cpu.regs.d;
    cpu.regs.b = d;
    4
}

/// Load 'E' into 'B'
fn ld_b_e(cpu: &mut CPU) -> u8
{
    let e = cpu.regs.e;
    cpu.regs.b = e;
    4
}

/// Load 'H' into 'B'
fn ld_b_h(cpu: &mut CPU) -> u8
{
    let h = cpu.regs.h;
    cpu.regs.b = h;
    4
}

/// Load 'L' into 'B'
fn ld_b_l(cpu: &mut CPU) -> u8
{
    let l = cpu.regs.l;
    cpu.regs.b = l;
    4
}

/// Load 'HL' into 'B'
fn ld_b_hl(cpu: &mut CPU) -> u8
{
    let hl = cpu.hl();
    let v = cpu.fetch_byte(hl);
    cpu.regs.b = v;
    8
}