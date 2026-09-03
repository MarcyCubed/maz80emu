//! Jumps and other things that change the program counter

use crate::instructions::ExecResult;
use crate::state::{Register16, State};

/// Jump to a relative address if the condition is true
///
/// This also sets `MEMPTR` to the address even if the jump doesn't happen
/// Jump if the zero flag isn't set
pub fn jr_cc_d(
    state: &mut State,
    cond: bool,
    cycles_jump: u32,
    cycles_not_jump: u32,
) -> ExecResult {
    let new_pc = state.pc().wrapping_add(state.z() as i8 as i16 as u16);
    state.memptr = new_pc;
    if cond {
        *state.pc_mut() = new_pc.to_le_bytes();
        ExecResult::Done(cycles_jump)
    } else {
        ExecResult::Done(cycles_not_jump)
    }
}

/// Decrement B and jump to immediate offset if not zero.
///
/// The number of cycles depends on if the instruction jumps or not.
pub fn djnz_d(state: &mut State, cycles_jump: u32, cycles_not_jump: u32) -> ExecResult {
    let b = state.b().wrapping_sub(1);
    *state.b_mut() = b;
    jr_cc_d(state, b != 0, cycles_jump, cycles_not_jump)
}

/// If the condition is true, the value pointed by sp is loaded to `WZ`. Otherwise, abort running
/// the instruction and return [[ExecResult::Done]].
pub fn load_sp_or_break(state: &mut State, cond: bool, cycles: u32) -> ExecResult {
    let address = state.sp();
    if cond {
        ExecResult::load16(address)
    } else {
        state.skip_instruction();
        ExecResult::Done(cycles)
    }
}

/// Pops the stack and jump to the value loaded into `WZ`.
///
/// This assumes the value pointed by `SP` was loaded into `WZ`
pub fn ret(state: &mut State, cycles: u32) -> ExecResult {
    state.memptr = state.wz();
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
    state.memptr = state.wz();
    if cond {
        *state.pc_mut() = state.wz_bytes();
    }
    ExecResult::Done(cycles)
}

/// Push `PC` into the stack if the condition is true. Otherwise, finish instruction execution
///
/// It also sets `MEMPTR` to `WZ`
pub fn push_pc_or_break(state: &mut State, cond: bool, cycles: u32) -> ExecResult {
    state.memptr = state.wz();
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
    state.memptr = state.wz();
    ExecResult::Done(cycles)
}

/// Jump to the address
pub fn jp(state: &mut State, address: u16, cycles: u32) -> ExecResult {
    *state.pc_mut() = address.to_le_bytes();
    ExecResult::Done(cycles)
}
