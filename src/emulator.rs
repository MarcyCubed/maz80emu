//! The core of the emulator

use crate::cpus::z80::Z80;
use crate::instructions::decoder::Decoder;
use crate::instructions::micro::{Microinstruction, jump};
use crate::instructions::{ExecResult, InstructionSet};
use crate::state::{InterruptMode, State};

/// The Z80 emulator
#[derive(Debug, Clone)]
pub struct Emulator {
    /// Internal state of the processor
    pub state: State,
    /// The instruction decoder
    decoder: Decoder,
    /// The microinstructions being executed
    micros: &'static [Microinstruction],
    /// Is a Nonmaskable Interrupt waiting to be handled?
    nmi_pending: bool,
    /// Is a regular interrupt waiting to be handled?
    int_pending: bool,
    /// Data for the interrupt
    interrupt_data: u8,
    /// The result of the last time `run` was executed
    last_result: ExecResult,
}

impl Emulator {
    /// Create an emulator to run programs for the instruction set
    pub fn new_with_instruction_set(instruction_set: &'static InstructionSet) -> Emulator {
        Emulator {
            state: Default::default(),
            decoder: Decoder::new(instruction_set),
            micros: &[],
            nmi_pending: false,
            int_pending: false,
            interrupt_data: 0,
            last_result: ExecResult::Done(0),
        }
    }

    /// Create a Z80 emulator
    pub fn new_z80() -> Self {
        Self::new_with_instruction_set(&Z80)
    }

    /// Check if the processor is halted
    pub fn is_halted(&self) -> bool {
        self.last_result == ExecResult::Halt
    }

    /// Try to service a pending interrupt
    ///
    /// if an interrupt can be serviced, return a list of microinstructions to run the interrupt
    /// handler and an `ExecResult` marking the time it took.
    /// Otherwise, return `None` if there is no interrupt to service, if they are disabled or if the
    /// processor is busy processing.
    fn service_interrupt(&mut self) -> Option<(&'static [Microinstruction], ExecResult)> {
        // Is the processor ready to run an interrupt?
        if !self.last_result.can_interrupt() {
            None
        } else if self.nmi_pending {
            // Service a nonmaskable interrupt
            self.nmi_pending = false;
            self.state.iff1 = false;
            self.state.advance_r();
            Some((
                &[jump::push_pc, |state| jump::jump_to(state, 0x66, 0)],
                ExecResult::Int(5),
            ))
        } else if self.int_pending && self.state.iff1 {
            self.int_pending = false;
            match self.state.interrupt_mode {
                InterruptMode::Instruction => {
                    // Dispatch the instruction
                    let injected = self.decoder.inject_opcode(self.interrupt_data);
                    assert!(
                        injected,
                        "Failed to inject instruction {:02x}h",
                        self.interrupt_data
                    );
                    Some((&[], ExecResult::Int(6)))
                }
                InterruptMode::Rst0038 => {
                    let injected = self.decoder.inject_opcode(0xff);
                    assert!(injected, "Failed to inject rst 0x38 instruction");
                    Some((&[], ExecResult::Int(6)))
                }
                InterruptMode::Vectored => {
                    // Put the address in WZ
                    *self.state.wz_mut() = [self.interrupt_data, self.state.i()];
                    Some((
                        &[
                            // Save the PC
                            jump::push_pc,
                            // Load the address of the interrupt handler
                            |state| ExecResult::load16(state.wz()),
                            // Finish the call to the interrupt handler
                            |state| jump::jr_mm(state, 0),
                        ],
                        ExecResult::Int(7),
                    ))
                }
            }
        } else {
            None
        }
    }

    /// Run the emulator
    pub fn run(&mut self) -> ExecResult {
        // If we're running microinstructions
        let result = if self.state.mpc < self.micros.len() {
            let micro = self.micros[self.state.mpc];
            self.state.mpc += 1;
            micro(&mut self.state)
        } else if let Some((micros, result)) = self.service_interrupt() {
            self.micros = micros;
            self.state.mpc = 0;
            result
        } else {
            // Need to decode the next instruction
            self.micros = self.decoder.decode(&mut self.state);
            self.state.mpc = 1; // We'll run the [0] now.
            self.micros[0](&mut self.state)
        };
        self.last_result = result;
        result
    }

    /// If the [ExecResult] is related to loading or storing data in the memory, try to perform it.
    ///
    /// If the request is a memory access and its address is within the memory's bounds, the
    /// requested action will be performed and the function will return `true`.
    /// Otherwise, nothing will be done and `false` will be returned instead.
    pub fn access_memory(&mut self, request: ExecResult, memory: &mut [u8]) -> bool {
        // Get the address for the memory access if it's valid
        fn get_address(memory: &[u8], address: u16) -> Option<usize> {
            let address = address as usize;
            if address < memory.len() {
                Some(address)
            } else {
                None
            }
        }

        // Get the addresses for the first and second index
        fn get_address_16(memory: &[u8], address: u16) -> Option<(usize, usize)> {
            let address_0 = get_address(memory, address)?;
            let address_1 = get_address(memory, address.wrapping_add(1))?;
            Some((address_0, address_1))
        }

        match request {
            ExecResult::Fetch { address } | ExecResult::Load { address } => {
                if let Some(address) = get_address(memory, address) {
                    self.send_byte(memory[address]);
                    true
                } else {
                    false
                }
            }
            ExecResult::Load16 { address } => {
                if let Some((address_0, address_1)) = get_address_16(memory, address) {
                    self.send_word([memory[address_0], memory[address_1]]);
                    true
                } else {
                    false
                }
            }
            ExecResult::Store { address, data } => {
                if let Some(address) = get_address(memory, address) {
                    memory[address] = data;
                    true
                } else {
                    false
                }
            }
            ExecResult::Store16 { address, data } => {
                if let Some((address_0, address_1)) = get_address_16(memory, address) {
                    memory[address_0] = data[0];
                    memory[address_1] = data[1];
                    true
                } else {
                    false
                }
            }
            _ => false,
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
            if !self.access_memory(result, memory) {
                return result;
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
