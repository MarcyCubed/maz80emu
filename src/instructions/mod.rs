//! # Instruction
//!
//! The emulator decodes instructions using tables of opcodes. The tables are arrays with 256
//! entries that map opcodes to instruction descriptions or in case of prefixes, other tables.
//!
//! The decoder just loads data from memory and walks over the tables until it read a complete
//! instruction, then it can be executed.
//!
//! Each actual decoded instruction is an array of microinstructions. Each microinstruction is just
//! a function pointer that operates on the machine [[State]] and return an [[ExecResult]]. After
//! each instruction is decoded, its microinstructions are executed one by one until the end. Then
//! the emulator reads and decodes the next instruction.

pub mod decoder;
pub mod micro;
#[cfg(test)]
mod test_cpu;

use micro::Microinstruction;
#[cfg(test)]
pub use test_cpu::TEST_CPU;

/// Value returned after a microinstruction.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ExecResult {
    /// Finished executing one instruction.
    ///
    /// The number is the amount of clock cycles it took to run the instruction.
    Done(u8),
    /// The emulator requests 1 byte of data from memory.
    Load {
        /// The memory address to be read.
        address: u16,
    },
    /// The emulator requests 2 bytes of data from memory.
    Load16 {
        /// The memory address to be read.
        address: u16,
    },
    /// The emulator wants to store a byte in the given memory `address`.
    Store {
        /// The memory address that will be changed.
        address: u16,
        /// The new value to be stored in the address
        data: u8,
    },
    /// The emulator wants to store two bytes sequentially starting in the given memory `address`.
    Store16 {
        /// The memory address that will be changed.
        address: u16,
        /// The values to be stored in the address
        data: [u8; 2],
    },
    /// The emulator wants to read data from an I/O port.
    In {
        /// The port address
        port: u16,
    },
    /// The emulator wants to write a byte to an I/O port.
    Out {
        /// The port that will receive data.
        port: u16,
        /// The data to be written to the port.
        data: u8,
    },
    /// Executed a `HALT` instruction.
    ///
    /// The processor stopped running until it receives an interruption
    Halt,
}

/// The processor's instruction set.
///
/// This is a table of instructions that map opcode bytes to instructions
pub type InstructionSet = [Instruction; 256];

/// How many extra bytes an instruction have after its opcode
#[derive(Debug, Clone, Copy)]
pub enum ExtraBytes {
    None,
    One,
    Two,
}

/// Representation of an instruction and how to run it
#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    /// Prefix for another instruction table
    Prefix(&'static InstructionSet),
    /// Actual instruction
    ///
    /// An instruction is just a sequence of microinstructions
    Instruction {
        /// How many extra bytes should be loaded
        extra_bytes: ExtraBytes,
        /// The list of micro instructions implementing the instruction
        micros: &'static [Microinstruction],
    },
}

/// NOP instruction
pub const NOP: Instruction = Instruction::Instruction {
    extra_bytes: ExtraBytes::None,
    micros: &[|_| ExecResult::Done(4)],
};

/// HALT instruction
pub const HALT: Instruction = Instruction::Instruction {
    extra_bytes: ExtraBytes::None,
    micros: &[|_| ExecResult::Halt],
};

/// An unimplemented instruction
///
/// This isn't a real instruction and will cause the program to panic
pub const UNIMPLEMENTED: Instruction = Instruction::Instruction {
    extra_bytes: ExtraBytes::None,
    micros: &[|_| unimplemented!("Instruction isn't implemented")],
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::decoder::Decoder;
    use crate::state::{Register, Register16, State};

    /// Run one instruction
    fn run_instruction(state: &mut State, decoder: &mut Decoder, ram: &mut [u8]) {
        loop {
            for instr in decoder.decode(state) {
                match instr(state) {
                    ExecResult::Load { address } => {
                        state.load_data_8(ram[address as usize]);
                    }
                    ExecResult::Load16 { address } => {
                        let address = address as usize;
                        let bytes = [ram[address], ram[address + 1]];
                        state.load_data_16(bytes);
                    }
                    ExecResult::Store { address, data } => {
                        ram[address as usize] = data;
                    }
                    ExecResult::Store16 { address, data } => {
                        ram[address as usize] = data[0];
                        ram[address as usize + 1] = data[1];
                    }
                    _ => return,
                }
            }
        }
    }

    /// Test the decoder and executer with a dummy CPU
    #[test]
    fn test_dummy_cpu() {
        // Memory with a loaded program
        let mut memory: [u8; _] = [
            7, // A = 7
            0, // A = 0
            1, // A = 1
            2, // A = 2
            3, // A = 3
            4, // A = 4
            5, // A = 5
            6, // A = 6
            7, // A = 7
            14, 0, 18, // Flags = 18
            8, 0xc3, // B = 0xc3
            11,   // A = A + B
            9, 0x73, // C = 0x73
            10, 0xaa, 0xbb, // DE = 0xbbaa
            12,   // HL = BC + DE = 0xc373 + 0xbbaa
            13, 4, 0,   // Mem[4,5] = HL,
            255, // Halt
        ];
        let mut state = State::new();
        let mut decoder = Decoder::new(&TEST_CPU);

        // Run setting A to various values
        run_instruction(&mut state, &mut decoder, &mut memory);
        assert_eq!(state.get_register_8(Register::A), 7);
        run_instruction(&mut state, &mut decoder, &mut memory);
        assert_eq!(state.get_register_8(Register::A), 0);
        run_instruction(&mut state, &mut decoder, &mut memory);
        assert_eq!(state.get_register_8(Register::A), 1);
        run_instruction(&mut state, &mut decoder, &mut memory);
        assert_eq!(state.get_register_8(Register::A), 2);
        run_instruction(&mut state, &mut decoder, &mut memory);
        assert_eq!(state.get_register_8(Register::A), 3);
        run_instruction(&mut state, &mut decoder, &mut memory);
        assert_eq!(state.get_register_8(Register::A), 4);
        run_instruction(&mut state, &mut decoder, &mut memory);
        assert_eq!(state.get_register_8(Register::A), 5);
        run_instruction(&mut state, &mut decoder, &mut memory);
        assert_eq!(state.get_register_8(Register::A), 6);
        run_instruction(&mut state, &mut decoder, &mut memory);
        assert_eq!(state.get_register_8(Register::A), 7);
        // Flags = 18 (prefix operation)
        run_instruction(&mut state, &mut decoder, &mut memory);
        assert_eq!(state.get_register_8(Register::Flags), 18);

        // B = 0xc3
        run_instruction(&mut state, &mut decoder, &mut memory);
        assert_eq!(state.get_register_8(Register::B), 0xc3);
        // A = A + B
        run_instruction(&mut state, &mut decoder, &mut memory);
        assert_eq!(state.get_register_8(Register::A), 0xc3 + 7);
        // C = 0x73
        run_instruction(&mut state, &mut decoder, &mut memory);
        assert_eq!(state.get_register_8(Register::C), 0x73);
        // DE = 0xbbaa
        run_instruction(&mut state, &mut decoder, &mut memory);
        assert_eq!(state.get_register_16(Register16::DE), 0xbbaa);
        // HL = BC + DE = 0xc373 + 0xbbaa
        run_instruction(&mut state, &mut decoder, &mut memory);
        assert_eq!(
            state.get_register_16(Register16::HL),
            0xc373u16.wrapping_add(0xbbaa)
        );
        // Mem[4,5] = HL,
        run_instruction(&mut state, &mut decoder, &mut memory);
        let bytes = 0xc373u16.wrapping_add(0xbbaa).to_le_bytes();
        assert_eq!(memory[4], bytes[0]);
        assert_eq!(memory[5], bytes[1]);
    }
}
