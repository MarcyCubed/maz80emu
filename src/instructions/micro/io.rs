use crate::instructions::{DataLoader, ExecResult};
use crate::state::State;

/// Write `A` to the port An
pub fn out_n_a(state: &mut State) -> ExecResult<'_> {
    let a = state.a();
    let port = (a as u16) << 8 | state.z() as u16;
    ExecResult::Out { port, data: a }
}

/// Read a byte from port `An` to `A`
pub fn in_n_a(state: &mut State) -> ExecResult<'_> {
    let a = state.a();
    let port = (a as u16) << 8 | state.z() as u16;
    ExecResult::In {
        port,
        loader: DataLoader(state.a_mut()),
    }
}
