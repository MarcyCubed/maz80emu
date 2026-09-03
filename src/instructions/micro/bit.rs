//! Bit operations

use crate::instructions::ExecResult;
use crate::state::{Flags, Register, State};

/// Rotate the accumulator left and copy the original most significant bit to the carry flag
pub fn rlca(state: &mut State, cycles: u32) -> ExecResult {
    let a = state.a();
    let a = a.rotate_left(1);
    // Flags S, Z and V are unchanged
    let mut flags = state.get_flags().select(Flags::S | Flags::Z | Flags::V);
    flags |= Flags::xy(a);
    // Flag C is the bit 7 (now moved to bit 0) of the accumulator
    if a & 1 << 0 != 0 {
        flags |= Flags::C;
    }
    *state.a_mut() = a;
    state.update_flags(flags);
    ExecResult::Done(cycles)
}

/// Rotate the accumulator right and copy the original least significant bit to the carry flag
pub fn rrca(state: &mut State, cycles: u32) -> ExecResult {
    let a = state.a();
    let a = a.rotate_right(1);
    // Flags S, Z and V are unchanged
    let mut flags = state.get_flags().select(Flags::S | Flags::Z | Flags::V);
    flags |= Flags::xy(a);
    // Flag C is the bit 0 (now moved to bit 7) of the accumulator
    if a & 1 << 7 != 0 {
        flags |= Flags::C;
    }
    *state.a_mut() = a;
    state.update_flags(flags);
    ExecResult::Done(cycles)
}

/// Rotate the 9-bit value composed by the C flag and the accumulator to the left
pub fn rla(state: &mut State, cycles: u32) -> ExecResult {
    let acc = state.a();
    // The MSB of the accumulator will move to the C flag
    let new_c_flag = Flags::C.set_if(acc & 0b10000000 != 0);
    // Rotate
    let a = acc << 1;
    let a = a | state.get_flags().is_set(Flags::C) as u8;
    // Flags S, Z and V are unchanged
    let flags = state.get_flags().select(Flags::S | Flags::Z | Flags::V)
        | new_c_flag // New C flag is the old MSB
        | Flags::xy(a);

    *state.a_mut() = a;
    state.update_flags(flags);
    ExecResult::Done(cycles)
}

/// Rotate the 9-bit value composed by the C flag and the accumulator to the right
pub fn rra(state: &mut State, cycles: u32) -> ExecResult {
    let acc = state.a();
    // The LSB of the accumulator will move to the C flag
    let new_c_flag = Flags::C.set_if(acc & 1 != 0);
    // Rotate
    let a = acc >> 1;
    let a = a | ((state.get_flags().is_set(Flags::C) as u8) << 7);
    // Flags S, Z and V are unchanged
    let flags = state.get_flags().select(Flags::S | Flags::Z | Flags::V)
        |new_c_flag // New C flag is the old MSB
        |Flags::xy(a);

    *state.a_mut() = a;
    state.update_flags(flags);
    ExecResult::Done(cycles)
}

/// Rotate the register left and copy the original most significant bit to the carry flag, updating
/// the flags.
pub fn rlc_r(state: &mut State, reg: Register, cycles: u32) -> ExecResult {
    let value = state.get_register_8(reg).rotate_left(1);
    state.set_register_8(reg, value);
    state.update_flags(
        Flags::from_value(value) | Flags::parity(value) | Flags::C.set_if(value & 0x1 != 0),
    );
    ExecResult::Done(cycles)
}

/// Rotate the register right and copy the original most significant bit to the carry flag, updating
/// the flags.
pub fn rrc_r(state: &mut State, reg: Register, cycles: u32) -> ExecResult {
    let value = state.get_register_8(reg).rotate_right(1);
    state.set_register_8(reg, value);
    state.update_flags(
        Flags::from_value(value) | Flags::parity(value) | Flags::C.set_if(value & 0b10000000 != 0),
    );
    ExecResult::Done(cycles)
}

/// Rotate left the 9-bit virtual register composed by the carry flag and the specified register.
pub fn rl_r(state: &mut State, reg: Register, cycles: u32) -> ExecResult {
    let value = state.get_register_8(reg);
    let carry = Flags::C.set_if(value & 0b10000000 != 0);
    let value = (value << 1) | (state.get_flags().is_set(Flags::C) as u8);
    state.set_register_8(reg, value);
    state.update_flags(Flags::from_value(value) | Flags::parity(value) | carry);
    ExecResult::Done(cycles)
}

/// Rotate right the 9-bit virtual register composed by the specified register and the carry flag.
pub fn rr_r(state: &mut State, reg: Register, cycles: u32) -> ExecResult {
    let value = state.get_register_8(reg);
    let carry = Flags::C.set_if(value & 0x1 != 0);
    let value = (value >> 1) | ((state.get_flags().is_set(Flags::C) as u8) << 7);
    state.set_register_8(reg, value);
    state.update_flags(Flags::from_value(value) | Flags::parity(value) | carry);
    ExecResult::Done(cycles)
}

/// Common core for shifts
///
/// `shift_func` is the function that actually performs the shift, `lost_bit` is the number of the
/// bit that falls out of the value
fn shift_common(
    state: &mut State,
    reg: Register,
    shift_func: fn(u8) -> u8,
    lost_bit: u32,
    cycles: u32,
) -> ExecResult {
    let value = state.get_register_8(reg);
    let carry = Flags::C.set_if(value & (1 << lost_bit) != 0);
    let value = shift_func(value);
    state.set_register_8(reg, value);
    state.update_flags(Flags::from_value(value) | Flags::parity(value) | carry);
    ExecResult::Done(cycles)
}

/// Arithmetic left shift
pub fn sla_r(state: &mut State, reg: Register, cycles: u32) -> ExecResult {
    shift_common(state, reg, |n| n << 1, 7, cycles)
}

/// Arithmetic right shift
pub fn sra_r(state: &mut State, reg: Register, cycles: u32) -> ExecResult {
    shift_common(state, reg, |n| (n as i8 >> 1) as u8, 0, cycles)
}

/// Logical left shift
///
/// This is an undocumented instruction
pub fn sll_r(state: &mut State, reg: Register, cycles: u32) -> ExecResult {
    shift_common(state, reg, |n| (n << 1) | 1, 7, cycles)
}

/// Logical right shift
pub fn srl_r(state: &mut State, reg: Register, cycles: u32) -> ExecResult {
    shift_common(state, reg, |n| n >> 1, 0, cycles)
}

/// Check if a bit is reset
///
/// If the bit is `0`, sets the `Z` flag
pub fn bit_r(state: &mut State, reg: Register, bit_number: u32, cycles: u32) -> ExecResult {
    let value = state.get_register_8(reg);
    let bit = value & (1 << bit_number);
    let flags = state.get_flags().select(Flags::C)
        | Flags::from_value(bit)
        | Flags::xy(value)
        | Flags::parity(bit)
        | Flags::H;
    state.update_flags(flags);
    ExecResult::Done(cycles)
}

/// Reset a bit
pub fn res_r(state: &mut State, reg: Register, bit_number: u32, cycles: u32) -> ExecResult {
    let value = state.get_register_8(reg);
    state.set_register_8(reg, value & !(1 << bit_number));
    ExecResult::Done(cycles)
}

/// Set a bit
pub fn set_r(state: &mut State, reg: Register, bit_number: u32, cycles: u32) -> ExecResult {
    let value = state.get_register_8(reg);
    state.set_register_8(reg, value | (1 << bit_number));
    ExecResult::Done(cycles)
}
