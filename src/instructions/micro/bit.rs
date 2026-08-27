//! Bit operations

use crate::instructions::ExecResult;
use crate::state::{Flags, State};

/// Rotate the accumulator left and copy the original most significant bit to the carry flag
pub fn rlca(state: &mut State, cycles: u32) -> ExecResult {
    let a = state.a();
    let a = a.rotate_left(1);
    // Flags S, Z and V are unchanged
    let mut flags = state.get_flags() & (Flags::S | Flags::Z | Flags::V);
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
    let mut flags = state.get_flags() & (Flags::S | Flags::Z | Flags::V);
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
    let new_c_flag = if acc & 0b10000000 != 0 {
        Flags::C
    } else {
        Flags::new()
    };
    // Rotate
    let a = acc << 1;
    let a = a | state.get_flags().is_set(Flags::C) as u8;
    // Flags S, Z and V are unchanged
    let flags = state.get_flags() & (Flags::S | Flags::Z | Flags::V);
    // New C flag is the old lsb
    let flags = flags | new_c_flag;

    *state.a_mut() = a;
    state.update_flags(flags);
    ExecResult::Done(cycles)
}

/// Rotate the 9-bit value composed by the C flag and the accumulator to the right
pub fn rra(state: &mut State, cycles: u32) -> ExecResult {
    let acc = state.a();
    // The LSB of the accumulator will move to the C flag
    let new_c_flag = if acc & 1 != 0 { Flags::C } else { Flags::new() };
    // Rotate
    let a = acc >> 1;
    let a = a | ((state.get_flags().is_set(Flags::C) as u8) << 7);
    // Flags S, Z and V are unchanged
    let flags = state.get_flags() & (Flags::S | Flags::Z | Flags::V);
    // New C flag is the old MSB
    let flags = flags | new_c_flag;

    *state.a_mut() = a;
    state.update_flags(flags);
    ExecResult::Done(cycles)
}
