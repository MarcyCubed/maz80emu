//! A dummy CPU to test if the execution infrastructure works

use crate::instructions::{
    ExecResult, HALT, Opcode, SimpleInstruction, Table, ThreeByteInstruction, TwoByteInstruction,
};
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
pub static TEST_CPU: Table = {
    let mut table = [HALT; 256];
    table[0] = Opcode::Simple(SimpleInstruction {
        cycles: 0,
        exec: |state| {
            *state.get_register_mut_8(Register::A) = 0;
            ExecResult::Ready
        },
    });
    table[1] = Opcode::Simple(SimpleInstruction {
        cycles: 0,
        exec: |state| {
            *state.get_register_mut_8(Register::A) = 1;
            ExecResult::Ready
        },
    });
    table[2] = Opcode::Simple(SimpleInstruction {
        cycles: 0,
        exec: |state| {
            *state.get_register_mut_8(Register::A) = 2;
            ExecResult::Ready
        },
    });
    table[3] = Opcode::Simple(SimpleInstruction {
        cycles: 0,
        exec: |state| {
            *state.get_register_mut_8(Register::A) = 3;
            ExecResult::Ready
        },
    });
    table[4] = Opcode::Simple(SimpleInstruction {
        cycles: 0,
        exec: |state| {
            *state.get_register_mut_8(Register::A) = 4;
            ExecResult::Ready
        },
    });
    table[5] = Opcode::Simple(SimpleInstruction {
        cycles: 0,
        exec: |state| {
            *state.get_register_mut_8(Register::A) = 5;
            ExecResult::Ready
        },
    });
    table[6] = Opcode::Simple(SimpleInstruction {
        cycles: 0,
        exec: |state| {
            *state.get_register_mut_8(Register::A) = 6;
            ExecResult::Ready
        },
    });
    table[7] = Opcode::Simple(SimpleInstruction {
        cycles: 0,
        exec: |state| {
            *state.get_register_mut_8(Register::A) = 7;
            ExecResult::Ready
        },
    });
    table[8] = Opcode::TwoByte(TwoByteInstruction {
        cycles: 0,
        exec: |state, n| {
            state.set_register_8(Register::B, n);
            ExecResult::Ready
        },
    });
    table[9] = Opcode::TwoByte(TwoByteInstruction {
        cycles: 0,
        exec: |state, n| {
            state.set_register_8(Register::C, n);
            ExecResult::Ready
        },
    });
    table[10] = Opcode::ThreeByte(ThreeByteInstruction {
        cycles: 0,
        exec: |state, nn| {
            state.set_register_16_bytes(Register16::DE, nn);
            ExecResult::Ready
        },
    });
    table[11] = Opcode::Simple(SimpleInstruction {
        cycles: 0,
        exec: |state| {
            let a = state.get_register_8(Register::A);
            let b = state.get_register_8(Register::B);
            state.set_register_8(Register::A, a.wrapping_add(b));
            ExecResult::Ready
        },
    });
    table[12] = Opcode::Simple(SimpleInstruction {
        cycles: 0,
        exec: |state| {
            let bc = state.get_register_16(Register16::BC);
            let de = state.get_register_16(Register16::DE);
            state.set_register_16(Register16::HL, bc.wrapping_add(de));
            ExecResult::Ready
        },
    });
    table[13] = Opcode::ThreeByte(ThreeByteInstruction {
        cycles: 0,
        exec: |state, nn| {
            let data = state.get_register_16_bytes(Register16::HL);
            ExecResult::Store16 {
                address: u16::from_le_bytes(nn),
                data,
            }
        },
    });
    table[14] = Opcode::Prefix(&TEST_PREFIX);
    table
};

pub static TEST_PREFIX: Table = [Opcode::TwoByte(TwoByteInstruction {
    cycles: 0,
    exec: |state, n| {
        state.set_register_8(Register::Flags, n);
        ExecResult::Ready
    },
}); 256];
