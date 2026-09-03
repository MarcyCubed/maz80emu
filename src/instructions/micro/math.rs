//! Math instructions

use crate::instructions::ExecResult;
use crate::state::{Flags, Register, Register16, State};

/// The bit representing the sign
const SIGN_BIT: u32 = 7;

/// The bit that can receive a half carry
const HALF_CARRY_BIT: u32 = 4;

/// The carry bit out of u8 range
const CARRY_BIT: u32 = 8;

/// Increment a 16-bit register
pub fn inc_rr(state: &mut State, register: Register16, cycles: u32) -> ExecResult {
    state.set_register_16(register, state.get_register_16(register).wrapping_add(1));
    ExecResult::Done(cycles)
}

/// Decrement a 16-bit register
pub fn dec_rr(state: &mut State, register: Register16, cycles: u32) -> ExecResult {
    state.set_register_16(register, state.get_register_16(register).wrapping_sub(1));
    ExecResult::Done(cycles)
}

/// Check if the operation `a + b = c` had a carry on the specified bit.
fn check_carry(bit: u32, a: u16, b: u16, sum: u16) -> bool {
    (sum ^ a ^ b) & (1 << bit) != 0
}

/// Add two values and a carry-in together, getting the sum and the flags
pub(crate) fn add_flags(a: u8, b: u8, carry_in: bool) -> (u8, Flags) {
    let a = a as u16;
    let b = b as u16;
    // Do the math
    let c = u16::wrapping_add(u16::wrapping_add(a, b), carry_in as u16);
    // Updates the S and Z flags
    let mut flags = Flags::from_value(c as u8);
    // Half carry
    flags |= Flags::H.set_if(check_carry(HALF_CARRY_BIT, a, b, c));
    // Overflow == if sign bit was overwritten
    flags |= Flags::V.set_if(check_carry(SIGN_BIT, a, b, c) != check_carry(CARRY_BIT, a, b, c));
    // Carry
    flags |= Flags::C.set_if(c & (1 << CARRY_BIT) != 0);
    (c as u8, flags)
}

/// Subtract one value and the carry from another, getting the difference and the flags
pub(crate) fn sub_flags(a: u8, b: u8, borrow_in: bool) -> (u8, Flags) {
    let (result, flags) = add_flags(a, !b, !borrow_in);
    (result, flags.flip(Flags::C | Flags::H) | Flags::N)
}

/// Increment an 8-bit register
///
/// Return Done
pub fn inc_r(state: &mut State, register: Register, cycles: u32) -> ExecResult {
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
pub fn dec_r(state: &mut State, register: Register, cycles: u32) -> ExecResult {
    let (dec, flags) = sub_flags(state.get_register_8(register), 1, false);
    state.set_register_8(register, dec);
    // Old C flag || Computed new flags
    let flags = (state.get_flags() & Flags::C) | (flags - Flags::C);
    state.update_flags(flags);
    ExecResult::Done(cycles)
}

/// Adds two 16-bit numbers and the carry together, returning the sum and the flags
fn add_16_flags(a: u16, b: u16, carry_in: bool) -> (u16, Flags) {
    let a = a.to_le_bytes();
    let b = b.to_le_bytes();
    let (low, flags) = add_flags(a[0], b[0], carry_in);
    let (high, flags) = add_flags(a[1], b[1], flags.is_set(Flags::C));
    let result = u16::from_le_bytes([low, high]);
    let flags = flags.reset(Flags::Z) | Flags::Z.set_if(result == 0);
    (u16::from_le_bytes([low, high]), flags)
}

/// Subtract one 16-bit value and the carry from another, getting the difference and the flags
fn sub_16_flags(a: u16, b: u16, carry_in: bool) -> (u16, Flags) {
    let (result, flags) = add_16_flags(a, !b, !carry_in);
    (result, flags.flip(Flags::C | Flags::H) | Flags::N)
}

/// Add two 16-bit registers together
pub fn add_rr_rr(state: &mut State, a: Register16, b: Register16, cycles: u32) -> ExecResult {
    let value = state.get_register_16(a);
    state.memptr = value.wrapping_add(1);
    let (result, flags) = add_16_flags(value, state.get_register_16(b), false);
    // Only use the flags C, H, X and Y
    let flags = flags & (Flags::C | Flags::H | Flags::X | Flags::Y);
    let old_flags = state.get_flags() & (Flags::S | Flags::Z | Flags::V);
    state.update_flags(flags | old_flags);
    // Store the result back in the accumulator register
    state.set_register_16(a, result);
    ExecResult::Done(cycles)
}

/// Adjust a BCD value after a math operation
pub fn daa(state: &mut State, cycles: u32) -> ExecResult {
    let a = state.a();
    let flags = state.get_flags();

    let mut diff = 0;
    if flags.is_set(Flags::H) || a & 0x0f > 9 {
        diff = 0x06;
    }
    if flags.is_set(Flags::C) || a > 0x99 {
        diff += 0x60;
    }

    let daa = if flags.is_set(Flags::N) {
        a.wrapping_sub(diff)
    } else {
        a.wrapping_add(diff)
    };

    *state.a_mut() = daa;
    state.update_flags(
        flags.select(Flags::N | Flags::C)
            | Flags::from_value(daa)
            | Flags::C.set_if(a > 0x99)
            | Flags::H.set_if((a ^ daa) & (1 << 4) != 0)
            | Flags::parity(daa),
    );
    ExecResult::Done(cycles)
}

/// Complement of the accumulator.
///
/// `!a`
pub fn cpl(state: &mut State, cycles: u32) -> ExecResult {
    let complement = !state.a();
    *state.a_mut() = complement;
    let flags = state
        .get_flags()
        .select(Flags::C | Flags::P | Flags::Z | Flags::S)
        | Flags::N
        | Flags::H
        | Flags::xy(complement);
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
pub fn scf(state: &mut State, cycles: u32) -> ExecResult {
    let flags =
        state.get_flags().select(Flags::S | Flags::Z | Flags::P) | Flags::C | Flags::xy(state.a());
    state.update_flags(flags);
    ExecResult::Done(cycles)
}

/// Complement the carry flag
pub fn ccf(state: &mut State, cycles: u32) -> ExecResult {
    let old_flags = state.get_flags();
    let flags = old_flags.select(Flags::S | Flags::Z | Flags::P)
        | Flags::H.set_if(old_flags.is_set(Flags::C))
        | Flags::C.set_if(!old_flags.is_set(Flags::C))
        | Flags::xy(state.a());
    state.update_flags(flags);
    ExecResult::Done(cycles)
}

/// Add the value of a register to A
pub fn add_a_r(state: &mut State, reg: Register, cycles: u32) -> ExecResult {
    add_a_r_common(state, reg, false, cycles)
}

/// Add the value of a register and the existing carry to A
pub fn adc_a_r(state: &mut State, reg: Register, cycles: u32) -> ExecResult {
    add_a_r_common(state, reg, state.get_flags().is_set(Flags::C), cycles)
}

/// Common part between add_a_r and adc_a_r
fn add_a_r_common(state: &mut State, reg: Register, carry_in: bool, cycles: u32) -> ExecResult {
    let a = state.a();
    let n = state.get_register_8(reg);
    let (a, flags) = add_flags(a, n, carry_in);
    //println!(" a = {:x}h", a);
    *state.a_mut() = a;
    state.update_flags(flags);
    ExecResult::Done(cycles)
}

/// Common part between sub_r and sbc_r
fn sub_r_common(state: &mut State, reg: Register, carry_in: bool, cycles: u32) -> ExecResult {
    let a = state.a();
    let n = state.get_register_8(reg);
    let (a, flags) = sub_flags(a, n, carry_in);
    //println!(" a = {:x}h  x = {:x}h  ", a, n);
    *state.a_mut() = a;
    state.update_flags(flags);
    ExecResult::Done(cycles)
}

/// Subtract the value of a register from A
pub fn sub_r(state: &mut State, reg: Register, cycles: u32) -> ExecResult {
    sub_r_common(state, reg, false, cycles)
}

/// Subtract the value of a register and the existing carry from A
pub fn sbc_r(state: &mut State, reg: Register, cycles: u32) -> ExecResult {
    sub_r_common(state, reg, state.get_flags().is_set(Flags::C), cycles)
}

/// Perform an `AND` operation between the register and the accumulator
pub fn and_r(state: &mut State, reg: Register, cycles: u32) -> ExecResult {
    let a = state.a() & state.get_register_8(reg);
    let flags = Flags::H | Flags::from_value(a) | Flags::parity(a);
    *state.a_mut() = a;
    state.update_flags(flags);
    ExecResult::Done(cycles)
}

/// Perform a `XOR` operation between the register and the accumulator
pub fn xor_r(state: &mut State, reg: Register, cycles: u32) -> ExecResult {
    let a = state.a() ^ state.get_register_8(reg);
    let flags = Flags::from_value(a) | Flags::parity(a);
    *state.a_mut() = a;
    state.update_flags(flags);
    ExecResult::Done(cycles)
}

/// Perform an `OR` operation between the register and the accumulator
pub fn or_r(state: &mut State, reg: Register, cycles: u32) -> ExecResult {
    let a = state.a() | state.get_register_8(reg);
    let flags = Flags::from_value(a) | Flags::parity(a);
    *state.a_mut() = a;
    state.update_flags(flags);
    ExecResult::Done(cycles)
}

/// Compare the accumulator and the register
pub fn cp_r(state: &mut State, reg: Register, cycles: u32) -> ExecResult {
    //println!(" a = {:x}h  x = {:x}h  ", state.a(), state.get_register_8(reg));
    let value = state.get_register_8(reg);
    let (_, flags) = sub_flags(state.a(), value, false);
    let flags = flags - Flags::X - Flags::Y;
    state.update_flags(flags | Flags::xy(value));
    ExecResult::Done(cycles)
}

/// 16-bit subtraction with carry
pub fn sbc_hl_rr(state: &mut State, reg: Register16, cycles: u32) -> ExecResult {
    state.memptr = state.hl().wrapping_add(1);
    let (hl, flags) = sub_16_flags(
        state.hl(),
        state.get_register_16(reg),
        state.get_flags().is_set(Flags::C),
    );
    *state.hl_mut() = hl.to_le_bytes();
    state.update_flags(flags);
    ExecResult::Done(cycles)
}

/// 16-bit addition with carry
pub fn adc_hl_rr(state: &mut State, reg: Register16, cycles: u32) -> ExecResult {
    state.memptr = state.hl().wrapping_add(1);
    let (hl, flags) = add_16_flags(
        state.hl(),
        state.get_register_16(reg),
        state.get_flags().is_set(Flags::C),
    );
    *state.hl_mut() = hl.to_le_bytes();
    state.update_flags(flags);
    ExecResult::Done(cycles)
}

/// Negation instruction
///
/// `A <- 0 - A`
pub fn neg(state: &mut State, cycles: u32) -> ExecResult {
    let (a, flags) = sub_flags(0, state.a(), false);
    *state.a_mut() = a;
    state.update_flags(flags);
    ExecResult::Done(cycles)
}

/// Perform a nybble rotate right between `Z` and the least significant nybble of `A`
pub fn rrd(state: &mut State) {
    state.memptr = state.hl().wrapping_add(1);
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
    state.memptr = state.hl().wrapping_add(1);
    let z = state.z();
    let a = state.a();
    *state.z_mut() = z << 4 | (a & 0x0f);
    let a = a & 0xf0 | (z >> 4);
    *state.a_mut() = a;
    let flags = (state.get_flags() & Flags::C) | Flags::from_value(a) | Flags::parity(a);
    state.update_flags(flags);
}
