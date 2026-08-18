use crate::instructions::ExecResult;
use crate::state::{Register, State};

/// Write `A` to the port An
pub fn out_n_a(state: &mut State) -> ExecResult {
    let a = state.a();
    let port = (a as u16) << 8 | state.z() as u16;
    ExecResult::Out { port, data: a }
}

/// Read a byte from port `An` to `A`
pub fn in_n_a(state: &mut State) -> ExecResult {
    let a = state.a();
    let port = (a as u16) << 8 | state.z() as u16;
    state.input_into(port, Register::A)
}
