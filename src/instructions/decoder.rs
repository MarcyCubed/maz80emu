//! Instruction decoder

use crate::instructions::micro::Microinstruction;
use crate::instructions::{ExtraBytes, Instruction, InstructionSet, micro};
use crate::state::State;

/// The instruction decoder.
///
/// The decoder loads data from memory and walks over the tables until it has decoded an
/// instruction. Then it returns to its initial state to fetch the next.
#[derive(Debug, Clone)]
pub(crate) struct Decoder {
    /// The main instruction table
    instruction_set: &'static InstructionSet,
    /// Current instruction table
    current: &'static InstructionSet,
    /// State of the decoder state machine
    state: DecoderState,
    /// The last instruction to be loaded
    last_instruction: &'static [Microinstruction],
    /// The printer of the last instruction
    last_printer: fn(&State),
    /// Is tracing enabled?
    is_tracing: bool,
    /// Is the state being printed before each instruction?
    show_state: bool,
    /// The last opcode to be fetched
    opcode: u8,
    /// Address of the instruction being executed
    address: u16,
}

/// Decode state machine states
#[derive(Debug, Clone)]
enum DecoderState {
    /// Fetch the opcode from memory
    FetchOpcode,
    /// Do a table lookup on the fetched opcode
    Table,
    /// The instruction is fully decoded is ready to be executed
    Decoded,
}

/// Initial state for the decoder
const INITIAL: DecoderState = DecoderState::FetchOpcode;

impl Decoder {
    /// Create a decoder to get instructions from the given instruction set
    pub(crate) fn new(instruction_set: &'static InstructionSet) -> Self {
        Decoder {
            instruction_set,
            current: instruction_set,
            state: INITIAL,
            last_instruction: &[],
            last_printer: |_| {},
            is_tracing: false,
            show_state: false,
            opcode: 0,
            address: 0,
        }
    }

    /// Go back to the initial state
    fn reset(&mut self) {
        self.current = self.instruction_set;
        self.state = INITIAL;
    }

    /// Display the current instruction
    fn display_instruction(&self, state: &State) {
        print!("{:0>4x}h   ", self.address);
        (self.last_printer)(state);
    }

    /// Advance on decoding the next instruction.
    pub(crate) fn decode(&mut self, state: &mut State) -> &'static [Microinstruction] {
        match self.state {
            DecoderState::FetchOpcode => {
                // Did nothing yet. We have to load the upcode
                self.address = state.pc();
                // Next state is to check the table
                self.state = DecoderState::Table;
                // Fetch the opcode
                &[micro::fetch]
            }
            DecoderState::Table => {
                self.opcode = state.z();
                if self.show_state {
                    state.print_debug(Some(self.opcode))
                }
                state.advance_r();
                // Get the instruction from the table
                match self.current[self.opcode as usize] {
                    Instruction::Prefix(table) => {
                        // It's a prefix, so we need to move to the inner table
                        // and fetch another opcode
                        self.current = table;
                        &[micro::fetch]
                    }
                    Instruction::Instruction {
                        extra_bytes,
                        micros,
                        printer,
                    } => {
                        self.last_printer = printer;
                        match extra_bytes {
                            ExtraBytes::None => {
                                // Fully decoded the instruction.
                                if self.is_tracing {
                                    self.display_instruction(state);
                                }
                                self.reset();
                                micros
                            }
                            ExtraBytes::One => {
                                self.last_instruction = micros;
                                self.state = DecoderState::Decoded;
                                &[micro::load_byte_parameter]
                            }
                            ExtraBytes::Two => {
                                self.last_instruction = micros;
                                self.state = DecoderState::Decoded;
                                &[micro::load_word_parameter]
                            }
                        }
                    }
                    Instruction::NoPrefix => {
                        // Discard the prefix and try again
                        self.current = self.instruction_set;
                        self.decode(state)
                    }
                }
            }
            DecoderState::Decoded => {
                // Decoded the instruction: Reset the decoder and return the micro instructions
                if self.is_tracing {
                    self.display_instruction(state);
                }
                self.reset();
                self.last_instruction
            }
        }
    }

    /// Show the instructions as they are decoded
    pub fn enable_tracing(&mut self) {
        self.is_tracing = true;
    }

    /// Don't show the instructions
    pub fn disable_tracing(&mut self) {
        self.is_tracing = false;
    }

    /// Show the state before each instruction
    pub fn enable_state_dump(&mut self) {
        self.show_state = true;
    }

    /// Don't show the state before each instruction
    pub fn disable_state_dump(&mut self) {
        self.show_state = false;
    }
}
