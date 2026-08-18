//! A dummy CPU to test if the execution infrastructure works

use crate::instructions::{ExecResult, ExtraBytes, HALT, Instruction, InstructionSet};
use crate::state::Register;

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
            *state.a_mut() = 0;
            ExecResult::Done(4)
        }],
        printer: |_| println!("A = 0"),
    };
    table[1] = Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| {
            *state.a_mut() = 1;
            ExecResult::Done(4)
        }],
        printer: |_| println!("A = 1"),
    };
    table[2] = Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| {
            *state.a_mut() = 2;
            ExecResult::Done(4)
        }],
        printer: |_| println!("A = 2"),
    };
    table[3] = Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| {
            *state.a_mut() = 3;
            ExecResult::Done(4)
        }],
        printer: |_| println!("A = 3"),
    };
    table[4] = Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| {
            *state.a_mut() = 4;
            ExecResult::Done(4)
        }],
        printer: |_| println!("A = 4"),
    };
    table[5] = Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| {
            *state.a_mut() = 5;
            ExecResult::Done(4)
        }],
        printer: |_| println!("A = 5"),
    };
    table[6] = Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| {
            *state.a_mut() = 6;
            ExecResult::Done(4)
        }],
        printer: |_| println!("A = 6"),
    };
    table[7] = Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| {
            *state.a_mut() = 7;
            ExecResult::Done(4)
        }],
        printer: |_| println!("A = 7"),
    };
    table[8] = Instruction::Instruction {
        extra_bytes: ExtraBytes::One,
        micros: &[|state| {
            *state.b_mut() = state.z();
            ExecResult::Done(4)
        }],
        printer: |state| println!("B = {}", state.z()),
    };
    table[9] = Instruction::Instruction {
        extra_bytes: ExtraBytes::One,
        micros: &[|state| {
            *state.c_mut() = state.z();
            ExecResult::Done(4)
        }],
        printer: |state| println!("C = {}", state.z()),
    };
    table[10] = Instruction::Instruction {
        extra_bytes: ExtraBytes::Two,
        micros: &[|state| {
            *state.de_mut() = state.wz_bytes();
            ExecResult::Done(4)
        }],
        printer: |state| println!("DE = {}", state.wz()),
    };
    table[11] = Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| {
            let a = state.a();
            let b = state.b();
            *state.a_mut() = a.wrapping_add(b);
            ExecResult::Done(4)
        }],
        printer: |state| println!("A = A + B"),
    };
    table[12] = Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| {
            let bc = state.bc();
            let de = state.de();
            *state.hl_mut() = bc.wrapping_add(de).to_le_bytes();
            ExecResult::Done(4)
        }],
        printer: |state| println!("HL = BC + DE"),
    };
    table[13] = Instruction::Instruction {
        extra_bytes: ExtraBytes::Two,
        micros: &[|state| ExecResult::Store16 {
            address: state.wz(),
            data: state.hl_bytes(),
        }],
        printer: |state| println!("({} = HL", state.wz()),
    };
    table[14] = Instruction::Prefix(&TEST_PREFIX);
    table
};

/// Table with the prefix instruction
pub static TEST_PREFIX: InstructionSet = [Instruction::Instruction {
    extra_bytes: ExtraBytes::One,
    micros: &[|state| {
        state.set_register_8(Register::Flags, state.z());
        ExecResult::Done(4)
    }],
    printer: |state| println!("F = {}", state.z()),
}; 256];
