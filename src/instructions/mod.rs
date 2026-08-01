//! # Instruction instructions.
//!
//! The emulator decodes instructions using tables of opcodes. The tables are arrays with 256
//! entries that map opcodes to instruction descriptions or in case of prefixes, other tables.
//!
//! The decoder just loads data from memory and walks over the tables until it read a complete
//! instruction, then it can be executed.

pub mod decoder;
#[cfg(test)]
mod test_cpu;

#[cfg(test)]
pub use test_cpu::TEST_CPU;

use crate::state::State;
use std::num::{NonZero, NonZeroU8};

/// Value returned after executing Z80 code.
#[derive(Debug)]
pub enum ExecResult<'a> {
    /// The emulator is ready to execute the next instruction.
    Ready,
    /// The emulator requests 1 byte of data from memory.
    Load {
        /// The memory address to be read.
        address: u16,
        /// The loader.
        loader: DataLoader<'a, u8>,
    },
    /// The emulator requests 2 bytes of data from memory.
    Load16 {
        /// The memory address to be read.
        address: u16,
        /// The loader.
        loader: DataLoader<'a, [u8; 2]>,
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
    ///
    /// `loader` should be used to store the value from the port in the register.
    /// If it's not used, the register will keep its original value.
    In {
        /// The port address
        port: u16,
        /// The thing pass the value read from the port to the processor
        loader: DataLoader<'a, u8>,
    },
    /// The emulator wants to write a byte to an I/O port.
    Out {
        /// The port that will receive data.
        port: u16,
        /// The data to be written to the port.
        data: u8,
    },
    /// Executed a `HALT` instruction.
    Halt,
}

/// The processor requests data.
///
/// A `DataLoader` is returned by the emulator whenever the Z80 processor needs to access external
/// data, from either the memory or I/O ports.
///
/// The loader is consumed after sending the data.
#[derive(Debug)]
pub struct DataLoader<'a, T>(&'a mut T);

impl<'a> DataLoader<'a, u8> {
    /// Send a byte of data to the processor
    pub fn load(self, data: u8) {
        *self.0 = data
    }
}

impl<'a> DataLoader<'a, [u8; 2]> {
    /// Send two bytes of data to the processor
    pub fn load_bytes(self, data: [u8; 2]) {
        *self.0 = data
    }

    /// Send a 16-bit number to the processor
    pub fn load_value(self, value: u16) {
        self.load_bytes(value.to_le_bytes())
    }
}

/// A table of instructions
pub type Table = [Opcode; 256];

/// A simple instruction
#[derive(Debug, Clone, Copy)]
pub struct SimpleInstruction {
    /// Number of clock cycles needed to execute the instruction
    cycles: u8,
    /// Function to execute the instruction
    exec: fn(&mut State) -> ExecResult,
}

/// Two byte instruction
#[derive(Debug, Clone, Copy)]
pub struct TwoByteInstruction {
    /// Number of clock cycles needed to execute the instruction
    cycles: u8,
    /// Function to execute the instruction
    exec: fn(&mut State, u8) -> ExecResult,
}

/// Three byte instruction
#[derive(Debug, Clone, Copy)]
pub struct ThreeByteInstruction {
    /// Number of clock cycles needed to execute the instruction
    cycles: u8,
    /// Function to execute the instruction
    exec: fn(&mut State, [u8; 2]) -> ExecResult,
}

/// Representation of an instruction and how to run it
#[derive(Debug, Clone, Copy)]
pub enum Opcode {
    /// Prefix for another instruction table
    Prefix(&'static Table),
    /// One byte instruction
    Simple(SimpleInstruction),
    /// Instructions two bytes long
    TwoByte(TwoByteInstruction),
    /// Instructions three bytes long
    ThreeByte(ThreeByteInstruction),
}

impl Opcode {
    /// Get the length of the instruction in bytes, excluding prefixes.
    ///
    /// If the opcode is a prefix, return `None`
    pub fn len(&self) -> Option<NonZeroU8> {
        NonZero::new(match self {
            Opcode::Prefix(_) => 0,
            Opcode::Simple { .. } => 1,
            Opcode::TwoByte { .. } => 2,
            Opcode::ThreeByte { .. } => 3,
        })
    }
}

/// NOP instruction
pub(crate) const NOP: Opcode = Opcode::Simple(SimpleInstruction {
    cycles: 4,
    exec: |_| ExecResult::Ready,
});

/// HALT instruction
pub(crate) const HALT: Opcode = Opcode::Simple(SimpleInstruction {
    cycles: 4,
    exec: |_| ExecResult::Halt,
});

/// Something that can be executed, either a memory fetch or a decoded instruction
#[derive(Debug)]
pub(crate) enum Executable<'a> {
    /// Get one byte from memory
    Fetch {
        /// The memory address to be read.
        address: u16,
        /// The loader.
        loader: DataLoader<'a, u8>,
    },
    /// Get two bytes from memory
    Fetch16 {
        /// The memory address to be read.
        address: u16,
        /// The loader.
        loader: DataLoader<'a, [u8; 2]>,
    },
    /// Decoded a simple instruction
    Simple {
        /// The instruction itself
        instruction: SimpleInstruction,
        /// The number of prefixes
        prefix_count: u8,
    },
    /// Decoded a two bytes long instruction
    TwoByte {
        /// The instruction itself
        instruction: TwoByteInstruction,
        /// The number of prefixes
        prefix_count: u8,
        /// The second byte of the instruction
        byte_2: u8,
    },
    /// Decoded a three bytes long instruction
    ThreeByte {
        /// The instruction itself
        instruction: ThreeByteInstruction,
        /// The number of prefixes
        prefix_count: u8,
        /// The other bytes of the instruction
        bytes: [u8; 2],
    },
}

impl<'a> Executable<'a> {
    /// Run the executable, consuming it in the process
    ///
    /// Return a pair with the number of clock cycles used by the instruction and the result of the
    /// instruction execution. The number of clock cycles can be used to simulate the speed of an
    /// actual Z80 processor.
    pub(crate) fn run(self, state: &'a mut State) -> (u8, ExecResult<'a>) {
        match self {
            Executable::Fetch { address, loader } => (0, ExecResult::Load { address, loader }),
            Executable::Fetch16 { address, loader } => (0, ExecResult::Load16 { address, loader }),
            Executable::Simple {
                instruction,
                prefix_count,
            } => {
                // Advance the PC
                state.advance_pc(prefix_count as u16 + 1);
                // Execute the instruction
                (instruction.cycles, (instruction.exec)(state))
            }
            Executable::TwoByte {
                instruction,
                prefix_count,
                byte_2,
            } => {
                // Advance the PC
                state.advance_pc(prefix_count as u16 + 2);
                // Execute the instruction
                (instruction.cycles, (instruction.exec)(state, byte_2))
            }
            Executable::ThreeByte {
                instruction,
                prefix_count,
                bytes,
            } => {
                // Advance the PC
                state.advance_pc(prefix_count as u16 + 3);
                // Execute the instruction
                (instruction.cycles, (instruction.exec)(state, bytes))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::decoder::Decoder;
    use crate::state::{Register, Register16};

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
        let mut decoder = Decoder::new_with_table(&TEST_CPU);

        fn run_until_result(state: &mut State, decoder: &mut Decoder, ram: &mut [u8]) {
            loop {
                let (_, result) = decoder
                    .decode(state.get_register_16(Register16::PC))
                    .run(state);
                match result {
                    ExecResult::Load { address, loader } => {
                        loader.load(ram[address as usize]);
                    }
                    ExecResult::Load16 { address, loader } => {
                        let address = address as usize;
                        let bytes = [ram[address], ram[address + 1]];
                        loader.load_bytes(bytes);
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
        // Run setting A to various values
        run_until_result(&mut state, &mut decoder, &mut memory);
        assert_eq!(state.get_register_8(Register::A), 7);
        run_until_result(&mut state, &mut decoder, &mut memory);
        assert_eq!(state.get_register_8(Register::A), 0);
        run_until_result(&mut state, &mut decoder, &mut memory);
        assert_eq!(state.get_register_8(Register::A), 1);
        run_until_result(&mut state, &mut decoder, &mut memory);
        assert_eq!(state.get_register_8(Register::A), 2);
        run_until_result(&mut state, &mut decoder, &mut memory);
        assert_eq!(state.get_register_8(Register::A), 3);
        run_until_result(&mut state, &mut decoder, &mut memory);
        assert_eq!(state.get_register_8(Register::A), 4);
        run_until_result(&mut state, &mut decoder, &mut memory);
        assert_eq!(state.get_register_8(Register::A), 5);
        run_until_result(&mut state, &mut decoder, &mut memory);
        assert_eq!(state.get_register_8(Register::A), 6);
        run_until_result(&mut state, &mut decoder, &mut memory);
        assert_eq!(state.get_register_8(Register::A), 7);
        // Flags = 18 (prefix operation)
        run_until_result(&mut state, &mut decoder, &mut memory);
        assert_eq!(state.get_register_8(Register::Flags), 18);

        // B = 0xc3
        run_until_result(&mut state, &mut decoder, &mut memory);
        assert_eq!(state.get_register_8(Register::B), 0xc3);
        // A = A + B
        run_until_result(&mut state, &mut decoder, &mut memory);
        assert_eq!(state.get_register_8(Register::A), 0xc3 + 7);
        // C = 0x73
        run_until_result(&mut state, &mut decoder, &mut memory);
        assert_eq!(state.get_register_8(Register::C), 0x73);
        // DE = 0xbbaa
        run_until_result(&mut state, &mut decoder, &mut memory);
        assert_eq!(state.get_register_16(Register16::DE), 0xbbaa);
        // HL = BC + DE = 0xc373 + 0xbbaa
        run_until_result(&mut state, &mut decoder, &mut memory);
        assert_eq!(
            state.get_register_16(Register16::HL),
            0xc373u16.wrapping_add(0xbbaa)
        );
        // Mem[4,5] = HL,
        run_until_result(&mut state, &mut decoder, &mut memory);
        let bytes = 0xc373u16.wrapping_add(0xbbaa).to_le_bytes();
        assert_eq!(memory[4], bytes[0]);
        assert_eq!(memory[5], bytes[1]);
    }
}
