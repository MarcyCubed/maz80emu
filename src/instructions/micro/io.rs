use crate::instructions::ExecResult;
use crate::state::State;

/// Write `A` to the port An
pub fn out_n_a(state: &mut State) -> ExecResult {
    let a = state.a();
    let port = (a as u16) << 8 | state.z() as u16;
    ExecResult::Out { port, data: a }
}
