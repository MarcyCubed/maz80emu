//! Zilog Z80 CPU

use crate::instructions::micro::{
    bit, jump, ld, load_8, load_16, load_16_or_break, math, transfer,
};
use crate::instructions::{ExecResult, ExtraBytes, HALT, Instruction, NOP, UNIMPLEMENTED};
use crate::state::{Flags, Register, Register16};

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
    // Instruction 0x40: ld b, b
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| ld::ld_r_r(state, Register::B, Register::B, 4)],
    },
    // Instruction 0x41: ld b, c
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| ld::ld_r_r(state, Register::B, Register::C, 4)],
    },
    // Instruction 0x42: ld b, d
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| ld::ld_r_r(state, Register::B, Register::D, 4)],
    },
    // Instruction 0x43: ld b, e
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| ld::ld_r_r(state, Register::B, Register::E, 4)],
    },
    // Instruction 0x44: ld b, h
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| ld::ld_r_r(state, Register::B, Register::H, 4)],
    },
    // Instruction 0x45: ld b, l
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| ld::ld_r_r(state, Register::B, Register::L, 4)],
    },
    // Instruction 0x46: ld b, (hl)
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| ld::ld_r_pp(state, Register::B, Register16::HL),
            |_| ExecResult::Done(7),
        ],
    },
    // Instruction 0x47: ld b, a
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| ld::ld_r_r(state, Register::B, Register::A, 4)],
    },
    // Instruction 0x48: ld c, b
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| ld::ld_r_r(state, Register::C, Register::B, 4)],
    },
    // Instruction 0x49: ld c, c
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| ld::ld_r_r(state, Register::C, Register::C, 4)],
    },
    // Instruction 0x4a: ld c, d
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| ld::ld_r_r(state, Register::C, Register::D, 4)],
    },
    // Instruction 0x4b: ld c, e
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| ld::ld_r_r(state, Register::C, Register::E, 4)],
    },
    // Instruction 0x4c: ld c, h
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| ld::ld_r_r(state, Register::C, Register::H, 4)],
    },
    // Instruction 0x4d: ld c, l
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| ld::ld_r_r(state, Register::C, Register::L, 4)],
    },
    // Instruction 0x4e: ld c, (hl)
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| ld::ld_r_pp(state, Register::C, Register16::HL),
            |_| ExecResult::Done(7),
        ],
    },
    // Instruction 0x4f: ld c, a
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| ld::ld_r_r(state, Register::C, Register::A, 4)],
    },
    // Instruction 0x50: ld d, b
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| ld::ld_r_r(state, Register::D, Register::B, 4)],
    },
    // Instruction 0x51: ld d, c
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| ld::ld_r_r(state, Register::D, Register::C, 4)],
    },
    // Instruction 0x52: ld d, d
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| ld::ld_r_r(state, Register::D, Register::D, 4)],
    },
    // Instruction 0x53: ld d, e
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| ld::ld_r_r(state, Register::D, Register::E, 4)],
    },
    // Instruction 0x54: ld d, h
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| ld::ld_r_r(state, Register::D, Register::H, 4)],
    },
    // Instruction 0x55: ld d, l
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| ld::ld_r_r(state, Register::D, Register::L, 4)],
    },
    // Instruction 0x56: ld d, (hl)
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| ld::ld_r_pp(state, Register::D, Register16::HL),
            |_| ExecResult::Done(7),
        ],
    },
    // Instruction 0x57: ld d, a
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| ld::ld_r_r(state, Register::D, Register::A, 4)],
    },
    // Instruction 0x58: ld e, b
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| ld::ld_r_r(state, Register::E, Register::B, 4)],
    },
    // Instruction 0x59: ld e, c
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| ld::ld_r_r(state, Register::E, Register::C, 4)],
    },
    // Instruction 0x5a: ld e, d
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| ld::ld_r_r(state, Register::E, Register::D, 4)],
    },
    // Instruction 0x5b: ld e, e
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| ld::ld_r_r(state, Register::E, Register::E, 4)],
    },
    // Instruction 0x5c: ld e, h
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| ld::ld_r_r(state, Register::E, Register::H, 4)],
    },
    // Instruction 0x5d: ld e, l
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| ld::ld_r_r(state, Register::E, Register::L, 4)],
    },
    // Instruction 0x5e: ld e, (hl)
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| ld::ld_r_pp(state, Register::E, Register16::HL),
            |_| ExecResult::Done(7),
        ],
    },
    // Instruction 0x5f: ld e, a
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| ld::ld_r_r(state, Register::E, Register::A, 4)],
    },
    // Instruction 0x60: ld h, b
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| ld::ld_r_r(state, Register::H, Register::B, 4)],
    },
    // Instruction 0x61: ld h, c
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| ld::ld_r_r(state, Register::H, Register::C, 4)],
    },
    // Instruction 0x62: ld h, d
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| ld::ld_r_r(state, Register::H, Register::D, 4)],
    },
    // Instruction 0x63: ld h, e
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| ld::ld_r_r(state, Register::H, Register::E, 4)],
    },
    // Instruction 0x64: ld h, h
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| ld::ld_r_r(state, Register::H, Register::H, 4)],
    },
    // Instruction 0x65: ld h, l
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| ld::ld_r_r(state, Register::H, Register::L, 4)],
    },
    // Instruction 0x66: ld h, (hl)
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| ld::ld_r_pp(state, Register::H, Register16::HL),
            |_| ExecResult::Done(7),
        ],
    },
    // Instruction 0x67: ld h, a
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| ld::ld_r_r(state, Register::H, Register::A, 4)],
    },
    // Instruction 0x68: ld l, b
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| ld::ld_r_r(state, Register::L, Register::B, 4)],
    },
    // Instruction 0x69: ld l, c
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| ld::ld_r_r(state, Register::L, Register::C, 4)],
    },
    // Instruction 0x6a: ld l, d
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| ld::ld_r_r(state, Register::L, Register::D, 4)],
    },
    // Instruction 0x6b: ld l, e
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| ld::ld_r_r(state, Register::L, Register::E, 4)],
    },
    // Instruction 0x6c: ld l, h
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| ld::ld_r_r(state, Register::L, Register::H, 4)],
    },
    // Instruction 0x6d: ld l, l
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| ld::ld_r_r(state, Register::L, Register::L, 4)],
    },
    // Instruction 0x6e: ld l, (hl)
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| ld::ld_r_pp(state, Register::L, Register16::HL),
            |_| ExecResult::Done(7),
        ],
    },
    // Instruction 0x6f: ld l, a
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| ld::ld_r_r(state, Register::L, Register::A, 4)],
    },
    // Instruction 0x70: ld (hl), b
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| ld::ld_pp_r(state, Register16::HL, Register::B),
            |_| ExecResult::Done(7),
        ],
    },
    // Instruction 0x71: ld (hl), c
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| ld::ld_pp_r(state, Register16::HL, Register::C),
            |_| ExecResult::Done(7),
        ],
    },
    // Instruction 0x72: ld (hl), d
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| ld::ld_pp_r(state, Register16::HL, Register::D),
            |_| ExecResult::Done(7),
        ],
    },
    // Instruction 0x73: ld (hl), e
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| ld::ld_pp_r(state, Register16::HL, Register::E),
            |_| ExecResult::Done(7),
        ],
    },
    // Instruction 0x74: ld (hl), h
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| ld::ld_pp_r(state, Register16::HL, Register::H),
            |_| ExecResult::Done(7),
        ],
    },
    // Instruction 0x75: ld (hl), l
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| ld::ld_pp_r(state, Register16::HL, Register::L),
            |_| ExecResult::Done(7),
        ],
    },
    // Instruction 0x76: halt
    HALT,
    // Instruction 0x77: ld (hl), a
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| ld::ld_pp_r(state, Register16::HL, Register::A),
            |_| ExecResult::Done(7),
        ],
    },
    // Instruction 0x78: ld a, b
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| ld::ld_r_r(state, Register::A, Register::B, 4)],
    },
    // Instruction 0x79: ld a, c
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| ld::ld_r_r(state, Register::A, Register::C, 4)],
    },
    // Instruction 0x7a: ld a, d
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| ld::ld_r_r(state, Register::A, Register::D, 4)],
    },
    // Instruction 0x7b: ld a, e
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| ld::ld_r_r(state, Register::A, Register::E, 4)],
    },
    // Instruction 0x7c: ld a, h
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| ld::ld_r_r(state, Register::A, Register::H, 4)],
    },
    // Instruction 0x7d: ld a, l
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| ld::ld_r_r(state, Register::A, Register::L, 4)],
    },
    // Instruction 0x7e: ld a, (hl)
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| ld::ld_r_pp(state, Register::A, Register16::HL),
            |_| ExecResult::Done(7),
        ],
    },
    // Instruction 0x7f: ld a, a
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| ld::ld_r_r(state, Register::A, Register::A, 4)],
    },
    // Instruction 0x80: add a, b
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::add_a_r(state, Register::B, 4)],
    },
    // Instruction 0x81: add a, c
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::add_a_r(state, Register::C, 4)],
    },
    // Instruction 0x82: add a, d
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::add_a_r(state, Register::D, 4)],
    },
    // Instruction 0x83: add a, e
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::add_a_r(state, Register::E, 4)],
    },
    // Instruction 0x84: add a, h
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::add_a_r(state, Register::H, 4)],
    },
    // Instruction 0x85: add a, l
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::add_a_r(state, Register::L, 4)],
    },
    // Instruction 0x86: add a, (hl)
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| load_8(state, state.hl()),
            |state| math::add_a_r(state, Register::Z, 7),
        ],
    },
    // Instruction 0x87: add a, a
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::add_a_r(state, Register::A, 4)],
    },
    // Instruction 0x88: adc a, b
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::adc_a_r(state, Register::B, 4)],
    },
    // Instruction 0x89: adc a, c
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::adc_a_r(state, Register::C, 4)],
    },
    // Instruction 0x8a: adc a, d
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::adc_a_r(state, Register::D, 4)],
    },
    // Instruction 0x8b: adc a, e
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::adc_a_r(state, Register::E, 4)],
    },
    // Instruction 0x8c: adc a, h
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::adc_a_r(state, Register::H, 4)],
    },
    // Instruction 0x8d: adc a, l
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::adc_a_r(state, Register::L, 4)],
    },
    // Instruction 0x8e: adc a, (hl)
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| load_8(state, state.hl()),
            |state| math::adc_a_r(state, Register::Z, 7),
        ],
    },
    // Instruction 0x8f: adc a, a
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::adc_a_r(state, Register::A, 4)],
    },
    // Instruction 0x90: sub b
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::sub_r(state, Register::B, 4)],
    },
    // Instruction 0x91: sub c
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::sub_r(state, Register::C, 4)],
    },
    // Instruction 0x92: sub d
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::sub_r(state, Register::D, 4)],
    },
    // Instruction 0x93: sub e
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::sub_r(state, Register::E, 4)],
    },
    // Instruction 0x94: sub h
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::sub_r(state, Register::H, 4)],
    },
    // Instruction 0x95: sub l
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::sub_r(state, Register::L, 4)],
    },
    // Instruction 0x96: sub (hl)
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| load_8(state, state.hl()),
            |state| math::sub_r(state, Register::Z, 7),
        ],
    },
    // Instruction 0x97: sub a
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::sub_r(state, Register::A, 4)],
    },
    // Instruction 0x98: sbc b
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::sbc_r(state, Register::B, 4)],
    },
    // Instruction 0x99: sbc c
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::sbc_r(state, Register::C, 4)],
    },
    // Instruction 0x9a: sbc d
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::sbc_r(state, Register::D, 4)],
    },
    // Instruction 0x9b: sbc e
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::sbc_r(state, Register::E, 4)],
    },
    // Instruction 0x9c: sbc h
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::sbc_r(state, Register::H, 4)],
    },
    // Instruction 0x9d: sbc l
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::sbc_r(state, Register::L, 4)],
    },
    // Instruction 0x9e: sbc (hl)
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| load_8(state, state.hl()),
            |state| math::sbc_r(state, Register::Z, 7),
        ],
    },
    // Instruction 0x9f: sbc a
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::sbc_r(state, Register::A, 4)],
    },
    // Instruction 0xa0: and b
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::and_r(state, Register::B, 4)],
    },
    // Instruction 0xa1: and c
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::and_r(state, Register::C, 4)],
    },
    // Instruction 0xa2: and d
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::and_r(state, Register::D, 4)],
    },
    // Instruction 0xa3: and e
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::and_r(state, Register::E, 4)],
    },
    // Instruction 0xa4: and h
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::and_r(state, Register::H, 4)],
    },
    // Instruction 0xa5: and l
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::and_r(state, Register::L, 4)],
    },
    // Instruction 0xa6: and (hl)
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| load_8(state, state.hl()),
            |state| math::and_r(state, Register::Z, 7),
        ],
    },
    // Instruction 0xa7: and a
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::and_r(state, Register::A, 4)],
    },
    // Instruction 0xa8: xor b
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::xor_r(state, Register::B, 4)],
    },
    // Instruction 0xa9: xor c
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::xor_r(state, Register::C, 4)],
    },
    // Instruction 0xaa: xor d
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::xor_r(state, Register::D, 4)],
    },
    // Instruction 0xab: xor e
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::xor_r(state, Register::E, 4)],
    },
    // Instruction 0xac: xor h
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::xor_r(state, Register::H, 4)],
    },
    // Instruction 0xad: xor l
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::xor_r(state, Register::L, 4)],
    },
    // Instruction 0xae: xor (hl)
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| load_8(state, state.hl()),
            |state| math::xor_r(state, Register::Z, 7),
        ],
    },
    // Instruction 0xaf: xor a
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::xor_r(state, Register::A, 4)],
    },
    // Instruction 0xba0: or b
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::or_r(state, Register::B, 4)],
    },
    // Instruction 0xb1: or c
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::or_r(state, Register::C, 4)],
    },
    // Instruction 0xb2: or d
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::or_r(state, Register::D, 4)],
    },
    // Instruction 0xb3: or e
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::or_r(state, Register::E, 4)],
    },
    // Instruction 0xb4: or h
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::or_r(state, Register::H, 4)],
    },
    // Instruction 0xb5: or l
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::or_r(state, Register::L, 4)],
    },
    // Instruction 0xb6: or (hl)
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| load_8(state, state.hl()),
            |state| math::or_r(state, Register::Z, 7),
        ],
    },
    // Instruction 0xb7: or a
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::or_r(state, Register::A, 4)],
    },
    // Instruction 0xb8: cp b
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::cp_r(state, Register::B, 4)],
    },
    // Instruction 0xb9: cp c
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::cp_r(state, Register::C, 4)],
    },
    // Instruction 0xba: cp d
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::cp_r(state, Register::D, 4)],
    },
    // Instruction 0xbb: cp e
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::cp_r(state, Register::E, 4)],
    },
    // Instruction 0xbc: cp h
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::cp_r(state, Register::H, 4)],
    },
    // Instruction 0xbd: cp l
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::cp_r(state, Register::L, 4)],
    },
    // Instruction 0xbe: cp (hl)
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| load_8(state, state.hl()),
            |state| math::cp_r(state, Register::Z, 7),
        ],
    },
    // Instruction 0xbf: cp a
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| math::cp_r(state, Register::A, 4)],
    },
    // Instruction 0xc0: ret nz
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| load_16_or_break(state, state.sp(), !state.get_flags().is_set(Flags::Z), 5),
            |state| jump::ret(state, 11),
        ],
    },
    // Instruction 0xc1: pop bc
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| jump::pop(state, Register16::BC),
            |_| ExecResult::Done(10),
        ],
    },
    // Instruction 0xc2: jp nz, nn
    Instruction::Instruction {
        extra_bytes: ExtraBytes::Two,
        micros: &[|state| jump::jp_cc_nn(state, !state.get_flags().is_set(Flags::Z), 10)],
    },
    // Instruction 0xc3: jp nn
    Instruction::Instruction {
        extra_bytes: ExtraBytes::Two,
        micros: &[|state| jump::jp_cc_nn(state, true, 10)],
    },
    // Instruction 0xc4: call nz, nn
    Instruction::Instruction {
        extra_bytes: ExtraBytes::Two,
        micros: &[
            |state| jump::push_pc_or_break(state, !state.get_flags().is_set(Flags::Z), 10),
            |state| jump::jr_mm(state, 17),
        ],
    },
    // Instruction 0xc5: push bc
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| jump::push(state, Register16::BC),
            |_| ExecResult::Done(11),
        ],
    },
    // Instruction 0xc6: add a, n
    Instruction::Instruction {
        extra_bytes: ExtraBytes::One,
        micros: &[|state| math::add_a_r(state, Register::Z, 7)],
    },
    // Instruction 0xc7: rst 00h
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| jump::push(state, Register16::PC),
            |state| jump::rst(state, 0, 11),
        ],
    },
    // Instruction 0xc8: ret z
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| load_16_or_break(state, state.sp(), state.get_flags().is_set(Flags::Z), 5),
            |state| jump::ret(state, 11),
        ],
    },
    // Instruction 0xc9: ret
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| load_16(state, state.sp()),
            |state| jump::ret(state, 11),
        ],
    },
    // Instruction 0xca: jp z, nn
    Instruction::Instruction {
        extra_bytes: ExtraBytes::Two,
        micros: &[|state| jump::jp_cc_nn(state, state.get_flags().is_set(Flags::Z), 10)],
    },
    // Bit instructions
    UNIMPLEMENTED,
    // Instruction 0xcc: call z, nn
    Instruction::Instruction {
        extra_bytes: ExtraBytes::Two,
        micros: &[
            |state| jump::push_pc_or_break(state, state.get_flags().is_set(Flags::Z), 10),
            |state| jump::jr_mm(state, 17),
        ],
    },
    // Instruction 0xcd: call nn
    Instruction::Instruction {
        extra_bytes: ExtraBytes::Two,
        micros: &[
            |state| jump::push(state, Register16::PC),
            |state| jump::jr_mm(state, 17),
        ],
    },
    // Instruction 0xce: adc a, n
    Instruction::Instruction {
        extra_bytes: ExtraBytes::One,
        micros: &[|state| math::adc_a_r(state, Register::Z, 7)],
    },
    // Instruction 0xcf: rst 08h
    Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| jump::push(state, Register16::PC),
            |state| jump::rst(state, 0x8, 11),
        ],
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
];
