use crate::cpu::registers::Registers;
use crate::mem::Memory;

/// Zero Flag is bit 7 in the F register
const Z: u8 = 0x80;

/// Subtract Flag is bit 6 in the F register
const N: u8 = 0x40;

/// Half-Carry Flag is bit 5 in the F register
const H: u8 = 0x20;

/// Carry Flag is bit 4 in the F register
const C: u8 = 0x10;

/// Execute the given opcode
pub fn exec(op: u8, regs: &mut Registers, mem: &mut Memory) -> u32
{
    // Load the value stored in register $r2 into register $r1
    macro_rules! ld {
        ($r1:ident, $r2:ident) => ({ 
            regs.$r1 = regs.$r2; 
            1 
        });
    }

    // Load the immediate 8-bit value into the given register
    macro_rules! ld_n {
        ($r:ident) => ({ 
            regs.$r = mem.read_byte(regs.adv()); 
            2 
        });
    }

    // Load the immediate 16-bit value into the given register
    macro_rules! ld_nn {
        ($r1:ident, $r2:ident) => ({
            regs.$r2 = mem.read_byte(regs.adv());
            regs.$r1 = mem.read_byte(regs.adv());
            3 
        });
    }

    // Push register pair $r1$r2 onto the stack & decrement SP twice
    macro_rules! push {
        ($r1:ident, $r2:ident) => ({ 
            mem.write_byte(regs.sp - 1, regs.$r1); 
            mem.write_byte(regs.sp - 2, regs.$r2);
            regs.sp = regs.sp.overflowing_sub(2).0;
            4
        });
    }

    // Pop two bytes off the stack into register pair $r1$r2
    // then increment SP twice
    macro_rules! pop {
        ($r1:ident, $r2:ident) => ({
            regs.$r2 = mem.read_byte(regs.sp);
            regs.$r1 = mem.read_byte(regs.sp + 1);
            regs.sp = regs.sp.overflowing_add(2).0;
            3 
        });
    }

    // Add n to A 
    macro_rules! add_a {
        ($n:expr) => ({
            let a = regs.a;
            let n = $n;
            regs.a = a.overflowing_add(n).0;
            regs.f = if (a & 0xF) + (n & 0xF) > 0xF { H } else { 0x0 };
            regs.f |= if (a as u16 + n as u16) > 0xFF { C } else { 0x0 };
            regs.f |= if regs.a == 0 { Z } else { 0x0 };
            1
        });
    }

    // Add n + Carry Flag to A
    macro_rules! adc_a {
        ($n:expr) => ({ 
            let a = regs.a;
            let n = $n;
            let c = if regs.f & C != 0 { 1 } else { 0x0 };
            regs.a = a.overflowing_add(n.overflowing_add(c).0).0;
            regs.f = if (a & 0xF) + (n & 0xF) + c > 0xF { H } else { 0x0 };
            regs.f |= 
                if (a as u16 + n as u16 + c as u16) > 0xFF { C } else { 0x0 };
            regs.f |= if regs.a == 0 { Z } else { 0x0 };
            1 
        });
    }
    
    // Subtract n from A
    macro_rules! sub_a {
        ($n:expr) => ({
            let a = regs.a;
            let n = $n;
            regs.a = a.overflowing_sub(n).0;
            regs.f = 
                N | 
                if a < n { C } else { 0x0 } | 
                if (a & 0xF) < (n & 0xF) { H } else { 0x0 };
            regs.f |= if regs.a == 0 { Z } else { 0x0 };
            1
        });
    }

    // Subtract n + Carry Flag from A
    macro_rules! sbc_a {
        ($n:expr) => ({
            let a = regs.a as u16;
            let n = $n as u16;
            let c = if regs.f & C != 0 { 1 } else { 0x0 };
            regs.f = 
                N | 
                if a < n + c { C } else { 0x0 } | 
                if (a & 0xF) < (n & 0xF) + c { H } else { 0x0 };
            regs.a = (a.overflowing_sub(n).0 - c) as u8;
            regs.f |= if regs.a == 0 { Z } else { 0x0 };
            1
        });
    }

    // Logically AND n with A and store result in A
    macro_rules! and_a {
        ($n:expr) => ({
            let val = regs.a & $n;
            regs.a = val;
            regs.f = H | if regs.a == 0 { Z } else { 0x0 };
            1
        });
    }

    // Logically OR n with A and store result in A
    macro_rules! or_a {
        ($n:expr) => ({
            let val = regs.a | $n;
            regs.a = val;
            regs.f = if regs.a == 0 { Z } else { 0x0 };
            1
        });
    }

    // Logically XOR n with A and store result in A
    macro_rules! xor_a {
        ($n:expr) => ({
            let val = regs.a ^ $n;
            regs.a = val;
            regs.f = if regs.a == 0 { Z } else { 0x0 };
            1
        });
    }

    // Compare A with n
    macro_rules! cp_a {
        ($n:expr) => ({
            let n = $n;
            regs.f = 
                N | 
                if regs.a == n { Z } else { 0x0 } | 
                if regs.a < n { C } else { 0x0 } | 
                if (regs.a & 0xF) < (n & 0xF) { H } else { 0x0 };
            1
        });
    }

    macro_rules! inc {
        // Increment 8-bit register
        ($r:ident) => ({
            regs.$r = regs.$r.overflowing_add(1).0;
            regs.f = 
                (regs.f & C) | 
                if regs.$r == 0 { Z } else { 0x0 } | 
                if regs.$r & 0xF == 0 { H } else { 0x0 };
            1
        });

        // Increment 16-bit register
        ($r1:ident, $r2:ident) => ({
            regs.$r2 = regs.$r2.overflowing_add(1).0;
            if regs.$r2 == 0x0 { regs.$r1 = regs.$r1.overflowing_add(1).0; }
            2
        });
    }

    macro_rules! dec {
        // Decrement 8-bit register
        ($r:ident) => ({
            regs.$r = regs.$r.overflowing_sub(1).0;
            regs.f &= 0x1F;
            regs.f |= 
                N | 
                if regs.$r == 0 { Z } else { 0x0 } | 
                ((((regs.$r & 0xF) == 0xF) as u8) << 5);
            1
        });

        // Decrement 16-bit register
        ($r1:ident, $r2:ident) => ({
            regs.$r2 = regs.$r2.overflowing_sub(1).0;
            if regs.$r2 == 0xFF { regs.$r1 = regs.$r1.overflowing_sub(1).0; }
            2
        });
    }

    // Add n to HL
    macro_rules! add_hl {
        ($reg:expr) => ({
            let a = regs.hl() as u32;
            let b = $reg as u32;
            let new_hl = a + b;
            regs.f = 
                (regs.f & Z) |
                if new_hl > 0xFFFF { C } else { 0x0 } |
                if (a as u32 & 0xFFF) > (new_hl & 0xFFF) { H } else { 0x0 };

            regs.l = new_hl as u8;
            regs.h = (new_hl >> 8) as u8;
            2
        });
    }

    // Decimal adjust register A for BCD operations
    macro_rules! daa {
        () => ({
            let a = regs.a;
            let mut adj = 0;

            // Check if we had a carry/borrow for low nibble in last operation
            if regs.f & H != 0x0 { adj |= 0x06; }

            // See if we had a carry/borrow for high nibble in last operation
            if regs.f & C != 0x0 { adj |= 0x60; }

            let res = if regs.f & N != 0 {
                a.overflowing_sub(adj).0
            } else {
                if a & 0x0F > 0x09 { adj |= 0x06; }
                if a > 0x99 { adj |= 0x60; }
                a.overflowing_add(adj).0
            };

            regs.a = res;
            regs.f = 
                if res == 0 { Z } else { 0x0 } | 
                if adj & 0x60 != 0 { C } else { 0x0 };
            
            1
        });
    }

    // Complement register A (flip all bits)
    macro_rules! cpl {
        () => ({
            regs.a ^= 0xFF;
            regs.f |= N | H;
            1
        });
    }

    // Complement carry flag. If C is set, reset it. If C is reset, set it.
    macro_rules! ccf {
        () => ({
            regs.f = (regs.f & Z) | ((regs.f & C) ^ C);
            1
        });
    }

    // Set carry flag
    macro_rules! scf {
        () => ({
            regs.f = (regs.f & Z) | C;
            1   
        });
    }

    // Power down CPU until interrupt occurs
    macro_rules! halt {
        () => ({
            regs.halt = 1;
            1   
        });
    }

    // Halt CPU & LCD display until a button is pressed
    macro_rules! stop {
        () => ({
            regs.stop = 1;
            1   
        });
    }

    // Disables interrupts after next instruction
    macro_rules! di {
        () => ({
            regs.di();
            1   
        });
    }

    // Enables interrupts after next instruction
    macro_rules! ei {
        () => ({
            regs.ei(mem);
            1   
        });
    }

    // Rotate A left and move old bit 7 to Carry flag
    macro_rules! rlca {
        () => ({
            let ci = if (regs.a & 0x80) != 0 { 1 } else { 0x0 };
            regs.a = (regs.a << 1) | ci;
            regs.f =
                if regs.a == 0 { Z } else { 0x0 } |
                if ci != 0 { C } else { 0x0 };
            1 
        });
    }

    // Rotate A left through Carry flag
    macro_rules! rla {
        () => ({
            let ci = if (regs.f & C) != 0 { 1 } else { 0x0 };
            let co = regs.a & 0x80;
            regs.a = (regs.a << 1) | ci;
            regs.f =
                if regs.a == 0 { Z } else { 0x0 } |
                if co != 0 { C } else { 0x0 };
            1 
        });
    }

    // Rotate A right and move old bit 0 to Carry flag
    macro_rules! rrca {
        () => ({
            let ci = regs.a & 0x01;
            regs.a = (regs.a >> 1) | (ci << 7);
            regs.f = 
                if regs.a == 0 { Z } else { 0x0 } | 
                if ci != 0 { C } else { 0x0 };
            1 
        });
    }

    // Rotate A right through carry flag
    macro_rules! rra {
        () => ({
            let ci = if (regs.f & C) != 0 { 0x80 } else { 0x0 };
            let co = if (regs.a & 0x01) != 0 { C } else { 0x0 };
            regs.a = (regs.a >> 1) | ci;
            regs.f = if regs.a == 0 { Z } else { 0x0 } | co;
            1  
        });
    }

    macro_rules! jp {
        // Jump to address of the two byte immediate value (LS byte first)
        () => ({
            regs.pc = mem.read_word(regs.adv());
            3
        });
        
        // Jump to address of the two byte immediate value (LS byte first)
        // if the condition $cc is true
        ($cc:expr) => ({ 
            if $cc { jp!() } else { regs.pc = regs.pc.overflowing_add(2).0; 3 }
        });
    }

    // Jump to address stored in HL
    macro_rules! jp_hl {
        () => ({
            regs.pc = regs.hl();
            1
        });
    }
    
    macro_rules! jr {
        // Add n to current address and then jump to the resulting address
        () => ({
            let n = mem.read_byte(regs.adv());
            regs.pc = (regs.pc as i16 + (n as i8 as i16)) as u16;
            3
        });

        // Add n to current address and then jump to the resulting address
        // if the condition $cc is true
        ($cc:expr) => ({
            if $cc { jr!() } else { regs.adv(); 2 }
        });
    }

    macro_rules! call {
        // Push address of next instruction onto stack and then jump to address
        // of two byte immediate value (LS byte first)
        () => ({
            regs.sp = regs.sp.overflowing_sub(2).0;
            mem.write_word(regs.sp, regs.pc + 2);
            regs.pc = mem.read_word(regs.pc);
            6  
        });
        
        // Push address of next instruction onto stack and then jump to address
        // of two byte immediate value (LS byte first) if condition $cc is true
        ($cc:expr) => ({
            if $cc { call!() } else { regs.pc = regs.pc.overflowing_add(2).0; 3 }
        });
    }

    /// Push present address onto stack and then jump to address 0x0000 + n
    macro_rules! rst {
        ($n:expr) => ({
            regs.rst($n, mem);
            8
        });
    }

    macro_rules! ret {
        // Pop two bytes from stack & jump to that address
        () => ({
            regs.ret(mem);
            4  
        });

        // Return if condition $cc is true
        ($cc:expr) => ({
            if $cc { regs.ret(mem); 5 } else { 2 }
        });
    }

    // Pop two bytes from stack & jump to that address then enable interrupts
    macro_rules! reti {
        () => ({
            regs.ret(mem);
            regs.ei(mem);
            4
        });
    }

    // Match opcode to instruction
    match op
    {
        0x00 => 1,
        0x01 => ld_nn!(b, c),
        0x02 => { mem.write_byte(regs.bc(), regs.a); 2 },
        0x03 => inc!(b, c),
        0x04 => inc!(b),
        0x05 => dec!(b),
        0x06 => ld_n!(b),
        0x07 => rlca!(),
        0x08 => { 
            let n = mem.read_word(regs.pc); 
            mem.write_word(n, regs.sp); 
            regs.pc += 2; 
            5 
        },
        0x09 => add_hl!(regs.bc()),
        0x0A => { regs.a = mem.read_byte(regs.bc()); 2 },
        0x0B => dec!(b, c),
        0x0C => inc!(c),
        0x0D => dec!(c),
        0x0E => ld_n!(c),
        0x0F => rrca!(),

        0x10 => stop!(),
        0x11 => ld_nn!(d, e),
        0x12 => { mem.write_byte(regs.de(), regs.a); 2 },
        0x13 => inc!(d, e),
        0x14 => inc!(d),
        0x15 => dec!(d),
        0x16 => ld_n!(d),
        0x17 => rla!(),
        0x18 => jr!(),
        0x19 => add_hl!(regs.de()),
        0x1A => { regs.a = mem.read_byte(regs.de()); 2 },
        0x1B => dec!(d, e),
        0x1C => inc!(e),
        0x1D => dec!(e),
        0x1E => ld_n!(e),
        0x1F => rra!(),

        0x20 => jr!((regs.f & Z) == 0),
        0x21 => ld_nn!(h, l),
        0x22 => { mem.write_byte(regs.hl(), regs.a); regs.inc_hl(); 2 },
        0x23 => inc!(h, l),
        0x24 => inc!(h),
        0x25 => dec!(h),
        0x26 => ld_n!(h),
        0x27 => daa!(),
        0x28 => jr!((regs.f & Z) != 0),
        0x29 => add_hl!(regs.hl()),
        0x2A => { regs.a = mem.read_byte(regs.hl()); regs.inc_hl(); 2 },
        0x2B => dec!(h, l),
        0x2C => inc!(l),
        0x2D => dec!(l),
        0x2E => ld_n!(l),
        0x2F => cpl!(),

        0x30 => jr!((regs.f & C) == 0),
        0x31 => { regs.sp = mem.read_word(regs.pc); regs.pc += 2; 3 },
        0x32 => { mem.write_byte(regs.hl(), regs.a); regs.dec_hl(); 2 },
        0x33 => { regs.sp = regs.sp.overflowing_add(1).0; 2 },
        0x34 => {
            let v = mem.read_byte(regs.hl()).overflowing_add(1).0;
            mem.write_byte(regs.hl(), v);
            regs.f =
                (regs.f & C) | 
                if v == 0 { Z } else { 0x0 } | 
                if v & 0xF == 0 { H } else { 0x0 };
            3
        },
        0x35 => {
            let v = mem.read_byte(regs.hl()).overflowing_sub(1).0;
            mem.write_byte(regs.hl(), v);
            regs.f = 
                N | 
                (regs.f & C) | 
                if v == 0 { Z } else { 0x0 } | 
                if v & 0xF == 0xF { H } else { 0x0 };
            3
        },
        0x36 => { 
            let pc = mem.read_byte(regs.adv()); 
            mem.write_byte(regs.hl(), pc); 
            3 
        },
        0x37 => scf!(),
        0x38 => jr!((regs.f & C) != 0),
        0x39 => {
            let hl = regs.hl() as u32;
            let sp = regs.sp as u32;
            let val = hl.overflowing_add(sp).0;
            regs.f = 
                if hl & 0xFFF > val & 0xFFF { H } else { 0 } | 
                if val > 0xFFFF { C } else { 0x0 } | 
                (regs.f & Z);
            regs.h = (val >> 8) as u8;
            regs.l = val as u8;
            2
        },
        0x3A => { regs.a = mem.read_byte(regs.hl()); regs.dec_hl(); 2 },
        0x3B => { regs.sp = regs.sp.overflowing_sub(1).0; 2 },
        0x3C => inc!(a),
        0x3D => dec!(a),
        0x3E => ld_n!(a),
        0x3F => ccf!(),

        0x40 => ld!(b, b),
        0x41 => ld!(b, c),
        0x42 => ld!(b, d),
        0x43 => ld!(b, e),
        0x44 => ld!(b, h),
        0x45 => ld!(b, l),
        0x46 => { regs.b = mem.read_byte(regs.hl()); 2 },
        0x47 => ld!(b, a),
        0x48 => ld!(c, b),
        0x49 => ld!(c, c),
        0x4A => ld!(c, d),
        0x4B => ld!(c, e),
        0x4C => ld!(c, h),
        0x4D => ld!(c, l),
        0x4E => { regs.c = mem.read_byte(regs.hl()); 2 },
        0x4F => ld!(c, a),

        0x50 => ld!(d, b),
        0x51 => ld!(d, c),
        0x52 => ld!(d, d),
        0x53 => ld!(d, e),
        0x54 => ld!(d, h),
        0x55 => ld!(d, l),
        0x56 => { regs.d = mem.read_byte(regs.hl()); 2 },
        0x57 => ld!(d, a),
        0x58 => ld!(e, b),
        0x59 => ld!(e, c),
        0x5A => ld!(e, d),
        0x5B => ld!(e, e),
        0x5C => ld!(e, h),
        0x5D => ld!(e, l),
        0x5E => { regs.e = mem.read_byte(regs.hl()); 2 },
        0x5F => ld!(e, a),

        0x60 => ld!(h, b),
        0x61 => ld!(h, c),
        0x62 => ld!(h, d),
        0x63 => ld!(h, e),
        0x64 => ld!(h, h),
        0x65 => ld!(h, l),
        0x66 => { regs.h = mem.read_byte(regs.hl()); 2 },
        0x67 => ld!(h, a),
        0x68 => ld!(l, b),
        0x69 => ld!(l, c),
        0x6A => ld!(l, d),
        0x6B => ld!(l, e),
        0x6C => ld!(l, h),
        0x6D => ld!(l, l),
        0x6E => { regs.l = mem.read_byte(regs.hl()); 2 },
        0x6F => ld!(l, a),

        0x70 => { mem.write_byte(regs.hl(), regs.b); 2 },
        0x71 => { mem.write_byte(regs.hl(), regs.c); 2 },
        0x72 => { mem.write_byte(regs.hl(), regs.d); 2 },
        0x73 => { mem.write_byte(regs.hl(), regs.e); 2 },
        0x74 => { mem.write_byte(regs.hl(), regs.h); 2 },
        0x75 => { mem.write_byte(regs.hl(), regs.l); 2 },
        0x76 => halt!(),
        0x77 => { mem.write_byte(regs.hl(), regs.a); 2 },
        0x78 => ld!(a, b),
        0x79 => ld!(a, c),
        0x7A => ld!(a, d),
        0x7B => ld!(a, e),
        0x7C => ld!(a, h),
        0x7D => ld!(a, l),
        0x7E => { regs.a = mem.read_byte(regs.hl()); 2 },
        0x7F => ld!(a, a),

        0x80 => add_a!(regs.b),
        0x81 => add_a!(regs.c),
        0x82 => add_a!(regs.d),
        0x83 => add_a!(regs.e),
        0x84 => add_a!(regs.h),
        0x85 => add_a!(regs.l),
        0x86 => { add_a!(mem.read_byte(regs.hl())); 2 },
        0x87 => add_a!(regs.a),
        0x88 => adc_a!(regs.b),
        0x89 => adc_a!(regs.c),
        0x8A => adc_a!(regs.d),
        0x8B => adc_a!(regs.e),
        0x8C => adc_a!(regs.h),
        0x8D => adc_a!(regs.l),
        0x8E => { adc_a!(mem.read_byte(regs.hl())); 2 },
        0x8F => adc_a!(regs.a),

        0x90 => sub_a!(regs.b),
        0x91 => sub_a!(regs.c),
        0x92 => sub_a!(regs.d),
        0x93 => sub_a!(regs.e),
        0x94 => sub_a!(regs.h),
        0x95 => sub_a!(regs.l),
        0x96 => { sub_a!(mem.read_byte(regs.hl())); 2 },
        0x97 => sub_a!(regs.a),
        0x98 => sbc_a!(regs.b),
        0x99 => sbc_a!(regs.c),
        0x9A => sbc_a!(regs.d),
        0x9B => sbc_a!(regs.e),
        0x9C => sbc_a!(regs.h),
        0x9D => sbc_a!(regs.l),
        0x9E => { sbc_a!(mem.read_byte(regs.hl())); 2 },
        0x9F => sbc_a!(regs.a),

        0xA0 => and_a!(regs.b),
        0xA1 => and_a!(regs.c),
        0xA2 => and_a!(regs.d),
        0xA3 => and_a!(regs.e),
        0xA4 => and_a!(regs.h),
        0xA5 => and_a!(regs.l),
        0xA6 => { and_a!(mem.read_byte(regs.hl())); 2 },
        0xA7 => and_a!(regs.a),
        0xA8 => xor_a!(regs.b),
        0xA9 => xor_a!(regs.c),
        0xAA => xor_a!(regs.d),
        0xAB => xor_a!(regs.e),
        0xAC => xor_a!(regs.h),
        0xAD => xor_a!(regs.l),
        0xAE => { xor_a!(mem.read_byte(regs.hl())); 2 },
        0xAF => xor_a!(regs.a),

        0xB0 => or_a!(regs.b),
        0xB1 => or_a!(regs.c),
        0xB2 => or_a!(regs.d),
        0xB3 => or_a!(regs.e),
        0xB4 => or_a!(regs.h),
        0xB5 => or_a!(regs.l),
        0xB6 => { or_a!(mem.read_byte(regs.hl())); 2 },
        0xB7 => or_a!(regs.a),
        0xB8 => cp_a!(regs.b),
        0xB9 => cp_a!(regs.c),
        0xBA => cp_a!(regs.d),
        0xBB => cp_a!(regs.e),
        0xBC => cp_a!(regs.h),
        0xBD => cp_a!(regs.l),
        0xBE => { cp_a!(mem.read_byte(regs.hl())); 2 },
        0xBF => cp_a!(regs.a),

        0xC0 => ret!((regs.f & Z) == 0),
        0xC1 => pop!(b, c),
        0xC2 => jp!((regs.f & Z) == 0),
        0xC3 => jp!(),
        0xC4 => call!((regs.f & Z) == 0),
        0xC5 => push!(b, c),
        0xC6 => { add_a!(mem.read_byte(regs.adv())); 2 },
        0xC7 => rst!(0x00),
        0xC8 => ret!((regs.f & Z) != 0),
        0xC9 => ret!(),
        0xCA => jp!((regs.f & Z) != 0),
        0xCB => { exec_cb(mem.read_byte(regs.adv()), regs, mem) },
        0xCC => call!((regs.f & Z) != 0),
        0xCD => call!(),
        0xCE => { adc_a!(mem.read_byte(regs.adv())); 2 },
        0xCF => rst!(0x08),

        0xD0 => ret!((regs.f & C) == 0),
        0xD1 => pop!(d, e),
        0xD2 => jp!((regs.f & C) == 0),
        0xD3 => 0u32,
        0xD4 => call!((regs.f & C) == 0),
        0xD5 => push!(d, e),
        0xD6 => { sub_a!(mem.read_byte(regs.adv())); 2 },
        0xD7 => rst!(0x10),
        0xD8 => ret!((regs.f & C) != 0),
        0xD9 => reti!(),
        0xDA => jp!((regs.f & C) != 0),
        0xDB => 0u32,
        0xDC => call!((regs.f & C) != 0),
        0xDD => 0u32,
        0xDE => { sbc_a!(mem.read_byte(regs.adv())); 2 },
        0xDF => rst!(0x18),

        0xE0 => { 
            let n = mem.read_byte(regs.adv()); 
            mem.write_byte(0xFF00 | (n as u16), regs.a); 
            3 
        },
        0xE1 => pop!(h, l),
        0xE2 => { mem.write_byte(0xFF00 | (regs.c as u16), regs.a); 2 },
        0xE3 => 0u32,
        0xE4 => 0u32,
        0xE5 => push!(h, l),
        0xE6 => { and_a!(mem.read_byte(regs.adv())); 2 },
        0xE7 => rst!(0x20),
        0xE8 => {
            let n = mem.read_byte(regs.adv()) as i8 as i16 as u16;
            let val = regs.sp.overflowing_add(n).0;
            let tmp = n ^ val ^ regs.sp;
            regs.f = 
                if tmp & 0x100 != 0 { C } else { 0 } | 
                if tmp & 0x010 != 0 { H } else { 0x0 };
            regs.sp = val;
            4
        },
        0xE9 => jp_hl!(),
        0xEA => { 
            let n = mem.read_word(regs.pc); 
            mem.write_byte(n, regs.a); 
            regs.pc += 2; 
            4 
        },
        0xEB => 0u32,
        0xEC => 0u32,
        0xED => 0u32,
        0xEE => { xor_a!(mem.read_byte(regs.adv())); 2 },
        0xEF => rst!(0x28),

        0xF0 => { 
            let n = mem.read_byte(regs.adv()); 
            regs.a = mem.read_byte(0xFF00 | (n as u16)); 
            3
        },
        0xF1 => pop!(a, f),
        0xF2 => { regs.a = mem.read_byte(0xFF00 | (regs.c as u16)); 2 },
        0xF3 => di!(),
        0xF4 => 0u32,
        0xF5 => push!(a, f),
        0xF6 => { or_a!(mem.read_byte(regs.adv())); 2 },
        0xF7 => rst!(0x30),
        0xF8 => {
            // Convert to signed value
            let sp = regs.sp as i32;
            let n = mem.read_byte(regs.adv()) as i8;
            let nn = n as i32;
            let res = sp.overflowing_add(nn).0;

            // store result of the operation in HL
            regs.h =  ((res as u16) >> 8) as u8;
            regs.l = (res as u16) as u8;

            // Set flags
            let tmp = sp ^ nn ^ res;
            regs.f = if tmp & 0x100 != 0 { C } else { 0 } |
                     if tmp & 0x010 != 0 { H } else { 0 };

            3
        },
        0xF9 => { regs.sp = regs.hl(); 2 },
        0xFA => { 
            regs.a = mem.read_byte(mem.read_word(regs.pc)); 
            regs.pc += 2; 
            4 
        },
        0xFB => ei!(),
        0xFC => 0u32,
        0xFD => 0u32,
        0xFE => { cp_a!(mem.read_byte(regs.adv())); 2 },
        0xFF => rst!(0x38)
    }
}

/// Execute an opcode that is preceded by the value 0xCB
pub fn exec_cb(op: u8, regs: &mut Registers, mem: &mut Memory) -> u32
{
    // Swap upper and lower nibbles of $n
    macro_rules! swap {
        ($reg:expr) => ({
            $reg = ($reg << 4) | (($reg & 0xF0) >> 4);
            regs.f = if $reg == 0 { Z } else { 0x0 };
            2
        });
    }

    // Rotate register n left and move the old bit 7 to Carry flag
    macro_rules! rlc {
        ($reg:expr) => ({ 
            let ci = if ($reg & 0x80) != 0 { 1 } else { 0 };
            $reg = ($reg << 1) | ci;
            regs.f = 
                if $reg == 0 { Z } else { 0x0 } | 
                if ci != 0 { C } else { 0x0 };
            2 
        });
    }

    // Rotate register n left through Carry flag
    macro_rules! rl {
        ($reg:expr) => ({
            let ci = if (regs.f & C) != 0 { 1 } else { 0 };
            let co = $reg & 0x80;
            $reg = ($reg << 1) | ci;
            regs.f = 
                if $reg == 0 { Z } else { 0x0 } | 
                if co != 0 { C } else { 0x0 };
            2
        });
    }

    // Rotate register n right and move the old bit 0 to Carry flag
    macro_rules! rrc {
        ($reg:expr) => ({
            let ci = $reg & 0x01;
            $reg = ($reg >> 1) | (ci << 7);
            regs.f = 
                if $reg == 0 { Z } else { 0x0 } | 
                if ci != 0 { C } else { 0x0 };
            2
        });
    }

    // Rotate register n right through Carry flag
    macro_rules! rr {
        ($reg:expr) => ({
            let ci = if (regs.f & C) != 0 { 0x80 } else { 0 };
            let co = if ($reg & 0x01) != 0 { C } else { 0 };
            $reg = ($reg >> 1) | ci;
            regs.f = if $reg == 0 { Z } else { 0x0 } | co;
            2
        });
    }

    // Shift register n left into Carry flag. LSB of n is set to 0.
    macro_rules! sla {
        ($reg:expr) => ({
            let co = ($reg >> 7) & 1;
            $reg = $reg << 1;
            regs.f = 
                if $reg == 0 { Z } else { 0x0 } |
                if co != 0 { C } else { 0x0 };
            2
        });
    }

    // Shift register n right into Carry flag. MSB of n doesn't change.
    macro_rules! sra {
        ($reg:expr) => ({
            let co = $reg & 1;
            $reg = (($reg as i8) >> 1) as u8;
            regs.f = 
                if $reg == 0 { Z } else { 0x0 } | 
                if co != 0 { C } else { 0x0 };
            2
        });
    }

    // Shift register n right into Carry flag. MSB of n is set to 0.
    macro_rules! srl {
        ($reg:expr) => ({
            let co = $reg & 1;
            $reg = $reg >> 1;
            regs.f =
                if $reg == 0 { Z } else { 0x0 } |
                if co != 0 { C } else { 0x0 };
            2
        });
    }

    // Test bit b in register n
    macro_rules! bit {
        ($reg:expr, $bit:expr) => ({
            regs.f = 
                (regs.f & C) |
                H |
                if $reg & (1 << $bit) == 0 { Z } else { 0x0 };
            2
        });
    }

    // Set bit b in register n
    macro_rules! set {
        ($n:ident, $bit:expr) => ({
            regs.$n |= 1 << $bit;
            2
        });
    }

    // Reset bit b in regsiter n
    macro_rules! res {
        ($n:ident, $bit:expr) => ({
            regs.$n &= !(1 << $bit);
            2
        });
    }

    // { hlm!(hl, rlc!(hl, 1)); 4 }

    match op
    {
        0x00 => rlc!(regs.b),
        0x01 => rlc!(regs.c),
        0x02 => rlc!(regs.d),
        0x03 => rlc!(regs.e),
        0x04 => rlc!(regs.h),
        0x05 => rlc!(regs.l),
        0x06 => {
            let mut v = mem.read_byte(regs.hl());
            rlc!(v);
            mem.write_byte(regs.hl(), v);
            4 
        },
        0x07 => rlc!(regs.a),
        0x08 => rrc!(regs.b),
        0x09 => rrc!(regs.c),
        0x0A => rrc!(regs.d),
        0x0B => rrc!(regs.e),
        0x0C => rrc!(regs.h),
        0x0D => rrc!(regs.l),
        0x0E => {
            let mut v = mem.read_byte(regs.hl());
            rrc!(v);
            mem.write_byte(regs.hl(), v);
            4 
        },
        0x0F => rrc!(regs.a),

        0x10 => rl!(regs.b),
        0x11 => rl!(regs.c),
        0x12 => rl!(regs.d),
        0x13 => rl!(regs.e),
        0x14 => rl!(regs.h),
        0x15 => rl!(regs.l),
        0x16 => {
            let mut v = mem.read_byte(regs.hl());
            rl!(v);
            mem.write_byte(regs.hl(), v);
            4 
        },
        0x17 => rl!(regs.a),
        0x18 => rr!(regs.b),
        0x19 => rr!(regs.c),
        0x1A => rr!(regs.d),
        0x1B => rr!(regs.e),
        0x1C => rr!(regs.h),
        0x1D => rr!(regs.l),
        0x1E => {
            let mut v = mem.read_byte(regs.hl());
            rr!(v);
            mem.write_byte(regs.hl(), v);
            4 
        },
        0x1F => rr!(regs.a),

        0x20 => sla!(regs.b),
        0x21 => sla!(regs.c),
        0x22 => sla!(regs.d),
        0x23 => sla!(regs.e),
        0x24 => sla!(regs.h),
        0x25 => sla!(regs.l),
        0x26 => {
            let mut v = mem.read_byte(regs.hl());
            sla!(v);
            mem.write_byte(regs.hl(), v);
            4 
        },
        0x27 => sla!(regs.a),
        0x28 => sra!(regs.b),
        0x29 => sra!(regs.c),
        0x2A => sra!(regs.d),
        0x2B => sra!(regs.e),
        0x2C => sra!(regs.h),
        0x2D => sra!(regs.l),
        0x2E => {
            let mut v = mem.read_byte(regs.hl());
            sra!(v);
            mem.write_byte(regs.hl(), v);
            4 
        },
        0x2F => sra!(regs.a),

        0x30 => swap!(regs.b),
        0x31 => swap!(regs.c),
        0x32 => swap!(regs.d),
        0x33 => swap!(regs.e),
        0x34 => swap!(regs.h),
        0x35 => swap!(regs.l),
        0x36 => {
            let mut v = mem.read_byte(regs.hl());
            swap!(v);
            mem.write_byte(regs.hl(), v);
            4 
        },
        0x37 => swap!(regs.a),
        0x38 => srl!(regs.b),
        0x39 => srl!(regs.c),
        0x3A => srl!(regs.d),
        0x3B => srl!(regs.e),
        0x3C => srl!(regs.h),
        0x3D => srl!(regs.l),
        0x3E => {
            let mut v = mem.read_byte(regs.hl());
            srl!(v);
            mem.write_byte(regs.hl(), v);
            4 
        },
        0x3F => srl!(regs.a),

        0x40 => bit!(regs.b, 0),
        0x41 => bit!(regs.c, 0),
        0x42 => bit!(regs.d, 0),
        0x43 => bit!(regs.e, 0),
        0x44 => bit!(regs.h, 0),
        0x45 => bit!(regs.l, 0),
        0x46 => { bit!(mem.read_byte(regs.hl()), 0); 3 },
        0x47 => bit!(regs.a, 0),
        0x48 => bit!(regs.b, 1),
        0x49 => bit!(regs.c, 1),
        0x4A => bit!(regs.d, 1),
        0x4B => bit!(regs.e, 1),
        0x4C => bit!(regs.h, 1),
        0x4D => bit!(regs.l, 1),
        0x4E => { bit!(mem.read_byte(regs.hl()), 1); 3 },
        0x4F => bit!(regs.a, 1),

        0x50 => bit!(regs.b, 2),
        0x51 => bit!(regs.c, 2),
        0x52 => bit!(regs.d, 2),
        0x53 => bit!(regs.e, 2),
        0x54 => bit!(regs.h, 2),
        0x55 => bit!(regs.l, 2),
        0x56 => { bit!(mem.read_byte(regs.hl()), 2); 3 },
        0x57 => bit!(regs.a, 2),
        0x58 => bit!(regs.b, 3),
        0x59 => bit!(regs.c, 3),
        0x5A => bit!(regs.d, 3),
        0x5B => bit!(regs.e, 3),
        0x5C => bit!(regs.h, 3),
        0x5D => bit!(regs.l, 3),
        0x5E => { bit!(mem.read_byte(regs.hl()), 3); 3 },
        0x5F => bit!(regs.a, 3),

        0x60 => bit!(regs.b, 4),
        0x61 => bit!(regs.c, 4),
        0x62 => bit!(regs.d, 4),
        0x63 => bit!(regs.e, 4),
        0x64 => bit!(regs.h, 4),
        0x65 => bit!(regs.l, 4),
        0x66 => { bit!(mem.read_byte(regs.hl()), 4); 3 },
        0x67 => bit!(regs.a, 4),
        0x68 => bit!(regs.b, 5),
        0x69 => bit!(regs.c, 5),
        0x6A => bit!(regs.d, 5),
        0x6B => bit!(regs.e, 5),
        0x6C => bit!(regs.h, 5),
        0x6D => bit!(regs.l, 5),
        0x6E => { bit!(mem.read_byte(regs.hl()), 5); 3 },
        0x6F => bit!(regs.a, 5),

        0x70 => bit!(regs.b, 6),
        0x71 => bit!(regs.c, 6),
        0x72 => bit!(regs.d, 6),
        0x73 => bit!(regs.e, 6),
        0x74 => bit!(regs.h, 6),
        0x75 => bit!(regs.l, 6),
        0x76 => { bit!(mem.read_byte(regs.hl()), 6); 3 },
        0x77 => bit!(regs.a, 6),
        0x78 => bit!(regs.b, 7),
        0x79 => bit!(regs.c, 7),
        0x7A => bit!(regs.d, 7),
        0x7B => bit!(regs.e, 7),
        0x7C => bit!(regs.h, 7),
        0x7D => bit!(regs.l, 7),
        0x7E => { bit!(mem.read_byte(regs.hl()), 7); 3 },
        0x7F => bit!(regs.a, 7),

        //{ hlfrob!(hl, hl & !(1 << 0)); 4 }

        0x80 => res!(b, 0),
        0x81 => res!(c, 0),
        0x82 => res!(d, 0),
        0x83 => res!(e, 0),
        0x84 => res!(h, 0),
        0x85 => res!(l, 0),
        0x86 => { 
            let v = mem.read_byte(regs.hl()); 
            mem.write_byte(regs.hl(), v & !(1 << 0)); 
            4 
        },
        0x87 => res!(a, 0),
        0x88 => res!(b, 1),
        0x89 => res!(c, 1),
        0x8A => res!(d, 1),
        0x8B => res!(e, 1),
        0x8C => res!(h, 1),
        0x8D => res!(l, 1),
        0x8E => { 
            let v = mem.read_byte(regs.hl()); 
            mem.write_byte(regs.hl(), v & !(1 << 1)); 
            4 
        },
        0x8F => res!(a, 1),

        0x90 => res!(b, 2),
        0x91 => res!(c, 2),
        0x92 => res!(d, 2),
        0x93 => res!(e, 2),
        0x94 => res!(h, 2),
        0x95 => res!(l, 2),
        0x96 => { 
            let v = mem.read_byte(regs.hl()); 
            mem.write_byte(regs.hl(), v & !(1 << 2)); 
            4 
        },
        0x97 => res!(a, 2),
        0x98 => res!(b, 3),
        0x99 => res!(c, 3),
        0x9A => res!(d, 3),
        0x9B => res!(e, 3),
        0x9C => res!(h, 3),
        0x9D => res!(l, 3),
        0x9E => { 
            let v = mem.read_byte(regs.hl()); 
            mem.write_byte(regs.hl(), v & !(1 << 3)); 
            4 
        },
        0x9F => res!(a, 3),

        0xA0 => res!(b, 4),
        0xA1 => res!(c, 4),
        0xA2 => res!(d, 4),
        0xA3 => res!(e, 4),
        0xA4 => res!(h, 4),
        0xA5 => res!(l, 4),
        0xA6 => { 
            let v = mem.read_byte(regs.hl()); 
            mem.write_byte(regs.hl(), v & !(1 << 4)); 
            4 
        },
        0xA7 => res!(a, 4),
        0xA8 => res!(b, 5),
        0xA9 => res!(c, 5),
        0xAA => res!(d, 5),
        0xAB => res!(e, 5),
        0xAC => res!(h, 5),
        0xAD => res!(l, 5),
        0xAE => { 
            let v = mem.read_byte(regs.hl()); 
            mem.write_byte(regs.hl(), v & !(1 << 5)); 
            4 
        },
        0xAF => res!(a, 5),

        0xB0 => res!(b, 6),
        0xB1 => res!(c, 6),
        0xB2 => res!(d, 6),
        0xB3 => res!(e, 6),
        0xB4 => res!(h, 6),
        0xB5 => res!(l, 6),
        0xB6 => { 
            let v = mem.read_byte(regs.hl()); 
            mem.write_byte(regs.hl(), v & !(1 << 6)); 
            4 
        },
        0xB7 => res!(a, 6),
        0xB8 => res!(b, 7),
        0xB9 => res!(c, 7),
        0xBA => res!(d, 7),
        0xBB => res!(e, 7),
        0xBC => res!(h, 7),
        0xBD => res!(l, 7),
        0xBE => { 
            let v = mem.read_byte(regs.hl()); 
            mem.write_byte(regs.hl(), v & !(1 << 7)); 
            4 
        },
        0xBF => res!(a, 7),

        0xC0 => set!(b, 0),
        0xC1 => set!(c, 0),
        0xC2 => set!(d, 0),
        0xC3 => set!(e, 0),
        0xC4 => set!(h, 0),
        0xC5 => set!(l, 0),
        0xC6 => { 
            let v = mem.read_byte(regs.hl()); 
            mem.write_byte(regs.hl(), v | (1 << 0)); 
            4 
        },
        0xC7 => set!(a, 0),
        0xC8 => set!(b, 1),
        0xC9 => set!(c, 1),
        0xCA => set!(d, 1),
        0xCB => set!(e, 1),
        0xCC => set!(h, 1),
        0xCD => set!(l, 1),
        0xCE => { 
            let v = mem.read_byte(regs.hl()); 
            mem.write_byte(regs.hl(), v | (1 << 1)); 
            4 
        },
        0xCF => set!(a, 1),

        0xD0 => set!(b, 2),
        0xD1 => set!(c, 2),
        0xD2 => set!(d, 2),
        0xD3 => set!(e, 2),
        0xD4 => set!(h, 2),
        0xD5 => set!(l, 2),
        0xD6 => { 
            let v = mem.read_byte(regs.hl()); 
            mem.write_byte(regs.hl(), v | (1 << 2)); 
            4 
        },
        0xD7 => set!(a, 2),
        0xD8 => set!(b, 3),
        0xD9 => set!(c, 3),
        0xDA => set!(d, 3),
        0xDB => set!(e, 3),
        0xDC => set!(h, 3),
        0xDD => set!(l, 3),
        0xDE => { 
            let v = mem.read_byte(regs.hl()); 
            mem.write_byte(regs.hl(), v | (1 << 3)); 
            4 
        },
        0xDF => set!(a, 3),

        0xE0 => set!(b, 4),
        0xE1 => set!(c, 4),
        0xE2 => set!(d, 4),
        0xE3 => set!(e, 4),
        0xE4 => set!(h, 4),
        0xE5 => set!(l, 4),
        0xE6 => { 
            let v = mem.read_byte(regs.hl()); 
            mem.write_byte(regs.hl(), v | (1 << 4)); 
            4 
        },
        0xE7 => set!(a, 4),
        0xE8 => set!(b, 5),
        0xE9 => set!(c, 5),
        0xEA => set!(d, 5),
        0xEB => set!(e, 5),
        0xEC => set!(h, 5),
        0xED => set!(l, 5),
        0xEE => { 
            let v = mem.read_byte(regs.hl()); 
            mem.write_byte(regs.hl(), v | (1 << 5)); 
            4 
        },
        0xEF => set!(a, 5),

        0xF0 => set!(b, 6),
        0xF1 => set!(c, 6),
        0xF2 => set!(d, 6),
        0xF3 => set!(e, 6),
        0xF4 => set!(h, 6),
        0xF5 => set!(l, 6),
        0xF6 => { 
            let v = mem.read_byte(regs.hl()); 
            mem.write_byte(regs.hl(), v | (1 << 6)); 
            4 
        },
        0xF7 => set!(a, 6),
        0xF8 => set!(b, 7),
        0xF9 => set!(c, 7),
        0xFA => set!(d, 7),
        0xFB => set!(e, 7),
        0xFC => set!(h, 7),
        0xFD => set!(l, 7),
        0xFE => { 
            let v = mem.read_byte(regs.hl()); 
            mem.write_byte(regs.hl(), v | (1 << 7)); 
            4 
        },
        0xFF => set!(a, 7)
    }
}