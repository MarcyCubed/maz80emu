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

/// Load the value pointed by rr to r
pub fn ld_r_pp(state: &mut State, dest: Register, src: Register16) -> ExecResult {
    let address = state.get_register_16(src);
    state.load_byte_into(address, dest)
}

/// Store the value of the 16-bit register in the immediate memory address
pub fn ld_mm_rr(state: &State, reg: Register16) -> ExecResult {
    let value = state.get_register_16_bytes(reg);
    ExecResult::Store16 {
        address: state.wz(),
        data: value,
    }
}

/// Load the data pointed by the immediate address into a 16-bit register
pub fn ld_rr_mm(state: &mut State, reg: Register16) -> ExecResult {
    state.load_word_into(state.wz(), reg)
}

/// Store the value of the 8-bit register in the immediate memory address
pub fn ld_mm_r(state: &State, reg: Register) -> ExecResult {
    let value = state.get_register_8(reg);
    ExecResult::Store {
        address: state.wz(),
        data: value,
    }
}

/// Load the data pointed by the immediate address into an 8-bit register
pub fn ld_r_mm(state: &mut State, reg: Register) -> ExecResult {
    state.load_byte_into(state.wz(), reg)
}

/// Load the contents of a register into another
pub fn ld_r_r(state: &mut State, dst: Register, src: Register, cycles: u8) -> ExecResult {
    state.set_register_8(dst, state.get_register_8(src));
    ExecResult::Done(cycles)
}
