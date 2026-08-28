//! Bit instructions with the CB prefix.

use crate::instructions::micro::bit;
use crate::instructions::{ExecResult, ExtraBytes, Instruction, InstructionSet, UNIMPLEMENTED};
use crate::state::Register;

/// Bit instruction without parameters
macro_rules! simple_bit_instruction {
    ($name:expr, $op: expr, $reg:expr) => {
        Instruction::Instruction {
            extra_bytes: ExtraBytes::None,
            micros: &[|state| $op(state, $reg, 0)],
            printer: |_| println!("{} {}", $name, $reg),
        }
    };
}

/// Define 8 bit instructions for each register and memory
macro_rules! bit_instruction_group {
    ($name:literal, $op: expr) => {
        [
            simple_bit_instruction!($name, $op, Register::B),
            simple_bit_instruction!($name, $op, Register::C),
            simple_bit_instruction!($name, $op, Register::D),
            simple_bit_instruction!($name, $op, Register::E),
            simple_bit_instruction!($name, $op, Register::H),
            simple_bit_instruction!($name, $op, Register::L),
            Instruction::Instruction {
                extra_bytes: ExtraBytes::None,
                micros: &[
                    |state| ExecResult::load(state.hl()),
                    |state| {
                        $op(state, Register::Z, 0);
                        ExecResult::Store {
                            address: state.hl(),
                            data: state.z(),
                        }
                    },
                    |_| ExecResult::Done(1),
                ],
                printer: |_| println!("{} (hl)", $name),
            },
            simple_bit_instruction!($name, $op, Register::A),
        ]
    };
}

/// Instruction on a specific bit
macro_rules! bit_select_instruction {
    ($name:expr, $op: expr, $bit:literal, $reg:expr) => {
        Instruction::Instruction {
            extra_bytes: ExtraBytes::None,
            micros: &[|state| $op(state, $reg, $bit, 0)],
            printer: |_| println!("{} {} {}", $name, $bit, $reg),
        }
    };
}

/// Define instructions on specific bits for each register and memory
macro_rules! bit_select_group {
    ($name:literal, $op: expr, $bit:literal) => {
        [
            bit_select_instruction!($name, $op, $bit, Register::B),
            bit_select_instruction!($name, $op, $bit, Register::C),
            bit_select_instruction!($name, $op, $bit, Register::D),
            bit_select_instruction!($name, $op, $bit, Register::E),
            bit_select_instruction!($name, $op, $bit, Register::H),
            bit_select_instruction!($name, $op, $bit, Register::L),
            Instruction::Instruction {
                extra_bytes: ExtraBytes::None,
                micros: &[
                    |state| ExecResult::load(state.hl()),
                    |state| {
                        $op(state, Register::Z, $bit, 0);
                        ExecResult::Store {
                            address: state.hl(),
                            data: state.z(),
                        }
                    },
                    |_| ExecResult::Done(1),
                ],
                printer: |_| println!("{} (hl)", $name),
            },
            bit_select_instruction!($name, $op, $bit, Register::A),
        ]
    };
}

/// Table with the bit instructions
pub static BIT_INSTRUCTIONS: InstructionSet = {
    // Static replacement for copy_from_slice
    const fn copy_8(dest: &mut InstructionSet, offset: usize, src: [Instruction; 8]) {
        let mut i = 0usize;
        while i < 8 {
            dest[offset + i] = src[i];
            i += 1;
        }
    }

    let mut instructions = [UNIMPLEMENTED; 256];
    copy_8(
        &mut instructions,
        0x00,
        bit_instruction_group!("rlc", bit::rlc_r),
    );
    copy_8(
        &mut instructions,
        0x08,
        bit_instruction_group!("rrc", bit::rrc_r),
    );
    copy_8(
        &mut instructions,
        0x10,
        bit_instruction_group!("rl", bit::rlc_r),
    );
    copy_8(
        &mut instructions,
        0x18,
        bit_instruction_group!("rr", bit::rrc_r),
    );
    copy_8(
        &mut instructions,
        0x20,
        bit_instruction_group!("sla", bit::sla_r),
    );
    copy_8(
        &mut instructions,
        0x28,
        bit_instruction_group!("sra", bit::sra_r),
    );
    copy_8(
        &mut instructions,
        0x30,
        bit_instruction_group!("sll", bit::sll_r),
    );
    copy_8(
        &mut instructions,
        0x38,
        bit_instruction_group!("srl", bit::srl_r),
    );
    copy_8(
        &mut instructions,
        0x40,
        bit_select_group!("bit", bit::bit_r, 0),
    );
    copy_8(
        &mut instructions,
        0x48,
        bit_select_group!("bit", bit::bit_r, 1),
    );
    copy_8(
        &mut instructions,
        0x50,
        bit_select_group!("bit", bit::bit_r, 2),
    );
    copy_8(
        &mut instructions,
        0x58,
        bit_select_group!("bit", bit::bit_r, 3),
    );
    copy_8(
        &mut instructions,
        0x60,
        bit_select_group!("bit", bit::bit_r, 4),
    );
    copy_8(
        &mut instructions,
        0x68,
        bit_select_group!("bit", bit::bit_r, 5),
    );
    copy_8(
        &mut instructions,
        0x70,
        bit_select_group!("bit", bit::bit_r, 6),
    );
    copy_8(
        &mut instructions,
        0x78,
        bit_select_group!("bit", bit::bit_r, 7),
    );
    copy_8(
        &mut instructions,
        0x80,
        bit_select_group!("res", bit::res_r, 0),
    );
    copy_8(
        &mut instructions,
        0x88,
        bit_select_group!("res", bit::res_r, 1),
    );
    copy_8(
        &mut instructions,
        0x90,
        bit_select_group!("res", bit::res_r, 2),
    );
    copy_8(
        &mut instructions,
        0x98,
        bit_select_group!("res", bit::res_r, 3),
    );
    copy_8(
        &mut instructions,
        0xa0,
        bit_select_group!("res", bit::res_r, 4),
    );
    copy_8(
        &mut instructions,
        0xa8,
        bit_select_group!("res", bit::res_r, 5),
    );
    copy_8(
        &mut instructions,
        0xb0,
        bit_select_group!("res", bit::res_r, 6),
    );
    copy_8(
        &mut instructions,
        0xb8,
        bit_select_group!("res", bit::res_r, 7),
    );
    copy_8(
        &mut instructions,
        0xc0,
        bit_select_group!("set", bit::set_r, 0),
    );
    copy_8(
        &mut instructions,
        0xc8,
        bit_select_group!("set", bit::set_r, 1),
    );
    copy_8(
        &mut instructions,
        0xd0,
        bit_select_group!("set", bit::set_r, 2),
    );
    copy_8(
        &mut instructions,
        0xd8,
        bit_select_group!("set", bit::set_r, 3),
    );
    copy_8(
        &mut instructions,
        0xe0,
        bit_select_group!("set", bit::set_r, 4),
    );
    copy_8(
        &mut instructions,
        0xe8,
        bit_select_group!("set", bit::set_r, 5),
    );
    copy_8(
        &mut instructions,
        0xf0,
        bit_select_group!("set", bit::set_r, 6),
    );
    copy_8(
        &mut instructions,
        0xf8,
        bit_select_group!("set", bit::set_r, 7),
    );

    instructions
};
