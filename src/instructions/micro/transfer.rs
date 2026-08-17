use crate::instructions::ExecResult;
use crate::state::{Register16, State};

/// Switch the data of AF and AF'
pub fn ex_af_af(state: &mut State, cycles: u8) -> ExecResult<'_> {
    let af = state.af_bytes();
    *state.af_mut() = state.alternate[Register16::AF as usize];
    state.alternate[Register16::AF as usize] = af;
    ExecResult::Done(cycles)
}
