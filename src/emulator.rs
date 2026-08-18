//! The core of the emulator

use crate::instructions::decoder::Decoder;
use crate::instructions::micro::Microinstruction;
use crate::instructions::{ExecResult, InstructionSet};
use crate::state::State;

/// The Z80 emulator
#[derive(Debug, Clone)]
pub struct Emulator {
    /// Internal state of the processor
    pub state: State,
    /// The instruction decoder
    decoder: Decoder,
    /// The microinstructions being executed
    micros: &'static [Microinstruction],
}

impl Emulator {
    /// Create an emulator to run programs for the instruction set
    pub fn new_with_instruction_set(instruction_set: &'static InstructionSet) -> Emulator {
        Emulator {
            state: Default::default(),
            decoder: Decoder::new(instruction_set),
            micros: &[],
        }
    }

    /// Runs the emulator
    pub fn run(&mut self) -> ExecResult<'_> {
        // If we're running microinstructions
        if self.state.mpc < self.micros.len() {
            let micro = self.micros[self.state.mpc];
            self.state.mpc += 1;
            micro(&mut self.state)
        } else {
            // Need to decode the next instruction
            self.micros = self.decoder.decode(&mut self.state);
            self.state.mpc = 1; // We'll run the [0] now.
            self.micros[0](&mut self.state)
        }
    }
}
