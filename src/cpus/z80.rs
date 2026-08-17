//! Zilog Z80 CPU

use crate::instructions::micro::{bit, jump, ld, load_8, math, transfer};
use crate::instructions::{ExecResult, ExtraBytes, Instruction, NOP, UNIMPLEMENTED};
use crate::state::{Register, Register16};

pub static Z80: [Instruction; 256] = [
    // Instruction 0x00: nop
    NOP,
    // Instruction 0x01: ld BC, nn
    Instruction::Instruction {
        extra_bytes: ExtraBytes::Two,
        micros: &[|state| ld::ld_rr_nn(state, Register16::BC, 10)],
    },
    // Instruction 0x02: ld (bc), a
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| ld::ld_pp_r(state, Register16::BC, Register::A),
            |_| ExecResult::Done(7),
        ],
    },
    // Instruction 0x03: inc bc
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::inc_rr(state, Register16::BC, 6)],
    },
    // Instruction 0x04: inc b
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::inc_r(state, Register::B, 4)],
    },
    // Instruction 0x05: dec b
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::dec_r(state, Register::B, 4)],
    },
    // Instruction 0x06: ld b, n
    Instruction::Instruction {
        extra_bytes: ExtraBytes::One,
        micros: &[|state| ld::ld_r_n(state, Register::B, 7)],
    },
    // Instruction 0x07: rlca
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| bit::rlca(state, 4)],
    },
    // Instruction 0x08: ex af, af'
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| transfer::ex_af_af(state, 4)],
    },
    // Instruction 0x09: add hl, bc
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::add_hl_rr(state, Register16::BC, 11)],
    },
    // Instruction 0x0a: ld a, (bc)
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| ld::ld_r_pp(state, Register::A, Register16::BC),
            |_| ExecResult::Done(7),
        ],
    },
    // Instruction 0x0b: dec bc
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::dec_rr(state, Register16::BC, 6)],
    },
    // Instruction 0x0c: inc c
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::inc_r(state, Register::C, 4)],
    },
    // Instruction 0x0d: dec c
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::dec_r(state, Register::C, 4)],
    },
    // Instruction 0x0e: ld c, n
    Instruction::Instruction {
        extra_bytes: ExtraBytes::One,
        micros: &[|state| ld::ld_r_n(state, Register::C, 7)],
    },
    // Instruction 0x0f: rrca
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| bit::rrca(state, 4)],
    },
    // Instruction 0x10: djnz d
    Instruction::Instruction {
        extra_bytes: ExtraBytes::One,
        micros: &[|state| jump::djnz_d(state, 13, 8)],
    },
    // Instruction 0x11: ld de, nn
    Instruction::Instruction {
        extra_bytes: ExtraBytes::Two,
        micros: &[|state| ld::ld_rr_nn(state, Register16::DE, 10)],
    },
    // Instruction 0x12: ld (de), a
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| ld::ld_pp_r(state, Register16::DE, Register::A),
            |_| ExecResult::Done(7),
        ],
    },
    // Instruction 0x13: inc de
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::inc_rr(state, Register16::DE, 6)],
    },
    // Instruction 0x14: inc d
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::inc_r(state, Register::D, 4)],
    },
    // Instruction 0x15: dec d
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::dec_r(state, Register::D, 4)],
    },
    // Instruction 0x16: ld d, n
    Instruction::Instruction {
        extra_bytes: ExtraBytes::One,
        micros: &[|state| ld::ld_r_n(state, Register::D, 7)],
    },
    // Instruction 0x17: rla
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| bit::rla(state, 4)],
    },
    // Instruction 0x18: jr d
    Instruction::Instruction {
        extra_bytes: ExtraBytes::One,
        micros: &[|state| jump::jr_d(state, 12)],
    },
    // Instruction 0x19: add hl, de
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::add_hl_rr(state, Register16::DE, 11)],
    },
    // Instruction 0x1a: ld a, (de)
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| ld::ld_r_pp(state, Register::A, Register16::DE),
            |_| ExecResult::Done(7),
        ],
    },
    // Instruction 0x1b: dec de
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::dec_rr(state, Register16::DE, 6)],
    },
    // Instruction 0x1c: inc e
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::inc_r(state, Register::E, 4)],
    },
    // Instruction 0x1d: dec e
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::dec_r(state, Register::E, 4)],
    },
    // Instruction 0x1e: ld e, n
    Instruction::Instruction {
        extra_bytes: ExtraBytes::One,
        micros: &[|state| ld::ld_r_n(state, Register::E, 7)],
    },
    // Instruction 0x1f: rra
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| bit::rra(state, 4)],
    },
    // Instruction 0x20: jr nz, d
    Instruction::Instruction {
        extra_bytes: ExtraBytes::One,
        micros: &[|state| jump::jr_nz_d(state, 12, 7)],
    },
    // Instruction 0x21: ld hl, nn
    Instruction::Instruction {
        extra_bytes: ExtraBytes::Two,
        micros: &[|state| ld::ld_rr_nn(state, Register16::HL, 10)],
    },
    // Instruction 0x22: ld (nn), hl
    Instruction::Instruction {
        extra_bytes: ExtraBytes::Two,
        micros: &[
            |state| ld::ld_mm_rr(state, Register16::HL),
            |_| ExecResult::Done(16),
        ],
    },
    // Instruction 0x23: inc hl
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::inc_rr(state, Register16::HL, 6)],
    },
    // Instruction 0x24: inc h
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::inc_r(state, Register::H, 4)],
    },
    // Instruction 0x25: dec h
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::dec_r(state, Register::H, 4)],
    },
    // Instruction 0x26: ld h, n
    Instruction::Instruction {
        extra_bytes: ExtraBytes::One,
        micros: &[|state| ld::ld_r_n(state, Register::H, 7)],
    },
    // Instruction 0x27: daa
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::daa(state, 4)],
    },
    // Instruction 0x28: jr z, d
    Instruction::Instruction {
        extra_bytes: ExtraBytes::One,
        micros: &[|state| jump::jr_z_d(state, 12, 7)],
    },
    // Instruction 0x29: add hl, hl
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::add_hl_rr(state, Register16::HL, 11)],
    },
    // Instruction 0x2a: ld hl, (nn)
    Instruction::Instruction {
        extra_bytes: ExtraBytes::Two,
        micros: &[
            |state| ld::ld_rr_mm(state, Register16::HL),
            |_| ExecResult::Done(16),
        ],
    },
    // Instruction 0x2b: dec hl
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::dec_rr(state, Register16::HL, 6)],
    },
    // Instruction 0x2c: inc l
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::inc_r(state, Register::L, 4)],
    },
    // Instruction 0x2d: dec l
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::dec_r(state, Register::L, 4)],
    },
    // Instruction 0x2e: ld l, n
    Instruction::Instruction {
        extra_bytes: ExtraBytes::One,
        micros: &[|state| ld::ld_r_n(state, Register::L, 7)],
    },
    // Instruction 0x2f: cpl
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::cpl(state, 4)],
    },
    // Instruction 0x30: jr nc, d
    Instruction::Instruction {
        extra_bytes: ExtraBytes::One,
        micros: &[|state| jump::jr_nc_d(state, 12, 7)],
    },
    // Instruction 0x31: ld sp, nn
    Instruction::Instruction {
        extra_bytes: ExtraBytes::Two,
        micros: &[|state| ld::ld_rr_nn(state, Register16::SP, 10)],
    },
    // Instruction 0x32: ld (nn), a
    Instruction::Instruction {
        extra_bytes: ExtraBytes::Two,
        micros: &[
            |state| ld::ld_mm_r(state, Register::A),
            |_| ExecResult::Done(13),
        ],
    },
    // Instruction 0x33: inc sp
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::inc_rr(state, Register16::SP, 6)],
    },
    // Instruction 0x34: inc (hl)
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| load_8(state, state.hl()),
            |state| math::inc_z_mem(state, state.hl()),
            |_| ExecResult::Done(11),
        ],
    },
    // Instruction 0x35: dec (hl)
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| load_8(state, state.hl()),
            |state| math::dec_z_mem(state, state.hl()),
            |_| ExecResult::Done(11),
        ],
    },
    // Instruction 0x36: ld (hl), n
    Instruction::Instruction {
        extra_bytes: ExtraBytes::One,
        micros: &[
            |state| ld::ld_pp_r(state, Register16::HL, Register::Z),
            |_| ExecResult::Done(10),
        ],
    },
    // Instruction 0x37: scf
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::scf(state, 4)],
    },
    // Instruction 0x38: jr c, d
    Instruction::Instruction {
        extra_bytes: ExtraBytes::One,
        micros: &[|state| jump::jr_c_d(state, 12, 7)],
    },
    // Instruction 0x39: add hl, sp
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::add_hl_rr(state, Register16::SP, 11)],
    },
    // Instruction 0x3a: ld a, (nn)
    Instruction::Instruction {
        extra_bytes: ExtraBytes::Two,
        micros: &[
            |state| ld::ld_r_mm(state, Register::A),
            |_| ExecResult::Done(13),
        ],
    },
    // Instruction 0x3b: dec sp
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::dec_rr(state, Register16::SP, 6)],
    },
    // Instruction 0x3c: inc a
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::inc_r(state, Register::A, 4)],
    },
    // Instruction 0x3d: dec a
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::dec_r(state, Register::A, 4)],
    },
    // Instruction 0x3e: ld a, n
    Instruction::Instruction {
        extra_bytes: ExtraBytes::One,
        micros: &[|state| ld::ld_r_n(state, Register::A, 7)],
    },
    // Instruction 0x3f: ccf
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::ccf(state, 4)],
    },
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
    UNIMPLEMENTED,
];
