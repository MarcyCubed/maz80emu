//! Jumps and other things that change the program counter

use crate::instructions::ExecResult;
use crate::state::{Flags, Register16, State};

/// Unconditional branch.
///
/// This can be used to implement any kind of branch with a one byte immediate operand by adding
/// the rest of the logic.
pub fn jr_d(state: &mut State, cycles: u32) -> ExecResult {
    let pc = state.pc().wrapping_add(state.z() as i8 as i16 as u16);
    *state.pc_mut() = pc.to_le_bytes();
    ExecResult::Done(cycles)
}

/// Decrement B and jump to immediate offset if not zero.
///
/// The number of cycles depends on if the instruction jumps or not.
pub fn djnz_d(state: &mut State, cycles_jump: u32, cycles_not_jump: u32) -> ExecResult {
    let b = state.b().wrapping_sub(1);
    *state.b_mut() = b;
    if b != 0 {
        jr_d(state, cycles_jump)
    } else {
        ExecResult::Done(cycles_not_jump)
    }
}

/// Jump if the zero flag isn't set
pub fn jr_nz_d(state: &mut State, cycles_jump: u32, cycles_not_jump: u32) -> ExecResult {
    if state.get_flags().is_set(Flags::Z) {
        ExecResult::Done(cycles_not_jump)
    } else {
        jr_d(state, cycles_jump)
    }
}

/// Jump if the zero flag is set
pub fn jr_z_d(state: &mut State, cycles_jump: u32, cycles_not_jump: u32) -> ExecResult {
    if state.get_flags().is_set(Flags::Z) {
        jr_d(state, cycles_jump)
    } else {
        ExecResult::Done(cycles_not_jump)
    }
}

/// Jump if the carry flag isn't set
pub fn jr_nc_d(state: &mut State, cycles_jump: u32, cycles_not_jump: u32) -> ExecResult {
    if state.get_flags().is_set(Flags::C) {
        ExecResult::Done(cycles_not_jump)
    } else {
        jr_d(state, cycles_jump)
    }
}

/// Jump if the carry flag is set
pub fn jr_c_d(state: &mut State, cycles_jump: u32, cycles_not_jump: u32) -> ExecResult {
    if state.get_flags().is_set(Flags::C) {
        jr_d(state, cycles_jump)
    } else {
        ExecResult::Done(cycles_not_jump)
    }
}

/// Pops the stack and jump to the value loaded into `WZ`.
///
/// This assumes the value pointed by `SP` was loaded into `WZ`
pub fn ret(state: &mut State, cycles: u32) -> ExecResult {
    // Jump
    *state.pc_mut() = state.wz_bytes();
    // Pop
    *state.sp_mut() = state.sp().wrapping_add(2).to_le_bytes();
    ExecResult::Done(cycles)
}

/// Pops the stack into the `WZ` 16-bit register
pub fn pop(state: &mut State) -> ExecResult {
    let address = state.sp();
    // Pop
    *state.sp_mut() = state.sp().wrapping_add(2).to_le_bytes();
    ExecResult::load16(address)
}

/// Push a 16-bit register into the stack into
pub fn push(state: &mut State, reg: Register16) -> ExecResult {
    let address = state.sp().wrapping_sub(2);
    *state.sp_mut() = address.to_le_bytes();

    ExecResult::Store16 {
        address,
        data: state.get_register_16_bytes(reg),
    }
}

/// Jump to the immediate value if the condition is true
pub fn jp_cc_nn(state: &mut State, cond: bool, cycles: u32) -> ExecResult {
    if cond {
        *state.pc_mut() = state.wz_bytes();
    }
    ExecResult::Done(cycles)
}

/// Push `PC` into the stack if the condition is true. Otherwise, finish instruction execution
pub fn push_pc_or_break(state: &mut State, cond: bool, cycles: u32) -> ExecResult {
    if cond {
        let sp = state.sp().wrapping_sub(2);
        *state.sp_mut() = sp.to_le_bytes();
        ExecResult::Store16 {
            address: sp,
            data: state.pc_bytes(),
        }
    } else {
        state.skip_instruction();
        ExecResult::Done(cycles)
    }
}

/// Jump to the immediate value
pub fn jr_mm(state: &mut State, cycles: u32) -> ExecResult {
    *state.pc_mut() = state.wz_bytes();
    ExecResult::Done(cycles)
}

/// Jump to the address
pub fn jp(state: &mut State, address: u16, cycles: u32) -> ExecResult {
    *state.pc_mut() = address.to_le_bytes();
    ExecResult::Done(cycles)
}
