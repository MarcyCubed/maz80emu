//! Jumps and other things that change the program counter

use crate::instructions::ExecResult;
use crate::state::{Flags, State};

/// Unconditional branch.
///
/// This can be used to implement any kind of branch with a one byte immediate operand by adding
/// the rest of the logic.
pub fn jr_d(state: &mut State, cycles: u8) -> ExecResult<'_> {
    let pc = state.pc().wrapping_add(state.z() as i8 as i16 as u16);
    *state.pc_mut() = pc.to_le_bytes();
    ExecResult::Done(cycles)
}

/// Decrement B and jump to immediate offset if not zero.
///
/// The number of cycles depends on if the instruction jumps or not.
pub fn djnz_d(state: &mut State, cycles_jump: u8, cycles_not_jump: u8) -> ExecResult<'_> {
    let b = state.b().wrapping_sub(1);
    *state.b_mut() = b;
    if b != 0 {
        jr_d(state, cycles_jump)
    } else {
        ExecResult::Done(cycles_not_jump)
    }
}

/// Jump if the zero flag isn't set
pub fn jr_nz_d(state: &mut State, cycles_jump: u8, cycles_not_jump: u8) -> ExecResult<'_> {
    if state.get_flags().is_set(Flags::Z) {
        ExecResult::Done(cycles_not_jump)
    } else {
        jr_d(state, cycles_jump)
    }
}

/// Jump if the zero flag is set
pub fn jr_z_d(state: &mut State, cycles_jump: u8, cycles_not_jump: u8) -> ExecResult<'_> {
    if state.get_flags().is_set(Flags::Z) {
        jr_d(state, cycles_jump)
    } else {
        ExecResult::Done(cycles_not_jump)
    }
}

/// Jump if the carry flag isn't set
pub fn jr_nc_d(state: &mut State, cycles_jump: u8, cycles_not_jump: u8) -> ExecResult<'_> {
    if state.get_flags().is_set(Flags::C) {
        ExecResult::Done(cycles_not_jump)
    } else {
        jr_d(state, cycles_jump)
    }
}

/// Jump if the carry flag is set
pub fn jr_c_d(state: &mut State, cycles_jump: u8, cycles_not_jump: u8) -> ExecResult<'_> {
    if state.get_flags().is_set(Flags::C) {
        jr_d(state, cycles_jump)
    } else {
        ExecResult::Done(cycles_not_jump)
    }
}
