//! Instruction decoder

use crate::instructions::{
    DataLoader, Executable, NOP, Opcode, Table, ThreeByteInstruction, TwoByteInstruction,
};

/// The instruction decoder.
///
/// The decoder loads data from memory and walks over the tables until it has decoded an
/// instruction. Then it returns to its initial state to fetch the next.
#[derive(Debug, Clone)]
pub(crate) struct Decoder {
    /// The main instruction table
    main_table: &'static Table,
    /// Current instruction table
    current: &'static Table,
    /// Number of prefixes already processed
    prefix_count: u8,
    /// Buffer to load instructions from memory
    buffer: [u8; 2],
    /// The instruction opcode
    opcode: Opcode,
    /// State of the decoder state machine
    state: DecoderState,
    /// The address of the instruction
    pc: u16,
}

/// Decode state machine states
#[derive(Debug, Clone)]
enum DecoderState {
    /// Fetch the opcode from memory
    FetchOpcode,
    /// Do a table lookup on the fetched opcode
    Table,
    /// Load one byte from memory
    LoadByte(TwoByteInstruction),
    /// Loads two bytes from memory
    LoadWord(ThreeByteInstruction),
}

/// Initial state for the decoder
const INITIAL: DecoderState = DecoderState::FetchOpcode;

impl Decoder {
    /// Create a decoder to get instructions from a table for a specific processor
    pub(crate) fn new_with_table(table: &'static Table) -> Self {
        Decoder {
            main_table: table,
            current: table,
            buffer: [0; 2],
            prefix_count: 0,
            opcode: NOP,
            state: INITIAL,
            pc: 0,
        }
    }

    /// Go back to the initial state
    fn reset(&mut self) {
        self.current = self.main_table;
        self.prefix_count = 0;
        self.state = INITIAL;
    }

    /// Advance on decoding the next instruction.
    ///
    /// The address is where the instruction is located.
    pub(crate) fn decode(&mut self, pc: u16) -> Executable<'_> {
        match self.state {
            DecoderState::FetchOpcode => {
                // Did nothing yet. We have to load the upcode

                // Next state is to check the table
                self.state = DecoderState::Table;
                // Save the instruction address. We'll ignore the parameter address on other calls
                self.pc = pc;
                // Fetch the opcode
                Executable::Fetch {
                    address: pc,
                    loader: DataLoader(&mut self.buffer[0]),
                }
            }
            DecoderState::Table => {
                // Get the opcode from the table
                self.opcode = self.current[self.buffer[0] as usize];
                match self.opcode {
                    Opcode::Prefix(table) => {
                        // It's a prefix, so we need to move to the inner table
                        // and fetch another opcode
                        self.current = table;
                        self.prefix_count += 1;
                        Executable::Fetch {
                            address: self.pc + self.prefix_count as u16,
                            loader: DataLoader(&mut self.buffer[0]),
                        }
                    }
                    Opcode::Simple(instruction) => {
                        let result = Executable::Simple {
                            instruction,
                            prefix_count: self.prefix_count,
                        };
                        // Finished decoding an instruction: reset the decoder state
                        self.reset();
                        result
                    }
                    Opcode::TwoByte(instruction) => {
                        // Set the state
                        self.state = DecoderState::LoadByte(instruction);
                        // Fetch the next byte
                        Executable::Fetch {
                            address: self.pc + self.prefix_count as u16 + 1,
                            loader: DataLoader(&mut self.buffer[0]),
                        }
                    }
                    Opcode::ThreeByte(instruction) => {
                        // Set the state
                        self.state = DecoderState::LoadWord(instruction);
                        // Fetch the next two bytes
                        Executable::Fetch16 {
                            address: self.pc + self.prefix_count as u16 + 1,
                            loader: DataLoader(&mut self.buffer),
                        }
                    }
                }
            }
            DecoderState::LoadByte(instruction) => {
                // The argument for the two byte instruction was loaded into buffer[0]

                let result = Executable::TwoByte {
                    instruction,
                    prefix_count: self.prefix_count,
                    byte_2: self.buffer[0],
                };
                // Finished decoding an instruction: reset the decoder state
                self.reset();
                result
            }
            DecoderState::LoadWord(instruction) => {
                // The arguments for the three byte instruction was loaded into buffer

                let result = Executable::ThreeByte {
                    instruction,
                    prefix_count: self.prefix_count,
                    bytes: self.buffer,
                };
                // Finished decoding an instruction: reset the decoder state
                self.reset();
                result
            }
        }
    }
}
