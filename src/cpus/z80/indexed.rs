//! IX and IY instructions

use crate::cpus::z80::{
    adc_r, add_r, add_rr_rr, and_r, cp_r, dec_r, dec_rr, inc_r, inc_rr, ld_mm_rr, ld_r_n, ld_r_r,
    ld_rr_mm, ld_rr_nn, math_r, or_r, pop_rr, push_rr, sbc_r, sub_r, xor_r,
};
use crate::instructions::micro::{jump, ld, math, store_16};
use crate::instructions::{
    ExecResult, ExtraBytes, Instruction, InstructionSet, NOP, UNIMPLEMENTED,
};
use crate::state::{Register, Register16, State};

/// A trait to abstract away IX and IY registers.
trait Index {
    /// The index register
    const REGISTER: Register16;

    /// The 8-bit register corresponding to the low byte of the register
    const LOW: Register;

    /// The 8-bit register corresponding to the high byte of the register
    const HIGH: Register;

    /// Get the value of the register + displacement in register `W`
    fn get_offset_w(state: &State) -> u16 {
        let d = state.w() as i8 as i16 as u16;

        let address = state.get_register_16(Self::REGISTER);
        address.wrapping_add(d)
    }

    /// Get the value of the register + displacement in register `Z`
    fn get_offset_z(state: &State) -> u16 {
        let d = state.z() as i8 as i16 as u16;

        let address = state.get_register_16(Self::REGISTER);
        address.wrapping_add(d)
    }
}

/// Selector for the IX register
struct IX;

impl Index for IX {
    const REGISTER: Register16 = Register16::IX;
    const LOW: Register = Register::IXL;
    const HIGH: Register = Register::IXH;
}

/// Selector for the IY register
struct IY;

impl Index for IY {
    const REGISTER: Register16 = Register16::IY;
    const LOW: Register = Register::IYL;
    const HIGH: Register = Register::IYH;
}

macro_rules! inc_dec_izd {
    ($name:literal, $op:expr) => {
        Instruction::Instruction {
            extra_bytes: ExtraBytes::One,
            printer: |state| println!("{} ({}+{})", $name, I::REGISTER, state.z() as i8),
            micros: &[
                |state| {
                    // Save d
                    *state.w_mut() = state.z();
                    ExecResult::load(I::get_offset_w(state))
                },
                |state| $op(state, I::get_offset_w(state)),
                |_| ExecResult::Done(6),
            ],
        }
    };
}

macro_rules! ld_r_izd {
    ($reg:expr) => {
        Instruction::Instruction {
            extra_bytes: ExtraBytes::One,
            printer: |state| println!("ld {}, ({}+{})", $reg, I::REGISTER, state.z() as i8),
            micros: &[
                |state| ExecResult::load(I::get_offset_z(state)),
                |state| ld::ld_r_r(state, $reg, Register::Z, 5),
            ],
        }
    };
}

macro_rules! ld_izd_r {
    ($reg:expr) => {
        Instruction::Instruction {
            extra_bytes: ExtraBytes::One,
            printer: |state| println!("ld ({}+{}), {}", I::REGISTER, state.z() as i8, $reg),
            micros: &[
                |state| ExecResult::Store {
                    address: I::get_offset_z(state),
                    data: state.get_register_8($reg),
                },
                |_| ExecResult::Done(4),
            ],
        }
    };
}

macro_rules! math_izd {
    ($name:literal, $op:expr) => {
        Instruction::Instruction {
            extra_bytes: ExtraBytes::One,
            micros: &[
                |state| ExecResult::load(I::get_offset_z(state)),
                |state| $op(state, Register::Z, 5),
            ],
            printer: |state| println!("{} ({}+{})", $name, I::REGISTER, state.z() as i8),
        }
    };
}

const fn make_indexed_instructions<I: Index>() -> InstructionSet {
    let mut instructions = [NOP; _];

    // Mark instructions as prefix-ignoring ones
    const fn un_prefix<const N: usize>(instructions: &mut InstructionSet, unprefixed: &[usize; N]) {
        let mut i = 0usize;
        while i < N {
            instructions[unprefixed[i]] = Instruction::NoPrefix;
            i += 1;
        }
    }

    // Undocumented instructions that ignore the prefix
    un_prefix(
        &mut instructions,
        &[
            0x4, 0x5, 0x6, 0xc, 0xd, 0xe, 0x14, 0x15, 0x16, 0x1c, 0x1d, 0x1e, 0x3c, 0x3d, 0x3e,
            0x40, 0x41, 0x42, 0x43, 0x47, 0x48, 0x49, 0x4a, 0x4b, 0x4f, 0x50, 0x51, 0x52, 0x53,
            0x57, 0x58, 0x59, 0x5a, 0x5b, 0x5f, 0x78, 0x79, 0x7a, 0x7b, 0x7f, 0x80, 0x81, 0x82,
            0x83, 0x87, 0x88, 0x89, 0x8a, 0x8b, 0x8f, 0x90, 0x91, 0x92, 0x93, 0x97, 0x98, 0x99,
            0x9a, 0x9b, 0x9f, 0xa0, 0xa1, 0xa2, 0xa3, 0xa7, 0xa8, 0xa9, 0xaa, 0xab, 0xaf, 0xb0,
            0xb1, 0xb2, 0xb3, 0xb7, 0xb8, 0xb9, 0xba, 0xbb, 0xbf, 0xdd, 0xfd,
        ],
    );
    // add i?, bc
    instructions[0x09] = add_rr_rr!(I::REGISTER, Register16::BC);
    // add i?, de
    instructions[0x09] = add_rr_rr!(I::REGISTER, Register16::DE);
    // ld i?, nn
    instructions[0x21] = ld_rr_nn!(I::REGISTER);
    // ld (mm), i?
    instructions[0x22] = ld_mm_rr!(I::REGISTER);
    // inc i?
    instructions[0x23] = inc_rr!(I::REGISTER);
    // inc i?h
    instructions[0x24] = inc_r!(I::HIGH);
    // dec i?h
    instructions[0x25] = dec_r!(I::HIGH);
    // ld i?h, n
    instructions[0x25] = ld_r_n!(I::HIGH);
    // add i?, i?
    instructions[0x29] = add_rr_rr!(I::REGISTER, I::REGISTER);
    // ld i?, (mm)
    instructions[0x2A] = ld_rr_mm!(I::REGISTER);
    // dec i?
    instructions[0x2B] = dec_rr!(I::REGISTER);
    // inc i?l
    instructions[0x2C] = inc_r!(I::LOW);
    // dec i?l
    instructions[0x2D] = dec_r!(I::LOW);
    // ld i?l, n
    instructions[0x2E] = ld_r_n!(I::LOW);
    // inc (i?+d)
    instructions[0x34] = inc_dec_izd!("inc", math::inc_z_mem);
    // dec (i?+d)
    instructions[0x35] = inc_dec_izd!("dec", math::dec_z_mem);
    // ld (i?+d), n
    instructions[0x36] = Instruction::Instruction {
        extra_bytes: ExtraBytes::Two,
        micros: &[
            |state| ExecResult::Store {
                address: I::get_offset_z(state),
                data: state.w(),
            },
            |_| ExecResult::Done(2),
        ],
        printer: |state| println!("ld ({}+{}, {:x})", I::REGISTER, state.z() as i8, state.w()),
    };
    // add i?, sp
    instructions[0x39] = add_rr_rr!(I::REGISTER, Register16::SP);
    // ld b, i?h
    instructions[0x44] = ld_r_r!(Register::B, I::HIGH);
    // ld b, i?l
    instructions[0x45] = ld_r_r!(Register::B, I::LOW);
    // ld b,(i?+d)
    instructions[0x46] = ld_r_izd!(Register::B);
    // ld c, i?h
    instructions[0x4c] = ld_r_r!(Register::C, I::HIGH);
    // ld c, i?l
    instructions[0x4d] = ld_r_r!(Register::C, I::LOW);
    // ld c,(i?+d)
    instructions[0x4e] = ld_r_izd!(Register::C);
    // ld d, i?h
    instructions[0x54] = ld_r_r!(Register::D, I::HIGH);
    // ld d, i?l
    instructions[0x55] = ld_r_r!(Register::D, I::LOW);
    // ld d,(i?+d)
    instructions[0x56] = ld_r_izd!(Register::D);
    // ld e, i?h
    instructions[0x5c] = ld_r_r!(Register::E, I::HIGH);
    // ld e, i?l
    instructions[0x5d] = ld_r_r!(Register::E, I::LOW);
    // ld e,(i?+d)
    instructions[0x5e] = ld_r_izd!(Register::E);
    // ld i?h, b
    instructions[0x60] = ld_r_r!(I::HIGH, Register::B);
    // ld i?h, c
    instructions[0x61] = ld_r_r!(I::HIGH, Register::C);
    // ld i?h, d
    instructions[0x62] = ld_r_r!(I::HIGH, Register::D);
    // ld i?h, e
    instructions[0x63] = ld_r_r!(I::HIGH, Register::E);
    // ld i?h, i?h
    instructions[0x64] = ld_r_r!(I::HIGH, I::HIGH);
    // ld i?h, i?l
    instructions[0x65] = ld_r_r!(I::HIGH, I::LOW);
    // ld h,(i?+d)
    instructions[0x66] = ld_r_izd!(Register::H);
    // ld i?h, a
    instructions[0x67] = ld_r_r!(I::LOW, Register::A);
    // ld i?l, b
    instructions[0x68] = ld_r_r!(I::LOW, Register::B);
    // ld i?l, c
    instructions[0x69] = ld_r_r!(I::LOW, Register::C);
    // ld i?l, d
    instructions[0x6a] = ld_r_r!(I::LOW, Register::D);
    // ld i?l, e
    instructions[0x6b] = ld_r_r!(I::LOW, Register::E);
    // ld i?l, i?h
    instructions[0x6c] = ld_r_r!(I::LOW, I::HIGH);
    // ld i?l, i?l
    instructions[0x6d] = ld_r_r!(I::LOW, I::LOW);
    // ld l,(i?+d)
    instructions[0x6e] = ld_r_izd!(Register::L);
    // ld i?l, a
    instructions[0x6f] = ld_r_r!(I::LOW, Register::A);
    // ld (i?+d), b
    instructions[0x70] = ld_izd_r!(Register::B);
    // ld (i?+d), c
    instructions[0x71] = ld_izd_r!(Register::C);
    // ld (i?+d), d
    instructions[0x72] = ld_izd_r!(Register::D);
    // ld (i?+d), e
    instructions[0x73] = ld_izd_r!(Register::E);
    // ld (i?+d), h
    instructions[0x74] = ld_izd_r!(Register::H);
    // ld (i?+d), l
    instructions[0x75] = ld_izd_r!(Register::L);
    // ld (i?+d), a
    instructions[0x77] = ld_izd_r!(Register::A);
    // ld a, i?h
    instructions[0x7c] = ld_r_r!(Register::A, I::HIGH);
    // ld a, i?l
    instructions[0x7d] = ld_r_r!(Register::A, I::LOW);
    // ld a,(i?+d)
    instructions[0x7e] = ld_r_izd!(Register::A);
    // add a, i?h
    instructions[0x84] = add_r!(I::HIGH);
    // add a, i?l
    instructions[0x85] = add_r!(I::LOW);
    // add a, (i?+d)
    instructions[0x86] = math_izd!("add a,", math::add_a_r);
    // adc a, i?h
    instructions[0x8c] = adc_r!(I::HIGH);
    // adc a, i?l
    instructions[0x8d] = adc_r!(I::LOW);
    // adc a, (i?+d)
    instructions[0x8e] = math_izd!("adc a,", math::adc_a_r);
    // sub i?h
    instructions[0x94] = sub_r!(I::HIGH);
    // sub i?l
    instructions[0x95] = sub_r!(I::LOW);
    // sub (i?+d)
    instructions[0x96] = math_izd!("sub", math::sub_r);
    // sbc a, i?h
    instructions[0x9c] = sbc_r!(I::HIGH);
    // sbc a, i?l
    instructions[0x9d] = sbc_r!(I::LOW);
    // sbc a, (i?+d)
    instructions[0x9e] = math_izd!("sbc a,", math::sbc_r);
    // and i?h
    instructions[0xa4] = and_r!(I::HIGH);
    // and i?l
    instructions[0xa5] = and_r!(I::LOW);
    // and (i?+d)
    instructions[0xa6] = math_izd!("and", math::and_r);
    // xor i?h
    instructions[0xac] = xor_r!(I::HIGH);
    // xor i?l
    instructions[0xad] = xor_r!(I::LOW);
    // xor (i?+d)
    instructions[0xae] = math_izd!("xor", math::xor_r);
    // or i?h
    instructions[0xb4] = or_r!(I::HIGH);
    // or i?l
    instructions[0xb5] = or_r!(I::LOW);
    // or (i?+d)
    instructions[0xb6] = math_izd!("or", math::or_r);
    // cp i?h
    instructions[0xbc] = cp_r!(I::HIGH);
    // cp i?l
    instructions[0xbd] = cp_r!(I::LOW);
    // cp (i?+d)
    instructions[0xbe] = math_izd!("cp", math::cp_r);
    // Bit operations
    instructions[0xcb] = UNIMPLEMENTED;
    // pop i?
    instructions[0xe1] = pop_rr!(I::REGISTER);
    // ex (sp), i?
    instructions[0xe3] = Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[
            |state| ExecResult::load16(state.sp()),
            |state| store_16(state.sp(), state.get_register_16(I::REGISTER)),
            |state| ld::ld_rr_rr(state, I::REGISTER, Register16::WZ, 3),
        ],
        printer: |_| println!("ex (sp), {}", I::REGISTER),
    };
    // push i?
    instructions[0xe5] = push_rr!(I::REGISTER);
    // jp (i?)
    instructions[0xe9] = Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| jump::jp(state, state.get_register_16(I::REGISTER), 0)],
        printer: |_| println!("jp {}", I::REGISTER),
    };
    // ld sp, i?
    instructions[0xf9] = Instruction::Instruction {
        extra_bytes: ExtraBytes::None,
        micros: &[|state| {
            *state.sp_mut() = state.get_register_16(I::REGISTER).to_le_bytes();
            ExecResult::Done(2)
        }],
        printer: |_| println!("ld sp, {}", I::REGISTER),
    };

    instructions
}

/// Table of IX instructions
pub(crate) static IX_TABLE: InstructionSet = make_indexed_instructions::<IX>();

/// Table of IY instructions
pub(crate) static IY_TABLE: InstructionSet = make_indexed_instructions::<IY>();
