//! Microinstructions and related fun.
//!
//! This module contains functions related to implementing microinstructions. Microinstructions are
//! function pointers of type `fn(&mut State) -> ExecResult`.
//!
//! All microinstructions used by the emulator are exposed by the library user for reuse.
//!
//! Since most Z80 instructions can be implemented with a single microinstruction, the functions
//! that implement them are named similarly to Z80 instructions and their parameters. These
//! functions aren't microinstructions and should be wrapped in a function or lambda expression with
//! the right type.
//!
//! This is the naming scheme of instruction arguments used in functions:
//!
//! * `r` - an 8-bit register.
//! * `rr` - a 16-bit register.
//! * `n` - an immediate 8-bit value.   
//! * `nn` - an immediate 16-bit value.
//! * `pp` - a 16-bit register that contains a memory address.
//! * `mm` - an immediate value that is used as a memory address.
//! * `d` - an immediate offset added to some register
//! * `cc` - a boolean condition
//!
//! Some of these functions have a `cycles` parameter. This means the function will return
//! [[ExecResult::Done]] by itself. Otherwise, there should be another microinstruction that returns
//! `Done` afterward.

use crate::instructions::{DataLoader, ExecResult};
use crate::state::{Register16, State};

pub mod bit;
pub mod io;
pub mod jump;
pub mod ld;
pub mod math;
pub mod transfer;

/// A microinstruction is just a function that operates on the state and yields an execution result.
///
/// In other words, it performs a simple operation.
pub type Microinstruction = fn(&mut State) -> ExecResult;

/// Microinstruction to load one byte as instruction argument.
///
/// The [[ExecResult::Load]] this function returns reads a single byte that is stored in the
/// register `Z`.
///
/// This can be used by two byte instructions to load their arguments.
pub fn fetch_byte(state: &mut State) -> ExecResult<'_> {
    let pc = state.get_register_16(Register16::PC);
    state.advance_pc(1);
    load_8(state, pc)
}

/// Microinstruction to load a two byte word as instruction arguments.
///
/// The [[ExecResult::Load16]] this function returns reads two bytes from memory, that are stored
/// in the register `WZ`.
///
/// This can be used by three byte instructions to load their arguments.
pub fn fetch_word(state: &mut State) -> ExecResult<'_> {
    let pc = state.get_register_16(Register16::PC);
    state.advance_pc(2);
    load_16(state, pc)
}

/// Microinstruction component to load a byte of memory to the `Z` register.
pub fn load_8(state: &mut State, address: u16) -> ExecResult<'_> {
    let reg = state.z_mut();
    ExecResult::Load {
        address,
        loader: DataLoader(reg),
    }
}

/// Microinstruction component to load a word of memory to the `WZ` register.
pub fn load_16(state: &mut State, address: u16) -> ExecResult<'_> {
    let reg = state.wz_mut();
    ExecResult::Load16 {
        address,
        loader: DataLoader(reg),
    }
}

/// If the condition is true, load a 16 bit value to `WZ`. Otherwise, abort running the instruction
/// and return [[ExecResult::Done]],
pub fn load_16_or_break(state: &mut State, address: u16, cond: bool, cycles: u8) -> ExecResult<'_> {
    if cond {
        load_16(state, address)
    } else {
        state.skip_instruction();
        ExecResult::Done(cycles)
    }
}
