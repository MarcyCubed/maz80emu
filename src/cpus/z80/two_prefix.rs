//! Two prefixes bit instructions.
//!
//! This file handles IX and IY bit instructions.

use crate::cpus::z80::indexed::{IX, IY, Index};
use crate::instructions::micro::bit;
use crate::instructions::{
    ExecResult, TwoPrefixInstruction, TwoPrefixTable,
};
use crate::state::Register;

macro_rules! load_store_instruction {
    ($name:expr, $op: expr, $reg:expr) => {
        TwoPrefixInstruction {
            printer: |state| {
                print!("{} ({}{:+})", $name, I::REGISTER, state.w() as i8);
                if $reg != Register::Z {
                    println!(", {}", $reg);
                } else {
                    println!();
                }
            },
            micros: &[
                |state| ExecResult::load(I::get_offset_w(state)),
                |state| {
                    $op(state, Register::Z, 0);
                    state.set_register_8($reg, state.z());
                    ExecResult::Store {
                        address: I::get_offset_w(state),
                        data: state.z(),
                    }
                },
                |_| ExecResult::Done(3),
            ],
        }
    };
}

/// Define 8 instructions for each register and memory that load change and save the value at `iz+d`
macro_rules! load_store_group {
    ($name:literal, $op: expr) => {
        [
            load_store_instruction!($name, $op, Register::B),
            load_store_instruction!($name, $op, Register::C),
            load_store_instruction!($name, $op, Register::D),
            load_store_instruction!($name, $op, Register::E),
            load_store_instruction!($name, $op, Register::H),
            load_store_instruction!($name, $op, Register::L),
            load_store_instruction!($name, $op, Register::Z),
            load_store_instruction!($name, $op, Register::A),
        ]
    };
}

// Make a bit instruction
macro_rules! bit_instruction {
    ($bit:literal) => {
        TwoPrefixInstruction {
            printer: |state| println!("bit {}, {}{:+}", $bit, I::REGISTER, state.w() as i8),
            micros: &[
                |state| ExecResult::load(I::get_offset_w(state)),
                |state| {
                    bit::bit_r(state, Register::Z, $bit, 0);
                    ExecResult::Store {
                        address: I::get_offset_w(state),
                        data: state.z(),
                    }
                },
                |_| ExecResult::Done(0),
            ],
        }
    };
}

macro_rules! bit_select_instruction {
    ($name:expr, $op: expr, $bit:literal, $reg:expr) => {
        TwoPrefixInstruction {
            printer: |state| {
                print!("{} {}, ({}{:+})", $name, $bit, I::REGISTER, state.w() as i8);
                if $reg != Register::Z {
                    println!(", {}", $reg);
                } else {
                    println!();
                }
            },
            micros: &[
                |state| ExecResult::load(I::get_offset_w(state)),
                |state| {
                    $op(state, Register::Z, $bit, 0);
                    state.set_register_8($reg, state.z());
                    ExecResult::Store {
                        address: I::get_offset_w(state),
                        data: state.z(),
                    }
                },
                |_| ExecResult::Done(3),
            ],
        }
    };
}

macro_rules! bit_group {
    ($name:expr, $op: expr, $bit:literal) => {
        [
            bit_select_instruction!($name, $op, $bit, Register::B),
            bit_select_instruction!($name, $op, $bit, Register::C),
            bit_select_instruction!($name, $op, $bit, Register::D),
            bit_select_instruction!($name, $op, $bit, Register::E),
            bit_select_instruction!($name, $op, $bit, Register::H),
            bit_select_instruction!($name, $op, $bit, Register::L),
            bit_select_instruction!($name, $op, $bit, Register::Z),
            bit_select_instruction!($name, $op, $bit, Register::A),
        ]
    };
}

pub(super) const fn make_two_prefixes_instructions<I: Index>() -> TwoPrefixTable {
    // Static replacement for copy_from_slice
    const fn copy_8(dest: &mut TwoPrefixTable, offset: usize, src: [TwoPrefixInstruction; 8]) {
        let mut i = 0usize;
        while i < 8 {
            dest[offset + i] = src[i];
            i += 1;
        }
    }

    /// Copy the same instruction 8 times in sequence to the table
    const fn repeat_8(dest: &mut TwoPrefixTable, offset: usize, instruction: TwoPrefixInstruction) {
        let mut i = 0usize;
        while i < 8 {
            dest[offset + i] = instruction;
            i += 1;
        }
    }

    let mut table = [TwoPrefixInstruction {
        micros: &[|_| unimplemented!("Instruction isn't implemented")],
        printer: |_| println!("crash"),
    }; 256];

    copy_8(&mut table, 0x00, load_store_group!("rlc", bit::rlc_r));
    copy_8(&mut table, 0x08, load_store_group!("rrc", bit::rrc_r));
    copy_8(&mut table, 0x10, load_store_group!("rl", bit::rl_r));
    copy_8(&mut table, 0x18, load_store_group!("rr", bit::rr_r));
    copy_8(&mut table, 0x20, load_store_group!("sla", bit::sla_r));
    copy_8(&mut table, 0x28, load_store_group!("sra", bit::sra_r));
    copy_8(&mut table, 0x30, load_store_group!("sll", bit::sll_r));
    copy_8(&mut table, 0x38, load_store_group!("srl", bit::srl_r));
    repeat_8(&mut table, 0x40, bit_instruction!(0));
    repeat_8(&mut table, 0x48, bit_instruction!(1));
    repeat_8(&mut table, 0x50, bit_instruction!(2));
    repeat_8(&mut table, 0x58, bit_instruction!(3));
    repeat_8(&mut table, 0x60, bit_instruction!(4));
    repeat_8(&mut table, 0x68, bit_instruction!(5));
    repeat_8(&mut table, 0x70, bit_instruction!(6));
    repeat_8(&mut table, 0x78, bit_instruction!(7));
    copy_8(&mut table, 0x80, bit_group!("res", bit::res_r, 0));
    copy_8(&mut table, 0x88, bit_group!("res", bit::res_r, 1));
    copy_8(&mut table, 0x90, bit_group!("res", bit::res_r, 2));
    copy_8(&mut table, 0x98, bit_group!("res", bit::res_r, 3));
    copy_8(&mut table, 0xa0, bit_group!("res", bit::res_r, 4));
    copy_8(&mut table, 0xa8, bit_group!("res", bit::res_r, 5));
    copy_8(&mut table, 0xb0, bit_group!("res", bit::res_r, 6));
    copy_8(&mut table, 0xb8, bit_group!("res", bit::res_r, 7));
    copy_8(&mut table, 0xc0, bit_group!("set", bit::set_r, 0));
    copy_8(&mut table, 0xc8, bit_group!("set", bit::set_r, 1));
    copy_8(&mut table, 0xd0, bit_group!("set", bit::set_r, 2));
    copy_8(&mut table, 0xd8, bit_group!("set", bit::set_r, 3));
    copy_8(&mut table, 0xe0, bit_group!("set", bit::set_r, 4));
    copy_8(&mut table, 0xe8, bit_group!("set", bit::set_r, 5));
    copy_8(&mut table, 0xf0, bit_group!("set", bit::set_r, 6));
    copy_8(&mut table, 0xf8, bit_group!("set", bit::set_r, 7));
    table
}

/// Table of IX bit instructions
pub(super) static IX_TABLE: TwoPrefixTable = make_two_prefixes_instructions::<IX>();

/// Table of IY bit instructions
pub(super) static IY_TABLE: TwoPrefixTable = make_two_prefixes_instructions::<IY>();
