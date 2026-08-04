//! The core of the emulator

use crate::instructions::decoder::Decoder;
use crate::instructions::{ExecResult, InstructionSet, Microinstruction};
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
    /// Which microinstruction is being executed
    mpc: usize,
}

impl Emulator {
    /// Create an emulator to run programs for the instruction set
    pub fn new_with_instruction_set(instruction_set: &'static InstructionSet) -> Emulator {
        Emulator {
            state: Default::default(),
            decoder: Decoder::new(instruction_set),
            micros: &[],
            mpc: 0,
        }
    }

    /// Runs the emulator
    pub fn run(&mut self) -> ExecResult<'_> {
        // If we're running microinstructions
        if self.mpc < self.micros.len() {
            let micro = self.micros[self.mpc];
            self.mpc += 1;
            micro(&mut self.state)
        } else {
            // Need to decode the next instruction
            self.micros = self.decoder.decode(&mut self.state);
            self.mpc = 1; // We'll run the [0] now.
            self.micros[0](&mut self.state)
        }
    }
}
