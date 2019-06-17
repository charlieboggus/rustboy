use crate::cpu::CPU;
use crate::cpu::registers::Registers;
use crate::mem::Memory;

/// Location of the Zero Flag byte
const Z: u8 = 0x80;

/// Location of the Add/Sub Flag byte
const N: u8 = 0x40;

/// Location of the Half-Carry Flag byte
const H: u8 = 0x20;

/// Location of the Carry Flag byte
const C: u8 = 0x10;

/// Execute the instruction for the given opcode and return number of cycles
pub fn exec(opcode: u8, cpu: &mut CPU, mem: &mut Memory) -> u32
{
    let mut regs = &mut cpu.regs;

    /// Load the value stored in register $r2 into register $r1
    macro_rules! ld (($r1: ident, $r2: ident) => 
    ({ 
        regs.$r1 = regs.$r2; 
        1 
    }));

    /// Load the next immediate byte into the 8-bit register $r
    macro_rules! ld_n (($r: ident) => 
    ({ 
        regs.$r = mem.read_byte(regs.adv()); 
        2 
    }));

    /// Load the next immediate word into the 16-bit register $r1$r2
    macro_rules! ld_nn (($r1: ident, $r2: ident) => 
    ({ 
        regs.$r2 = mem.read_byte(regs.adv()); 
        regs.$r1 = mem.read_byte(regs.adv());
        3
    }));

    /// Load the byte stored in the 16-bit register HL into the 
    /// 8-bit register $r
    macro_rules! ld_xhlm (($r: ident) => 
    ({ 
        regs.$r = mem.read_byte(regs.hl()); 
        2 
    }));

    /// Load the byte stored in register $r to memory address stored in the 
    /// 16-bit register HL
    macro_rules! ld_hlmx (($r: ident) => 
    ({ 
        mem.write_byte(regs.hl(), regs.$r); 
        2 
    }));

    /// Write the next immediate byte to memory address stored in the 16-bit
    /// register HL
    macro_rules! ld_hlm_n (() => 
    ({ 
        let pc = mem.read_byte(regs.adv());
        mem.write_byte(regs.hl(), pc);
        3 
    }));

    /// Push the 16-bit value stored in register $r1$r2 onto the stack
    macro_rules! push (($r1: ident, $r2: ident) => 
    ({
        mem.write_byte(regs.sp - 1, regs.$r1);
        mem.write_byte(regs.sp - 2, regs.$r2);
        regs.sp -= 2;
        4
    }));

    /// Pop a 16-bit value from the stack and store it in 16-bit register $r1$r2
    macro_rules! pop (($r1: ident, $r2: ident) => 
    ({
        regs.$r1 = mem.read_byte(regs.sp);
        regs.$r2 = mem.read_byte(regs.sp + 1);
        regs.sp += 2;
        3
    }));

    /// Add some value to A
    macro_rules! add_a (($r: expr) => 
    ({
        let i = regs.a;
        let j = $r;
        regs.f = if (i & 0xF) + (j & 0xF) > 0xF { H } else { 0 };
        regs.f |= if (i as u16 + j as u16) > 0xFF { C } else { 0 };
        regs.a = i + j;
        regs.f |= if regs.a != 0 { 0 } else { Z };
        1
    }));

    /// Add some value + Carry Flag to A
    macro_rules! adc_a (($r: expr) => 
    ({
        let i = regs.a;
        let j = $r;
        regs.f = if (i & 0xF) + (j & 0xF) > 0xF { H } else { 0 };
        regs.f |= if (i as u16 + j as u16) > 0xFF { C } else { 0 };
        regs.a = i + j;
        regs.f |= if regs.a != 0 { 0 } else { Z };
        1
    }));

    /// Subtract some value from A
    macro_rules! sub_a (($r: expr) => 
    ({
        let a = regs.a;
        let b = $r;
        regs.f = N | 
                if a < b { C } else { 0 } | 
                if (a & 0xF) < (b & 0xF) { H } else { 0 };
        regs.a = a - b;
        regs.f |= if regs.a != 0 { 0 } else { Z };
        1
    }));

    /// Subtract some value + Carry Flag from A
    macro_rules! sbc_a (($r: expr) => 
    ({
        let a = regs.a as u16;
        let b = $r as u16;
        let c = if regs.f & C != 0 { 1 } else { 0 };
        regs.f = N | 
                if a < b + c { C } else { 0 } | 
                if (a & 0xF) < (b & 0xF) + c { H } else { 0 };
        regs.a = (a - b - c) as u8;
        regs.f |= if regs.a != 0 { 0 } else { Z };
        1
    }));

    /// Logically AND some value with A and store the result in A
    macro_rules! and_a (($r: expr) => 
    ({
        regs.a &= $r;
        regs.f = H | if regs.a != 0 { 0 } else { Z };
        1
    }));

    /// Logically OR some value with A and store the result in A
    macro_rules! or_a (($r: expr) => 
    ({
        regs.a |= $r;
        regs.f = if regs.a != 0 { 0 } else { Z };
        1
    }));

    /// Logically XOR some value with A and store the result in A
    macro_rules! xor_a (($r: expr) => 
    ({
        regs.a ^= $r;
        regs.f = if regs.a != 0 { 0 } else { Z };
        1
    }));

    /// Compare A with some value. 
    /// Basically A - n subtraction but results are thrown away.
    macro_rules! cp_a (($r: expr) => 
    ({
        let b = $r;
        regs.f = N | 
                if regs.a == b { Z } else { 0 } |
                if regs.a < b { C } else { 0 } |
                if (regs.a & 0xF) < (b & 0xF) { H } else { 0 };
        1
    }));

    /// Increment register $r
    macro_rules! inc (($r: ident) => 
    ({
        regs.$r += 1;
        regs.f = (regs.f & C) | 
                if regs.$r == 0 { Z } else { 0 } |
                if regs.$r & 0xF == 0 { H } else { 0 };
        1
    }));

    /// Decrement register $r
    macro_rules! dec (($r: ident) => 
    ({
        regs.$r -= 1;
        regs.f &= 0x1F;
        regs.f |= N | 
                (if regs.$r == 0 { Z } else { 0 }) | 
                ((((regs.$r & 0xF) == 0xF) as u8) << 5);
        1
    }));

    /// Add some value to HL
    macro_rules! add_hl (($r: expr) => 
    ({
        let a = regs.hl() as u32;
        let b = $r as u32;
        let hl = a + b;
        regs.f = (regs.f & Z) | 
                if hl > 0xFFFF { C } else { 0 } | 
                if (a as u32 & 0xFFF) > (hl & 0xFFF) { H } else { 0 };
        regs.l = hl as u8;
        regs.h = (hl >> 8) as u8;
        2
    }));

    /// Increment the 16-bit register $r1$r2
    macro_rules! inc_word (($r1: ident, $r2: ident) => 
    ({
        regs.$r2 += 1;
        if regs.$r2 == 0 { regs.$r1 += 1; }
        2
    }));

    /// Decrement the 16-bit register $r1$r2
    macro_rules! dec_word (($r1: ident, $r2: ident) => 
    ({
        regs.$r2 -= 1;
        if regs.$r2 == 0xFF { regs.$r1 -= 1; }
        2
    }));

    /// Rotate the given register left through carry flag
    macro_rules! rl (($r: expr, $cy: expr) => 
    ({
        let ci = if (regs.f & C) != 0 { 1 } else { 0 };
        let co = $r & 0x80;
        $r = ($r << 1) | ci;
        regs.f = if co != 0 { C } else { 0 };
        $cy
    }));

    /// Rotate the given register Left. Old bit 7 goes to Carry Flag
    macro_rules! rlc (($r: expr, $cy: expr) => 
    ({
        let ci = if ($r & 0x80) != 0 { 1 } else { 0 };
        $r = ($r << 1) | ci;
        regs.f = if ci != 0 { C } else { 0 };
        $cy
    }));
    
    /// Rotate the given register right through Carry Flag
    macro_rules! rr (($r: expr, $cy: expr) => 
    ({
        let ci = if (regs.f & C) != 0 { 0x80 } else { 0 };
        let co = if ($r & 0x01) != 0 { C } else { 0 };
        $r = ($r >> 1) | ci;
        regs.f = co;
        $cy
    }));

    /// Rotate the given register right. Old bit 0 goes to Carry Flag.
    macro_rules! rrc (($r: expr, $cy: expr) => 
    ({
        let ci = $r & 0x01;
        $r = ($r >> 1) | (ci << 7);
        regs.f = if ci != 0 { C } else { 0 };
        $cy
    }));

    /// Jump to the next two byte immediate value
    macro_rules! jp (() => 
    ({
        regs.pc = mem.read_word(regs.pc);
        4 
    }));

    /// Jump to next immediate byte if expression $e is true
    macro_rules! jp_n (($e: expr) => 
    ({
        if $e { jp!() } else { regs.pc += 2; 3 }
    }));

    /// Add n to current address and then jump to it
    macro_rules! jr (() => 
    ({ 
        let v = mem.read_byte(regs.adv());
        regs.pc = add(regs.pc, v);
        3 
    }));

    /// Add n to current address and then jump to it if $e is true
    macro_rules! jr_n (($e: expr) => 
    ({
        if $e { jr!() } else { regs.adv(); 2 }
    }));

    /// Push addr of next instruction onto stack and then jump to address nn
    macro_rules! call (() => 
    ({
        regs.sp -= 2;
        mem.write_word(regs.sp, regs.pc + 2);
        regs.pc = mem.read_word(regs.pc);
        6 
    }));

    /// Call address n if $e is true
    macro_rules! call_if (($e: expr) => 
    ({
        if $e { call!() } else { regs.pc += 2; 3 }
    }));

    /// Push present address onto stack then jump to address $0000 + n.
    macro_rules! rst (($e: expr) => 
    ({
        regs.rst($e, mem);
        4
    }));

    /// Pop two bytes from stack & jump to that address if $e is true
    macro_rules! ret_if (($e: expr) => 
    ({
        if $e { regs.ret(mem); 5 } else { 2 }
    }));

    // Match the given opcode to an instruction
    match opcode
    {
        0x00 => 1,                                                  // NOP

        // If opcode is 0xCB we should execute a 0xCB instruction using the
        // next immediate opcode read from memory
        0xCB => exec_cb(mem.read_byte(regs.adv()), cpu, mem),

        // ----------------------------- 8-bit Loads ---------------------------

        0x06 => ld_n!(b),                                           // LD B, n
        0x0E => ld_n!(c),                                           // LD C, n
        0x16 => ld_n!(d),                                           // LD D, n
        0x1E => ld_n!(e),                                           // LD E, n
        0x26 => ld_n!(h),                                           // LD H, n
        0x2E => ld_n!(l),                                           // LD L, n
        0x7F => ld!(a, a),                                          // LD A, A
        0x78 => ld!(a, b),                                          // LD A, B
        0x79 => ld!(a, c),                                          // LD A, C
        0x7A => ld!(a, d),                                          // LD A, D
        0x7B => ld!(a, e),                                          // LD A, E
        0x7C => ld!(a, h),                                          // LD A, H
        0x7D => ld!(a, l),                                          // LD A, L
        0x7E => ld_xhlm!(a),                                        // LD A, (HL)
        0x40 => ld!(b, b),                                          // LD B, B
        0x41 => ld!(b, c),                                          // LD B, C
        0x42 => ld!(b, d),                                          // LD B, D
        0x43 => ld!(b, e),                                          // LD B, E
        0x44 => ld!(b, h),                                          // LD B, H
        0x45 => ld!(b, l),                                          // LD B, L
        0x46 => ld_xhlm!(b),                                        // LD B, (HL)
        0x48 => ld!(c, b),                                          // LD C, B
        0x49 => ld!(c, c),                                          // LD C, C
        0x4A => ld!(c, d),                                          // LD C, D
        0x4B => ld!(c, e),                                          // LD C, E
        0x4C => ld!(c, h),                                          // LD C, H
        0x4D => ld!(c, l),                                          // LD C, L
        0x4E => ld_xhlm!(c),                                        // LD C, (HL)
        0x50 => ld!(d, b),                                          // LD D, B
        0x51 => ld!(d, c),                                          // LD D, C
        0x52 => ld!(d, d),                                          // LD D, D
        0x53 => ld!(d, e),                                          // LD D, E
        0x54 => ld!(d, h),                                          // LD D, H
        0x55 => ld!(d, l),                                          // LD D, L
        0x56 => ld_xhlm!(d),                                        // LD D, (HL)
        0x58 => ld!(e, b),                                          // LD E, B
        0x59 => ld!(e, c),                                          // LD E, C
        0x5A => ld!(e, d),                                          // LD E, D
        0x5B => ld!(e, e),                                          // LD E, E
        0x5C => ld!(e, h),                                          // LD E, H
        0x5D => ld!(e, l),                                          // LD E, L
        0x5E => ld_xhlm!(e),                                        // LD E, (HL)
        0x60 => ld!(h, b),                                          // LD H, B
        0x61 => ld!(h, c),                                          // LD H, C
        0x62 => ld!(h, d),                                          // LD H, D
        0x63 => ld!(h, e),                                          // LD H, E
        0x64 => ld!(h, h),                                          // LD H, H
        0x65 => ld!(h, l),                                          // LD H, L
        0x66 => ld_xhlm!(h),                                        // LD H, (HL)
        0x68 => ld!(l, b),                                          // LD L, B
        0x69 => ld!(l, c),                                          // LD L, C
        0x6A => ld!(l, d),                                          // LD L, D
        0x6B => ld!(l, e),                                          // LD L, E
        0x6C => ld!(l, h),                                          // LD L, H
        0x6D => ld!(l, l),                                          // LD L, L
        0x6E => ld_xhlm!(l),                                        // LD L, (HL)
        0x70 => ld_hlmx!(b),                                        // LD (HL), B
        0x71 => ld_hlmx!(c),                                        // LD (HL), C
        0x72 => ld_hlmx!(d),                                        // LD (HL), D
        0x73 => ld_hlmx!(e),                                        // LD (HL), E
        0x74 => ld_hlmx!(h),                                        // LD (HL), H
        0x75 => ld_hlmx!(l),                                        // LD (HL), L
        0x36 => ld_hlm_n!(),                                        // LD (HL), n
        0x0A => { regs.a = mem.read_byte(regs.bc()); 2 },           // LD A, (BC)
        0x1A => { regs.a = mem.read_byte(regs.de()); 2 },           // LD A, (DE)
        0xFA => {                                                   // LD A, (nn)
            regs.a = mem.read_byte(mem.read_word(regs.pc)); 
            regs.pc += 2; 
            4 
        },
        0x3E => ld_n!(a),                                           // LD A, n
        0x47 => ld!(b, a),                                          // LD B, A
        0x4F => ld!(c, a),                                          // LD C, A
        0x57 => ld!(d, a),                                          // LD D, A
        0x5F => ld!(e, a),                                          // LD E, A
        0x67 => ld!(h, a),                                          // LD H, A
        0x6F => ld!(l, a),                                          // LD L, A
        0x02 => { mem.write_byte(regs.bc(), regs.a); 2 },           // LD (BC), A
        0x12 => { mem.write_byte(regs.de(), regs.a); 2 },           // LD (DE), A
        0x77 => { mem.write_byte(regs.hl(), regs.a); 2 },           // LD (HL), A
        0xEA => {                                                   // LD (nn), A
            let n = mem.read_word(regs.pc); 
            mem.write_byte(n, regs.a); 
            regs.pc += 2; 
            4
        },
        0xF2 => { regs.a = mem.read_byte(0xFF00 | (regs.c as u16)); 2 },    // LD A, (C)
        0xE2 => { mem.write_byte(0xFF00 | (regs.c as u16), regs.a); 2 },    // LD (C), A
        0x3A => { regs.a = mem.read_byte(regs.hl()); regs.hlmm(); 2 },      // LDD A, (HL)
        0x32 => { mem.write_byte(regs.hl(), regs.a); regs.hlmm(); 2 },      // LDD (HL), A
        0x2A => { regs.a = mem.read_byte(regs.hl()); regs.hlpp(); 2 },      // LDI A, (HL)
        0x22 => { mem.write_byte(regs.hl(), regs.a); regs.hlpp(); 2 },      // LDI (HL), A
        0xE0 => { ld_ioan(regs, mem); 3 },                                  // LDH (n), A
        0xF0 => {                                                           // LDH A, n
            regs.a = mem.read_byte(0xFF00 | (mem.read_byte(regs.adv()) as u16)); 
            3 
        }

        // ---------------------------- 16-bit Loads ---------------------------

        0x01 => ld_nn!(b, c),                                       // LD BC, nn
        0x11 => ld_nn!(d, e),                                       // LD DE, nn
        0x21 => ld_nn!(h, l),                                       // LD HL, nn
        0x31 => {                                                   // LD SP, nn
            regs.sp = mem.read_word(regs.pc); 
            regs.pc += 2; 
            3 
        },
        0xF9 => { regs.sp = regs.hl(); 2 },                         // LD SP, HL
        0xF8 => { ld_hlspn(regs, mem); 3 },                         // LDHL SP, n
        0x08 => {                                                   // LD (nn), SP
            let a = mem.read_word(regs.pc); 
            mem.write_word(a, regs.sp); 
            regs.pc += 2; 
            5 
        },
        0xF5 => push!(a, f),                                        // PUSH AF
        0xC5 => push!(b, c),                                        // PUSH BC
        0xD5 => push!(d, e),                                        // PUSH DE
        0xE5 => push!(h, l),                                        // PUSH HL
        0xF1 => { pop_af(regs, mem); 3 },                           // POP AF
        0xC1 => pop!(b, c),                                         // POP BC
        0xD1 => pop!(d, e),                                         // POP DE
        0xE1 => pop!(h, l),                                         // POP HL

        // ----------------------------- 8-bit ALU -----------------------------

        0x87 => add_a!(regs.a),                                     // ADD A
        0x80 => add_a!(regs.b),                                     // ADD B
        0x81 => add_a!(regs.c),                                     // ADD C
        0x82 => add_a!(regs.d),                                     // ADD D
        0x83 => add_a!(regs.e),                                     // ADD E
        0x84 => add_a!(regs.h),                                     // ADD H
        0x85 => add_a!(regs.l),                                     // ADD L
        0x86 => { add_a!(mem.read_byte(regs.hl())); 2 },            // ADD A, (HL)
        0xC6 => { add_a!(mem.read_byte(regs.adv())); 2 },           // ADD A, #
        0x8F => adc_a!(regs.a),                                     // ADC A, A
        0x88 => adc_a!(regs.b),                                     // ADC A, B
        0x89 => adc_a!(regs.c),                                     // ADC A, C
        0x8A => adc_a!(regs.d),                                     // ADC A, D
        0x8B => adc_a!(regs.e),                                     // ADC A, E
        0x8C => adc_a!(regs.h),                                     // ADC A, H
        0x8D => adc_a!(regs.l),                                     // ADC A, L
        0x8E => { adc_a!(mem.read_byte(regs.hl())); 2 },            // ADC A, (HL)
        0xCE => { adc_a!(mem.read_byte(regs.adv())); 2 },           // ADC A, #
        0x97 => sub_a!(regs.a),                                     // SUB A
        0x90 => sub_a!(regs.b),                                     // SUB B
        0x91 => sub_a!(regs.c),                                     // SUB C
        0x92 => sub_a!(regs.d),                                     // SUB D
        0x93 => sub_a!(regs.e),                                     // SUB E
        0x94 => sub_a!(regs.h),                                     // SUB H
        0x95 => sub_a!(regs.l),                                     // SUB L
        0x96 => { sub_a!(mem.read_byte(regs.hl())); 2 },            // SUB (HL)
        0xD6 => { sub_a!(mem.read_byte(regs.adv())); 2 },           // SUB #
        0x9F => sbc_a!(regs.a),                                     // SBC A, A
        0x98 => sbc_a!(regs.a),                                     // SBC A, B
        0x99 => sbc_a!(regs.a),                                     // SBC A, C
        0x9A => sbc_a!(regs.a),                                     // SBC A, D
        0x9B => sbc_a!(regs.a),                                     // SBC A, E
        0x9C => sbc_a!(regs.a),                                     // SBC A, H
        0x9D => sbc_a!(regs.a),                                     // SBC A, L
        0x9E => { sbc_a!(mem.read_byte(regs.hl())); 2 },            // SBC A, (HL)
        0xDE => { sbc_a!(mem.read_byte(regs.adv())); 2 },           // SBC A, #
        0xA7 => and_a!(regs.a),                                     // AND A
        0xA0 => and_a!(regs.b),                                     // AND B
        0xA1 => and_a!(regs.c),                                     // AND C
        0xA2 => and_a!(regs.d),                                     // AND D
        0xA3 => and_a!(regs.e),                                     // AND E
        0xA4 => and_a!(regs.h),                                     // AND H
        0xA5 => and_a!(regs.l),                                     // AND L
        0xA6 => { and_a!(mem.read_byte(regs.hl())); 2 },            // AND (HL)
        0xE6 => { and_a!(mem.read_byte(regs.adv())); 2 },           // AND #
        0xB7 => or_a!(regs.a),                                      // OR A
        0xB0 => or_a!(regs.b),                                      // OR B
        0xB1 => or_a!(regs.c),                                      // OR C
        0xB2 => or_a!(regs.d),                                      // OR D
        0xB3 => or_a!(regs.e),                                      // OR E
        0xB4 => or_a!(regs.h),                                      // OR H
        0xB5 => or_a!(regs.l),                                      // OR L
        0xB6 => { or_a!(mem.read_byte(regs.hl())); 2 },             // OR (HL)
        0xF6 => { or_a!(mem.read_byte(regs.adv())); 2 },            // OR #
        0xAF => xor_a!(regs.a),                                     // XOR A
        0xA8 => xor_a!(regs.b),                                     // XOR B
        0xA9 => xor_a!(regs.c),                                     // XOR C
        0xAA => xor_a!(regs.d),                                     // XOR D
        0xAB => xor_a!(regs.e),                                     // XOR E
        0xAC => xor_a!(regs.h),                                     // XOR H
        0xAD => xor_a!(regs.l),                                     // XOR L
        0xAE => { xor_a!(mem.read_byte(regs.hl())); 2 },            // XOR (HL)
        0xEE => { xor_a!(mem.read_byte(regs.adv())); 2 },           // XOR #
        0xBF => cp_a!(regs.a),                                      // CP A
        0xB8 => cp_a!(regs.b),                                      // CP B
        0xB9 => cp_a!(regs.c),                                      // CP C
        0xBA => cp_a!(regs.d),                                      // CP D
        0xBB => cp_a!(regs.e),                                      // CP E
        0xBC => cp_a!(regs.h),                                      // CP H
        0xBD => cp_a!(regs.l),                                      // CP L
        0xBE => { cp_a!(mem.read_byte(regs.hl())); 2 },             // CP (HL)
        0xFE => { cp_a!(mem.read_byte(regs.adv())); 2 },            // CP #
        0x3C => inc!(a),                                            // INC A
        0x04 => inc!(b),                                            // INC B
        0x0C => inc!(c),                                            // INC C
        0x14 => inc!(d),                                            // INC D
        0x1C => inc!(e),                                            // INC E
        0x24 => inc!(h),                                            // INC H
        0x2C => inc!(l),                                            // INC L
        0x34 => { inc_hlm(regs, mem); 3 },                          // INC (HL)
        0x3D => dec!(a),                                            // DEC A
        0x05 => dec!(b),                                            // DEC B
        0x0D => dec!(c),                                            // DEC C
        0x15 => dec!(d),                                            // DEC D
        0x1D => dec!(e),                                            // DEC E
        0x25 => dec!(h),                                            // DEC H
        0x2D => dec!(l),                                            // DEC L
        0x35 => { dec_hlm(regs, mem); 3 },                          // DEC (HL)

        // ---------------------------- 16-bit Arithmetic ----------------------

        0x09 => add_hl!(regs.bc()),                                 // ADD HL, BC
        0x19 => add_hl!(regs.de()),                                 // ADD HL, DE
        0x29 => add_hl!(regs.hl()),                                 // ADD HL, HL
        0x39 => { add_hlsp(regs); 2 },                         // ADD HL, SP
        0xE8 => { add_spn(regs, mem); 4 },                          // ADD SP, #
        0x03 => inc_word!(b, c),                                    // INC BC
        0x13 => inc_word!(d, e),                                    // INC DE
        0x23 => inc_word!(h, l),                                    // INC HL
        0x33 => { regs.sp += 1; 2 },                                // INC SP
        0x0B => dec_word!(b, c),                                    // DEC BC
        0x1B => dec_word!(d, e),                                    // DEC DE
        0x2B => dec_word!(h, l),                                    // DEC HL
        0x3B => { regs.sp -= 1; 2 },                                // DEC SP

        // ----------------------------- Miscellaneous -------------------------

        0x27 => { daa(regs); 1 },                                   // DAA
        0x2F => { regs.a ^= 0xFF; regs.f |= N | H; 1 },             // CPL
        0x3F => { regs.f = (regs.f & Z) | ((regs.f & C) ^ C); 1 },  // CCF
        0x37 => { regs.f = (regs.f & Z) | C; 1 },                   // SCF
        0x76 => { regs.halt = 1; 1 },                               // HALT
        0x10 => { regs.stop = 1; 1 },                               // STOP
        0xF3 => { regs.di(); 1 },                                   // DI
        0xFB => { regs.ei(mem); 1 },                                // EI

        // -------------------------- Rotates & Shifts -------------------------

        0x07 => rlc!(regs.a, 1),                                    // RLCA
        0x17 => rl!(regs.a, 1),                                     // RLA
        0x0F => rrc!(regs.a, 1),                                    // RRCA
        0x1F => rr!(regs.a, 1),                                     // RRA

        // ------------------------------- Jumps -------------------------------

        0xC3 => jp!(),                                              // JP nn
        0xC2 => jp_n!((regs.f & Z) == 0),                           // JP NZ, nn
        0xCA => jp_n!((regs.f & Z) != 0),                           // JP Z, nn
        0xD2 => jp_n!((regs.f & C) == 0),                           // JP NC, nn
        0xDA => jp_n!((regs.f & C) != 0),                           // JP C, nn
        0xE9 => { regs.pc = regs.hl(); 1 },                         // JP (HL)
        0x18 => jr!(),                                              // JR n
        0x20 => jr_n!((regs.f & Z) == 0),                           // JR NZ, n
        0x28 => jr_n!((regs.f & Z) != 0),                           // JR Z, n
        0x30 => jr_n!((regs.f & C) == 0),                           // JR NC, n
        0x38 => jr_n!((regs.f & C) != 0),                           // JR C, n

        // ------------------------------- Calls -------------------------------

        0xCD => call!(),                                            // CALL nn
        0xC4 => call_if!((regs.f & Z) == 0),                        // CALL NZ, n
        0xCC => call_if!((regs.f & Z) != 0),                        // CALL Z, n
        0xD4 => call_if!((regs.f & C) == 0),                        // CALL NC, n
        0xDC => call_if!((regs.f & C) != 0),                        // CALL C, n

        // ----------------------------- Restarts ------------------------------

        0xC7 => rst!(0x00),                                         // RST 00
        0xCF => rst!(0x08),                                         // RST 08
        0xD7 => rst!(0x10),                                         // RST 10
        0xDF => rst!(0x18),                                         // RST 18
        0xE7 => rst!(0x20),                                         // RST 20
        0xEF => rst!(0x28),                                         // RST 28
        0xF7 => rst!(0x30),                                         // RST 30
        0xFF => rst!(0x38),                                         // RST 38

        // ----------------------------- Returns -------------------------------

        0xC9 => { regs.ret(mem); 4 },                               // RET
        0xC0 => ret_if!((regs.f & Z) == 0),                         // RET NZ
        0xC8 => ret_if!((regs.f & Z) != 0),                         // RET Z
        0xD0 => ret_if!((regs.f & C) == 0),                         // RET NC
        0xD8 => ret_if!((regs.f & C) == 0),                         // RET C
        0xD9 => { regs.ei(mem); regs.ret(mem); 4 },                 // RETI

        _ => xx()
    }
}

/// Execute an instruction for an opcode prefixed by 0xCB and return number of
/// cycles it took to execute
fn exec_cb(opcode: u8, cpu: &mut CPU, mem: &mut Memory) -> u32
{
    // TODO: comments/rustdocs for everything in this function

    let mut regs = &mut cpu.regs;

    /// Swap the upper and lower nibbles of the given value
    macro_rules! swap (($e: expr) => 
    ({
        $e = ($e << 4) | (($e & 0xF0) >> 4);
        regs.f = if $e != 0 { 0 } else { Z };
        2 as u32
    }));

    macro_rules! rl (($r: expr, $cy: expr) =>
    ({
        let ci = if (regs.f & C) != 0 { 1 } else { 0 };
        let co = $r & 0x80;
        $r = ($r << 1) | ci;
        regs.f = if $r != 0 { 0 } else { Z } | if co != 0 { C } else { 0 };
        $cy as u32
    }));

    macro_rules! rlc (($r: expr, $cy: expr) =>
    ({
        let ci = if ($r & 0x80) != 0 { 1 } else { 0 };
        $r = ($r << 1) | ci;
        regs.f = if $r != 0 { 0 } else { Z } | if ci != 0 { C } else { 0 };
        $cy as u32
    }));

    macro_rules! rr (($r: expr, $cy: expr) =>
    ({
        let ci = if (regs.f & C) != 0 { 0x80 } else { 0 };
        let co = if ($r & 0x01) != 0 { C } else { 0 };
        $r = ($r >> 1) | ci;
        regs.f = if $r != 0 { 0 } else { Z } | co;
        $cy as u32
    }));

    macro_rules! rrc (($r: expr, $cy: expr) =>
    ({
        let ci = $r & 0x01;
        $r = ($r >> 1) | (ci << 7);
        regs.f = if $r != 0 { 0 } else { Z } | if ci != 0 { C } else { 0 };
        $cy as u32
    }));

    macro_rules! sra (($e: expr, $cy: expr) => 
    ({
        let co = $e & 1;
        $e = (($e as i8) >> 1) as u8;
        regs.f = if $e != 0 { 0 } else { Z } | if co != 0 { C } else { 0 };
        $cy as u32
    }));

    macro_rules! srl (($e: expr, $cy: expr) => 
    ({
        let co = ($e >> 7) & 1;
        $e = $e << 1;
        regs.f = if $e != 0 { 0 } else { Z } | if co != 0 { C } else { 0 };
        $cy as u32
    }));

    macro_rules! sla (($e: expr, $cy: expr) => 
    ({
        let co = ($e >> 7) & 1;
        $e = $e << 1;
        regs.f = if $e != 0 { 0 } else { Z } | if co != 0 { C } else { 0 };
        $cy as u32
    }));

    macro_rules! bit (($e: expr, $bit: expr) => 
    ({
        regs.f = (regs.f & C) | H | if $e & (1 << $bit) != 0 { 0 } else { Z };
        2 as u32
    }));

    macro_rules! set (($i: ident, $bit: expr) => 
    ({
        regs.$i |= 1 << $bit;
        2 as u32
    }));

    macro_rules! res (($i: ident, $bit: expr) => 
    ({
        regs.$i &= !1 << $bit;
        2 as u32
    }));

    macro_rules! hlm (($i: ident, $e: expr) => 
    ({
        let mut $i = mem.read_byte(regs.hl());
        $e;
        mem.write_byte(regs.hl(), $i);
    }));

    macro_rules! hlfrob (($i: ident, $e: expr) => 
    ({
        let $i = mem.read_byte(regs.hl());
        mem.write_byte(regs.hl(), $e);
    }));

    // Match the opcode to the correct instruction
    match opcode
    {
        0x37 => swap!(regs.a),                                      // SWAP A
        0x30 => swap!(regs.b),                                      // SWAP B
        0x31 => swap!(regs.c),                                      // SWAP C
        0x32 => swap!(regs.d),                                      // SWAP D
        0x33 => swap!(regs.e),                                      // SWAP E
        0x34 => swap!(regs.h),                                      // SWAP H
        0x35 => swap!(regs.l),                                      // SWAP L
        0x36 => { hlm!(hl, swap!(hl)); 4 },                         // SWAP (HL)

        0x07 => rlc!(regs.a, 2),                                    // RLC A
        0x00 => rlc!(regs.b, 2),                                    // RLC B
        0x01 => rlc!(regs.c, 2),                                    // RLC C
        0x02 => rlc!(regs.d, 2),                                    // RLC D
        0x03 => rlc!(regs.e, 2),                                    // RLC E
        0x04 => rlc!(regs.h, 2),                                    // RLC H
        0x05 => rlc!(regs.l, 2),                                    // RLC L
        0x06 => { hlm!(hl, rlc!(hl, 1)); 4 },                       // RLC (HL)
        0x17 => rl!(regs.a, 2),                                     // RL A
        0x10 => rl!(regs.b, 2),                                     // RL B
        0x11 => rl!(regs.c, 2),                                     // RL C
        0x12 => rl!(regs.d, 2),                                     // RL D
        0x13 => rl!(regs.e, 2),                                     // RL E
        0x14 => rl!(regs.h, 2),                                     // RL H
        0x15 => rl!(regs.l, 2),                                     // RL L
        0x16 => { hlm!(hl, rl!(hl, 1)); 4 },                        // RL (HL)
        0x0F => rrc!(regs.a, 2),                                    // RRC A
        0x08 => rrc!(regs.b, 2),                                    // RRC B
        0x09 => rrc!(regs.c, 2),                                    // RRC C
        0x0A => rrc!(regs.d, 2),                                    // RRC D
        0x0B => rrc!(regs.e, 2),                                    // RRC E
        0x0C => rrc!(regs.h, 2),                                    // RRC H
        0x0D => rrc!(regs.l, 2),                                    // RRC L
        0x0E => { hlm!(hl, rrc!(hl, 1)); 4 },                       // RRC (HL)
        0x1F => rr!(regs.a, 2),                                     // RR A
        0x18 => rr!(regs.b, 2),                                     // RR B
        0x19 => rr!(regs.c, 2),                                     // RR C
        0x1A => rr!(regs.d, 2),                                     // RR D
        0x1B => rr!(regs.e, 2),                                     // RR E
        0x1C => rr!(regs.h, 2),                                     // RR H
        0x1D => rr!(regs.l, 2),                                     // RR L
        0x1E => { hlm!(hl, rr!(hl, 1)); 4 },                        // RR (HL)
        0x27 => sla!(regs.a, 2),                                    // SLA A
        0x20 => sla!(regs.b, 2),                                    // SLA B
        0x21 => sla!(regs.c, 2),                                    // SLA C
        0x22 => sla!(regs.d, 2),                                    // SLA D
        0x23 => sla!(regs.e, 2),                                    // SLA E
        0x24 => sla!(regs.h, 2),                                    // SLA H
        0x25 => sla!(regs.l, 2),                                    // SLA L
        0x26 => { hlm!(hl, sla!(hl, 1)); 4 },                       // SLA (HL)
        0x2F => sra!(regs.a, 2),                                    // SRA A
        0x28 => sra!(regs.b, 2),                                    // SRA B
        0x29 => sra!(regs.c, 2),                                    // SRA C
        0x2A => sra!(regs.d, 2),                                    // SRA D
        0x2B => sra!(regs.e, 2),                                    // SRA E
        0x2C => sra!(regs.h, 2),                                    // SRA H
        0x2D => sra!(regs.l, 2),                                    // SRA L
        0x2E => { hlm!(hl, sra!(hl, 1)); 4 },                       // SRA (HL)
        0x3F => srl!(regs.a, 2),                                    // SRL A
        0x38 => srl!(regs.b, 2),                                    // SRL B
        0x39 => srl!(regs.c, 2),                                    // SRL C
        0x3A => srl!(regs.d, 2),                                    // SRL D
        0x3B => srl!(regs.e, 2),                                    // SRL E
        0x3C => srl!(regs.h, 2),                                    // SRL H
        0x3D => srl!(regs.l, 2),                                    // SRL L
        0x3E => { hlm!(hl, srl!(hl, 1)); 4 },                       // SRL (HL)

        0x47 => bit!(regs.a, 0),                                    // BIT 0A
        0x40 => bit!(regs.b, 0),                                    // BIT 0B
        0x41 => bit!(regs.c, 0),                                    // BIT 0C
        0x42 => bit!(regs.d, 0),                                    // BIT 0D
        0x43 => bit!(regs.e, 0),                                    // BIT 0E
        0x44 => bit!(regs.h, 0),                                    // BIT 0H
        0x45 => bit!(regs.l, 0),                                    // BIT 0L
        0x46 => { bit!(mem.read_byte(regs.hl()), 0); 3 },           // BIT 0HLm
        0x4F => bit!(regs.a, 1),                                    // BIT 1A
        0x48 => bit!(regs.b, 1),                                    // BIT 1B
        0x49 => bit!(regs.c, 1),                                    // BIT 1C
        0x4A => bit!(regs.d, 1),                                    // BIT 1D
        0x4B => bit!(regs.e, 1),                                    // BIT 1E
        0x4C => bit!(regs.h, 1),                                    // BIT 1H
        0x4D => bit!(regs.l, 1),                                    // BIT 1L
        0x4E => { bit!(mem.read_byte(regs.hl()), 1); 3 },           // BIT 1HLm
        0x57 => bit!(regs.a, 2),                                    // BIT 2A
        0x50 => bit!(regs.b, 2),                                    // BIT 2B
        0x51 => bit!(regs.c, 2),                                    // BIT 2C
        0x52 => bit!(regs.d, 2),                                    // BIT 2D
        0x53 => bit!(regs.e, 2),                                    // BIT 2E
        0x54 => bit!(regs.h, 2),                                    // BIT 2H
        0x55 => bit!(regs.l, 2),                                    // BIT 2L
        0x56 => { bit!(mem.read_byte(regs.hl()), 2); 3 },           // BIT 2HLm
        0x5F => bit!(regs.a, 3),                                    // BIT 3A
        0x58 => bit!(regs.b, 3),                                    // BIT 3B
        0x59 => bit!(regs.c, 3),                                    // BIT 3C
        0x5A => bit!(regs.d, 3),                                    // BIT 3D
        0x5B => bit!(regs.e, 3),                                    // BIT 3E
        0x5C => bit!(regs.h, 3),                                    // BIT 3H
        0x5D => bit!(regs.l, 3),                                    // BIT 3L
        0x5E => { bit!(mem.read_byte(regs.hl()), 3); 3 },           // BIT 3HLm
        0x67 => bit!(regs.a, 4),                                    // BIT 4A
        0x60 => bit!(regs.b, 4),                                    // BIT 4B
        0x61 => bit!(regs.c, 4),                                    // BIT 4C
        0x62 => bit!(regs.d, 4),                                    // BIT 4D
        0x63 => bit!(regs.e, 4),                                    // BIT 4E
        0x64 => bit!(regs.h, 4),                                    // BIT 4H
        0x65 => bit!(regs.l, 4),                                    // BIT 4L
        0x66 => { bit!(mem.read_byte(regs.hl()), 4); 3 },           // BIT 4HLm
        0x6F => bit!(regs.a, 5),                                    // BIT 5A
        0x68 => bit!(regs.b, 5),                                    // BIT 5B
        0x69 => bit!(regs.c, 5),                                    // BIT 5C
        0x6A => bit!(regs.d, 5),                                    // BIT 5D
        0x6B => bit!(regs.e, 5),                                    // BIT 5E
        0x6C => bit!(regs.h, 5),                                    // BIT 5H
        0x6D => bit!(regs.l, 5),                                    // BIT 5L
        0x6E => { bit!(mem.read_byte(regs.hl()), 5); 3 },           // BIT 5HLm
        0x77 => bit!(regs.a, 6),                                    // BIT 6A
        0x70 => bit!(regs.b, 6),                                    // BIT 6B
        0x71 => bit!(regs.c, 6),                                    // BIT 6C
        0x72 => bit!(regs.d, 6),                                    // BIT 6D
        0x73 => bit!(regs.e, 6),                                    // BIT 6E
        0x74 => bit!(regs.h, 6),                                    // BIT 6H
        0x75 => bit!(regs.l, 6),                                    // BIT 6L
        0x76 => { bit!(mem.read_byte(regs.hl()), 6); 3 },           // BIT 6HLm
        0x7F => bit!(regs.a, 7),                                    // BIT 7A
        0x78 => bit!(regs.b, 7),                                    // BIT 7B
        0x79 => bit!(regs.c, 7),                                    // BIT 7C
        0x7A => bit!(regs.d, 7),                                    // BIT 7D
        0x7B => bit!(regs.e, 7),                                    // BIT 7E
        0x7C => bit!(regs.h, 7),                                    // BIT 7H
        0x7D => bit!(regs.l, 7),                                    // BIT 7L
        0x7E => { bit!(mem.read_byte(regs.hl()), 7); 3 },           // BIT 7HLm

        0xC7 => set!(a, 0),                                         // SET 0A
        0xC0 => set!(b, 0),                                         // SET 0B
        0xC1 => set!(c, 0),                                         // SET 0C
        0xC2 => set!(d, 0),                                         // SET 0D
        0xC3 => set!(e, 0),                                         // SET 0E
        0xC4 => set!(h, 0),                                         // SET 0H
        0xC5 => set!(l, 0),                                         // SET 0L
        0xC6 => { hlfrob!(hl, hl | (1 << 0)); 4 },                  // SET 0HLm
        0xCF => set!(a, 1),                                         // SET 1A
        0xC8 => set!(b, 1),                                         // SET 1B
        0xC9 => set!(c, 1),                                         // SET 1C
        0xCA => set!(d, 1),                                         // SET 1D
        0xCB => set!(e, 1),                                         // SET 1E
        0xCC => set!(h, 1),                                         // SET 1H
        0xCD => set!(l, 1),                                         // SET 1L
        0xCE => { hlfrob!(hl, hl | (1 << 1)); 4 },                  // SET 1HLm
        0xD7 => set!(a, 2),                                         // SET 2A
        0xD0 => set!(b, 2),                                         // SET 2B
        0xD1 => set!(c, 2),                                         // SET 2C
        0xD2 => set!(d, 2),                                         // SET 2D
        0xD3 => set!(e, 2),                                         // SET 2E
        0xD4 => set!(h, 2),                                         // SET 2H
        0xD5 => set!(l, 2),                                         // SET 2L
        0xD6 => { hlfrob!(hl, hl | (1 << 2)); 4 },                  // SET 2HLm
        0xDF => set!(a, 3),                                         // SET 3A
        0xD8 => set!(b, 3),                                         // SET 3B
        0xD9 => set!(c, 3),                                         // SET 3C
        0xDA => set!(d, 3),                                         // SET 3D
        0xDB => set!(e, 3),                                         // SET 3E
        0xDC => set!(h, 3),                                         // SET 3H
        0xDD => set!(l, 3),                                         // SET 3L
        0xDE => { hlfrob!(hl, hl | (1 << 3)); 4 },                  // SET 3HLm
        0xE7 => set!(a, 4),                                         // SET 4A
        0xE0 => set!(b, 4),                                         // SET 4B
        0xE1 => set!(c, 4),                                         // SET 4C
        0xE2 => set!(d, 4),                                         // SET 4D
        0xE3 => set!(e, 4),                                         // SET 4E
        0xE4 => set!(h, 4),                                         // SET 4H
        0xE5 => set!(l, 4),                                         // SET 4L
        0xE6 => { hlfrob!(hl, hl | (1 << 4)); 4 },                  // SET 4HLm
        0xEF => set!(a, 5),                                         // SET 5A
        0xE8 => set!(b, 5),                                         // SET 5B
        0xE9 => set!(c, 5),                                         // SET 5C
        0xEA => set!(d, 5),                                         // SET 5D
        0xEB => set!(e, 5),                                         // SET 5E
        0xEC => set!(h, 5),                                         // SET 5H
        0xED => set!(l, 5),                                         // SET 5L
        0xEE => { hlfrob!(hl, hl | (1 << 5)); 4 },                  // SET 5HLm
        0xF7 => set!(a, 6),                                         // SET 6A
        0xF0 => set!(b, 6),                                         // SET 6B
        0xF1 => set!(c, 6),                                         // SET 6C
        0xF2 => set!(d, 6),                                         // SET 6D
        0xF3 => set!(e, 6),                                         // SET 6E
        0xF4 => set!(h, 6),                                         // SET 6H
        0xF5 => set!(l, 6),                                         // SET 6L
        0xF6 => { hlfrob!(hl, hl | (1 << 6)); 4 },                  // SET 6HLm
        0xFF => set!(a, 7),                                         // SET 7A
        0xF8 => set!(b, 7),                                         // SET 7B
        0xF9 => set!(c, 7),                                         // SET 7C
        0xFA => set!(d, 7),                                         // SET 7D
        0xFB => set!(e, 7),                                         // SET 7E
        0xFC => set!(h, 7),                                         // SET 7H
        0xFD => set!(l, 7),                                         // SET 7L
        0xFE => { hlfrob!(hl, hl | (1 << 7)); 4 },                  // SET 7HLm

        0x87 => res!(a, 0),                                         // RES 0A
        0x80 => res!(b, 0),                                         // RES 0B
        0x81 => res!(c, 0),                                         // RES 0C
        0x82 => res!(d, 0),                                         // RES 0D
        0x83 => res!(e, 0),                                         // RES 0E
        0x84 => res!(h, 0),                                         // RES 0H
        0x85 => res!(l, 0),                                         // RES 0L
        0x86 => { hlfrob!(hl, hl & !(1 << 0)); 4 },                 // RES 0HLm
        0x8F => res!(a, 1),                                         // RES 1A
        0x88 => res!(b, 1),                                         // RES 1B
        0x89 => res!(c, 1),                                         // RES 1C
        0x8A => res!(d, 1),                                         // RES 1D
        0x8B => res!(e, 1),                                         // RES 1E
        0x8C => res!(h, 1),                                         // RES 1H
        0x8D => res!(l, 1),                                         // RES 1L
        0x8E => { hlfrob!(hl, hl & !(1 << 1)); 4 },                 // RES 1HLm
        0x97 => res!(a, 2),                                         // RES 2A
        0x90 => res!(b, 2),                                         // RES 2B
        0x91 => res!(c, 2),                                         // RES 2C
        0x92 => res!(d, 2),                                         // RES 2D
        0x93 => res!(e, 2),                                         // RES 2E
        0x94 => res!(h, 2),                                         // RES 2H
        0x95 => res!(l, 2),                                         // RES 2L
        0x96 => { hlfrob!(hl, hl & !(1 << 2)); 4 },                 // RES 2HLm
        0x9F => res!(a, 3),                                         // RES 3A
        0x98 => res!(b, 3),                                         // RES 3B
        0x99 => res!(c, 3),                                         // RES 3C
        0x9A => res!(d, 3),                                         // RES 3D
        0x9B => res!(e, 3),                                         // RES 3E
        0x9C => res!(h, 3),                                         // RES 3H
        0x9D => res!(l, 3),                                         // RES 3L
        0x9E => { hlfrob!(hl, hl & !(1 << 3)); 4 },                 // RES 3HLm
        0xA7 => res!(a, 4),                                         // RES 4A
        0xA0 => res!(b, 4),                                         // RES 4B
        0xA1 => res!(c, 4),                                         // RES 4C
        0xA2 => res!(d, 4),                                         // RES 4D
        0xA3 => res!(e, 4),                                         // RES 4E
        0xA4 => res!(h, 4),                                         // RES 4H
        0xA5 => res!(l, 4),                                         // RES 4L
        0xA6 => { hlfrob!(hl, hl & !(1 << 4)); 4 },                 // RES 4HLm
        0xAF => res!(a, 5),                                         // RES 5A
        0xA8 => res!(b, 5),                                         // RES 5B
        0xA9 => res!(c, 5),                                         // RES 5C
        0xAA => res!(d, 5),                                         // RES 5D
        0xAB => res!(e, 5),                                         // RES 5E
        0xAC => res!(h, 5),                                         // RES 5H
        0xAD => res!(l, 5),                                         // RES 5L
        0xAE => { hlfrob!(hl, hl & !(1 << 5)); 4 },                 // RES 5HLm
        0xB7 => res!(a, 6),                                         // RES 6A
        0xB0 => res!(b, 6),                                         // RES 6B
        0xB1 => res!(c, 6),                                         // RES 6C
        0xB2 => res!(d, 6),                                         // RES 6D
        0xB3 => res!(e, 6),                                         // RES 6E
        0xB4 => res!(h, 6),                                         // RES 6H
        0xB5 => res!(l, 6),                                         // RES 6L
        0xB6 => { hlfrob!(hl, hl & !(1 << 6)); 4 },                 // RES 6HLm
        0xBF => res!(a, 7),                                         // RES 7A
        0xB8 => res!(b, 7),                                         // RES 7B
        0xB9 => res!(c, 7),                                         // RES 7C
        0xBA => res!(d, 7),                                         // RES 7D
        0xBB => res!(e, 7),                                         // RES 7E
        0xBC => res!(h, 7),                                         // RES 7H
        0xBD => res!(l, 7),                                         // RES 7L
        0xBE => { hlfrob!(hl, hl & !(1 << 7)); 4 }                  // RES 7HLm
    }
}

fn add(a: u16, b: u8) -> u16
{
    (a as i16 + (b as i8 as i16)) as u16
}

fn daa(r: &mut Registers)
{
    if r.f & N == 0
    {
        if r.f & C != 0 || r.a > 0x99
        {
            r.a += 0x60;
            r.f |= C;
        }

        if r.f & H != 0 || (r.a & 0xF) > 0x9
        {
            r.a += 0x06;
            r.f &= !H;
        }
    }
    else if r.f & C != 0 && r.f & H != 0
    {
        r.a += 0x9A;
        r.f &= !H;
    }
    else if r.f & C != 0
    {
        r.a += 0xA0;
    }
    else if r.f & H != 0
    {
        r.a += 0xFA;
        r.f &= !H;
    }

    if r.a == 0
    {
        r.f |= Z;
    }
    else
    {
        r.f &= !Z;
    }
}

fn inc_hlm(r: &mut Registers, m: &mut Memory)
{
    let hl = r.hl();
    let k = m.read_byte(hl) + 1;
    m.write_byte(hl, k);
    r.f = (r.f & C) | if k != 0 { 0 } else { Z } | if k & 0xF == 0 { H } else { 0 };
}

fn dec_hlm(r: &mut Registers, m: &mut Memory)
{
    let hl = r.hl();
    let k = m.read_byte(hl) - 1;
    m.write_byte(hl, k);
    r.f = (r.f & C) | if k != 0 { 0 } else { Z } | if k & 0xF == 0 { H } else { 0 };
}

fn ld_hlspn(r: &mut Registers, m: &mut Memory)
{
    let b = m.read_byte(r.adv()) as i8 as i16 as u16;
    let res = b + r.sp;
    r.h = (res >> 8) as u8;
    r.l = res as u8;
    let tmp = b ^ r.sp ^ r.hl();
    r.f = if tmp & 0x100 != 0 { C } else { 0 } | if tmp & 0x010 != 0 { H } else { 0 };
}

fn ld_ioan(r: &mut Registers, m: &mut Memory)
{
    let n = m.read_byte(r.adv());
    m.write_byte(0xFF00 | (n as u16), r.a);
}

fn add_spn(r: &mut Registers, m: &mut Memory)
{
    let b = m.read_byte(r.adv()) as i8 as i16 as u16;
    let res = r.sp + b;
    let tmp = b ^ res ^ r.sp;
    r.f = if tmp & 0x100 != 0 { C } else { 0 } | if tmp & 0x010 != 0 { H } else { 0 };
    r.sp = res;
}

fn add_hlsp(r: &mut Registers)
{
    let s = r.hl() as u32 + r.sp as u32;
    r.f = if r.hl() as u32 & 0xFFF > s & 0xFFF { H } else { 0 } | 
             if s > 0xFFFF { C } else { 0 } | 
             (r.f & Z);
    r.h = (s >> 8) as u8;
    r.l = s as u8;
}

fn pop_af(r: &mut Registers, m: &mut Memory)
{
    r.f = m.read_byte(r.sp) & 0xF0;
    r.a = m.read_byte(r.sp + 1);
    r.sp += 2;
}

fn xx() -> u32 { 0 }