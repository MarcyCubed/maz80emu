use crate::instructions::ExecResult;
use crate::state::{Flags, InterruptMode, Register, State};

/// Write `A` to the port An
pub fn out_n_a(state: &mut State) -> ExecResult {
    let a = state.a();
    let port = (a as u16) << 8 | state.z() as u16;
    ExecResult::Out { port, data: a }
}

/// Perform the processing part of an `in r, (c)` instruction
///
/// The data received from the device should be in the `Z` register
pub fn in_r_bc(state: &mut State, reg: Register, cycles: u32) -> ExecResult {
    let c_flag = state.get_flags() & Flags::C;
    let data = state.z();
    state.update_flags(c_flag | Flags::from_value(data) | Flags::parity(data));
    state.set_register_8(reg, data);
    ExecResult::Done(cycles)
}

/// Sets the interrupt mode
pub fn im_n(state: &mut State, interrupt_mode: InterruptMode, cycles: u32) -> ExecResult {
    state.interrupt_mode = interrupt_mode;
    ExecResult::Done(cycles)
}
