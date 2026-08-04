//! Microinstructions and related fun.

use crate::instructions::{DataLoader, ExecResult};
use crate::state::{Register16, State};

/// Microinstruction to load one byte as instruction argument.
///
/// The _Load_ this function returns reads a single byte to `state.instruction_arguments[0]`.
///
/// This can be used by two byte instructions to load their arguments.
pub fn fetch_byte(state: &mut State) -> ExecResult<'_> {
    let pc = state.get_register_16(Register16::PC);
    state.advance_pc(1);
    let argument = state.get_fetched_byte_mut();
    ExecResult::Load {
        address: pc,
        loader: DataLoader(argument),
    }
}

/// Microinstruction to load a two byte word as instruction arguments.
///
/// The _Load_ this function returns reads two bytes to `state.instruction_arguments`.
///
/// This can be used by three byte instructions to load their arguments.
pub fn fetch_word(state: &mut State) -> ExecResult<'_> {
    let pc = state.get_register_16(Register16::PC);
    state.advance_pc(2);
    let argument = state.get_fetched_word_mut();
    ExecResult::Load16 {
        address: pc,
        loader: DataLoader(argument),
    }
}
