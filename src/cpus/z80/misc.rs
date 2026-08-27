//! Miscellaneous instructions.
//!
//! These are instructions with the `EB` prefix.

use crate::instructions::ExecResult::Done;
use crate::instructions::micro::{io, jump, ld, math, transfer};
use crate::instructions::{ExecResult, ExtraBytes, Instruction, InstructionSet, micro};
use crate::state::Flags;
use crate::state::{InterruptMode, Register, Register16};
use crate::{one_byte_instruction, simple_instruction};

/// NOP instruction. Since it's a 2-bit instruction it takes longer that the standard NOP
const MISC_NOP: Instruction = Instruction::Instruction {
    extra_bytes: ExtraBytes::None,
    micros: &[|_| Done(8)],
    printer: |_| println!("nop     ; ED prefix"),
};

/// Generate instruction `in r, (c)`
macro_rules! in_r_c {
    ( $reg:expr ) => {
        Instruction::Instruction {
            extra_bytes: ExtraBytes::None,
            micros: &[
                |state| ExecResult::input(state.bc()),
                |state| io::in_r_bc(state, $reg, 12),
            ],
            printer: |_| println!("in {}, (c)", $reg),
        }
    };
}

/// Generate instruction `out (c), r`
macro_rules! out_c_r {
    ( $reg:expr ) => {
        Instruction::Instruction {
            extra_bytes: ExtraBytes::None,
            micros: &[
                |state| ExecResult::Out {
                    port: state.bc(),
                    data: state.get_register_8($reg),
                },
                |_| ExecResult::Done(12),
            ],
            printer: |_| println!("out (c), {}", $reg),
        }
    };
}

/// Generate instruction `sbc hl, rr`
macro_rules! sbc_hl_rr {
    ( $reg:expr ) => {
        Instruction::Instruction {
            extra_bytes: ExtraBytes::None,
            micros: &[|state| math::sbc_hl_rr(state, $reg, 15)],
            printer: |_| println!("sbc hl, {}", $reg),
        }
    };
}

/// Generate instruction `adc hl, rr`
macro_rules! adc_hl_rr {
    ( $reg:expr ) => {
        Instruction::Instruction {
            extra_bytes: ExtraBytes::None,
            micros: &[|state| math::adc_hl_rr(state, $reg, 15)],
            printer: |_| println!("adc hl, {}", $reg),
        }
    };
}

/// Generate instruction `ld (mm), rr`
macro_rules! ld_mm_rr {
    ( $reg:expr ) => {
        Instruction::Instruction {
            extra_bytes: ExtraBytes::Two,
            micros: &[|state| ld::ld_mm_rr(state, $reg), |_| ExecResult::Done(20)],
            printer: |state| println!("ld ({:x}h), {}", state.wz(), $reg),
        }
    };
}

/// Generate instruction `ld rr, (mm)`
macro_rules! ld_rr_mm {
    ( $reg:expr ) => {
        Instruction::Instruction {
            extra_bytes: ExtraBytes::Two,
            micros: &[
                |state| ExecResult::load16(state.wz()),
                |state| ld::ld_rr_rr(state, $reg, Register16::WZ, 20),
            ],
            printer: |state| println!("ld {}, ({:x}h)", $reg, state.wz()),
        }
    };
}

/// Create a ld r, a instruction
macro_rules! ld_r_a {
    ( $dst:expr ) => {
        Instruction::Instruction {
            extra_bytes: ExtraBytes::None,
            micros: &[|state| ld::ld_r_r(state, $dst, Register::A, 9)],
            printer: |_| println!("ld {}, a", $dst),
        }
    };
}

/// Create a ld a, r instruction
macro_rules! ld_a_r {
    ( $src:expr ) => {
        Instruction::Instruction {
            extra_bytes: ExtraBytes::None,
            micros: &[|state| {
                let value = state.get_register_8($src);
                let flags = Flags::from_value(value)
                    | Flags::V.set_if(state.iff2)
                    | (state.get_flags() & Flags::C);
                state.update_flags(flags);
                *state.a_mut() = value;
                ExecResult::Done(9)
            }],
            printer: |_| println!("ld a, {}", $src),
        }
    };
}

/// Instruction that loads something from memory, operates on it and stores it back
macro_rules! load_change_store {
    ( $name:literal,  $operation:expr, $cycles:literal ) => {
        one_byte_instruction!(
            $name,
            &[
                |state| ExecResult::load16(state.hl()),
                |state| {
                    $operation(state);
                    ExecResult::Store {
                        address: state.hl(),
                        data: state.z(),
                    }
                },
                |_| ExecResult::Done($cycles),
            ]
        )
    };
}

/// Instructions that transfers a byte from `(HL)` to `(BC)` then do something.
macro_rules! ldx {
    ( $name:literal,  $operation:expr ) => {
        one_byte_instruction!(
            $name,
            &[
                |state| ExecResult::load(state.hl()),
                |state| ExecResult::Store {
                    address: state.de(),
                    data: state.z(),
                },
                $operation
            ]
        )
    };
}

/// Instructions that load a byte from `(HL)` to `Z` then do something with it.
macro_rules! hl_load {
    ( $name:literal,  $operation:expr ) => {
        one_byte_instruction!($name, &[|state| ExecResult::load(state.hl()), $operation])
    };
}

/// Read a byte from the I/O port in `BC` to `(HL)` then do something with it.
macro_rules! hl_in {
    ( $name:literal,  $operation:expr ) => {
        one_byte_instruction!(
            $name,
            &[
                |state| ExecResult::input(state.bc()),
                |state| ExecResult::Store {
                    address: state.hl(),
                    data: state.z()
                },
                $operation
            ]
        )
    };
}

/// Write the byte in `(HL)` to the I/O port in `BC` to  then do something with it.
macro_rules! hl_out {
    ( $name:literal,  $operation:expr ) => {
        one_byte_instruction!(
            $name,
            &[
                micro::fetch_byte,
                |state| {
                    *state.b_mut() = state.b().wrapping_sub(1);
                    ExecResult::Store {
                        address: state.bc(),
                        data: state.z(),
                    }
                },
                $operation
            ]
        )
    };
}

/// Table of miscellaneous instructions
pub static MISC_INSTRUCTIONS: InstructionSet = {
    // Initialize all instructions with NOP.
    let mut table = [MISC_NOP; _];
    // in r, (c) instructions
    table[0x40] = in_r_c!(Register::B);
    table[0x48] = in_r_c!(Register::C);
    table[0x50] = in_r_c!(Register::D);
    table[0x58] = in_r_c!(Register::E);
    table[0x60] = in_r_c!(Register::H);
    table[0x68] = in_r_c!(Register::L);
    table[0x70] = in_r_c!(Register::Z); // Undocumented instruction
    table[0x78] = in_r_c!(Register::A);
    // out (c), r instructions
    table[0x41] = out_c_r!(Register::B);
    table[0x49] = out_c_r!(Register::C);
    table[0x51] = out_c_r!(Register::D);
    table[0x59] = out_c_r!(Register::E);
    table[0x61] = out_c_r!(Register::H);
    table[0x69] = out_c_r!(Register::L);
    table[0x71] = out_c_r!(Register::Z); // Undocumented instruction
    table[0x79] = out_c_r!(Register::A);
    // sbc hl, rr
    table[0x42] = sbc_hl_rr!(Register16::BC);
    table[0x52] = sbc_hl_rr!(Register16::DE);
    table[0x62] = sbc_hl_rr!(Register16::HL);
    table[0x72] = sbc_hl_rr!(Register16::SP);
    // adc hl, rr
    table[0x4A] = adc_hl_rr!(Register16::BC);
    table[0x5A] = adc_hl_rr!(Register16::DE);
    table[0x6A] = adc_hl_rr!(Register16::HL);
    table[0x7A] = adc_hl_rr!(Register16::SP);
    // ld (mm), rr
    table[0x43] = ld_mm_rr!(Register16::BC);
    table[0x53] = ld_mm_rr!(Register16::DE);
    table[0x63] = ld_mm_rr!(Register16::HL); // Undocumented instruction
    table[0x73] = ld_mm_rr!(Register16::SP);
    // ld rr, (mm)
    table[0x4b] = ld_rr_mm!(Register16::BC);
    table[0x5b] = ld_rr_mm!(Register16::DE);
    table[0x6b] = ld_rr_mm!(Register16::HL); // Undocumented instruction
    table[0x7b] = ld_rr_mm!(Register16::SP);
    // neg
    table[0x44] = simple_instruction!("neg", |state| math::neg(state, 8));
    // im n
    table[0x46] = simple_instruction!("im 0", |state| {
        state.interrupt_mode = InterruptMode::Instruction;
        ExecResult::Done(8)
    });
    table[0x56] = simple_instruction!("im 0", |state| {
        state.interrupt_mode = InterruptMode::Rst0038;
        ExecResult::Done(8)
    });
    table[0x53] = simple_instruction!("im 0", |state| {
        state.interrupt_mode = InterruptMode::Vectored;
        ExecResult::Done(8)
    });
    // Extra ld r, r
    table[0x47] = ld_r_a!(Register::I);
    table[0x4f] = ld_r_a!(Register::R);
    table[0x57] = ld_a_r!(Register::I);
    table[0x5f] = ld_a_r!(Register::R);
    // nybble rotation
    table[0x67] = load_change_store!("rrd", |state| math::rrd(state), 18);
    table[0x6f] = load_change_store!("rld", |state| math::rld(state), 18);
    // Interrupt returns
    table[0x45] = one_byte_instruction!(
        "retn",
        &[
            |state| ExecResult::load16(state.sp()),
            |state| {
                state.iff1 = state.iff2;
                jump::ret(state, 14)
            },
        ]
    );
    table[0x4d] = one_byte_instruction!(
        "reti",
        &[
            |state| ExecResult::load16(state.sp()),
            |state| {
                jump::ret(state, 0);
                ExecResult::Reti(14)
            },
        ]
    );
    // Block transfer
    table[0xa0] = ldx!("ldi", |state| transfer::ldi_registers(state, 16));
    table[0xb0] = ldx!("ldir", |state| transfer::ldir_registers(state, 21, 16));
    table[0xa8] = ldx!("ldd", |state| transfer::ldd_registers(state, 16));
    table[0xb8] = ldx!("lddr", |state| transfer::lddr_registers(state, 21, 16));
    table[0xa1] = hl_load!("cpi", |state| transfer::cpi_registers(state, 16));
    table[0xb1] = hl_load!("cpir", |state| transfer::cpir_registers(state, 21, 16));
    table[0xa9] = hl_load!("cpd", |state| transfer::cpd_registers(state, 16));
    table[0xb9] = hl_load!("cpdr", |state| transfer::cpdr_registers(state, 21, 16));
    table[0xa2] = hl_in!("ini", |state| transfer::ini_registers(state, 16));
    table[0xb2] = hl_in!("inir", |state| transfer::inir_registers(state, 21, 16));
    table[0xaa] = hl_in!("ind", |state| transfer::ini_registers(state, 16));
    table[0xba] = hl_in!("indr", |state| transfer::inir_registers(state, 21, 16));
    table[0xa3] = hl_out!("outi", |state| transfer::outi_registers(state, 16));
    table[0xb3] = hl_out!("otir", |state| transfer::otir_registers(state, 21, 16));
    table[0xab] = hl_out!("outd", |state| transfer::outd_registers(state, 16));
    table[0xbb] = hl_out!("otdr", |state| transfer::otdr_registers(state, 21, 16));

    table
};
