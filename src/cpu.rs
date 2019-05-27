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

    /// Fetch a byte from the given address
    fn fetch_byte(&mut self, addr: u16) -> u8
    {
        let b = self.mem.read_byte(addr);
        b
    }

    /// Returns the next immediate byte
    fn next_byte(&mut self) -> u8
    {
        self.regs.pc += 1;
        let b = self.fetch_byte(self.regs.pc);
        b
    }

    /// Store a byte at the given address
    fn store_byte(&mut self, addr: u16, byte: u8)
    {
        self.mem.write_byte(addr, byte);
    }

    /// Push one byte onto the stack and decrement the stack pointer
    fn push_byte(&mut self, byte: u8)
    {
        self.regs.sp -= 1;
        self.store_byte(self.regs.sp, byte);
    }

    /// Pop a single byte from the stack and increment stack pointer
    fn pop_byte(&mut self) -> u8
    {
        let b = self.fetch_byte(self.regs.sp);
        self.regs.sp += 1;
        b
    }

    /// Fetch a word from the given address
    fn fetch_word(&mut self, addr: u16) -> u16
    {
        let w = self.mem.read_word(addr);
        w
    }

    /// Returns the next immediate word
    fn next_word(&mut self) -> u16
    {
        let b1 = self.next_byte() as u16;
        let b2 = self.next_byte() as u16;

        b1 | (b2 << 8)
    }

    /// Store a word at the given address
    fn store_word(&mut self, addr: u16, word: u16)
    {
        self.mem.write_word(addr, word);
    }

    /// Push two bytes onto the stack and decrement the stack pointer twice
    fn push_word(&mut self, word: u16)
    {
        self.push_byte((word >> 8) as u8);
        self.push_byte(word as u8);
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

// ---------------------------- BEGIN INSTRUCTIONS -----------------------------
//           http://marc.rawer.de/Gameboy/Docs/GBCPUman.pdf - pg. 65+

lazy_static!
{
    /// HashMap that indexes instruction function pointers and their names by opcode
    static ref OPCODES: HashMap< u8, (Instruction, &'static str) > =
    {
        use crate::cpu::*;
        let mut m = HashMap::new();
        
        m.insert(0x0, (nop as Instruction, "NOP"));

        // ---------------------------- 8-bit loads ----------------------------  

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
        m.insert(0x7E, (ld_a_hl as Instruction, "LD A, HL"));
        m.insert(0x40, (ld_b_b as Instruction, "LD B, B"));
        m.insert(0x41, (ld_b_c as Instruction, "LD B, C"));
        m.insert(0x42, (ld_b_d as Instruction, "LD B, D"));
        m.insert(0x43, (ld_b_e as Instruction, "LD B, E"));
        m.insert(0x44, (ld_b_h as Instruction, "LD B, H"));
        m.insert(0x45, (ld_b_l as Instruction, "LD B, L"));
        m.insert(0x46, (ld_b_hl as Instruction, "LD B, HL"));
        m.insert(0x48, (ld_c_b as Instruction, "LD C, B"));
        m.insert(0x49, (ld_c_c as Instruction, "LD C, C"));
        m.insert(0x4A, (ld_c_d as Instruction, "LD C, D"));
        m.insert(0x4B, (ld_c_e as Instruction, "LD C, E"));
        m.insert(0x4C, (ld_c_h as Instruction, "LD C, H"));
        m.insert(0x4D, (ld_c_l as Instruction, "LD C, L"));
        m.insert(0x4E, (ld_c_hl as Instruction, "LD C, HL"));
        m.insert(0x50, (ld_d_b as Instruction, "LD D, B"));
        m.insert(0x51, (ld_d_c as Instruction, "LD D, C"));
        m.insert(0x52, (ld_d_d as Instruction, "LD D, D"));
        m.insert(0x53, (ld_d_e as Instruction, "LD D, E"));
        m.insert(0x54, (ld_d_h as Instruction, "LD D, H"));
        m.insert(0x55, (ld_d_l as Instruction, "LD D, L"));
        m.insert(0x56, (ld_d_hl as Instruction, "LD D, HL"));
        m.insert(0x58, (ld_e_b as Instruction, "LD E, B"));
        m.insert(0x59, (ld_e_c as Instruction, "LD E, C"));
        m.insert(0x5A, (ld_e_d as Instruction, "LD E, D"));
        m.insert(0x5B, (ld_e_e as Instruction, "LD E, E"));
        m.insert(0x5C, (ld_e_h as Instruction, "LD E, H"));
        m.insert(0x5D, (ld_e_l as Instruction, "LD E, L"));
        m.insert(0x5E, (ld_e_hl as Instruction, "LD E, HL"));
        m.insert(0x60, (ld_h_b as Instruction, "LD H, B"));
        m.insert(0x61, (ld_h_c as Instruction, "LD H, C"));
        m.insert(0x62, (ld_h_d as Instruction, "LD H, D"));
        m.insert(0x63, (ld_h_e as Instruction, "LD H, E"));
        m.insert(0x64, (ld_h_h as Instruction, "LD H, H"));
        m.insert(0x65, (ld_h_l as Instruction, "LD H, L"));
        m.insert(0x66, (ld_h_hl as Instruction, "LD H, HL"));
        m.insert(0x68, (ld_l_b as Instruction, "LD L, B"));
        m.insert(0x69, (ld_l_c as Instruction, "LD L, C"));
        m.insert(0x6A, (ld_l_d as Instruction, "LD L, D"));
        m.insert(0x6B, (ld_l_e as Instruction, "LD L, E"));
        m.insert(0x6C, (ld_l_h as Instruction, "LD L, H"));
        m.insert(0x6D, (ld_l_l as Instruction, "LD L, L"));
        m.insert(0x6E, (ld_l_hl as Instruction, "LD L, HL"));
        m.insert(0x70, (ld_hl_b as Instruction, "LD HL, B"));
        m.insert(0x71, (ld_hl_c as Instruction, "LD HL, C"));
        m.insert(0x72, (ld_hl_d as Instruction, "LD HL, D"));
        m.insert(0x73, (ld_hl_e as Instruction, "LD HL, E"));
        m.insert(0x74, (ld_hl_h as Instruction, "LD HL, H"));
        m.insert(0x75, (ld_hl_l as Instruction, "LD HL, L"));
        m.insert(0x36, (ld_hl_n as Instruction, "LD HL, n"));
        m.insert(0x0A, (ld_a_bc as Instruction, "LD A, BC"));
        m.insert(0x1A, (ld_a_de as Instruction, "LD A, DE"));
        m.insert(0xFA, (ld_a_nn as Instruction, "LD A, nn"));
        m.insert(0x3E, (ld_a_n as Instruction, "LD A, #"));
        m.insert(0x47, (ld_b_a as Instruction, "LD B, A"));
        m.insert(0x4F, (ld_c_a as Instruction, "LD C, A"));
        m.insert(0x57, (ld_d_a as Instruction, "LD D, A"));
        m.insert(0x5F, (ld_e_a as Instruction, "LD E, A"));
        m.insert(0x67, (ld_h_a as Instruction, "LD H, A"));
        m.insert(0x6F, (ld_l_a as Instruction, "LD L, A"));
        m.insert(0x02, (ld_bc_a as Instruction, "LD BC, A"));
        m.insert(0x12, (ld_de_a as Instruction, "LD DE, A"));
        m.insert(0x77, (ld_hl_a as Instruction, "LD HL, A"));
        m.insert(0xEA, (ld_nn_a as Instruction, "LD nn, A"));
        m.insert(0xF2, (ld_a_c_ff00 as Instruction, "LD A, C"));
        m.insert(0xE2, (ld_c_ff00_a as Instruction, "LD C, A"));
        m.insert(0x3A, (ldd_a_hl as Instruction, "LDD A, HL"));
        m.insert(0x32, (ldd_hl_a as Instruction, "LDD HL, A"));
        m.insert(0x2A, (ldi_a_hl as Instruction, "LDI A, HL"));
        m.insert(0x22, (ldi_hl_a as Instruction, "LDI HL, A"));
        m.insert(0xE0, (ldh_n_a as Instruction, "LDH n, A"));
        m.insert(0xF0, (ldh_a_n as Instruction, "LDH A, n"));

        // ---------------------------- 16-bit loads ---------------------------

        m.insert(0x01, (ld_bc_nn as Instruction, "LD BC, nn"));
        m.insert(0x11, (ld_de_nn as Instruction, "LD DE), nn"));
        m.insert(0x21, (ld_hl_nn as Instruction, "LD HL, nn"));
        m.insert(0x31, (ld_sp_nn as Instruction, "LD SP, nn"));
        m.insert(0xF9, (ld_sp_hl as Instruction, "LD SP, HL"));
        m.insert(0xF8, (ldhl_sp_n as Instruction, "LDHL SP, n"));
        m.insert(0x08, (ld_nn_sp as Instruction, "LD nn, SP"));
        m.insert(0xF5, (push_af as Instruction, "PUSH AF"));
        m.insert(0xC5, (push_bc as Instruction, "PUSH BC"));
        m.insert(0xD5, (push_de as Instruction, "PUSH DE"));
        m.insert(0xE5, (push_hl as Instruction, "PUSH HL"));
        m.insert(0xF1, (pop_af as Instruction, "POP AF"));
        m.insert(0xC1, (pop_bc as Instruction, "POP BC"));
        m.insert(0xD1, (pop_de as Instruction, "POP DE"));
        m.insert(0xE1, (pop_hl as Instruction, "POP HL"));

        // ----------------------------- 8-bit ALU -----------------------------

        m.insert(0x87, (add_a_a as Instruction, "ADD A, A"));
        m.insert(0x80, (add_a_b as Instruction, "ADD A, B"));
        m.insert(0x81, (add_a_c as Instruction, "ADD A, C"));
        m.insert(0x82, (add_a_d as Instruction, "ADD A, D"));
        m.insert(0x83, (add_a_e as Instruction, "ADD A, E"));
        m.insert(0x84, (add_a_h as Instruction, "ADD A, H"));
        m.insert(0x85, (add_a_l as Instruction, "ADD A, L"));
        m.insert(0x86, (add_a_hl as Instruction, "ADD A, HL"));
        m.insert(0xC6, (add_a_n as Instruction, "ADD A, #"));

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
    let n = cpu.next_byte();
    cpu.regs.b = n;
    8
}

/// Load the immediate 8-bit value into register 'C'
fn ld_c_n(cpu: &mut CPU) -> u8
{
    let n = cpu.next_byte();
    cpu.regs.c = n;
    8
}

/// Load the immediate 8-bit value into register 'D'
fn ld_d_n(cpu: &mut CPU) -> u8
{
    let n = cpu.next_byte();
    cpu.regs.d = n;
    8
}

/// Load the immediate 8-bit value into register 'E'
fn ld_e_n(cpu: &mut CPU) -> u8
{
    let n = cpu.next_byte();
    cpu.regs.e = n;
    8
}

/// Load the immediate 8-bit value into register 'H'
fn ld_h_n(cpu: &mut CPU) -> u8
{
    let n = cpu.next_byte();
    cpu.regs.h = n;
    8
}

/// Load the immediate 8-bit value into register 'L'
fn ld_l_n(cpu: &mut CPU) -> u8
{
    let n = cpu.next_byte();
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

/// Load 'B' into 'C'
fn ld_c_b(cpu: &mut CPU) -> u8
{
    let b = cpu.regs.b;
    cpu.regs.c = b;
    4
}

/// Load 'C' into 'C' (no operation)
fn ld_c_c(_: &mut CPU) -> u8
{
    4
}

/// Load 'D' into 'C'
fn ld_c_d(cpu: &mut CPU) -> u8
{
    let d = cpu.regs.d;
    cpu.regs.c = d;
    4
}

/// Load 'E' into 'C'
fn ld_c_e(cpu: &mut CPU) -> u8
{
    let e = cpu.regs.e;
    cpu.regs.c = e;
    4
}

/// Load 'H' into 'C'
fn ld_c_h(cpu: &mut CPU) -> u8
{
    let h = cpu.regs.h;
    cpu.regs.c = h;
    4
}

/// Load 'L' into 'C'
fn ld_c_l(cpu: &mut CPU) -> u8
{
    let l = cpu.regs.l;
    cpu.regs.c = l;
    4
}

/// Load 'HL' into 'C'
fn ld_c_hl(cpu: &mut CPU) -> u8
{
    let hl = cpu.hl();
    let v = cpu.fetch_byte(hl);
    cpu.regs.c = v;
    8
}

/// Load 'B' into 'D'
fn ld_d_b(cpu: &mut CPU) -> u8
{
    let b = cpu.regs.b;
    cpu.regs.d = b;
    4
}

/// Load 'C' into 'D'
fn ld_d_c(cpu: &mut CPU) -> u8
{
    let c = cpu.regs.c;
    cpu.regs.d = c;
    4
}

/// Load 'D' into 'D' (no operation)
fn ld_d_d(_: &mut CPU) -> u8
{
    4
}

/// Load 'E' into 'D'
fn ld_d_e(cpu: &mut CPU) -> u8
{
    let e = cpu.regs.e;
    cpu.regs.d = e;
    4
}

/// Load 'H' into 'D'
fn ld_d_h(cpu: &mut CPU) -> u8
{
    let h = cpu.regs.h;
    cpu.regs.d = h;
    4
}

/// Load 'L' into 'D'
fn ld_d_l(cpu: &mut CPU) -> u8
{
    let l = cpu.regs.l;
    cpu.regs.d = l;
    4
}

/// Load 'HL' into 'D'
fn ld_d_hl(cpu: &mut CPU) -> u8
{
    let hl = cpu.hl();
    let v = cpu.fetch_byte(hl);
    cpu.regs.d = v;
    8
}

/// Load 'B' into 'E'
fn ld_e_b(cpu: &mut CPU) -> u8
{
    let b = cpu.regs.b;
    cpu.regs.e = b;
    4
}

/// Load 'C' into 'E'
fn ld_e_c(cpu: &mut CPU) -> u8
{
    let c = cpu.regs.c;
    cpu.regs.e = c;
    4
}

/// Load 'D' into 'E'
fn ld_e_d(cpu: &mut CPU) -> u8
{
    let d = cpu.regs.d;
    cpu.regs.e = d;
    4
}

/// Load 'E' into 'E' (no operation)
fn ld_e_e(_: &mut CPU) -> u8
{
    4
}

/// Load 'H' into 'E'
fn ld_e_h(cpu: &mut CPU) -> u8
{
    let h = cpu.regs.h;
    cpu.regs.e = h;
    4
}

/// Load 'L' into 'E'
fn ld_e_l(cpu: &mut CPU) -> u8
{
    let l = cpu.regs.l;
    cpu.regs.e = l;
    4
}

/// Load 'HL' into 'E'
fn ld_e_hl(cpu: &mut CPU) -> u8
{
    let hl = cpu.hl();
    let v = cpu.fetch_byte(hl);
    cpu.regs.e = v;
    8
}

/// Load 'B' into 'H'
fn ld_h_b(cpu: &mut CPU) -> u8
{
    let b = cpu.regs.b;
    cpu.regs.h = b;
    4
}

/// Load 'C' into 'H'
fn ld_h_c(cpu: &mut CPU) -> u8
{
    let c = cpu.regs.c;
    cpu.regs.h = c;
    4
}

/// Load 'D' into 'H'
fn ld_h_d(cpu: &mut CPU) -> u8
{
    let d = cpu.regs.d;
    cpu.regs.h = d;
    4
}

/// Load 'E' into 'H'
fn ld_h_e(cpu: &mut CPU) -> u8
{
    let e = cpu.regs.e;
    cpu.regs.h = e;
    4
}

/// Load 'H' into 'H' (no operation)
fn ld_h_h(_: &mut CPU) -> u8
{
    4
}

/// Load 'L' into 'H'
fn ld_h_l(cpu: &mut CPU) -> u8
{
    let l = cpu.regs.l;
    cpu.regs.h = l;
    4
}

/// Load 'HL' into 'H'
fn ld_h_hl(cpu: &mut CPU) -> u8
{
    let hl = cpu.hl();
    let v = cpu.fetch_byte(hl);
    cpu.regs.h = v;
    8
}

/// Load 'B' into 'L'
fn ld_l_b(cpu: &mut CPU) -> u8
{
    let b = cpu.regs.b;
    cpu.regs.l = b;
    4
}

/// Load 'C' into 'L'
fn ld_l_c(cpu: &mut CPU) -> u8
{
    let c = cpu.regs.c;
    cpu.regs.l = c;
    4
}

/// Load 'D' into 'L'
fn ld_l_d(cpu: &mut CPU) -> u8
{
    let d = cpu.regs.d;
    cpu.regs.l = d;
    4
}

/// Load 'E' into 'L'
fn ld_l_e(cpu: &mut CPU) -> u8
{
    let e = cpu.regs.e;
    cpu.regs.l = e;
    4
}

/// Load 'H' into 'L'
fn ld_l_h(cpu: &mut CPU) -> u8
{
    let h = cpu.regs.h;
    cpu.regs.l = h;
    4
}

/// Load 'L' into 'L' (no operation)
fn ld_l_l(_: &mut CPU) -> u8
{
    4
}

/// Load 'HL' into 'L'
fn ld_l_hl(cpu: &mut CPU) -> u8
{
    let hl = cpu.hl();
    let v = cpu.fetch_byte(hl);
    cpu.regs.l = v;
    8
}

/// Load 'B' into 'HL'
fn ld_hl_b(cpu: &mut CPU) -> u8
{
    let b = cpu.regs.b;
    let hl = cpu.hl();
    cpu.store_byte(hl, b);
    8
}

/// Load 'C' into 'HL'
fn ld_hl_c(cpu: &mut CPU) -> u8
{
    let c = cpu.regs.c;
    let hl = cpu.hl();
    cpu.store_byte(hl, c);
    8
}

/// Load 'D' into 'HL'
fn ld_hl_d(cpu: &mut CPU) -> u8
{
    let d = cpu.regs.d;
    let hl = cpu.hl();
    cpu.store_byte(hl, d);
    8
}

/// Load 'E' into 'HL'
fn ld_hl_e(cpu: &mut CPU) -> u8
{
    let e = cpu.regs.e;
    let hl = cpu.hl();
    cpu.store_byte(hl, e);
    8
}

/// Load 'H' into 'HL'
fn ld_hl_h(cpu: &mut CPU) -> u8
{
    let h = cpu.regs.h;
    let hl = cpu.hl();
    cpu.store_byte(hl, h);
    8
}

/// Load 'L' into 'HL'
fn ld_hl_l(cpu: &mut CPU) -> u8
{
    let l = cpu.regs.l;
    let hl = cpu.hl();
    cpu.store_byte(hl, l);
    8
}

/// Load the next immediate byte into 'HL'
fn ld_hl_n(cpu: &mut CPU) -> u8
{
    let n = cpu.next_byte();
    let hl = cpu.hl();
    cpu.store_byte(hl, n);
    12
}

/// Load 'BC' into 'A'
fn ld_a_bc(cpu: &mut CPU) -> u8
{
    let bc = cpu.bc();
    let v = cpu.fetch_byte(bc);
    cpu.regs.a = v;
    8
}

/// Load 'DE' into 'A'
fn ld_a_de(cpu: &mut CPU) -> u8
{
    let de = cpu.de();
    let v = cpu.fetch_byte(de);
    cpu.regs.a = v;
    8
}

/// Load the next immediate word into 'A'
fn ld_a_nn(cpu: &mut CPU) -> u8
{
    let nn = cpu.next_word();
    let v = cpu.fetch_byte(nn);
    cpu.regs.a = v;
    16
}

/// Load the next immediate byte into 'A'
fn ld_a_n(cpu: &mut CPU) -> u8
{
    let n = cpu.next_byte();
    cpu.regs.a = n;
    8
}

/// Load 'A' into 'B'
fn ld_b_a(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    cpu.regs.b = a;
    4
}

/// Load 'A' into 'C'
fn ld_c_a(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    cpu.regs.c = a;
    4
}

/// Load 'A' into 'D'
fn ld_d_a(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    cpu.regs.d = a;
    4
}

/// Load 'A' into 'E'
fn ld_e_a(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    cpu.regs.e = a;
    4
}

/// Load 'A' into 'H'
fn ld_h_a(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    cpu.regs.h = a;
    4
}

/// Load 'A' into 'L'
fn ld_l_a(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    cpu.regs.l = a;
    4
}

/// Load 'A' into 'BC'
fn ld_bc_a(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let bc = cpu.bc();
    cpu.store_byte(bc, a);
    8
}

/// Load 'A' into 'DE'
fn ld_de_a(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let de = cpu.de();
    cpu.store_byte(de, a);
    8
}

/// Load 'A' into 'HL'
fn ld_hl_a(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let hl = cpu.hl();
    cpu.store_byte(hl, a);
    8
}

/// Load 'A' into the next immediate word
fn ld_nn_a(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let nn = cpu.next_word();
    cpu.store_byte(nn, a);
    16
}

/// Load '[0xFF00 + C]' into 'A'
fn ld_a_c_ff00(cpu: &mut CPU) -> u8
{
    let c = cpu.regs.c as u16;
    let v = cpu.fetch_byte(0xFF00 | c);
    cpu.regs.a = v;
    8
}

/// Load 'A' into '[0xFF00 + C]'
fn ld_c_ff00_a(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let c = cpu.regs.c as u16;
    cpu.store_byte(0xFF00 | c, a);
    8
}

/// Load 'HL' into 'A' and then decrement HL
fn ldd_a_hl(cpu: &mut CPU) -> u8
{
    let hl = cpu.hl();
    let a = cpu.fetch_byte(hl);
    cpu.regs.a = a;
    cpu.set_hl(hl - 1);
    8
}

/// Load 'A' into 'HL' and then decrement HL
fn ldd_hl_a(cpu: &mut CPU) -> u8
{
    let hl = cpu.hl();
    let a = cpu.regs.a;
    cpu.store_byte(hl, a);
    cpu.set_hl(hl - 1);
    8
}

/// Load 'HL' into 'A' and then increment HL
fn ldi_a_hl(cpu: &mut CPU) -> u8
{
    let hl = cpu.hl();
    let a = cpu.fetch_byte(hl);
    cpu.regs.a = a;
    cpu.set_hl(hl + 1);
    8
}

/// Load 'A' into 'HL' and then increment HL
fn ldi_hl_a(cpu: &mut CPU) -> u8
{
    let hl = cpu.hl();
    let a = cpu.regs.a;
    cpu.store_byte(hl, a);
    cpu.set_hl(hl + 1);
    8
}

/// Load 'A' into memory address [0xFF00 + n], where n is the next immediate byte
fn ldh_n_a(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let n = cpu.next_byte() as u16;
    cpu.store_byte(0xFF00 | n, a);
    12
}

/// Load [0xFF00 + n] into 'A', where n is the next immediate byte
fn ldh_a_n(cpu: &mut CPU) -> u8
{
    let n = cpu.next_byte() as u16;
    let v = cpu.fetch_byte(0xFF00 | n);
    cpu.regs.a = v;
    12
}

/// Load the next immediate word into 'BC'
fn ld_bc_nn(cpu: &mut CPU) -> u8
{
    let n = cpu.next_word();
    cpu.set_bc(n);
    12
}

/// Load the next immediate word into 'DE'
fn ld_de_nn(cpu: &mut CPU) -> u8
{
    let n = cpu.next_word();
    cpu.set_de(n);
    12
}

/// Load the next immediate word into 'HL'
fn ld_hl_nn(cpu: &mut CPU) -> u8
{
    let n = cpu.next_word();
    cpu.set_hl(n);
    12
}

/// Load the next immediate word into the Stack Pointer
fn ld_sp_nn(cpu: &mut CPU) -> u8
{
    let n = cpu.next_word();
    cpu.regs.sp = n;
    12
}

/// Load HL into the Stack Pointer
fn ld_sp_hl(cpu: &mut CPU) -> u8
{
    let hl = cpu.hl();
    cpu.regs.sp = hl;
    8
}

/// Load the memory address at '[SP + n]' into 'HL', where n is the next immediate byte
fn ldhl_sp_n(cpu: &mut CPU) -> u8
{
    let sp = cpu.regs.sp as i32;
    let n = cpu.next_byte() as i8;
    let nn = n as i32;
    let r = sp + nn;

    cpu.flags.z = false;
    cpu.flags.n = false;
    cpu.flags.h = (sp ^ nn ^ r) & 0x10 != 0;
    cpu.flags.c = (sp ^ nn ^ r) & 0x100 != 0;

    cpu.set_hl(r as u16);

    12
}

/// Store the Stack Pointer into the next immediate word
fn ld_nn_sp(cpu: &mut CPU) -> u8
{
    let sp = cpu.regs.sp;
    let nn = cpu.next_word();
    cpu.store_word(nn, sp);
    20
}

/// Push register pair AF onto the stack and decrement stack pointer twice
fn push_af(cpu: &mut CPU) -> u8
{
    let af = cpu.af();
    cpu.push_word(af);
    16
}

/// Push register pair BC onto the stack and decrement stack pointer twice
fn push_bc(cpu: &mut CPU) -> u8
{
    let bc = cpu.bc();
    cpu.push_word(bc);
    16
}

/// Push register pair DE onto the stack and decrement stack pointer twice
fn push_de(cpu: &mut CPU) -> u8
{
    let de = cpu.de();
    cpu.push_word(de);
    16
}

/// Push register pair HL onto the stack and decrement stack pointer twice
fn push_hl(cpu: &mut CPU) -> u8
{
    let hl = cpu.hl();
    cpu.push_word(hl);
    16
}

/// Pop two bytes off the stack into register pair 'AF' and increment SP twice
fn pop_af(cpu: &mut CPU) -> u8
{
    let n = cpu.pop_word();
    cpu.set_af(n);
    12
}

/// Pop two bytes off the stack into register pair 'BC' and increment SP twice
fn pop_bc(cpu: &mut CPU) -> u8
{
    let n = cpu.pop_word();
    cpu.set_bc(n);
    12
}

/// Pop two bytes off the stack into register pair 'DE' and increment SP twice
fn pop_de(cpu: &mut CPU) -> u8
{
    let n = cpu.pop_word();
    cpu.set_de(n);
    12
}

/// Pop two bytes off the stack into register pair 'HL' and increment SP twice
fn pop_hl(cpu: &mut CPU) -> u8
{
    let n = cpu.pop_word();
    cpu.set_hl(n);
    12
}

/// Helper function to add two bytes and update the CPU flags register and
/// return the result of the addition as a single byte
fn add_and_update_flags(cpu: &mut CPU, b1: u8, b2: u8) -> u8
{
    let b1 = b1 as u32;
    let b2 = b2 as u32;
    let result = b1 + b2;
    let result_u8 = result as u8;

    cpu.flags.z = result_u8 == 0;
    cpu.flags.n = false;
    cpu.flags.h = (b1 ^ b2 ^ result) & 0x10 != 0;
    cpu.flags.c = result & 0x100 != 0;

    result_u8
}

/// Add 'A' to 'A'
fn add_a_a(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let v = add_and_update_flags(cpu, a, a);
    cpu.regs.a = v;
    4
}

/// Add 'B' to 'A'
fn add_a_b(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let b = cpu.regs.b;
    let v = add_and_update_flags(cpu, a, b);
    cpu.regs.a = v;
    4
}

/// Add 'C' to 'A'
fn add_a_c(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let c = cpu.regs.c;
    let v = add_and_update_flags(cpu, a, c);
    cpu.regs.a = v;
    4
}

/// Add 'D' to 'A'
fn add_a_d(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let d = cpu.regs.d;
    let v = add_and_update_flags(cpu, a, d);
    cpu.regs.a = v;
    4
}

/// Add 'E' to 'A'
fn add_a_e(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let e = cpu.regs.e;
    let v = add_and_update_flags(cpu, a, e);
    cpu.regs.a = v;
    4
}

/// Add 'H' to 'A'
fn add_a_h(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let h = cpu.regs.h;
    let v = add_and_update_flags(cpu, a, h);
    cpu.regs.a = v;
    4
}

/// Add 'L' to 'A'
fn add_a_l(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let l = cpu.regs.l;
    let v = add_and_update_flags(cpu, a, l);
    4
}

/// Add 'HL' to 'A'
fn add_a_hl(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let hl = cpu.hl();
    let n = cpu.fetch_byte(hl);
    let v = add_and_update_flags(cpu, a, n);
    cpu.regs.a = v;
    8
}

/// Add the next immediate byte to 'A'
fn add_a_n(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let n = cpu.next_byte();
    let v = add_and_update_flags(cpu, a, n);
    cpu.regs.a = v;
    8
}