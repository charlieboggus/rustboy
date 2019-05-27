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
        let mut op = self.fetch_byte(self.regs.pc);
        self.regs.pc += 1;

        // Check if the opcode is a prefixed opcode
        if op != 0xCB
        {
            // If it's not we can just get the instruction from the regular
            // opcode hashmap
            if let Some((instruction, _)) = OPCODES.get(&op)
            {
                *instruction
            }
            else
            {
                crate::cpu::undefined
            }
        }
        else
        {
            // If it is we have to fetch the next byte and use it to search for
            // an instruction in the prefixed opcode hashmap
            op = self.fetch_byte(self.regs.pc);
            self.regs.pc += 1;
            
            if let Some((instruction, _)) = OPCODES_CB.get(&op)
            {
                *instruction
            }
            else
            {
                crate::cpu::undefined
            }
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

    /// Enable interrupts immediately
    fn enable_interrupts(&mut self)
    {
        self.ime = true;
    }

    /// Enable interrupts after next instruction
    fn enable_interrupts_after_next(&mut self)
    {
    }

    /// Halt and wait for interrupts
    fn halt(&mut self)
    {
        self.halted = true;
    }

    /// Stop CPU and LCD display until button press
    fn stop(&mut self)
    {
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
    /// HashMap that indexes instruction function pointers and their names by 
    /// their opcode
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
        m.insert(0x8F, (adc_a_a as Instruction, "ADC A, A"));
        m.insert(0x88, (adc_a_b as Instruction, "ADC A, B"));
        m.insert(0x89, (adc_a_c as Instruction, "ADC A, C"));
        m.insert(0x8A, (adc_a_d as Instruction, "ADC A, D"));
        m.insert(0x8B, (adc_a_e as Instruction, "ADC A, E"));
        m.insert(0x8C, (adc_a_h as Instruction, "ADC A, H"));
        m.insert(0x8D, (adc_a_l as Instruction, "ADC A, L"));
        m.insert(0x8E, (adc_a_hl as Instruction, "ADC A, HL"));
        m.insert(0xCE, (adc_a_n as Instruction, "ADC A, #"));
        m.insert(0x97, (sub_a_a as Instruction, "SUB A, A"));
        m.insert(0x90, (sub_a_b as Instruction, "SUB A, B"));
        m.insert(0x91, (sub_a_c as Instruction, "SUB A, C"));
        m.insert(0x92, (sub_a_d as Instruction, "SUB A, D"));
        m.insert(0x93, (sub_a_e as Instruction, "SUB A, E"));
        m.insert(0x94, (sub_a_h as Instruction, "SUB A, H"));
        m.insert(0x95, (sub_a_l as Instruction, "SUB A, L"));
        m.insert(0x96, (sub_a_hl as Instruction, "SUB A, HL"));
        m.insert(0xD6, (sub_a_n as Instruction, "SUB A, #"));
        m.insert(0x9F, (sbc_a_a as Instruction, "SBC A, A"));
        m.insert(0x98, (sbc_a_b as Instruction, "SBC A, B"));
        m.insert(0x99, (sbc_a_c as Instruction, "SBC A, C"));
        m.insert(0x9A, (sbc_a_d as Instruction, "SBC A, D"));
        m.insert(0x9B, (sbc_a_e as Instruction, "SBC A, E"));
        m.insert(0x9C, (sbc_a_h as Instruction, "SBC A, H"));
        m.insert(0x9D, (sbc_a_l as Instruction, "SBC A, L"));
        m.insert(0x9E, (sbc_a_hl as Instruction, "SBC A, HL"));
        m.insert(0xDE, (sbc_a_n as Instruction, "SBC A, #"));
        m.insert(0xA7, (and_a_a as Instruction, "AND A, A"));
        m.insert(0xA0, (and_a_b as Instruction, "AND A, B"));
        m.insert(0xA1, (and_a_c as Instruction, "AND A, C"));
        m.insert(0xA2, (and_a_d as Instruction, "AND A, D"));
        m.insert(0xA3, (and_a_e as Instruction, "AND A, E"));
        m.insert(0xA4, (and_a_h as Instruction, "AND A, H"));
        m.insert(0xA5, (and_a_l as Instruction, "AND A, L"));
        m.insert(0xA6, (and_a_hl as Instruction, "AND A, HL"));
        m.insert(0xE6, (and_a_n as Instruction, "AND A, #"));
        m.insert(0xB7, (or_a_a as Instruction, "OR A, A"));
        m.insert(0xB0, (or_a_b as Instruction, "OR A, B"));
        m.insert(0xB1, (or_a_c as Instruction, "OR A, C"));
        m.insert(0xB2, (or_a_d as Instruction, "OR A, D"));
        m.insert(0xB3, (or_a_e as Instruction, "OR A, E"));
        m.insert(0xB4, (or_a_h as Instruction, "OR A, H"));
        m.insert(0xB5, (or_a_l as Instruction, "OR A, L"));
        m.insert(0xB6, (or_a_hl as Instruction, "OR A, HL"));
        m.insert(0xF6, (or_a_n as Instruction, "OR A, #"));
        m.insert(0xAF, (xor_a_a as Instruction, "XOR A, A"));
        m.insert(0xA8, (xor_a_b as Instruction, "XOR A, B"));
        m.insert(0xA9, (xor_a_c as Instruction, "XOR A, C"));
        m.insert(0xAA, (xor_a_d as Instruction, "XOR A, D"));
        m.insert(0xAB, (xor_a_e as Instruction, "XOR A, E"));
        m.insert(0xAC, (xor_a_h as Instruction, "XOR A, H"));
        m.insert(0xAD, (xor_a_l as Instruction, "XOR A, L"));
        m.insert(0xAE, (xor_a_hl as Instruction, "XOR A, HL"));
        m.insert(0xEE, (xor_a_n as Instruction, "XOR A, #"));
        m.insert(0xBF, (cp_a_a as Instruction, "CP A, A"));
        m.insert(0xB8, (cp_a_b as Instruction, "CP A, B"));
        m.insert(0xB9, (cp_a_c as Instruction, "CP A, C"));
        m.insert(0xBA, (cp_a_d as Instruction, "CP A, D"));
        m.insert(0xBB, (cp_a_e as Instruction, "CP A, E"));
        m.insert(0xBC, (cp_a_h as Instruction, "CP A, H"));
        m.insert(0xBD, (cp_a_l as Instruction, "CP A, L"));
        m.insert(0xBE, (cp_a_hl as Instruction, "CP A, HL"));
        m.insert(0xFE, (cp_a_n as Instruction, "CP A, #"));
        m.insert(0x3C, (inc_a as Instruction, "INC A"));
        m.insert(0x04, (inc_b as Instruction, "INC B"));
        m.insert(0x0C, (inc_c as Instruction, "INC C"));
        m.insert(0x14, (inc_d as Instruction, "INC D"));
        m.insert(0x1C, (inc_e as Instruction, "INC E"));
        m.insert(0x24, (inc_h as Instruction, "INC H"));
        m.insert(0x2C, (inc_l as Instruction, "INC L"));
        m.insert(0x34, (inc_hl as Instruction, "INC HL"));
        m.insert(0x3D, (dec_a as Instruction, "DEC A"));
        m.insert(0x05, (dec_b as Instruction, "DEC B"));
        m.insert(0x0D, (dec_c as Instruction, "DEC C"));
        m.insert(0x15, (dec_d as Instruction, "DEC D"));
        m.insert(0x1D, (dec_e as Instruction, "DEC E"));
        m.insert(0x25, (dec_h as Instruction, "DEC H"));
        m.insert(0x2D, (dec_l as Instruction, "DEC L"));
        m.insert(0x35, (dec_hl as Instruction, "DEC HL"));

        // ------------------------- 16-bit Arithmetic -------------------------

        m.insert(0x09, (add_hl_bc as Instruction, "ADD HL, BC"));
        m.insert(0x19, (add_hl_de as Instruction, "ADD HL, DE"));
        m.insert(0x29, (add_hl_hl as Instruction, "ADD HL, HL"));
        m.insert(0x39, (add_hl_sp as Instruction, "ADD HL, SP"));
        m.insert(0xE8, (add_sp_n as Instruction, "ADD SP, #"));
        m.insert(0x03, (inc_bc as Instruction, "INC BC"));
        m.insert(0x13, (inc_de as Instruction, "INC DE"));
        m.insert(0x23, (inc_hl_16 as Instruction, "INC HL"));
        m.insert(0x33, (inc_sp as Instruction, "INC SP"));
        m.insert(0x0B, (dec_bc as Instruction, "DEC BC"));
        m.insert(0x1B, (dec_de as Instruction, "DEC DE"));
        m.insert(0x2B, (dec_hl_16 as Instruction, "DEC HL"));
        m.insert(0x3B, (dec_sp as Instruction, "DEC SP"));

        // ------------------------------- Misc. -------------------------------

        m.insert(0x27, (daa as Instruction, "DAA"));
        m.insert(0x2F, (cpl as Instruction, "CPL"));
        m.insert(0x3F, (ccf as Instruction, "CCF"));
        m.insert(0x37, (scf as Instruction, "SCF"));
        m.insert(0x76, (halt as Instruction, "HALT"));
        m.insert(0x10, (stop as Instruction, "STOP"));
        m.insert(0xF3, (di as Instruction, "DI"));
        m.insert(0xFB, (ei as Instruction, "EI"));

        // ------------------------- Rotates & Shifts --------------------------

        m.insert(0x07, (rlca as Instruction, "RLCA"));
        m.insert(0x17, (rla as Instruction, "RLA"));
        m.insert(0x0F, (rrca as Instruction, "RRCA"));
        m.insert(0x1F, (rra as Instruction, "RRA"));

        // ------------------------------- Jumps -------------------------------

        m.insert(0xC3, (jp as Instruction, "JP"));

        m.insert(0xC2, (jp_nz_nn as Instruction, "JP Z, nn"));
        m.insert(0xCA, (jp_z_nn as Instruction, "JP Z, nn"));
        m.insert(0xD2, (jp_nc_nn as Instruction, "JP NC, nn"));
        m.insert(0xDA, (jp_c_nn as Instruction, "JP C, nn"));

        m.insert(0xE9, (jp_hl as Instruction, "JP HL"));

        m.insert(0x18, (jr_n as Instruction, "JR n"));

        m.insert(0x20, (jr_nz as Instruction, "JR NZ, *"));
        m.insert(0x28, (jr_z as Instruction, "JR Z, *"));
        m.insert(0x30, (jr_nc as Instruction, "JR NC, *"));
        m.insert(0x38, (jr_c as Instruction, "JR C, *"));

        // ------------------------------- Calls -------------------------------

        m.insert(0xCD, (call_nn as Instruction, "CALL nn"));

        m.insert(0xC4, (call_nz_nn as Instruction, "CALL NZ, nn"));
        m.insert(0xCC, (call_z_nn as Instruction, "CALL Z, nn"));
        m.insert(0xD4, (call_nc_nn as Instruction, "CALL NC, nn"));
        m.insert(0xDC, (call_c_nn as Instruction, "CALL C, nn"));

        // ----------------------------- Restarts ------------------------------

        m.insert(0xC7, (rst_00h as Instruction, "RST 00H"));
        m.insert(0xCF, (rst_08h as Instruction, "RST 08H"));
        m.insert(0xD7, (rst_10h as Instruction, "RST 10H"));
        m.insert(0xDF, (rst_18h as Instruction, "RST 18H"));
        m.insert(0xE7, (rst_20h as Instruction, "RST 20H"));
        m.insert(0xEF, (rst_28h as Instruction, "RST 28H"));
        m.insert(0xF7, (rst_30h as Instruction, "RST 30H"));
        m.insert(0xFF, (rst_38h as Instruction, "RST 38H"));

        // ----------------------------- Returns -------------------------------

        m.insert(0xC9, (ret as Instruction, "RET"));

        m.insert(0xC0, (ret_nz as Instruction, "RET NZ"));
        m.insert(0xC8, (ret_z as Instruction, "RET Z"));
        m.insert(0xD0, (ret_nc as Instruction, "RET NC"));
        m.insert(0xD8, (ret_c as Instruction, "RET C"));

        m.insert(0xD9, (reti as Instruction, "RETI"));

        m
    };

    /// HashMap that indexes instruction function pointers and their names by
    /// opcodes prefixed by 0xCB
    static ref OPCODES_CB: HashMap< u8, (Instruction, &'static str) > =
    {
        use crate::cpu::*;
        let mut m = HashMap::new();

        m.insert(0x37, (swap_a as Instruction, "SWAP A"));
        m.insert(0x30, (swap_b as Instruction, "SWAP B"));
        m.insert(0x31, (swap_c as Instruction, "SWAP C"));
        m.insert(0x32, (swap_d as Instruction, "SWAP D"));
        m.insert(0x33, (swap_e as Instruction, "SWAP E"));
        m.insert(0x34, (swap_h as Instruction, "SWAP H"));
        m.insert(0x35, (swap_l as Instruction, "SWAP L"));
        m.insert(0x36, (swap_hl as Instruction, "SWAP HL"));

        m.insert(0x07, (rlc_a as Instruction, "RLC A"));
        m.insert(0x00, (rlc_b as Instruction, "RLC B"));
        m.insert(0x01, (rlc_c as Instruction, "RLC C"));
        m.insert(0x02, (rlc_d as Instruction, "RLC D"));
        m.insert(0x03, (rlc_e as Instruction, "RLC E"));
        m.insert(0x04, (rlc_h as Instruction, "RLC H"));
        m.insert(0x05, (rlc_l as Instruction, "RLC L"));
        m.insert(0x06, (rlc_hl as Instruction, "RLC HL"));

        m.insert(0x17, (rl_a as Instruction, "RL A"));
        m.insert(0x10, (rl_b as Instruction, "RL B"));
        m.insert(0x11, (rl_c as Instruction, "RL C"));
        m.insert(0x12, (rl_d as Instruction, "RL D"));
        m.insert(0x13, (rl_e as Instruction, "RL E"));
        m.insert(0x14, (rl_h as Instruction, "RL H"));
        m.insert(0x15, (rl_l as Instruction, "RL L"));
        m.insert(0x16, (rl_hl as Instruction, "RL HL"));

        m.insert(0x0F, (rrc_a as Instruction, "RRC A"));
        m.insert(0x08, (rrc_b as Instruction, "RRC B"));
        m.insert(0x09, (rrc_c as Instruction, "RRC C"));
        m.insert(0x0A, (rrc_d as Instruction, "RRC D"));
        m.insert(0x0B, (rrc_e as Instruction, "RRC E"));
        m.insert(0x0C, (rrc_h as Instruction, "RRC H"));
        m.insert(0x0D, (rrc_l as Instruction, "RRC L"));
        m.insert(0x0E, (rrc_hl as Instruction, "RRC HL"));

        m.insert(0x1F, (rr_a as Instruction, "RR A"));
        m.insert(0x18, (rr_b as Instruction, "RR B"));
        m.insert(0x19, (rr_c as Instruction, "RR C"));
        m.insert(0x1A, (rr_d as Instruction, "RR D"));
        m.insert(0x1B, (rr_e as Instruction, "RR E"));
        m.insert(0x1C, (rr_h as Instruction, "RR H"));
        m.insert(0x1D, (rr_l as Instruction, "RR L"));
        m.insert(0x1E, (rr_hl as Instruction, "RR HL"));

        m.insert(0x27, (sla_a as Instruction, "SLA A"));
        m.insert(0x20, (sla_b as Instruction, "SLA B"));
        m.insert(0x21, (sla_c as Instruction, "SLA C"));
        m.insert(0x22, (sla_d as Instruction, "SLA D"));
        m.insert(0x23, (sla_e as Instruction, "SLA E"));
        m.insert(0x24, (sla_h as Instruction, "SLA H"));
        m.insert(0x25, (sla_l as Instruction, "SLA L"));
        m.insert(0x26, (sla_hl as Instruction, "SLA HL"));

        m.insert(0x2F, (sra_a as Instruction, "SRA A"));
        m.insert(0x28, (sra_b as Instruction, "SRA B"));
        m.insert(0x29, (sra_c as Instruction, "SRA C"));
        m.insert(0x2A, (sra_d as Instruction, "SRA D"));
        m.insert(0x2B, (sra_e as Instruction, "SRA E"));
        m.insert(0x2C, (sra_h as Instruction, "SRA H"));
        m.insert(0x2D, (sra_l as Instruction, "SRA L"));
        m.insert(0x2E, (sra_hl as Instruction, "SRA HL"));

        m.insert(0x3F, (srl_a as Instruction, "SRL A"));
        m.insert(0x38, (srl_b as Instruction, "SRL B"));
        m.insert(0x39, (srl_c as Instruction, "SRL C"));
        m.insert(0x3A, (srl_d as Instruction, "SRL D"));
        m.insert(0x3B, (srl_e as Instruction, "SRL E"));
        m.insert(0x2C, (srl_h as Instruction, "SRL H"));
        m.insert(0x3D, (srl_l as Instruction, "SRL L"));
        m.insert(0x3E, (srl_hl as Instruction, "SRL HL"));

        m.insert(0x47, (bit_b_a as Instruction, "BIT b, A"));
        m.insert(0x40, (bit_b_b as Instruction, "BIT b, B"));
        m.insert(0x41, (bit_b_c as Instruction, "BIT b, C"));
        m.insert(0x42, (bit_b_d as Instruction, "BIT b, D"));
        m.insert(0x43, (bit_b_e as Instruction, "BIT b, E"));
        m.insert(0x44, (bit_b_h as Instruction, "BIT b, H"));
        m.insert(0x45, (bit_b_l as Instruction, "BIT b, L"));
        m.insert(0x46, (bit_b_hl as Instruction, "BIT b, HL"));

        m.insert(0xC7, (set_b_a as Instruction, "SET b, A"));
        m.insert(0xC0, (set_b_b as Instruction, "SET b, B"));
        m.insert(0xC1, (set_b_c as Instruction, "SET b, C"));
        m.insert(0xC2, (set_b_d as Instruction, "SET b, D"));
        m.insert(0xC3, (set_b_e as Instruction, "SET b, E"));
        m.insert(0xC4, (set_b_h as Instruction, "SET b, H"));
        m.insert(0xC5, (set_b_l as Instruction, "SET b, L"));
        m.insert(0xC6, (set_b_hl as Instruction, "SET b, HL"));

        m.insert(0x87, (res_b_a as Instruction, "RES b, A"));
        m.insert(0x80, (res_b_b as Instruction, "RES b, B"));
        m.insert(0x81, (res_b_c as Instruction, "RES b, C"));
        m.insert(0x82, (res_b_d as Instruction, "RES b, D"));
        m.insert(0x83, (res_b_e as Instruction, "RES b, E"));
        m.insert(0x84, (res_b_h as Instruction, "RES b, H"));
        m.insert(0x85, (res_b_l as Instruction, "RES b, L"));
        m.insert(0x86, (res_b_hl as Instruction, "RES b, HL"));

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
    cpu.regs.a = v;
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

/// Helper function to add two bytes with carry and update the CPU flags register
fn add_with_carry_update_flags(cpu: &mut CPU, b1: u8, b2: u8) -> u8
{
    let b1 = b1 as u32;
    let b2 = b2 as u32;
    let carry = cpu.flags.c as u32;
    let result = b1 + b2 + carry;
    let result_u8 = result as u8;

    cpu.flags.z = result_u8 == 0;
    cpu.flags.n = false;
    cpu.flags.h = (b1 ^ b2 ^ result) & 0x10 != 0;
    cpu.flags.c = result & 0x100 != 0;

    result_u8
}

/// Add ['A' + carry] to 'A'
fn adc_a_a(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let v = add_with_carry_update_flags(cpu, a, a);
    cpu.regs.a = v;
    4
}

/// Add ['B' + carry] to 'A'
fn adc_a_b(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let b = cpu.regs.b;
    let v = add_with_carry_update_flags(cpu, a, b);
    cpu.regs.a = v;
    4
}

/// Add ['C' + carry] to 'A'
fn adc_a_c(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let c = cpu.regs.c;
    let v = add_with_carry_update_flags(cpu, a, c);
    cpu.regs.a = v;
    4
}

/// Add ['D' + carry] to 'A'
fn adc_a_d(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let d = cpu.regs.d;
    let v = add_with_carry_update_flags(cpu, a, d);
    cpu.regs.a = v;
    4
}

/// Add ['E' + carry] to 'A'
fn adc_a_e(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let e = cpu.regs.e;
    let v = add_with_carry_update_flags(cpu, a, e);
    cpu.regs.a = v;
    4
}

/// Add ['H' + carry] to 'A'
fn adc_a_h(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let h = cpu.regs.h;
    let v = add_with_carry_update_flags(cpu, a, h);
    cpu.regs.a = v;
    4
}

/// Add ['L' + carry] to 'A'
fn adc_a_l(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let l = cpu.regs.l;
    let v = add_with_carry_update_flags(cpu, a, l);
    cpu.regs.a = v;
    4
}

/// Add ['HL' + carry] to 'A'
fn adc_a_hl(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let hl = cpu.hl();
    let n = cpu.fetch_byte(hl);
    let v = add_with_carry_update_flags(cpu, a, n);
    cpu.regs.a = v;
    8
}

/// Add [next immediate byte + carry] to 'A'
fn adc_a_n(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let n = cpu.next_byte();
    let v = add_with_carry_update_flags(cpu, a, n);
    cpu.regs.a = v;
    8
}

/// Helper function to subtract two bytes and update the CPU flags register
/// and return the result of the subtraction as a single byte
fn sub_and_update_flags(cpu: &mut CPU, b1: u8, b2: u8) -> u8
{
    let b1 = b1 as u32;
    let b2 = b2 as u32;
    let result = b1 - b2;
    let result_u8 = result as u8;

    cpu.flags.z = result_u8 == 0;
    cpu.flags.n = true;
    cpu.flags.h = (b1 ^ b2 ^ result) & 0x10 != 0;
    cpu.flags.c = result & 0x100 != 0;

    result_u8
}

/// Subtract 'A' from 'A'
fn sub_a_a(cpu: &mut CPU) -> u8
{
    cpu.flags.z = true;
    cpu.flags.n = true;
    cpu.flags.h = false;
    cpu.flags.c = false;
    cpu.regs.a = 0;
    4
}

/// Subtract 'B' from 'A'
fn sub_a_b(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let b = cpu.regs.b;
    let v = sub_and_update_flags(cpu, a, b);
    cpu.regs.a = v;
    4
}

/// Subtract 'C' from 'A'
fn sub_a_c(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let c = cpu.regs.c;
    let v = sub_and_update_flags(cpu, a, c);
    cpu.regs.a = v;
    4
}

/// Subtract 'D' from 'A'
fn sub_a_d(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let d = cpu.regs.d;
    let v = sub_and_update_flags(cpu, a, d);
    cpu.regs.a = v;
    4
}

/// Subtract 'E' from 'A'
fn sub_a_e(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let e = cpu.regs.e;
    let v = sub_and_update_flags(cpu, a, e);
    cpu.regs.a = v;
    4
}

/// Subtract 'H' from 'A'
fn sub_a_h(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let h = cpu.regs.h;
    let v = sub_and_update_flags(cpu, a, h);
    cpu.regs.a = v;
    4
}

/// Subtract 'B' from 'A'
fn sub_a_l(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let l = cpu.regs.l;
    let v = sub_and_update_flags(cpu, a, l);
    cpu.regs.a = v;
    4
}

/// Subtract 'HL' from 'A'
fn sub_a_hl(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let hl = cpu.hl();
    let n = cpu.fetch_byte(hl);
    let v = sub_and_update_flags(cpu, a, n);
    cpu.regs.a = v;
    8
}

/// Subtract the next immediate byte from 'A'
fn sub_a_n(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let n = cpu.next_byte();
    let v = sub_and_update_flags(cpu, a, n);
    cpu.regs.a = v;
    8
}

/// Helper function to subtract two bytes with carry and update the CPU flags
/// register and then return the result of the subtraction as a single byte
fn sub_with_carry_update_flags(cpu: &mut CPU, b1: u8, b2: u8) -> u8
{
    let b1 = b1 as u32;
    let b2 = b2 as u32;
    let carry = cpu.flags.c as u32;
    let result = b1 - b2 - carry;
    let result_u8 = result as u8;

    cpu.flags.z = result_u8 == 0;
    cpu.flags.n = true;
    cpu.flags.h = (b1 ^ b2 ^ result) & 0x10 != 0;
    cpu.flags.c = result & 0x100 != 0;

    result_u8
}

/// Subtract ['A' + carry] from 'A'
fn sbc_a_a(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let v = sub_with_carry_update_flags(cpu, a, a);
    cpu.regs.a = v;
    4
}

/// Subtract ['B' + carry] from 'A'
fn sbc_a_b(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let b = cpu.regs.b;
    let v = sub_with_carry_update_flags(cpu, a, b);
    cpu.regs.a = v;
    4
}

/// Subtract ['C' + carry] from 'A'
fn sbc_a_c(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let c = cpu.regs.c;
    let v = sub_with_carry_update_flags(cpu, a, c);
    cpu.regs.a = v;
    4
}

/// Subtract ['D' + carry] from 'A'
fn sbc_a_d(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let d = cpu.regs.d;
    let v = sub_with_carry_update_flags(cpu, a, d);
    cpu.regs.a = v;
    4
}

/// Subtract ['E' + carry] from 'A'
fn sbc_a_e(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let e = cpu.regs.e;
    let v = sub_with_carry_update_flags(cpu, a, e);
    cpu.regs.a = v;
    4
}

/// Subtract ['H' + carry] from 'A'
fn sbc_a_h(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let h = cpu.regs.h;
    let v = sub_with_carry_update_flags(cpu, a, h);
    cpu.regs.a = v;
    4
}

/// Subtract ['L' + carry] from 'A'
fn sbc_a_l(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let l = cpu.regs.l;
    let v = sub_with_carry_update_flags(cpu, a, l);
    cpu.regs.a = v;
    4
}

/// Subtract ['HL' + carry] from 'A'
fn sbc_a_hl(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let hl = cpu.hl();
    let n = cpu.fetch_byte(hl);
    let v = sub_with_carry_update_flags(cpu, a, n);
    cpu.regs.a = v;
    8
}

/// Subtract [next immediate byte + carry] from 'A'
fn sbc_a_n(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let n = cpu.next_byte();
    let v = sub_with_carry_update_flags(cpu, a, n);
    cpu.regs.a = v;
    8
}

/// AND 'A' with 'A'
fn and_a_a(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    cpu.flags.z = a == 0;
    cpu.flags.n = false;
    cpu.flags.h = true;
    cpu.flags.c = false;
    4
}

/// AND 'B' with 'A'
fn and_a_b(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let b = cpu.regs.b;
    let v = a & b;

    cpu.flags.z = v == 0;
    cpu.flags.n = false;
    cpu.flags.h = true;
    cpu.flags.c = false;

    cpu.regs.a = v;
    4
}

/// AND 'C' with 'A'
fn and_a_c(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let c = cpu.regs.c;
    let v = a & c;

    cpu.flags.z = v == 0;
    cpu.flags.n = false;
    cpu.flags.h = true;
    cpu.flags.c = false;

    cpu.regs.a = v;
    4
}

/// AND 'D' with 'A'
fn and_a_d(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let d = cpu.regs.d;
    let v = a & d;

    cpu.flags.z = v == 0;
    cpu.flags.n = false;
    cpu.flags.h = true;
    cpu.flags.c = false;

    cpu.regs.a = v;
    4
}

/// AND 'E' with 'A'
fn and_a_e(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let e = cpu.regs.e;
    let v = a & e;

    cpu.flags.z = v == 0;
    cpu.flags.n = false;
    cpu.flags.h = true;
    cpu.flags.c = false;

    cpu.regs.a = v;
    4
}

/// AND 'H' with 'A'
fn and_a_h(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let h = cpu.regs.h;
    let v = a & h;

    cpu.flags.z = v == 0;
    cpu.flags.n = false;
    cpu.flags.h = true;
    cpu.flags.c = false;

    cpu.regs.a = v;
    4
}

/// AND 'L' with 'A'
fn and_a_l(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let l = cpu.regs.l;
    let v = a & l;

    cpu.flags.z = v == 0;
    cpu.flags.n = false;
    cpu.flags.h = true;
    cpu.flags.c = false;

    cpu.regs.a = v;
    4
}

/// AND 'HL' with 'A'
fn and_a_hl(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let hl = cpu.hl();
    let n = cpu.fetch_byte(hl);
    let v = a & n;

    cpu.flags.z = v == 0;
    cpu.flags.n = false;
    cpu.flags.h = true;
    cpu.flags.c = false;

    cpu.regs.a = v;
    8
}

/// AND the next immediate byte with 'A'
fn and_a_n(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let n = cpu.next_byte();
    let v = a & n;

    cpu.flags.z = v == 0;
    cpu.flags.n = false;
    cpu.flags.h = true;
    cpu.flags.c = false;

    cpu.regs.a = v;
    8
}

/// OR 'A' with 'A'
fn or_a_a(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    cpu.flags.z = a == 0;
    cpu.flags.n = false;
    cpu.flags.h = false;
    cpu.flags.c = false;
    4
}

/// OR 'B' with 'A'
fn or_a_b(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let b = cpu.regs.b;
    let v = a | b;

    cpu.flags.z = v == 0;
    cpu.flags.n = false;
    cpu.flags.h = false;
    cpu.flags.c = false;

    cpu.regs.a = v;
    4
}

/// OR 'C' with 'A'
fn or_a_c(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let c = cpu.regs.c;
    let v = a | c;

    cpu.flags.z = v == 0;
    cpu.flags.n = false;
    cpu.flags.h = false;
    cpu.flags.c = false;

    cpu.regs.a = v;
    4
}

/// OR 'D' with 'A'
fn or_a_d(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let d = cpu.regs.d;
    let v = a | d;

    cpu.flags.z = v == 0;
    cpu.flags.n = false;
    cpu.flags.h = false;
    cpu.flags.c = false;

    cpu.regs.a = v;
    4
}

/// OR 'E' with 'A'
fn or_a_e(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let e = cpu.regs.e;
    let v = a | e;

    cpu.flags.z = v == 0;
    cpu.flags.n = false;
    cpu.flags.h = false;
    cpu.flags.c = false;

    cpu.regs.a = v;
    4
}

/// OR 'H' with 'A'
fn or_a_h(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let h = cpu.regs.h;
    let v = a | h;

    cpu.flags.z = v == 0;
    cpu.flags.n = false;
    cpu.flags.h = false;
    cpu.flags.c = false;

    cpu.regs.a = v;
    4
}

/// OR 'L' with 'A'
fn or_a_l(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let l = cpu.regs.l;
    let v = a | l;

    cpu.flags.z = v == 0;
    cpu.flags.n = false;
    cpu.flags.h = false;
    cpu.flags.c = false;

    cpu.regs.a = v;
    4
}

/// OR 'HL' with 'A'
fn or_a_hl(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let hl = cpu.hl();
    let n = cpu.fetch_byte(hl);
    let v = a | n;

    cpu.flags.z = v == 0;
    cpu.flags.n = false;
    cpu.flags.h = false;
    cpu.flags.c = false;

    cpu.regs.a = v;
    8
}

/// or the next immediate byte with 'A'
fn or_a_n(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let n = cpu.next_byte();
    let v = a | n;

    cpu.flags.z = v == 0;
    cpu.flags.n = false;
    cpu.flags.h = false;
    cpu.flags.c = false;

    cpu.regs.a = v;
    8
}

/// XOR 'A' with 'A'
fn xor_a_a(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    cpu.flags.z = a == 0;
    cpu.flags.n = false;
    cpu.flags.h = false;
    cpu.flags.c = false;
    4
}

/// XOR 'B' with 'A'
fn xor_a_b(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let b = cpu.regs.b;
    let v = a ^ b;

    cpu.flags.z = v == 0;
    cpu.flags.n = false;
    cpu.flags.h = false;
    cpu.flags.c = false;

    cpu.regs.a = v;
    4
}

/// XOR 'C' with 'A'
fn xor_a_c(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let c = cpu.regs.c;
    let v = a ^ c;

    cpu.flags.z = v == 0;
    cpu.flags.n = false;
    cpu.flags.h = false;
    cpu.flags.c = false;

    cpu.regs.a = v;
    4
}

/// XOR 'D' with 'A'
fn xor_a_d(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let d = cpu.regs.d;
    let v = a ^ d;

    cpu.flags.z = v == 0;
    cpu.flags.n = false;
    cpu.flags.h = false;
    cpu.flags.c = false;

    cpu.regs.a = v;
    4
}

/// XOR 'E' with 'A'
fn xor_a_e(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let e = cpu.regs.e;
    let v = a ^ e;

    cpu.flags.z = v == 0;
    cpu.flags.n = false;
    cpu.flags.h = false;
    cpu.flags.c = false;

    cpu.regs.a = v;
    4
}

/// XOR 'H' with 'A'
fn xor_a_h(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let h = cpu.regs.h;
    let v = a ^ h;

    cpu.flags.z = v == 0;
    cpu.flags.n = false;
    cpu.flags.h = false;
    cpu.flags.c = false;

    cpu.regs.a = v;
    4
}

/// XOR 'L' with 'A'
fn xor_a_l(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let l = cpu.regs.l;
    let v = a ^ l;

    cpu.flags.z = v == 0;
    cpu.flags.n = false;
    cpu.flags.h = false;
    cpu.flags.c = false;

    cpu.regs.a = v;
    4
}

/// XOR 'HL' with 'A'
fn xor_a_hl(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let hl = cpu.hl();
    let n = cpu.fetch_byte(hl);
    let v = a ^ n;

    cpu.flags.z = v == 0;
    cpu.flags.n = false;
    cpu.flags.h = false;
    cpu.flags.c = false;

    cpu.regs.a = v;
    8
}

/// Xor the next immediate byte with 'A'
fn xor_a_n(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let n = cpu.next_byte();
    let v = a ^ n;

    cpu.flags.z = v == 0;
    cpu.flags.n = false;
    cpu.flags.h = false;
    cpu.flags.c = false;

    cpu.regs.a = v;
    8
}

/// Compare 'A' with 'A'
fn cp_a_a(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    sub_and_update_flags(cpu, a, a);
    4
}

/// Compare 'A' with 'B'
fn cp_a_b(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let b = cpu.regs.b;
    sub_and_update_flags(cpu, a, b);
    4
}

/// Compare 'A' with 'C'
fn cp_a_c(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let c = cpu.regs.c;
    sub_and_update_flags(cpu, a, c);
    4
}

/// Compare 'A' with 'D'
fn cp_a_d(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let d = cpu.regs.d;
    sub_and_update_flags(cpu, a, d);
    4
}

/// Compare 'A' with 'E'
fn cp_a_e(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let e = cpu.regs.e;
    sub_and_update_flags(cpu, a, e);
    4
}

/// Compare 'A' with 'H'
fn cp_a_h(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let h = cpu.regs.h;
    sub_and_update_flags(cpu, a, h);
    4
}

/// Compare 'A' with 'L'
fn cp_a_l(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let l = cpu.regs.l;
    sub_and_update_flags(cpu, a, l);
    4
}

/// Compare 'A' with 'HL'
fn cp_a_hl(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let hl = cpu.hl();
    let n = cpu.fetch_byte(hl);
    sub_and_update_flags(cpu, a, n);
    8
}

/// Compare 'A' with the next immediate byte
fn cp_a_n(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let n = cpu.next_byte();
    sub_and_update_flags(cpu, a, n);
    8
}

/// Increment 'A'
fn inc_a(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let v = a + 1;

    cpu.flags.z = v == 0;
    cpu.flags.n = false;
    cpu.flags.h = a & 0xF == 0xF;

    cpu.regs.a = v;
    4
}

/// Increment 'B'
fn inc_b(cpu: &mut CPU) -> u8
{
    let b = cpu.regs.b;
    let v = b + 1;

    cpu.flags.z = v == 0;
    cpu.flags.n = false;
    cpu.flags.h = b & 0xF == 0xF;

    cpu.regs.b = v;
    4
}

/// Increment 'C'
fn inc_c(cpu: &mut CPU) -> u8
{
    let c = cpu.regs.c;
    let v = c + 1;

    cpu.flags.z = v == 0;
    cpu.flags.n = false;
    cpu.flags.h = c & 0xF == 0xF;

    cpu.regs.c = v;
    4
}

/// Increment 'D'
fn inc_d(cpu: &mut CPU) -> u8
{
    let d = cpu.regs.d;
    let v = d + 1;

    cpu.flags.z = v == 0;
    cpu.flags.n = false;
    cpu.flags.h = d & 0xF == 0xF;

    cpu.regs.d = v;
    4
}

/// Increment 'E'
fn inc_e(cpu: &mut CPU) -> u8
{
    let e = cpu.regs.e;
    let v = e + 1;

    cpu.flags.z = v == 0;
    cpu.flags.n = false;
    cpu.flags.h = e & 0xF == 0xF;

    cpu.regs.e = v;
    4
}

/// Increment 'H'
fn inc_h(cpu: &mut CPU) -> u8
{
    let h = cpu.regs.h;
    let v = h + 1;

    cpu.flags.z = v == 0;
    cpu.flags.n = false;
    cpu.flags.h = h & 0xF == 0xF;

    cpu.regs.h = v;
    4
}

/// Increment 'L'
fn inc_l(cpu: &mut CPU) -> u8
{
    let l = cpu.regs.l;
    let v = l + 1;

    cpu.flags.z = v == 0;
    cpu.flags.n = false;
    cpu.flags.h = l & 0xF == 0xF;

    cpu.regs.l = v;
    4
}

/// Increment 'HL'
fn inc_hl(cpu: &mut CPU) -> u8
{
    let hl = cpu.hl();
    let n = cpu.fetch_byte(hl);
    let v = n + 1;

    cpu.flags.z = v == 0;
    cpu.flags.n = false;
    cpu.flags.h = n & 0xF == 0xF;

    cpu.store_byte(hl, v);
    12
}

/// Decrement 'A'
fn dec_a(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let v = a - 1;

    cpu.flags.z = v == 0;
    cpu.flags.n = true;
    cpu.flags.h = a & 0xF == 0xF;

    cpu.regs.a = v;
    4
}

/// Decrement 'B'
fn dec_b(cpu: &mut CPU) -> u8
{
    let b = cpu.regs.b;
    let v = b - 1;

    cpu.flags.z = v == 0;
    cpu.flags.n = true;
    cpu.flags.h = b & 0xF == 0xF;

    cpu.regs.b = v;
    4
}

/// Decrement 'C'
fn dec_c(cpu: &mut CPU) -> u8
{
    let c = cpu.regs.c;
    let v = c - 1;

    cpu.flags.z = v == 0;
    cpu.flags.n = true;
    cpu.flags.h = c & 0xF == 0xF;

    cpu.regs.c = v;
    4
}

/// Decrement 'D'
fn dec_d(cpu: &mut CPU) -> u8
{
    let d = cpu.regs.d;
    let v = d - 1;

    cpu.flags.z = v == 0;
    cpu.flags.n = true;
    cpu.flags.h = d & 0xF == 0xF;

    cpu.regs.d = v;
    4
}

/// Decrement 'E'
fn dec_e(cpu: &mut CPU) -> u8
{
    let e = cpu.regs.e;
    let v = e - 1;

    cpu.flags.z = v == 0;
    cpu.flags.n = true;
    cpu.flags.h = e & 0xF == 0xF;

    cpu.regs.e = v;
    4
}

/// Decrement 'H'
fn dec_h(cpu: &mut CPU) -> u8
{
    let h = cpu.regs.h;
    let v = h - 1;

    cpu.flags.z = v == 0;
    cpu.flags.n = true;
    cpu.flags.h = h & 0xF == 0xF;

    cpu.regs.h = v;
    4
}

/// Decrement 'L'
fn dec_l(cpu: &mut CPU) -> u8
{
    let l = cpu.regs.l;
    let v = l - 1;

    cpu.flags.z = v == 0;
    cpu.flags.n = true;
    cpu.flags.h = l & 0xF == 0xF;

    cpu.regs.l = v;
    4
}

/// Decrement 'HL'
fn dec_hl(cpu: &mut CPU) -> u8
{
    let hl = cpu.hl();
    let n = cpu.fetch_byte(hl);
    let v = n - 1;

    cpu.flags.z = v == 0;
    cpu.flags.n = false;
    cpu.flags.h = n & 0xF == 0xF;

    cpu.store_byte(hl, v);
    12
}

/// Helper function to add two words and update the CPU flags register
/// then return the result of the addition as a single 2 byte word
fn add_words_and_update_flags(cpu: &mut CPU, w1: u16, w2: u16) -> u16
{
    let w1 = w1 as u32;
    let w2 = w2 as u32;
    let v = w1 + w2;
    
    cpu.flags.n = false;
    cpu.flags.h = (w1 ^ w2 ^ v) & 0x1000 != 0;
    cpu.flags.c = v & 0x10000 != 0;

    v as u16
}

/// Add 'BC' to 'HL'
fn add_hl_bc(cpu: &mut CPU) -> u8
{
    let hl = cpu.hl();
    let bc = cpu.bc();
    let v = add_words_and_update_flags(cpu, hl, bc);
    cpu.set_hl(v);
    8
}

/// Add 'DE' to 'HL'
fn add_hl_de(cpu: &mut CPU) -> u8
{
    let hl = cpu.hl();
    let de = cpu.de();
    let v = add_words_and_update_flags(cpu, hl, de);
    cpu.set_hl(v);
    8
}

/// Add 'HL' to 'HL'
fn add_hl_hl(cpu: &mut CPU) -> u8
{
    let hl = cpu.hl();
    let v = add_words_and_update_flags(cpu, hl, hl);
    cpu.set_hl(v);
    8
}

/// Add Stack Pointer to 'HL'
fn add_hl_sp(cpu: &mut CPU) -> u8
{
    let hl = cpu.hl();
    let sp = cpu.regs.sp;
    let v = add_words_and_update_flags(cpu, hl, sp);
    cpu.set_hl(v);
    8
}

/// Add the next immediate byte to the stack pointer
fn add_sp_n(cpu: &mut CPU) -> u8
{
    let sp = cpu.regs.sp as i32;
    let n = cpu.next_byte() as i8;
    let nn = n as i32;
    let v = sp + nn;

    cpu.flags.z = false;
    cpu.flags.n = false;
    cpu.flags.h = (sp ^ nn ^ v) & 0x10 != 0;
    cpu.flags.c = (sp ^ nn ^ v) & 0x100 != 0;

    cpu.regs.sp = v as u16;
    16
}

/// Increment 'BC'
fn inc_bc(cpu: &mut CPU) -> u8
{
    let bc = cpu.bc();
    cpu.set_bc(bc + 1);
    8
}

/// Increment 'DE'
fn inc_de(cpu: &mut CPU) -> u8
{
    let de = cpu.de();
    cpu.set_de(de + 1);
    8
}

/// Increment 'HL' using 16-bit arithmetic
fn inc_hl_16(cpu: &mut CPU) -> u8
{
    let hl = cpu.hl();
    cpu.set_hl(hl + 1);
    8
}

/// Increment the Stack Pointer
fn inc_sp(cpu: &mut CPU) -> u8
{
    let sp = cpu.regs.sp;
    cpu.regs.sp = sp + 1;
    8
}

/// Decrement 'BC'
fn dec_bc(cpu: &mut CPU) -> u8
{
    let bc = cpu.bc();
    cpu.set_bc(bc - 1);
    8
}

/// Decrement 'DE'
fn dec_de(cpu: &mut CPU) -> u8
{
    let de = cpu.de();
    cpu.set_de(de - 1);
    8
}

/// Decrement 'HL' using 16-bit arithmetic
fn dec_hl_16(cpu: &mut CPU) -> u8
{
    let hl = cpu.hl();
    cpu.set_hl(hl - 1);
    8
}

/// Decrement the Stack Pointer
fn dec_sp(cpu: &mut CPU) -> u8
{
    let sp = cpu.regs.sp;
    cpu.regs.sp = sp - 1;
    8
}

/// Helper function to swap the two nibbles of a byte and update the CPU
/// flags register
fn swap(cpu: &mut CPU, v: u8) -> u8
{
    cpu.flags.z = v == 0;
    cpu.flags.n = false;
    cpu.flags.h = false;
    cpu.flags.c = false;

    (v << 4) | (v >> 4)
}

/// Swap the low and high nibbles of 'A'
fn swap_a(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let v = swap(cpu, a);
    cpu.regs.a = v;   
    8
}

/// Swap the low and high nibbles of 'B'
fn swap_b(cpu: &mut CPU) -> u8
{
    let b = cpu.regs.b;
    let v = swap(cpu, b);
    cpu.regs.b = v;   
    8
}

/// Swap the low and high nibbles of 'C'
fn swap_c(cpu: &mut CPU) -> u8
{
    let c = cpu.regs.c;
    let v = swap(cpu, c);
    cpu.regs.c = v;   
    8
}

/// Swap the low and high nibbles of 'D'
fn swap_d(cpu: &mut CPU) -> u8
{
    let d = cpu.regs.d;
    let v = swap(cpu, d);
    cpu.regs.d = v;   
    8
}

/// Swap the low and high nibbles of 'E'
fn swap_e(cpu: &mut CPU) -> u8
{
    let e = cpu.regs.e;
    let v = swap(cpu, e);
    cpu.regs.e = v;   
    8
}

/// Swap the low and high nibbles of 'H'
fn swap_h(cpu: &mut CPU) -> u8
{
    let h = cpu.regs.h;
    let v = swap(cpu, h);
    cpu.regs.h = v;   
    8
}

/// Swap the low and high nibbles of 'L'
fn swap_l(cpu: &mut CPU) -> u8
{
    let l = cpu.regs.l;
    let v = swap(cpu, l);
    cpu.regs.l = v;   
    8
}

/// Swap the low and high nibbles of 'HL'
fn swap_hl(cpu: &mut CPU) -> u8
{
    let hl = cpu.hl();
    let n = cpu.fetch_byte(hl);
    let v = swap(cpu, n);
    cpu.store_byte(hl, v);
    16
}

/// Decimal adjust 'A' for BCD operations
fn daa(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let mut adj = 0;

    if cpu.flags.h
    {
        adj |= 0x06;
    }

    if cpu.flags.c
    {
        adj |= 0x60;
    }

    let res = if cpu.flags.n 
    {
        a - adj
    } 
    else 
    {
        if a & 0x0F > 0x09 
        { 
            adj |= 0x06; 
        }

        if a > 0x99 
        {
            adj |= 0x60; 
        }

        a + adj
    };

    cpu.regs.a = res;
    cpu.flags.z = res == 0;
    cpu.flags.h = false;
    cpu.flags.c = adj & 0x60 != 0;
    4
}

/// Compliment 'A' (flip all bits)
fn cpl(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    cpu.regs.a = !a;
    cpu.flags.n = true;
    cpu.flags.h = true;
    4
}

/// Compliment carry flag (flip all bits)
fn ccf(cpu: &mut CPU) -> u8
{
    let carry = cpu.flags.c;
    cpu.flags.c = !carry;
    cpu.flags.n = false;
    cpu.flags.h = false;
    4
}

/// Set carry flag
fn scf(cpu: &mut CPU) -> u8
{
    cpu.flags.c = true;
    cpu.flags.n = false;
    cpu.flags.h = false;
    4
}

/// Power down CPU until an interrupt occurs
fn halt(cpu: &mut CPU) -> u8
{
    cpu.halt();
    4
}

/// Halt CPU & LCD display until button press
fn stop(cpu: &mut CPU) -> u8
{
    let _ = cpu.next_byte();
    cpu.stop();
    4
}

/// Disable interrupts but not immediately. Disabled after next instruction
fn di(cpu: &mut CPU) -> u8
{
    cpu.disable_interrupts();
    4
}

/// Enables interrupts but not immediately. Enabled after next instruction
fn ei(cpu: &mut CPU) -> u8
{
    cpu.enable_interrupts_after_next();
    4
}

/// Rotate 'A' Left
fn rlca(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let c = a >> 7;
    cpu.regs.a = (a << 1) | c;
    cpu.flags.c = c != 0;
    cpu.flags.z = false;
    cpu.flags.n = false;
    cpu.flags.h = false;
    4
}

/// Rotate 'A' Left through carry
fn rla(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let new_c = (a >> 7) != 0;
    let old_c = cpu.flags.c as u8;

    cpu.regs.a = (a << 1) | old_c;

    cpu.flags.c = new_c;
    cpu.flags.z = false;
    cpu.flags.n = false;
    cpu.flags.h = false;
    4
}

/// Rotate 'A' Right
fn rrca(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let c = a & 1;

    cpu.regs.a = (a >> 1) | (c << 7);

    cpu.flags.c = c != 0;
    cpu.flags.z = false;
    cpu.flags.h = false;
    cpu.flags.n = false;
    4
}

/// Rotate 'A' Right through carry
fn rra(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let new_c = (a & 1) != 0;
    let old_c = cpu.flags.c as u8;

    cpu.regs.a = (a >> 1) | (old_c << 7);

    cpu.flags.c = new_c;
    cpu.flags.z = false;
    cpu.flags.n = false;
    cpu.flags.h = false;
    4
}

/// Helper function to rotate a byte left and update CPU flags register
fn rlc(cpu: &mut CPU, v: u8) -> u8
{
    cpu.flags.c = v & 0x80 != 0;

    let r = (v << 1) | (v >> 7);

    cpu.flags.z = r == 0;
    cpu.flags.n = false;
    cpu.flags.h = false;

    r
}

/// Rotate A left. Old bit is used for carry flag.
fn rlc_a(cpu: &mut CPU) -> u8
{
    let a = cpu.regs.a;
    let r = rlc(cpu, a);
    cpu.regs.a = r;
    8
}

/// Rotate B left. Old bit is used for carry flag.
fn rlc_b(cpu: &mut CPU) -> u8
{
    let b = cpu.regs.b;
    let r = rlc(cpu, b);
    cpu.regs.b = r;
    8
}

/// Rotate C left. Old bit is used for carry flag.
fn rlc_c(cpu: &mut CPU) -> u8
{
    let c = cpu.regs.c;
    let r = rlc(cpu, c);
    cpu.regs.c = r;
    8
}

/// Rotate D left. Old bit is used for carry flag.
fn rlc_d(cpu: &mut CPU) -> u8
{
    let d = cpu.regs.d;
    let r = rlc(cpu, d);
    cpu.regs.d = r;
    8
}

/// Rotate E left. Old bit is used for carry flag.
fn rlc_e(cpu: &mut CPU) -> u8
{
    let e = cpu.regs.e;
    let r = rlc(cpu, e);
    cpu.regs.e = r;
    8
}

/// Rotate H left. Old bit is used for carry flag.
fn rlc_h(cpu: &mut CPU) -> u8
{
    let h = cpu.regs.h;
    let r = rlc(cpu, h);
    cpu.regs.h = r;
    8
}

/// Rotate L left. Old bit is used for carry flag.
fn rlc_l(cpu: &mut CPU) -> u8
{
    let l = cpu.regs.l;
    let r = rlc(cpu, l);
    cpu.regs.l = r;
    8
}

/// Rotate HL left. Old bit is used for carry flag.
fn rlc_hl(cpu: &mut CPU) -> u8
{
    let hl = cpu.hl();
    let n = cpu.fetch_byte(hl);
    let r = rlc(cpu, n);
    cpu.store_byte(hl, r);
    16
}

/// Helper function to rotate a byte left through carry and update CPU flags
fn rl(cpu: &mut CPU, v: u8) -> u8
{
    let old_c = cpu.flags.c as u8;
    cpu.flags.c = v & 0x80 != 0;
    let r = (v << 1) | old_c;

    cpu.flags.z = r == 0;
    cpu.flags.n = false;
    cpu.flags.h = false;

    r
}

/// Rotate A left through carry flag
fn rl_a(cpu: &mut CPU) -> u8
{
    let v = cpu.regs.a;
    let r = rl(cpu, v);
    cpu.regs.a = r;
    8
}

/// Rotate B left through carry flag
fn rl_b(cpu: &mut CPU) -> u8
{
    let v = cpu.regs.b;
    let r = rl(cpu, v);
    cpu.regs.b = r;
    8
}

/// Rotate C left through carry flag
fn rl_c(cpu: &mut CPU) -> u8
{
    let v = cpu.regs.c;
    let r = rl(cpu, v);
    cpu.regs.c = r;
    8
}

/// Rotate D left through carry flag
fn rl_d(cpu: &mut CPU) -> u8
{
    let v = cpu.regs.d;
    let r = rl(cpu, v);
    cpu.regs.d = r;
    8
}

/// Rotate E left through carry flag
fn rl_e(cpu: &mut CPU) -> u8
{
    let v = cpu.regs.e;
    let r = rl(cpu, v);
    cpu.regs.e = r;
    8
}

/// Rotate H left through carry flag
fn rl_h(cpu: &mut CPU) -> u8
{
    let v = cpu.regs.h;
    let r = rl(cpu, v);
    cpu.regs.h = r;
    8
}

/// Rotate L left through carry flag
fn rl_l(cpu: &mut CPU) -> u8
{
    let v = cpu.regs.l;
    let r = rl(cpu, v);
    cpu.regs.l = r;
    8
}

/// Rotate HL left through carry flag
fn rl_hl(cpu: &mut CPU) -> u8
{
    let hl = cpu.hl();
    let n = cpu.fetch_byte(hl);
    let r = rl(cpu, n);
    cpu.store_byte(hl, r);
    16
}

/// Helper function to rotate a byte right and update CPU flags register
fn rrc(cpu: &mut CPU, v: u8) -> u8
{
    cpu.flags.c = v & 1 != 0;

    let r = (v >> 1) | (v << 7);

    cpu.flags.z = r == 0;
    cpu.flags.n = false;
    cpu.flags.h = false;

    r
}

/// Rotate A to the right
fn rrc_a(cpu: &mut CPU) -> u8
{
    let v = cpu.regs.a;
    let r = rrc(cpu, v);
    cpu.regs.a = r;
    8
}

/// Rotate B to the right
fn rrc_b(cpu: &mut CPU) -> u8
{
    let v = cpu.regs.b;
    let r = rrc(cpu, v);
    cpu.regs.b = r;
    8
}

/// Rotate C to the right
fn rrc_c(cpu: &mut CPU) -> u8
{
    let v = cpu.regs.c;
    let r = rrc(cpu, v);
    cpu.regs.c = r;
    8
}

/// Rotate D to the right
fn rrc_d(cpu: &mut CPU) -> u8
{
    let v = cpu.regs.d;
    let r = rrc(cpu, v);
    cpu.regs.d = r;
    8
}

/// Rotate E to the right
fn rrc_e(cpu: &mut CPU) -> u8
{
    let v = cpu.regs.e;
    let r = rrc(cpu, v);
    cpu.regs.e = r;
    8
}

/// Rotate H to the right
fn rrc_h(cpu: &mut CPU) -> u8
{
    let v = cpu.regs.h;
    let r = rrc(cpu, v);
    cpu.regs.h = r;
    8
}

/// Rotate L to the right
fn rrc_l(cpu: &mut CPU) -> u8
{
    let v = cpu.regs.l;
    let r = rrc(cpu, v);
    cpu.regs.l = r;
    8
}

/// Rotate HL to the right
fn rrc_hl(cpu: &mut CPU) -> u8
{
    let hl = cpu.hl();
    let n = cpu.fetch_byte(hl);
    let r = rrc(cpu, n);
    cpu.store_byte(hl, r);
    16
}