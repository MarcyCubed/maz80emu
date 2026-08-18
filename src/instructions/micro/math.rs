//! Math instructions

use crate::instructions::ExecResult;
use crate::state::{Flags, Register, Register16, State};

/// The bit representing the sign
const SIGN_BIT: u8 = 0b10000000;

/// Increment a 16-bit register
pub fn inc_rr(state: &mut State, register: Register16, cycles: u8) -> ExecResult<'_> {
    state.set_register_16(register, state.get_register_16(register).wrapping_add(1));
    ExecResult::Done(cycles)
}

/// Decrement a 16-bit register
pub fn dec_rr(state: &mut State, register: Register16, cycles: u8) -> ExecResult<'_> {
    state.set_register_16(register, state.get_register_16(register).wrapping_sub(1));
    ExecResult::Done(cycles)
}

/// Add two values and a carry-in together, getting the flags
///
/// Return the sum, the flags and the carry-out. The flags S, Z, V and H are updated. The C flag
/// isn't.
fn add_flags(a: u8, b: u8, carry_in: bool) -> (u8, Flags, bool) {
    let a16 = a as u16;
    let b16 = b as u16;
    // We'll add the numbers nybble by nybble
    let sum_low = (a16 & 0xF) + (b16 & 0xF) + carry_in as u16;
    let sum = (a16 & 0xF0) + (b16 & 0xF0) + sum_low;
    // Updates the S and Z flags
    let mut flags = Flags::from_value(sum as u8);
    // Half carry
    if sum_low > 0xF {
        flags |= Flags::H
    }
    // Overflow == if sign bit was overwritten
    if (a ^ b) & SIGN_BIT == 0 && (a ^ sum as u8) & SIGN_BIT != 0 {
        // Overflow
        flags |= Flags::V;
    }

    (sum as u8, flags, sum > u8::MAX as u16)
}

/// Subtract one value from the other
///
/// Return the difference, the flags and the borrow. The C flag is never set in flags
fn sub_flags(a: u8, b: u8, borrow_in: bool) -> (u8, Flags, bool) {
    let b = -(b as i8) as u8;
    let (sum, flags, borrow) = add_flags(a, b, borrow_in);
    // subtraction flag
    let flags = flags | Flags::N;
    (sum, flags, borrow)
}

/// Increment an 8-bit register
///
/// Return Done
pub fn inc_r(state: &mut State, register: Register, cycles: u8) -> ExecResult<'_> {
    let (inc, flags, _) = add_flags(state.get_register_8(register), 1, false);
    state.set_register_8(register, inc);
    // Old C flag || Computed new flags
    let flags = (state.get_flags() & Flags::C) | flags;
    state.update_flags(flags);
    ExecResult::Done(cycles)
}

/// Decrement an 8-bit register
///
/// Return Done
pub fn dec_r(state: &mut State, register: Register, cycles: u8) -> ExecResult<'_> {
    let (dec, flags, _) = sub_flags(state.get_register_8(register), 1, false);
    state.set_register_8(register, dec);
    // Old C flag || Computed new flags
    let flags = (state.get_flags() & Flags::C) | flags;
    state.update_flags(flags);
    ExecResult::Done(cycles)
}

/// Add a 16-bit value to HL
pub fn add_hl_rr(state: &mut State, register: Register16, cycles: u8) -> ExecResult<'_> {
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
    if (hl ^ other ^ result) & (1 << 12) != 0 {
        flags |= Flags::H;
    }
    state.update_flags(flags);
    // Store the result in HL
    *state.hl_mut() = hl.to_le_bytes();
    ExecResult::Done(cycles)
}

/// Adjust a BCD value after a math operation
pub fn daa(state: &mut State, cycles: u8) -> ExecResult<'_> {
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
    let a = if flags_0.is_set(Flags::N) {
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
pub fn cpl(state: &mut State, cycles: u8) -> ExecResult<'_> {
    *state.a_mut() = !state.a();
    let flags = state.get_flags() | Flags::N | Flags::H;
    state.update_flags(flags);
    ExecResult::Done(cycles)
}

/// Increment the contents of the `Z` register then store the result in the given address
pub fn inc_z_mem(state: &mut State, address: u16) -> ExecResult<'_> {
    inc_r(state, Register::Z, 0);
    ExecResult::Store {
        address,
        data: state.z(),
    }
}

/// Decrement the contents of the `Z` register then store the result in the given address
pub fn dec_z_mem(state: &mut State, address: u16) -> ExecResult<'_> {
    dec_r(state, Register::Z, 0);
    ExecResult::Store {
        address,
        data: state.z(),
    }
}

/// Set the carry flag
pub fn scf(state: &mut State, cycles: u8) -> ExecResult<'_> {
    let flags = (state.get_flags() | Flags::C) - (Flags::H | Flags::N);
    state.update_flags(flags);
    ExecResult::Done(cycles)
}

/// Complement the carry flag
pub fn ccf(state: &mut State, cycles: u8) -> ExecResult<'_> {
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
pub fn add_a_r(state: &mut State, reg: Register, cycles: u8) -> ExecResult<'_> {
    add_a_r_common(state, reg, false, cycles)
}

/// Add the value of a register and the existing carry to A
pub fn adc_a_r(state: &mut State, reg: Register, cycles: u8) -> ExecResult<'_> {
    add_a_r_common(state, reg, state.get_flags().is_set(Flags::C), cycles)
}

/// Common part between add_a_r and adc_a_r
fn add_a_r_common(state: &mut State, reg: Register, carry_in: bool, cycles: u8) -> ExecResult<'_> {
    let a = state.a();
    let n = state.get_register_8(reg);
    let (a, flags, c) = add_flags(a, n, carry_in);
    *state.a_mut() = a;
    let flags = if c { flags | Flags::C } else { flags };
    state.update_flags(flags);
    ExecResult::Done(cycles)
}

/// Common part between sub_r and sbc_r
fn sub_r_common(state: &mut State, reg: Register, carry_in: bool, cycles: u8) -> ExecResult<'_> {
    let a = state.a();
    let n = state.get_register_8(reg);
    let (a, flags, c) = sub_flags(a, n, carry_in);
    *state.a_mut() = a;
    let flags = if c { flags | Flags::C } else { flags } | Flags::N;
    state.update_flags(flags);
    ExecResult::Done(cycles)
}

/// Subtract the value of a register from A
pub fn sub_r(state: &mut State, reg: Register, cycles: u8) -> ExecResult<'_> {
    sub_r_common(state, reg, false, cycles)
}

/// Subtract the value of a register and the existing carry from A
pub fn sbc_r(state: &mut State, reg: Register, cycles: u8) -> ExecResult<'_> {
    sub_r_common(state, reg, state.get_flags().is_set(Flags::C), cycles)
}

/// Perform an `AND` operation between the register and the accumulator
pub fn and_r(state: &mut State, reg: Register, cycles: u8) -> ExecResult<'_> {
    let a = state.a() & state.get_register_8(reg);
    let flags = Flags::H | Flags::from_value(a) | Flags::parity(a);
    *state.a_mut() = a;
    state.update_flags(flags);
    ExecResult::Done(cycles)
}

/// Perform a `XOR` operation between the register and the accumulator
pub fn xor_r(state: &mut State, reg: Register, cycles: u8) -> ExecResult<'_> {
    let a = state.a() ^ state.get_register_8(reg);
    let flags = Flags::from_value(a) | Flags::parity(a);
    *state.a_mut() = a;
    state.update_flags(flags);
    ExecResult::Done(cycles)
}

/// Perform an `OR` operation between the register and the accumulator
pub fn or_r(state: &mut State, reg: Register, cycles: u8) -> ExecResult<'_> {
    let a = state.a() | state.get_register_8(reg);
    let flags = Flags::from_value(a) | Flags::parity(a);
    *state.a_mut() = a;
    state.update_flags(flags);
    ExecResult::Done(cycles)
}

/// Compare the accumulator and the register
pub fn cp_r(state: &mut State, reg: Register, cycles: u8) -> ExecResult<'_> {
    let (_, flags, carry_out) = sub_flags(state.a(), state.get_register_8(reg), false);
    state.update_flags(flags | Flags::C.set_if(carry_out) | Flags::N);
    ExecResult::Done(cycles)
}
