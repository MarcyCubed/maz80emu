//! A dummy CPU to test if the execution infrastructure works

use crate::instructions::{ExecResult, ExtraBytes, HALT, Instruction, InstructionSet};
use crate::state::{Register, Register16};

/// Dummy CPU with a few testing instructions.
///
/// * Instructions 0 to 7 load their corresponding number to A
/// * Instruction 8 loads an immediate value to B
/// * Instruction 9 loads an immediate value to C
/// * Instruction 10 loads an immediate value to DE
/// * Instruction 11 add B to A, storing the result in A
/// * Instruction 12 adds BC to DE, storing the result in HL
/// * Instruction 13 writes the contents of HL to an immediate memory address
/// * 14 is a prefix and all instructions in that table set Flags to an immediate value
/// * Everything else halts execution
pub static TEST_CPU: InstructionSet = {
    let mut table = [HALT; 256];
    table[0] = Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| {
            state.set_register_8(Register::A, 0);
            ExecResult::Done(4)
        }],
    };
    table[1] = Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| {
            state.set_register_8(Register::A, 1);
            ExecResult::Done(4)
        }],
    };
    table[2] = Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| {
            state.set_register_8(Register::A, 2);
            ExecResult::Done(4)
        }],
    };
    table[3] = Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| {
            state.set_register_8(Register::A, 3);
            ExecResult::Done(4)
        }],
    };
    table[4] = Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| {
            state.set_register_8(Register::A, 4);
            ExecResult::Done(4)
        }],
    };
    table[5] = Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| {
            state.set_register_8(Register::A, 5);
            ExecResult::Done(4)
        }],
    };
    table[6] = Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| {
            state.set_register_8(Register::A, 6);
            ExecResult::Done(4)
        }],
    };
    table[7] = Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| {
            state.set_register_8(Register::A, 7);
            ExecResult::Done(4)
        }],
    };
    table[8] = Instruction::Instruction {
        extra_bytes: ExtraBytes::One,
        micros: &[|state| {
            state.set_register_8(Register::B, state.get_fetched_byte());
            ExecResult::Done(4)
        }],
    };
    table[9] = Instruction::Instruction {
        extra_bytes: ExtraBytes::One,
        micros: &[|state| {
            state.set_register_8(Register::C, state.get_fetched_byte());
            ExecResult::Done(4)
        }],
    };
    table[10] = Instruction::Instruction {
        extra_bytes: ExtraBytes::Two,
        micros: &[|state| {
            state.set_register_16(Register16::DE, state.get_fetched_word());
            ExecResult::Done(4)
        }],
    };
    table[11] = Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| {
            let a = state.get_register_8(Register::A);
            let b = state.get_register_8(Register::B);
            state.set_register_8(Register::A, a.wrapping_add(b));
            ExecResult::Done(4)
        }],
    };
    table[12] = Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| {
            let bc = state.get_register_16(Register16::BC);
            let de = state.get_register_16(Register16::DE);
            state.set_register_16(Register16::HL, bc.wrapping_add(de));
            ExecResult::Done(4)
        }],
    };
    table[13] = Instruction::Instruction {
        extra_bytes: ExtraBytes::Two,
        micros: &[|state| ExecResult::Store16 {
            address: state.get_fetched_word(),
            data: state.get_register_16_bytes(Register16::HL),
        }],
    };
    table[14] = Instruction::Prefix(&TEST_PREFIX);
    table
};

/// Table with the prefix instruction
pub static TEST_PREFIX: InstructionSet = [Instruction::Instruction {
    extra_bytes: ExtraBytes::One,
    micros: &[|state| {
        state.set_register_8(Register::Flags, state.get_fetched_byte());
        ExecResult::Done(4)
    }],
}; 256];
