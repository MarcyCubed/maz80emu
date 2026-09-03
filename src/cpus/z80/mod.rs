//! Zilog Z80 CPU

mod bit_instructions;
mod indexed;
mod misc;
mod two_prefix;

use crate::instructions::micro::{bit, io, jump, ld, math, store_16, transfer};
use crate::instructions::{ExecResult, ExtraBytes, HALT, Instruction, NOP};
use crate::simple_instruction;
use crate::state::{Flags, Register, Register16};

/// Create a ld rr, nn instruction
macro_rules! ld_rr_nn {
    ($reg: expr) => {
        Instruction::Instruction {
            extra_bytes: ExtraBytes::Two,
            micros: &[|state| ld::ld_rr_nn(state, $reg, 0)],
            printer: |state| println!("ld {}, {:x}h", $reg, state.wz()),
        }
    };
}

pub(crate) use ld_rr_nn;

/// Create a ld (mm), rr instruction
macro_rules! ld_mm_rr {
    ($reg: expr) => {
        Instruction::Instruction {
            extra_bytes: ExtraBytes::Two,
            micros: &[|state| ld::ld_mm_rr(state, $reg), |_| ExecResult::Done(0)],
            printer: |state| println!("ld ({:x}h), {}", state.wz(), $reg),
        }
    };
}

pub(crate) use ld_mm_rr;

/// Create a ld rr, (mm) instruction
macro_rules! ld_rr_mm {
    ($reg: expr) => {
        Instruction::Instruction {
            extra_bytes: ExtraBytes::Two,
            micros: &[
                |state| {
                    state.memptr = state.wz().wrapping_add(1);
                    ExecResult::load16(state.wz())
                },
                |state| ld::ld_rr_rr(state, $reg, Register16::WZ, 0),
            ],
            printer: |state| println!("ld {}, ({:x}h)", $reg, state.wz()),
        }
    };
}

pub(crate) use ld_rr_mm;

/// Create a ld r, r instruction
macro_rules! ld_r_r {
    ( $dst:expr,  $src:expr ) => {
        Instruction::Instruction {
            extra_bytes: ExtraBytes::None,
            micros: &[|state| ld::ld_r_r(state, $dst, $src, 0)],
            printer: |_| println!("ld {}, {}", $dst, $src),
        }
    };
}
pub(crate) use ld_r_r;

/// Create a ld (pp), r instruction
macro_rules! ld_pp_r {
    ( $pointer:expr,  $reg:expr ) => {
        Instruction::Instruction {
            extra_bytes: ExtraBytes::None,
            micros: &[
                |state| ld::ld_pp_r(state, $pointer, $reg),
                |_| ExecResult::Done(0),
            ],
            printer: |_| println!("ld ({}), {}", $pointer, $reg),
        }
    };
}

/// Create a `ld (pp), a` instruction that affects `MEMPTR`
macro_rules! ld_pp_a_memptr {
    ( $pointer:expr ) => {
        Instruction::Instruction {
            extra_bytes: ExtraBytes::None,
            micros: &[
                |state| {
                    let mut memptr = state
                        .get_register_16($pointer)
                        .wrapping_add(1)
                        .to_le_bytes();
                    memptr[1] = state.a();
                    state.memptr = u16::from_le_bytes(memptr);
                    ld::ld_pp_r(state, $pointer, Register::A)
                },
                |_| ExecResult::Done(0),
            ],
            printer: |_| println!("ld ({}), a", $pointer),
        }
    };
}

/// Create an inc rr instruction
macro_rules! inc_rr {
    ( $reg:expr ) => {
        Instruction::Instruction {
            extra_bytes: ExtraBytes::None,
            micros: &[|state| math::inc_rr(state, $reg, 2)],
            printer: |_| println!("inc {}", $reg),
        }
    };
}

pub(crate) use inc_rr;

/// Create an inc r instruction
macro_rules! inc_r {
    ( $reg:expr ) => {
        Instruction::Instruction {
            extra_bytes: ExtraBytes::None,
            micros: &[|state| math::inc_r(state, $reg, 0)],
            printer: |_| println!("inc {}", $reg),
        }
    };
}
pub(crate) use inc_r;

/// Create a dec r instruction
macro_rules! dec_r {
    ( $reg:expr ) => {
        Instruction::Instruction {
            extra_bytes: ExtraBytes::None,
            micros: &[|state| math::dec_r(state, $reg, 0)],
            printer: |_| println!("dec {}", $reg),
        }
    };
}
pub(crate) use dec_r;

/// Create a ld r, n instruction
macro_rules! ld_r_n {
    ( $reg:expr ) => {
        Instruction::Instruction {
            extra_bytes: ExtraBytes::One,
            micros: &[|state| ld::ld_r_n(state, $reg, 0)],
            printer: |state| println!("ld {}, {:x}h", $reg, state.z()),
        }
    };
}
pub(crate) use ld_r_n;

/// Create an add rr, rr instruction
macro_rules! add_rr_rr {
    ( $dest:expr, $src:expr ) => {
        Instruction::Instruction {
            extra_bytes: ExtraBytes::None,
            micros: &[|state| math::add_rr_rr(state, $dest, $src, 7)],
            printer: |_| println!("add {}, {}", $dest, $src),
        }
    };
}

pub(crate) use add_rr_rr;

/// Create a ld r, (pp) instruction
macro_rules! ld_r_pp {
    ( $reg:expr, $pointer:expr ) => {
        Instruction::Instruction {
            extra_bytes: ExtraBytes::None,
            micros: &[
                |state| ExecResult::load(state.get_register_16($pointer)),
                |state| ld::ld_r_r(state, $reg, Register::Z, 0),
            ],
            printer: |_| println!("ld {}, ({})", $reg, $pointer),
        }
    };
}

/// Create a ld a, (pp) instruction that affects `MEMPTR`
macro_rules! ld_a_pp_memptr {
    ( $pointer:expr ) => {
        Instruction::Instruction {
            extra_bytes: ExtraBytes::None,
            micros: &[
                |state| {
                    let address = state.get_register_16($pointer);
                    state.memptr = address.wrapping_add(1);
                    ExecResult::load(address)
                },
                |state| ld::ld_r_r(state, Register::A, Register::Z, 0),
            ],
            printer: |_| println!("ld a, ({})", $pointer),
        }
    };
}

/// Create a dec rr instruction
macro_rules! dec_rr {
    ( $reg:expr ) => {
        Instruction::Instruction {
            extra_bytes: ExtraBytes::None,
            micros: &[|state| math::dec_rr(state, $reg, 2)],
            printer: |_| println!("dec {}", $reg),
        }
    };
}

pub(crate) use dec_rr;

/// Create a logic or arithmetic instruction
macro_rules! math_r {
    ( $function:ident, $text:expr, $reg:expr ) => {
        Instruction::Instruction {
            extra_bytes: ExtraBytes::None,
            micros: &[|state| math::$function(state, $reg, 0)],
            printer: |_| println!("{} {}", $text, $reg),
        }
    };
}
pub(crate) use math_r;

/// Create an add_a_r instruction
macro_rules! add_r {
    ( $reg:expr ) => {
        math_r!(add_a_r, "add a,", $reg)
    };
}
pub(crate) use add_r;

/// Create an adc_r instruction
macro_rules! adc_r {
    ( $reg:expr ) => {
        math_r!(adc_a_r, "adc a,", $reg)
    };
}
pub(crate) use adc_r;

/// Create a sub_r instruction
macro_rules! sub_r {
    ( $reg:expr ) => {
        math_r!(sub_r, "sub", $reg)
    };
}
pub(crate) use sub_r;

/// Create a sbc_r instruction
macro_rules! sbc_r {
    ( $reg:expr ) => {
        math_r!(sbc_r, "sbc", $reg)
    };
}
pub(crate) use sbc_r;

/// Create an and_r instruction
macro_rules! and_r {
    ( $reg:expr ) => {
        math_r!(and_r, "and", $reg)
    };
}
pub(crate) use and_r;

/// Create an or_r instruction
macro_rules! or_r {
    ( $reg:expr ) => {
        math_r!(or_r, "or", $reg)
    };
}
pub(crate) use or_r;

/// Create a xor_r instruction
macro_rules! xor_r {
    ( $reg:expr ) => {
        math_r!(xor_r, "xor", $reg)
    };
}
pub(crate) use xor_r;

/// Create a cp_r instruction
macro_rules! cp_r {
    ( $reg:expr ) => {
        math_r!(cp_r, "cp", $reg)
    };
}
pub(crate) use cp_r;

/// Create a pop rr instruction
macro_rules! pop_rr {
    ( $reg:expr ) => {
        Instruction::Instruction {
            extra_bytes: ExtraBytes::None,
            micros: &[jump::pop, |state| {
                ld::ld_rr_rr(state, $reg, Register16::WZ, 0)
            }],
            printer: |_| println!("pop {}", $reg),
        }
    };
}
pub(crate) use pop_rr;

macro_rules! push_rr {
    ( $reg:expr ) => {
        Instruction::Instruction {
            extra_bytes: ExtraBytes::None,
            micros: &[|state| jump::push(state, $reg), |_| ExecResult::Done(1)],
            printer: |_| println!("push {}", $reg),
        }
    };
}
pub(crate) use push_rr;

macro_rules! rst {
    ( $addr:literal ) => {
        Instruction::Instruction {
            extra_bytes: ExtraBytes::None,
            micros: &[
                |state| jump::push(state, Register16::PC),
                |state| {
                    state.memptr = $addr;
                    jump::jp(state, $addr, 1)
                },
            ],
            printer: |_| println!("rst {:x}h", $addr),
        }
    };
}

/// Create an `ex (sp), rr` instruction
macro_rules! ex_sp_rr {
    ($reg:expr) => {
        Instruction::Instruction {
            extra_bytes: ExtraBytes::None,
            micros: &[
                |state| ExecResult::load16(state.sp()),
                |state| store_16(state.sp(), state.get_register_16($reg)),
                |state| {
                    state.memptr = state.wz();
                    ld::ld_rr_rr(state, $reg, Register16::WZ, 3)
                },
            ],
            printer: |_| println!("ex (sp), {}", $reg),
        }
    };
}
pub(crate) use ex_sp_rr;

pub static Z80: [Instruction; 256] = [
    // Instruction 0x00: nop
    NOP,
    // Instruction 0x01: ld BC, nn
    ld_rr_nn!(Register16::BC),
    // Instruction 0x02: ld (bc), a
    ld_pp_a_memptr!(Register16::BC),
    // Instruction 0x03: inc bc
    inc_rr!(Register16::BC),
    // Instruction 0x04: inc b
    inc_r!(Register::B),
    // Instruction 0x05: dec b
    dec_r!(Register::B),
    // Instruction 0x06: ld b, n
    ld_r_n!(Register::B),
    // Instruction 0x07: rlca
    simple_instruction!("rlca", |state| bit::rlca(state, 0)),
    // Instruction 0x08: ex af, af'
    simple_instruction!("ex af, af'", |state| transfer::ex_af_af(state, 0)),
    // Instruction 0x09: add hl, bc
    add_rr_rr!(Register16::HL, Register16::BC),
    // Instruction 0x0a: ld a, (bc)
    ld_a_pp_memptr!(Register16::BC),
    // Instruction 0x0b: dec bc
    dec_rr!(Register16::BC),
    // Instruction 0x0c: inc c
    inc_r!(Register::C),
    // Instruction 0x0d: dec c
    dec_r!(Register::C),
    // Instruction 0x0e: ld c, n
    ld_r_n!(Register::C),
    // Instruction 0x0f: rrca
    simple_instruction!("rrca", |state| bit::rrca(state, 0)),
    // Instruction 0x10: djnz d
    Instruction::Instruction {
        extra_bytes: ExtraBytes::One,
        micros: &[|state| jump::djnz_d(state, 6, 1)],
        printer: |state| println!("djnz {:x}h", state.z() as i8),
    },
    // Instruction 0x11: ld de, nn
    ld_rr_nn!(Register16::DE),
    // Instruction 0x12: ld (de), a
    ld_pp_a_memptr!(Register16::DE),
    // Instruction 0x13: inc de
    inc_rr!(Register16::DE),
    // Instruction 0x14: inc d
    inc_r!(Register::D),
    // Instruction 0x15: dec d
    dec_r!(Register::D),
    // Instruction 0x16: ld d, n
    ld_r_n!(Register::D),
    // Instruction 0x17: rla
    simple_instruction!("rla", |state| bit::rla(state, 0)),
    // Instruction 0x18: jr d
    Instruction::Instruction {
        extra_bytes: ExtraBytes::One,
        micros: &[|state| jump::jr_cc_d(state, true, 5, 5)],
        printer: |state| println!("jr {}", state.z() as i8),
    },
    // Instruction 0x19: add hl, de
    add_rr_rr!(Register16::HL, Register16::DE),
    // Instruction 0x1a: ld a, (de)
    ld_a_pp_memptr!(Register16::DE),
    // Instruction 0x1b: dec de
    dec_rr!(Register16::DE),
    // Instruction 0x1c: inc e
    inc_r!(Register::E),
    // Instruction 0x1d: dec e
    dec_r!(Register::E),
    // Instruction 0x1e: ld e, n
    ld_r_n!(Register::E),
    // Instruction 0x1f: rra
    simple_instruction!("rra", |state| bit::rra(state, 0)),
    // Instruction 0x20: jr nz, d
    Instruction::Instruction {
        extra_bytes: ExtraBytes::One,
        micros: &[|state| jump::jr_cc_d(state, !state.get_flags().is_set(Flags::Z), 5, 0)],
        printer: |state| println!(" jr nz,{}", state.z() as i8),
    },
    // Instruction 0x21: ld hl, nn
    ld_rr_nn!(Register16::HL),
    // Instruction 0x22: ld (mm), hl
    ld_mm_rr!(Register16::HL),
    // Instruction 0x23: inc hl
    inc_rr!(Register16::HL),
    // Instruction 0x24: inc h
    inc_r!(Register::H),
    // Instruction 0x25: dec h
    dec_r!(Register::H),
    // Instruction 0x26: ld h, n
    ld_r_n!(Register::H),
    // Instruction 0x27: daa
    simple_instruction!("daa", |state| math::daa(state, 0)),
    // Instruction 0x28: jr z, d
    Instruction::Instruction {
        extra_bytes: ExtraBytes::One,
        micros: &[|state| jump::jr_cc_d(state, state.get_flags().is_set(Flags::Z), 5, 0)],
        printer: |state| println!("jr z, {}", state.z() as i8),
    },
    // Instruction 0x29: add hl, hl
    add_rr_rr!(Register16::HL, Register16::HL),
    // Instruction 0x2a: ld hl, (nn)
    ld_rr_mm!(Register16::HL),
    // Instruction 0x2b: dec hl
    dec_rr!(Register16::HL),
    // Instruction 0x2c: inc l
    inc_r!(Register::L),
    // Instruction 0x2d: dec l
    dec_r!(Register::L),
    // Instruction 0x2e: ld l, n
    ld_r_n!(Register::L),
    // Instruction 0x2f: cpl
    simple_instruction!("cpl", |state| math::cpl(state, 0)),
    // Instruction 0x30: jr nc, d
    Instruction::Instruction {
        extra_bytes: ExtraBytes::One,
        micros: &[|state| jump::jr_cc_d(state, !state.get_flags().is_set(Flags::C), 5, 0)],
        printer: |state| println!("jr nc, {}", state.z() as i8),
    },
    // Instruction 0x31: ld sp, nn
    ld_rr_nn!(Register16::SP),
    // Instruction 0x32: ld (nn), a
    Instruction::Instruction {
        extra_bytes: ExtraBytes::Two,
        micros: &[
            |state| {
                let mut memptr = state.wz().wrapping_add(1).to_le_bytes();
                memptr[1] = state.a();
                state.memptr = u16::from_le_bytes(memptr);
                ld::ld_mm_r(state, Register::A)
            },
            |_| ExecResult::Done(0),
        ],
        printer: |state| println!("ld ({:x}h) a", state.wz()),
    },
    // Instruction 0x33: inc sp
    inc_rr!(Register16::SP),
    // Instruction 0x34: inc (hl)
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| ExecResult::load(state.hl()),
            |state| math::inc_z_mem(state, state.hl()),
            |_| ExecResult::Done(1),
        ],
        printer: |_| println!("inc (hl)"),
    },
    // Instruction 0x35: dec (hl)
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| ExecResult::load(state.hl()),
            |state| math::dec_z_mem(state, state.hl()),
            |_| ExecResult::Done(1),
        ],
        printer: |_| println!("dec (hl)"),
    },
    // Instruction 0x36: ld (hl), n
    Instruction::Instruction {
        extra_bytes: ExtraBytes::One,
        micros: &[
            |state| ld::ld_pp_r(state, Register16::HL, Register::Z),
            |_| ExecResult::Done(0),
        ],
        printer: |state| println!("ld (hl), {:x}h", state.z()),
    },
    // Instruction 0x37: scf
    simple_instruction!("scf", |state| math::scf(state, 0)),
    // Instruction 0x38: jr c, d
    Instruction::Instruction {
        extra_bytes: ExtraBytes::One,
        micros: &[|state| jump::jr_cc_d(state, state.get_flags().is_set(Flags::C), 5, 0)],
        printer: |state| println!("jr c, {}", state.z() as i8),
    },
    // Instruction 0x39: add hl, sp
    add_rr_rr!(Register16::HL, Register16::SP),
    // Instruction 0x3a: ld a, (nn)
    Instruction::Instruction {
        extra_bytes: ExtraBytes::Two,
        micros: &[
            |state| {
                let address = state.wz();
                state.memptr = address.wrapping_add(1);
                ExecResult::load(address)
            },
            |state| ld::ld_r_r(state, Register::A, Register::Z, 0),
        ],
        printer: |state| println!("ld a, ({:x}h)", state.wz()),
    },
    // Instruction 0x3b: dec sp
    dec_rr!(Register16::SP),
    // Instruction 0x3c: inc a
    inc_r!(Register::A),
    // Instruction 0x3d: dec a
    dec_r!(Register::A),
    // Instruction 0x3e: ld a, n
    ld_r_n!(Register::A),
    // Instruction 0x3f: ccf
    simple_instruction!("ccf", |state| math::ccf(state, 0)),
    // Instruction 0x40: ld b, b
    ld_r_r!(Register::B, Register::B),
    // Instruction 0x41: ld b, c
    ld_r_r!(Register::B, Register::C),
    // Instruction 0x42: ld b, d
    ld_r_r!(Register::B, Register::D),
    // Instruction 0x43: ld b, e
    ld_r_r!(Register::B, Register::E),
    // Instruction 0x44: ld b, h
    ld_r_r!(Register::B, Register::H),
    // Instruction 0x45: ld b, l
    ld_r_r!(Register::B, Register::L),
    // Instruction 0x46: ld b, (hl)
    ld_r_pp!(Register::B, Register16::HL),
    // Instruction 0x47: ld b, a
    ld_r_r!(Register::B, Register::A),
    // Instruction 0x48: ld c, b
    ld_r_r!(Register::C, Register::B),
    // Instruction 0x49: ld c, c
    ld_r_r!(Register::C, Register::C),
    // Instruction 0x4a: ld c, d
    ld_r_r!(Register::C, Register::D),
    // Instruction 0x4b: ld c, e
    ld_r_r!(Register::C, Register::E),
    // Instruction 0x4c: ld c, h
    ld_r_r!(Register::C, Register::H),
    // Instruction 0x4d: ld c, l
    ld_r_r!(Register::C, Register::L),
    // Instruction 0x4e: ld c, (hl)
    ld_r_pp!(Register::C, Register16::HL),
    // Instruction 0x4f: ld c, a
    ld_r_r!(Register::C, Register::A),
    // Instruction 0x50: ld d, b
    ld_r_r!(Register::D, Register::B),
    // Instruction 0x51: ld d, c
    ld_r_r!(Register::D, Register::C),
    // Instruction 0x52: ld d, d
    ld_r_r!(Register::D, Register::D),
    // Instruction 0x53: ld d, e
    ld_r_r!(Register::D, Register::E),
    // Instruction 0x54: ld d, h
    ld_r_r!(Register::D, Register::H),
    // Instruction 0x55: ld d, l
    ld_r_r!(Register::D, Register::L),
    // Instruction 0x56: ld d, (hl)
    ld_r_pp!(Register::D, Register16::HL),
    // Instruction 0x57: ld d, a
    ld_r_r!(Register::D, Register::A),
    // Instruction 0x58: ld e, b
    ld_r_r!(Register::E, Register::B),
    // Instruction 0x59: ld e, c
    ld_r_r!(Register::E, Register::C),
    // Instruction 0x5a: ld e, d
    ld_r_r!(Register::E, Register::D),
    // Instruction 0x5b: ld e, e
    ld_r_r!(Register::E, Register::E),
    // Instruction 0x5c: ld e, h
    ld_r_r!(Register::E, Register::H),
    // Instruction 0x5d: ld e, l
    ld_r_r!(Register::E, Register::L),
    // Instruction 0x5e: ld e, (hl)
    ld_r_pp!(Register::E, Register16::HL),
    // Instruction 0x5f: ld e, a
    ld_r_r!(Register::E, Register::A),
    // Instruction 0x60: ld h, b
    ld_r_r!(Register::H, Register::B),
    // Instruction 0x61: ld h, c
    ld_r_r!(Register::H, Register::C),
    // Instruction 0x62: ld h, d
    ld_r_r!(Register::H, Register::D),
    // Instruction 0x63: ld h, e
    ld_r_r!(Register::H, Register::E),
    // Instruction 0x64: ld h, h
    ld_r_r!(Register::H, Register::H),
    // Instruction 0x65: ld h, l
    ld_r_r!(Register::H, Register::L),
    // Instruction 0x66: ld h, (hl)
    ld_r_pp!(Register::H, Register16::HL),
    // Instruction 0x67: ld h, a
    ld_r_r!(Register::H, Register::A),
    // Instruction 0x68: ld l, b
    ld_r_r!(Register::L, Register::B),
    // Instruction 0x69: ld l, c
    ld_r_r!(Register::L, Register::C),
    // Instruction 0x6a: ld l, d
    ld_r_r!(Register::L, Register::D),
    // Instruction 0x6b: ld l, e
    ld_r_r!(Register::L, Register::E),
    // Instruction 0x6c: ld l, h
    ld_r_r!(Register::L, Register::H),
    // Instruction 0x6d: ld l, l
    ld_r_r!(Register::L, Register::L),
    // Instruction 0x6e: ld l, (hl)
    ld_r_pp!(Register::L, Register16::HL),
    // Instruction 0x6f: ld l, a
    ld_r_r!(Register::L, Register::A),
    // Instruction 0x70: ld (hl), b
    ld_pp_r!(Register16::HL, Register::B),
    // Instruction 0x71: ld (hl), c
    ld_pp_r!(Register16::HL, Register::C),
    // Instruction 0x72: ld (hl), d
    ld_pp_r!(Register16::HL, Register::D),
    // Instruction 0x73: ld (hl), e
    ld_pp_r!(Register16::HL, Register::E),
    // Instruction 0x74: ld (hl), h
    ld_pp_r!(Register16::HL, Register::H),
    // Instruction 0x75: ld (hl), l
    ld_pp_r!(Register16::HL, Register::L),
    // Instruction 0x76: halt
    HALT,
    // Instruction 0x77: ld (hl), a
    ld_pp_r!(Register16::HL, Register::A),
    // Instruction 0x78: ld a, b
    ld_r_r!(Register::A, Register::B),
    // Instruction 0x79: ld a, c
    ld_r_r!(Register::A, Register::C),
    // Instruction 0x7a: ld a, d
    ld_r_r!(Register::A, Register::D),
    // Instruction 0x7b: ld a, e
    ld_r_r!(Register::A, Register::E),
    // Instruction 0x7c: ld a, h
    ld_r_r!(Register::A, Register::H),
    // Instruction 0x7d: ld a, l
    ld_r_r!(Register::A, Register::L),
    // Instruction 0x7e: ld a, (hl)
    ld_r_pp!(Register::A, Register16::HL),
    // Instruction 0x7f: ld a, a
    ld_r_r!(Register::A, Register::A),
    // Instruction 0x80: add a, b
    add_r!(Register::B),
    // Instruction 0x81: add a, c
    add_r!(Register::C),
    // Instruction 0x82: add a, d
    add_r!(Register::D),
    // Instruction 0x83: add a, e
    add_r!(Register::E),
    // Instruction 0x84: add a, h
    add_r!(Register::H),
    // Instruction 0x85: add a, l
    add_r!(Register::L),
    // Instruction 0x86: add a, (hl)
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| ExecResult::load(state.hl()),
            |state| math::add_a_r(state, Register::Z, 0),
        ],
        printer: |_| println!("add a, (hl)"),
    },
    // Instruction 0x87: add a, a
    add_r!(Register::A),
    // Instruction 0x88: adc a, b
    adc_r!(Register::B),
    // Instruction 0x89: adc a, c
    adc_r!(Register::C),
    // Instruction 0x8a: adc a, d
    adc_r!(Register::D),
    // Instruction 0x8b: adc a, e
    adc_r!(Register::E),
    // Instruction 0x8c: adc a, h
    adc_r!(Register::H),
    // Instruction 0x8d: adc a, l
    adc_r!(Register::L),
    // Instruction 0x8e: adc a, (hl)
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| ExecResult::load(state.hl()),
            |state| math::adc_a_r(state, Register::Z, 0),
        ],
        printer: |_| println!("adc a, (hl)"),
    },
    // Instruction 0x8f: adc a, a
    adc_r!(Register::A),
    // Instruction 0x90: sub b
    sub_r!(Register::B),
    // Instruction 0x91: sub c
    sub_r!(Register::C),
    // Instruction 0x92: sub d
    sub_r!(Register::D),
    // Instruction 0x93: sub e
    sub_r!(Register::E),
    // Instruction 0x94: sub h
    sub_r!(Register::H),
    // Instruction 0x95: sub l
    sub_r!(Register::L),
    // Instruction 0x96: sub (hl)
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| ExecResult::load(state.hl()),
            |state| math::sub_r(state, Register::Z, 0),
        ],
        printer: |_| println!("sub (hl)"),
    },
    // Instruction 0x97: sub a
    sub_r!(Register::A),
    // Instruction 0x98: sbc b
    sbc_r!(Register::B),
    // Instruction 0x99: sbc c
    sbc_r!(Register::C),
    // Instruction 0x9a: sbc d
    sbc_r!(Register::D),
    // Instruction 0x9b: sbc e
    sbc_r!(Register::E),
    // Instruction 0x9c: sbc h
    sbc_r!(Register::H),
    // Instruction 0x9d: sbc l
    sbc_r!(Register::L),
    // Instruction 0x9e: sbc (hl)
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| ExecResult::load(state.hl()),
            |state| math::sbc_r(state, Register::Z, 0),
        ],
        printer: |_| println!("sbc (hl)"),
    },
    // Instruction 0x9f: sbc a
    sbc_r!(Register::A),
    // Instruction 0xa0: and b
    and_r!(Register::B),
    // Instruction 0xa1: and c
    and_r!(Register::C),
    // Instruction 0xa2: and d
    and_r!(Register::D),
    // Instruction 0xa3: and e
    and_r!(Register::E),
    // Instruction 0xa4: and h
    and_r!(Register::H),
    // Instruction 0xa5: and l
    and_r!(Register::L),
    // Instruction 0xa6: and (hl)
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| ExecResult::load(state.hl()),
            |state| math::and_r(state, Register::Z, 0),
        ],
        printer: |_| println!("and (hl)"),
    },
    // Instruction 0xa7: and a
    and_r!(Register::A),
    // Instruction 0xa8: xor b
    xor_r!(Register::B),
    // Instruction 0xa9: xor c
    xor_r!(Register::C),
    // Instruction 0xaa: xor d
    xor_r!(Register::D),
    // Instruction 0xab: xor e
    xor_r!(Register::E),
    // Instruction 0xac: xor h
    xor_r!(Register::H),
    // Instruction 0xad: xor l
    xor_r!(Register::L),
    // Instruction 0xae: xor (hl)
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| ExecResult::load(state.hl()),
            |state| math::xor_r(state, Register::Z, 0),
        ],
        printer: |_| println!("xor (hl)"),
    },
    // Instruction 0xaf: xor a
    xor_r!(Register::A),
    // Instruction 0xba0: or b
    or_r!(Register::B),
    // Instruction 0xb1: or c
    or_r!(Register::C),
    // Instruction 0xb2: or d
    or_r!(Register::D),
    // Instruction 0xb3: or e
    or_r!(Register::E),
    // Instruction 0xb4: or h
    or_r!(Register::H),
    // Instruction 0xb5: or l
    or_r!(Register::L),
    // Instruction 0xb6: or (hl)
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| ExecResult::load(state.hl()),
            |state| math::or_r(state, Register::Z, 0),
        ],
        printer: |_| println!("or (hl)"),
    },
    // Instruction 0xb7: or a
    or_r!(Register::A),
    // Instruction 0xb8: cp b
    cp_r!(Register::B),
    // Instruction 0xb9: cp c
    cp_r!(Register::C),
    // Instruction 0xba: cp d
    cp_r!(Register::D),
    // Instruction 0xbb: cp e
    cp_r!(Register::E),
    // Instruction 0xbc: cp h
    cp_r!(Register::H),
    // Instruction 0xbd: cp l
    cp_r!(Register::L),
    // Instruction 0xbe: cp (hl)
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| ExecResult::load(state.hl()),
            |state| math::cp_r(state, Register::Z, 0),
        ],
        printer: |_| println!("cp (hl)"),
    },
    // Instruction 0xbf: cp a
    cp_r!(Register::A),
    // Instruction 0xc0: ret nz
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| jump::load_sp_or_break(state, !state.get_flags().is_set(Flags::Z), 1),
            |state| jump::ret(state, 1),
        ],
        printer: |_| println!("ret nz"),
    },
    // Instruction 0xc1: pop bc
    pop_rr!(Register16::BC),
    // Instruction 0xc2: jp nz, nn
    Instruction::Instruction {
        extra_bytes: ExtraBytes::Two,
        micros: &[|state| jump::jp_cc_nn(state, !state.get_flags().is_set(Flags::Z), 0)],
        printer: |state| println!("jp nz, {:x}h", state.wz()),
    },
    // Instruction 0xc3: jp nn
    Instruction::Instruction {
        extra_bytes: ExtraBytes::Two,
        micros: &[|state| jump::jp_cc_nn(state, true, 0)],
        printer: |state| println!("jp {:x}h", state.wz() as i16),
    },
    // Instruction 0xc4: call nz, nn
    Instruction::Instruction {
        extra_bytes: ExtraBytes::Two,
        micros: &[
            |state| jump::push_pc_or_break(state, !state.get_flags().is_set(Flags::Z), 0),
            |state| jump::jr_mm(state, 1),
        ],
        printer: |state| println!("call nz, {:x}h", state.wz()),
    },
    // Instruction 0xc5: push bc
    push_rr!(Register16::BC),
    // Instruction 0xc6: add a, n
    Instruction::Instruction {
        extra_bytes: ExtraBytes::One,
        micros: &[|state| math::add_a_r(state, Register::Z, 0)],
        printer: |state| println!("add a, {:x}h", state.z()),
    },
    // Instruction 0xc7: rst 00h
    rst!(0x00),
    // Instruction 0xc8: ret z
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| jump::load_sp_or_break(state, state.get_flags().is_set(Flags::Z), 1),
            |state| jump::ret(state, 1),
        ],
        printer: |_| println!("ret z"),
    },
    // Instruction 0xc9: ret
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| ExecResult::load16(state.sp()),
            |state| jump::ret(state, 1),
        ],
        printer: |_| println!("ret"),
    },
    // Instruction 0xca: jp z, nn
    Instruction::Instruction {
        extra_bytes: ExtraBytes::Two,
        micros: &[|state| jump::jp_cc_nn(state, state.get_flags().is_set(Flags::Z), 0)],
        printer: |state| println!("jp z, {:x}h", state.wz()),
    },
    // Bit instructions
    Instruction::Prefix(&bit_instructions::BIT_INSTRUCTIONS),
    // Instruction 0xcc: call z, nn
    Instruction::Instruction {
        extra_bytes: ExtraBytes::Two,
        micros: &[
            |state| jump::push_pc_or_break(state, state.get_flags().is_set(Flags::Z), 0),
            |state| jump::jr_mm(state, 1),
        ],
        printer: |state| println!("call z, {:x}h", state.wz()),
    },
    // Instruction 0xcd: call nn
    Instruction::Instruction {
        extra_bytes: ExtraBytes::Two,
        micros: &[
            |state| jump::push(state, Register16::PC),
            |state| {
                state.memptr = state.wz();
                jump::jr_mm(state, 1)
            },
        ],
        printer: |state| println!("call {:x}h", state.wz()),
    },
    // Instruction 0xce: adc a, n
    Instruction::Instruction {
        extra_bytes: ExtraBytes::One,
        micros: &[|state| math::adc_a_r(state, Register::Z, 0)],
        printer: |state| println!("adc a, {:x}h", state.z()),
    },
    // Instruction 0xcf: rst 08h
    rst!(0x8),
    // Instruction 0xd0: ret nc
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| jump::load_sp_or_break(state, !state.get_flags().is_set(Flags::C), 1),
            |state| jump::ret(state, 1),
        ],
        printer: |_| println!("ret nc"),
    },
    // Instruction 0xd1: pop de
    pop_rr!(Register16::DE),
    // Instruction 0xd2: jp nc, nn
    Instruction::Instruction {
        extra_bytes: ExtraBytes::Two,
        micros: &[|state| jump::jp_cc_nn(state, !state.get_flags().is_set(Flags::C), 0)],
        printer: |state| println!("jp nc, {:x}h", state.wz()),
    },
    // Instruction 0xd3: out (m), a
    Instruction::Instruction {
        extra_bytes: ExtraBytes::One,
        micros: &[|state| io::out_n_a(state), |_| ExecResult::Done(0)],
        printer: |state| println!("out ({:x}h), a", state.z()),
    },
    // Instruction 0xd4: call nc, nn
    Instruction::Instruction {
        extra_bytes: ExtraBytes::Two,
        micros: &[
            |state| jump::push_pc_or_break(state, !state.get_flags().is_set(Flags::C), 0),
            |state| jump::jr_mm(state, 1),
        ],
        printer: |state| println!("call nc, {:x}h", state.wz()),
    },
    // Instruction 0xd5: push de
    push_rr!(Register16::DE),
    // Instruction 0xd6: sub n
    Instruction::Instruction {
        extra_bytes: ExtraBytes::One,
        micros: &[|state| math::sub_r(state, Register::Z, 0)],
        printer: |state| println!("sub {:x}h", state.z()),
    },
    // Instruction 0xd7: rst 10h
    rst!(0x10),
    // Instruction 0xd8: ret c
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| jump::load_sp_or_break(state, state.get_flags().is_set(Flags::C), 1),
            |state| jump::ret(state, 1),
        ],
        printer: |_| println!("ret c"),
    },
    // Instruction 0xd9: exx
    simple_instruction!("exx", |state| transfer::exx(state, 0)),
    // Instruction 0xda: jp c, nn
    Instruction::Instruction {
        extra_bytes: ExtraBytes::Two,
        micros: &[|state| jump::jp_cc_nn(state, state.get_flags().is_set(Flags::C), 0)],
        printer: |state| println!("jp c, {:x}h", state.wz()),
    },
    // Instruction 0xdb: in a, (n)
    Instruction::Instruction {
        extra_bytes: ExtraBytes::One,
        micros: &[
            |state| {
                let port = (state.a() as u16) << 8 | (state.z() as u16);
                state.memptr = port.wrapping_add(1);
                ExecResult::input(port)
            },
            |state| ld::ld_r_r(state, Register::A, Register::Z, 0),
        ],
        printer: |state| println!("in a, ({:x}h)", state.z()),
    },
    // Instruction 0xdc: call c, nn
    Instruction::Instruction {
        extra_bytes: ExtraBytes::Two,
        micros: &[
            |state| jump::push_pc_or_break(state, state.get_flags().is_set(Flags::C), 0),
            |state| jump::jr_mm(state, 1),
        ],
        printer: |state| println!("call c, {:x}h", state.wz()),
    },
    // IX instructions
    Instruction::Prefix(&indexed::IX_TABLE),
    // Instruction 0xde: sbc a, n
    Instruction::Instruction {
        extra_bytes: ExtraBytes::One,
        micros: &[|state| math::sbc_r(state, Register::Z, 0)],
        printer: |state| println!("sbc a, {:x}h", state.z()),
    },
    // Instruction 0xdf: rst 18h
    rst!(0x18),
    // Instruction 0xe0: ret po
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| jump::load_sp_or_break(state, !state.get_flags().is_set(Flags::P), 1),
            |state| jump::ret(state, 1),
        ],
        printer: |_| println!("ret po"),
    },
    // Instruction 0xe1: pop hl
    pop_rr!(Register16::HL),
    // Instruction 0xe2: jp po, nn
    Instruction::Instruction {
        extra_bytes: ExtraBytes::Two,
        micros: &[|state| jump::jp_cc_nn(state, !state.get_flags().is_set(Flags::P), 0)],
        printer: |state| println!("jp po, {:x}h", state.wz()),
    },
    // Instruction 0xe3: ex (sp), hl
    ex_sp_rr!(Register16::HL),
    // Instruction 0xe4: call po, nn
    Instruction::Instruction {
        extra_bytes: ExtraBytes::Two,
        micros: &[
            |state| jump::push_pc_or_break(state, !state.get_flags().is_set(Flags::P), 0),
            |state| jump::jr_mm(state, 1),
        ],
        printer: |state| println!("call po, {:x}h", state.wz()),
    },
    // Instruction 0xe5: push hl
    push_rr!(Register16::HL),
    // Instruction 0xe6: and n
    Instruction::Instruction {
        extra_bytes: ExtraBytes::One,
        micros: &[|state| math::and_r(state, Register::Z, 0)],
        printer: |state| println!("and {:x}h", state.z()),
    },
    // Instruction 0xe7: rst 20h
    rst!(0x20),
    // Instruction 0xe8: ret pe
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| jump::load_sp_or_break(state, state.get_flags().is_set(Flags::P), 1),
            |state| jump::ret(state, 1),
        ],
        printer: |_| println!("ret pe"),
    },
    // Instruction 0xe9: jp (hl)
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| jump::jp(state, state.hl(), 0)],
        printer: |_| println!("jp (hl)"),
    },
    // Instruction 0xea: jp pe, nn
    Instruction::Instruction {
        extra_bytes: ExtraBytes::Two,
        micros: &[|state| jump::jp_cc_nn(state, state.get_flags().is_set(Flags::P), 0)],
        printer: |state| println!("jp pe, {:x}h", state.wz()),
    },
    // Instruction 0xeb: ex de, hl
    simple_instruction!("ex de, hl", |state| {
        let de = state.de_bytes();
        *state.de_mut() = state.hl_bytes();
        *state.hl_mut() = de;
        ExecResult::Done(0)
    }),
    // Instruction 0xec: call pe, nn
    Instruction::Instruction {
        extra_bytes: ExtraBytes::Two,
        micros: &[
            |state| jump::push_pc_or_break(state, state.get_flags().is_set(Flags::P), 0),
            |state| jump::jr_mm(state, 1),
        ],
        printer: |state| println!("call pe, {:x}h", state.wz()),
    },
    // Misc. instructions
    Instruction::Prefix(&misc::MISC_INSTRUCTIONS),
    // Instruction 0xee: xor n
    Instruction::Instruction {
        extra_bytes: ExtraBytes::One,
        micros: &[|state| math::xor_r(state, Register::Z, 0)],
        printer: |state| println!("xor {:x}h", state.z()),
    },
    // Instruction 0xef: rst 28h
    rst!(0x28),
    // Instruction 0xf0: ret p
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| jump::load_sp_or_break(state, !state.get_flags().is_set(Flags::S), 1),
            |state| jump::ret(state, 1),
        ],
        printer: |_| println!("ret p"),
    },
    // Instruction 0xf1: pop af
    pop_rr!(Register16::AF),
    // Instruction 0xf2: jp p, nn
    Instruction::Instruction {
        extra_bytes: ExtraBytes::Two,
        micros: &[|state| jump::jp_cc_nn(state, !state.get_flags().is_set(Flags::S), 0)],
        printer: |state| println!("jp p, {:x}h", state.wz()),
    },
    // Instruction 0xf3: di
    simple_instruction!("di", |state| {
        state.iff1 = false;
        state.iff2 = false;
        ExecResult::Done(0)
    }),
    // Instruction 0xf4: call p, nn
    Instruction::Instruction {
        extra_bytes: ExtraBytes::Two,
        micros: &[
            |state| jump::push_pc_or_break(state, !state.get_flags().is_set(Flags::S), 0),
            |state| jump::jr_mm(state, 1),
        ],
        printer: |state| println!("call p, {:x}h", state.wz()),
    },
    // Instruction 0xf5: push af
    push_rr!(Register16::AF),
    // Instruction 0xf6: or n
    Instruction::Instruction {
        extra_bytes: ExtraBytes::One,
        micros: &[|state| math::or_r(state, Register::Z, 0)],
        printer: |state| println!("or {:x}h", state.z()),
    },
    // Instruction 0xf7: rst 30h
    rst!(0x30),
    // Instruction 0xe8: ret m
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| jump::load_sp_or_break(state, state.get_flags().is_set(Flags::S), 1),
            |state| jump::ret(state, 1),
        ],
        printer: |_| println!("ret m"),
    },
    // Instruction 0xf9: ld sp, hl
    simple_instruction!("ld sp, hl", |state| {
        *state.sp_mut() = state.hl_bytes();
        ExecResult::Done(2)
    }),
    // Instruction 0xfa: jp m, nn
    Instruction::Instruction {
        extra_bytes: ExtraBytes::Two,
        micros: &[|state| jump::jp_cc_nn(state, state.get_flags().is_set(Flags::S), 0)],
        printer: |state| println!("jp m, {:x}h", state.wz()),
    },
    // Instruction 0xfb: ei
    simple_instruction!("ei", |state| {
        state.iff1 = true;
        state.iff2 = true;
        ExecResult::Ei(0)
    }),
    // Instruction 0xfc: call m, nn
    Instruction::Instruction {
        extra_bytes: ExtraBytes::Two,
        micros: &[
            |state| jump::push_pc_or_break(state, state.get_flags().is_set(Flags::S), 0),
            |state| jump::jr_mm(state, 1),
        ],
        printer: |state| println!("call m, {:x}h", state.wz()),
    },
    // IY instructions
    Instruction::Prefix(&indexed::IY_TABLE),
    // Instruction 0xfe: cp n
    Instruction::Instruction {
        extra_bytes: ExtraBytes::One,
        micros: &[|state| math::cp_r(state, Register::Z, 0)],
        printer: |state| println!("cp {:x}h", state.z()),
    },
    // Instruction 0xff: rst 38h
    rst!(0x38),
];
