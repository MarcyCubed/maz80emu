//! `LD` instructions

use crate::instructions::ExecResult;
use crate::state::{Register, Register16, State};

/// Load an immediate value into a 16 bits register
///
/// Returns `Done`
pub fn ld_rr_nn(state: &mut State, register: Register16, cycles: u8) -> ExecResult {
    state.set_register_16(register, state.wz());
    ExecResult::Done(cycles)
}

/// Store the value of r in the memory pointed by rr
///
/// Doesn't return `Done`
pub fn ld_pp_r(state: &State, reg16: Register16, reg8: Register) -> ExecResult {
    ExecResult::Store {
        address: state.get_register_16(reg16),
        data: state.get_register_8(reg8),
    }
}

/// Load an immediate value to an 8-bit register
///
/// Returns `Done`
pub fn ld_r_n(state: &mut State, register: Register, cycles: u8) -> ExecResult {
    state.set_register_8(register, state.z());
    ExecResult::Done(cycles)
}

/// Store the value of the 16-bit register in the immediate memory address
pub fn ld_mm_rr(state: &State, reg: Register16) -> ExecResult {
    let value = state.get_register_16_bytes(reg);
    ExecResult::Store16 {
        address: state.wz(),
        data: value,
    }
}

/// Store the value of the 8-bit register in the immediate memory address
pub fn ld_mm_r(state: &State, reg: Register) -> ExecResult {
    let value = state.get_register_8(reg);
    ExecResult::Store {
        address: state.wz(),
        data: value,
    }
}

/// Load the contents of a register into another
pub fn ld_r_r(state: &mut State, dst: Register, src: Register, cycles: u8) -> ExecResult {
    state.set_register_8(dst, state.get_register_8(src));
    ExecResult::Done(cycles)
}

/// Load the contents of a 16-bit register into another
pub fn ld_rr_rr(state: &mut State, dst: Register16, src: Register16, cycles: u8) -> ExecResult {
    state.set_register_16(dst, state.get_register_16(src));
    ExecResult::Done(cycles)
}
