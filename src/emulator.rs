//! The core of the emulator

use crate::cpus::z80::Z80;
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

    /// Create a Z80 emulator
    pub fn new_z80() -> Self {
        Self::new_with_instruction_set(&Z80)
    }

    /// Runs the emulator
    pub fn run(&mut self) -> ExecResult {
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

    /// Run the emulator with a given memory, optionally disabling automatic handling of any
    /// operation.
    ///
    /// If the requested address is within the memory slice and `trap` returns `false`, the memory
    /// request will be processed automatically.
    /// Otherwise, the load or store request will be returned as in [[Emulator::run]].
    pub fn run_with_memory_trap<F: Fn(ExecResult) -> bool>(
        &mut self,
        memory: &mut [u8],
        trap: F,
    ) -> ExecResult {
        loop {
            let result = self.run();
            if trap(result) {
                return result;
            }
            match result {
                ExecResult::Load { address } | ExecResult::Fetch { address } => {
                    match memory.get(address as usize).copied() {
                        None => return result,
                        Some(data) => {
                            self.state.load_data_8(data);
                        }
                    }
                }
                ExecResult::Load16 { address } => {
                    let address_0 = address as usize;
                    let address_1 = address_0.wrapping_add(1);
                    if address_0 >= memory.len() || address_1 >= memory.len() {
                        return result;
                    } else {
                        self.state
                            .load_data_16([memory[address_0], memory[address_1]]);
                    }
                }
                ExecResult::Store { address, data } => match memory.get_mut(address as usize) {
                    None => return result,
                    Some(cell) => *cell = data,
                },
                ExecResult::Store16 { address, data } => {
                    let address_0 = address as usize;
                    let address_1 = address_0.wrapping_add(1);
                    if address_0 >= memory.len() || address_1 >= memory.len() {
                        return result;
                    } else {
                        memory[address_0] = data[0];
                        memory[address_1] = data[1];
                    }
                }
                _ => return result,
            }
        }
    }

    /// Run the emulator with memory.
    ///
    /// Handle memory access for the emulator. If the requested address is out of bounds, the
    /// load or store request will be returned as in [[Emulator::run]].
    pub fn run_with_memory(&mut self, memory: &mut [u8]) -> ExecResult {
        self.run_with_memory_trap(memory, |_| false)
    }

    /// Run the emulator with 64 kilobytes of memory.
    pub fn run_with_full_memory(&mut self, memory: &mut [u8; 0x10000]) -> ExecResult {
        loop {
            let result = self.run();
            match result {
                ExecResult::Load { address } | ExecResult::Fetch { address } => {
                    self.state.load_data_8(memory[address as usize])
                }
                ExecResult::Load16 { address } => {
                    let address = address as usize;
                    self.state
                        .load_data_16([memory[address], memory[address + 1]]);
                }
                ExecResult::Store { address, data } => {
                    memory[address as usize] = data;
                }
                ExecResult::Store16 { address, data } => {
                    let address = address as usize;
                    memory[address] = data[0];
                    memory[address + 1] = data[1];
                }
                _ => return result,
            }
        }
    }

    /// Pass a byte to the emulator to complete a memory or input instruction
    pub fn send_byte(&mut self, byte: u8) {
        self.state.load_data_8(byte)
    }

    /// Pass a word to the emulator to complete a memory load
    pub fn send_word(&mut self, word: [u8; 2]) {
        self.state.load_data_16(word)
    }

    /// Show the instructions as they are executed
    pub fn enable_tracing(&mut self) {
        self.decoder.enable_tracing();
    }

    /// Stop showing the instructions
    pub fn disable_tracing(&mut self) {
        self.decoder.disable_tracing();
    }

    /// Show the state before each instruction
    pub fn enable_state_dump(&mut self) {
        self.decoder.enable_state_dump();
    }

    /// Don't show the state before each instruction
    pub fn disable_state_dump(&mut self) {
        self.decoder.disable_state_dump();
    }
}
