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

use crate::instructions::ExecResult;
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
/// The result is stored in the `Z` register.
pub fn fetch(state: &mut State) -> ExecResult {
    let pc = state.get_register_16(Register16::PC);
    state.advance_pc(1);
    // Advance the R register
    let r = state.r();
    *state.r_mut() = (r.wrapping_add(1) & !(1 << 7)) | r & 1 << 7;
    ExecResult::fetch(pc)
}

/// Load the parameter for a two byte instruction.
///
/// The result is stored in the `Z` register.
pub fn load_byte_parameter(state: &mut State) -> ExecResult {
    let pc = state.get_register_16(Register16::PC);
    state.advance_pc(1);
    ExecResult::load(pc)
}

/// Loads the parameter for a three byte instruction
///
/// The result is stored in the `WZ` register.
pub fn load_word_parameter(state: &mut State) -> ExecResult {
    let pc = state.get_register_16(Register16::PC);
    state.advance_pc(2);
    ExecResult::load16(pc)
}

/// If the condition is true, load a 16 bit value to `WZ`. Otherwise, abort running the instruction
/// and return [[ExecResult::Done]],
pub fn load_16_or_break(state: &mut State, address: u16, cond: bool, cycles: u32) -> ExecResult {
    if cond {
        ExecResult::load16(address)
    } else {
        state.skip_instruction();
        ExecResult::Done(cycles)
    }
}

/// Microinstruction component to write a 16-bit value to a memory location
pub fn store_16(address: u16, data: u16) -> ExecResult {
    ExecResult::Store16 {
        address,
        data: data.to_le_bytes(),
    }
}
