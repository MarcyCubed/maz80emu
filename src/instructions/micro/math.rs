//! Math instructions

use crate::instructions::ExecResult;
use crate::state::{Flags, Register, Register16, State};

/// The bit representing the sign
const SIGN_BIT: u16 = 1 << 7;

/// The bit that can receive a half carry
const HALF_CARRY_BIT: u16 = 1 << 4;

/// The bit out of u8 range
const CARRY_BIT: u16 = 1 << 8;

/// The bit representing the sign
const SIGN_BIT_16: u32 = 1 << 15;

/// The bit that can receive a half carry
const HALF_CARRY_BIT_16: u32 = 1 << 12;

/// The bit out of u16 range
const CARRY_BIT_16: u32 = 1 << 16;

/// Increment a 16-bit register
pub fn inc_rr(state: &mut State, register: Register16, cycles: u8) -> ExecResult {
    state.set_register_16(register, state.get_register_16(register).wrapping_add(1));
    ExecResult::Done(cycles)
}

/// Decrement a 16-bit register
pub fn dec_rr(state: &mut State, register: Register16, cycles: u8) -> ExecResult {
    state.set_register_16(register, state.get_register_16(register).wrapping_sub(1));
    ExecResult::Done(cycles)
}

/// Perform an arithmetic operation on the values, getting the assigned flags
///
/// Return the sum, the flags and the carry-out. The flags S, Z, V, C and H are updated.
fn arithmetic_flags(a: u8, b: u8, carry_in: bool, op: fn(u16, u16) -> u16) -> (u8, Flags) {
    let a = a as u16;
    let b = b as u16;
    // Do the math
    let c = op(op(a, b), carry_in as u16);
    // Updates the S and Z flags
    let mut flags = Flags::from_value(c as u8);
    // Half carry
    if (a ^ b) & HALF_CARRY_BIT != c & HALF_CARRY_BIT {
        flags |= Flags::H;
    }
    // Overflow == if sign bit was overwritten
    if a & SIGN_BIT == b & SIGN_BIT && a & SIGN_BIT != c & SIGN_BIT {
        // Overflow
        flags |= Flags::V;
    }
    // Carry
    if c & CARRY_BIT != 0 {
        flags |= Flags::C;
    }
    (c as u8, flags)
}

/// Add two values and a carry-in together, getting the sum and the flags
fn add_flags(a: u8, b: u8, carry_in: bool) -> (u8, Flags) {
    arithmetic_flags(a, b, carry_in, u16::wrapping_add)
}

/// Subtract one value and the carry from another, getting the difference and the flags
fn sub_flags(a: u8, b: u8, borrow_in: bool) -> (u8, Flags) {
    let (result, flags) = arithmetic_flags(a, b, borrow_in, u16::wrapping_sub);
    (result, flags | Flags::N)
}

/// Increment an 8-bit register
///
/// Return Done
pub fn inc_r(state: &mut State, register: Register, cycles: u8) -> ExecResult {
    let (inc, flags) = add_flags(state.get_register_8(register), 1, false);
    state.set_register_8(register, inc);
    // Old C flag || Computed new flags
    let flags = (state.get_flags() & Flags::C) | (flags - Flags::C);
    state.update_flags(flags);
    ExecResult::Done(cycles)
}

/// Decrement an 8-bit register
///
/// Return Done
pub fn dec_r(state: &mut State, register: Register, cycles: u8) -> ExecResult {
    let (dec, flags) = sub_flags(state.get_register_8(register), 1, false);
    state.set_register_8(register, dec);
    // Old C flag || Computed new flags
    let flags = (state.get_flags() & Flags::C) | (flags - Flags::C);
    state.update_flags(flags);
    ExecResult::Done(cycles)
}

/// Add a 16-bit value to HL
pub fn add_hl_rr(state: &mut State, register: Register16, cycles: u8) -> ExecResult {
    let hl = state.hl();
    let other = state.get_register_16(register);
    // Flags S, Z and V are unchanged
    let mut flags = state.get_flags() & (Flags::S | Flags::Z | Flags::V);
    let (result, carry) = hl.overflowing_add(other);
    // Set carry flag
    if carry {
        flags |= Flags::C;
    }
    // Set half carry flag
    if (hl ^ other ^ result) & HALF_CARRY_BIT_16 as u16 != 0 {
        flags |= Flags::H;
    }
    state.update_flags(flags);
    // Store the result in HL
    *state.hl_mut() = result.to_le_bytes();
    ExecResult::Done(cycles)
}

/// Adjust a BCD value after a math operation
pub fn daa(state: &mut State, cycles: u8) -> ExecResult {
    let a = state.a();
    let flags_0 = state.get_flags();
    //let high_nybble = a >> 4;
    let low_nybble = a & 0xf;
    // What we'll add to / subtract form A
    let mut diff = 0u8;
    // Keep N
    let mut flags = flags_0 & Flags::N;

    if low_nybble > 0x9 || flags_0.is_set(Flags::H) {
        diff += 0x6;
    }
    if flags_0.is_set(Flags::C) || a > 0x99 {
        diff += 0x60;
        flags |= Flags::C;
    }
    // Get the result
    let a = if !flags_0.is_set(Flags::N) {
        a.wrapping_add(diff)
    } else {
        a.wrapping_sub(diff)
    };
    // S and Z set according to value
    flags |= Flags::from_value(a);
    if (flags_0.is_set(Flags::N) && flags_0.is_set(Flags::H) && low_nybble <= 0x5)
        || (low_nybble > 0x9 && !flags_0.is_set(Flags::N))
    {
        flags |= Flags::H;
    }
    *state.a_mut() = a;
    state.update_flags(flags);
    ExecResult::Done(cycles)
}

/// Complement of the accumulator.
///
/// `!a`
pub fn cpl(state: &mut State, cycles: u8) -> ExecResult {
    *state.a_mut() = !state.a();
    let flags = state.get_flags() | Flags::N | Flags::H;
    state.update_flags(flags);
    ExecResult::Done(cycles)
}

/// Increment the contents of the `Z` register then store the result in the given address
pub fn inc_z_mem(state: &mut State, address: u16) -> ExecResult {
    inc_r(state, Register::Z, 0);
    ExecResult::Store {
        address,
        data: state.z(),
    }
}

/// Decrement the contents of the `Z` register then store the result in the given address
pub fn dec_z_mem(state: &mut State, address: u16) -> ExecResult {
    dec_r(state, Register::Z, 0);
    ExecResult::Store {
        address,
        data: state.z(),
    }
}

/// Set the carry flag
pub fn scf(state: &mut State, cycles: u8) -> ExecResult {
    let flags = (state.get_flags() | Flags::C) - (Flags::H | Flags::N);
    state.update_flags(flags);
    ExecResult::Done(cycles)
}

/// Complement the carry flag
pub fn ccf(state: &mut State, cycles: u8) -> ExecResult {
    let flags_0 = state.get_flags();
    let mut flags = flags_0 - (Flags::H | Flags::N | Flags::C);
    flags |= Flags::C & !flags_0;
    if flags_0.is_set(Flags::C) {
        flags |= Flags::H
    }
    state.update_flags(flags);
    ExecResult::Done(cycles)
}

/// Add the value of a register to A
pub fn add_a_r(state: &mut State, reg: Register, cycles: u8) -> ExecResult {
    add_a_r_common(state, reg, false, cycles)
}

/// Add the value of a register and the existing carry to A
pub fn adc_a_r(state: &mut State, reg: Register, cycles: u8) -> ExecResult {
    add_a_r_common(state, reg, state.get_flags().is_set(Flags::C), cycles)
}

/// Common part between add_a_r and adc_a_r
fn add_a_r_common(state: &mut State, reg: Register, carry_in: bool, cycles: u8) -> ExecResult {
    let a = state.a();
    let n = state.get_register_8(reg);
    let (a, flags) = add_flags(a, n, carry_in);
    //println!(" a = {:x}h", a);
    *state.a_mut() = a;
    state.update_flags(flags);
    ExecResult::Done(cycles)
}

/// Common part between sub_r and sbc_r
fn sub_r_common(state: &mut State, reg: Register, carry_in: bool, cycles: u8) -> ExecResult {
    let a = state.a();
    let n = state.get_register_8(reg);
    let (a, flags) = sub_flags(a, n, carry_in);
    //println!(" a = {:x}h  x = {:x}h  ", a, n);
    *state.a_mut() = a;
    state.update_flags(flags);
    ExecResult::Done(cycles)
}

/// Subtract the value of a register from A
pub fn sub_r(state: &mut State, reg: Register, cycles: u8) -> ExecResult {
    sub_r_common(state, reg, false, cycles)
}

/// Subtract the value of a register and the existing carry from A
pub fn sbc_r(state: &mut State, reg: Register, cycles: u8) -> ExecResult {
    sub_r_common(state, reg, state.get_flags().is_set(Flags::C), cycles)
}

/// Perform an `AND` operation between the register and the accumulator
pub fn and_r(state: &mut State, reg: Register, cycles: u8) -> ExecResult {
    let a = state.a() & state.get_register_8(reg);
    let flags = Flags::H | Flags::from_value(a) | Flags::parity(a);
    *state.a_mut() = a;
    state.update_flags(flags);
    ExecResult::Done(cycles)
}

/// Perform a `XOR` operation between the register and the accumulator
pub fn xor_r(state: &mut State, reg: Register, cycles: u8) -> ExecResult {
    let a = state.a() ^ state.get_register_8(reg);
    let flags = Flags::from_value(a) | Flags::parity(a);
    *state.a_mut() = a;
    state.update_flags(flags);
    ExecResult::Done(cycles)
}

/// Perform an `OR` operation between the register and the accumulator
pub fn or_r(state: &mut State, reg: Register, cycles: u8) -> ExecResult {
    let a = state.a() | state.get_register_8(reg);
    let flags = Flags::from_value(a) | Flags::parity(a);
    *state.a_mut() = a;
    state.update_flags(flags);
    ExecResult::Done(cycles)
}

/// Compare the accumulator and the register
pub fn cp_r(state: &mut State, reg: Register, cycles: u8) -> ExecResult {
    //println!(" a = {:x}h  x = {:x}h  ", state.a(), state.get_register_8(reg));
    let (_, flags) = sub_flags(state.a(), state.get_register_8(reg), false);
    state.update_flags(flags);
    ExecResult::Done(cycles)
}

/// Perform an arithmetic operation on 2 16-bit values, getting the assigned flags
///
/// Return the sum, the flags and the carry-out. The flags S, Z, V, C and H are updated.
fn word_arithmetic_flags(a: u16, b: u16, carry_in: bool, op: fn(u32, u32) -> u32) -> (u16, Flags) {
    let a = a as u32;
    let b = b as u32;
    // Do the math
    let c = op(op(a, b), carry_in as u32);
    // Updates the S and Z flags
    let mut flags = if c & SIGN_BIT_16 != 0 {
        Flags::S
    } else if c == 0 {
        Flags::Z
    } else {
        Flags::new()
    };
    // Half carry
    if (a ^ b) & HALF_CARRY_BIT_16 != c & HALF_CARRY_BIT_16 {
        flags |= Flags::H;
    }
    // Overflow == if sign bit was overwritten
    if a & SIGN_BIT_16 == b & SIGN_BIT_16 && a & SIGN_BIT_16 != c & SIGN_BIT_16 {
        // Overflow
        flags |= Flags::V;
    }
    // Carry
    if c & CARRY_BIT_16 != 0 {
        flags |= Flags::C;
    }
    (c as u16, flags)
}

/// 16-bit subtraction with carry
pub fn sbc_hl_rr(state: &mut State, reg: Register16, cycles: u8) -> ExecResult {
    let (hl, flags) = word_arithmetic_flags(
        state.hl(),
        state.get_register_16(reg),
        state.get_flags().is_set(Flags::C),
        u32::wrapping_sub,
    );
    *state.hl_mut() = hl.to_le_bytes();
    state.update_flags(flags);
    ExecResult::Done(cycles)
}

/// 16-bit addition with carry
pub fn adc_hl_rr(state: &mut State, reg: Register16, cycles: u8) -> ExecResult {
    let (hl, flags) = word_arithmetic_flags(
        state.hl(),
        state.get_register_16(reg),
        state.get_flags().is_set(Flags::C),
        u32::wrapping_add,
    );
    *state.hl_mut() = hl.to_le_bytes();
    state.update_flags(flags);
    ExecResult::Done(cycles)
}

/// Negation instruction
///
/// `A <- 0 - A`
pub fn neg(state: &mut State, cycles: u8) -> ExecResult {
    let (a, flags) = sub_flags(0, state.a(), false);
    *state.a_mut() = a;
    state.update_flags(flags);
    ExecResult::Done(cycles)
}

/// Perform a nybble rotate right between `Z` and the least significant nybble of `A`
pub fn rrd(state: &mut State) {
    let z = state.z();
    let a = state.a();
    *state.z_mut() = (z >> 4) | (a << 4);
    let a = (a & 0xf0) | (z & 0x0f);
    *state.a_mut() = a;
    let flags = (state.get_flags() & Flags::C) | Flags::from_value(a) | Flags::parity(a);
    state.update_flags(flags);
}

/// Perform a nybble rotate left between `Z` and the least significant nybble of `A`
pub fn rld(state: &mut State) {
    let z = state.z();
    let a = state.a();
    *state.z_mut() = z << 4 | (a & 0x0f);
    let a = a & 0xf0 | (z >> 4);
    *state.a_mut() = a;
    let flags = (state.get_flags() & Flags::C) | Flags::from_value(a) | Flags::parity(a);
    state.update_flags(flags);
}
