//! State of a Z80 CPU.
//!
//! The state is formed by the normal registers, the alternate register sets and the interruption
//! flip-flops. This is basically the same as described in the Z80 User Manual. Unlike most
//! emulators we don't manage memory, so the user is free to use any memory layout as they please.
//!
//! The registers are implemented as an array of two element arrays of bytes (`[[u8;2];N]`). Since
//! the Z80 can transfer both 8 and 16 bits to memory in a single instruction, this gives us
//! flexibility to pass around references to both 8-bit and 16-bit data types.

use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not, Sub, SubAssign};
use strum::{Display, EnumCount};

/// The list of registers in a Z80 CPU.
///
/// The order may seem a bit strange, but that's because the Z80 is a little endian CPU. This means
/// when we store in memory the value of 16 bit registers, the least significant byte goes first.
/// If we keep our registers pairs also in little endian order we may avoid some format translation.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Default, Display)]
#[strum(serialize_all = "lowercase")]
pub enum Register {
    /// The flags
    Flags,
    /// The accumulator
    #[default]
    A,
    /// General purpose C register
    C,
    /// General purpose B register
    B,
    /// General purpose E register
    E,
    /// General purpose D register
    D,
    /// L register
    L,
    /// H register
    H,
    /// Interrupt page address register
    I,
    /// Memory refresh register
    R,
    /// Low byte of the IX register
    IXL,
    /// High byte of the IX register
    IXH,
    /// Low byte of the IY register
    IYL,
    /// High byte of the IY register
    IYH,
    /// Low byte of the stack pointer
    SPL,
    /// High byte of the stack pointer
    SPH,
    /// Low byte of the program counter
    PCL,
    /// High byte of the program counter.
    PCH,
    /// Low byte of the instruction buffer.
    Z,
    /// High byte of the instruction buffer.
    W,
    /// Alternate F register
    #[strum(serialize = "flags'")]
    FAlt,
    /// Alternate A register
    #[strum(serialize = "a'")]
    AAlt,
    /// Alternate C register
    #[strum(serialize = "c'")]
    CAlt,
    /// Alternate B register
    #[strum(serialize = "b'")]
    BAlt,
    /// Alternate E register
    #[strum(serialize = "e'")]
    EAlt,
    /// Alternate D register
    #[strum(serialize = "d'")]
    DAlt,
    /// Alternate L register
    #[strum(serialize = "l'")]
    LAlt,
    /// Alternate H register
    #[strum(serialize = "h'")]
    HAlt,
}

/// The list of 16 bit registers.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Default, Display, EnumCount)]
#[strum(serialize_all = "lowercase")]
pub enum Register16 {
    /// Accumulator and flags
    #[default]
    AF,
    /// B and C
    BC,
    /// D and E
    DE,
    /// H and L
    HL,
    /// R and I.
    ///
    /// This isn't a real Z80 register, but we'll use it internally in the emulator
    RI,
    /// Index register IX
    IX,
    /// Index register IY
    IY,
    /// Stack pointer
    SP,
    /// Program counter
    PC,
    /// Instruction buffer
    WZ,
    /// Alternate AF register
    #[strum(serialize = "af'")]
    AfAlt,
    /// Alternate BC register
    #[strum(serialize = "bc'")]
    BcAlt,
    /// Alternate DE register
    #[strum(serialize = "de'")]
    DeAlt,
    /// Alternate HL register
    #[strum(serialize = "hl'")]
    HlAlt,
}

/// The state of a Z80 CPU.
#[derive(Debug, Clone, Copy)]
pub struct State {
    /// The set of registers
    pub registers: [[u8; 2]; Register16::COUNT],
    /// Interrupt flip-flop
    pub iff1: bool,
    /// Temporary storage for `iff1``
    pub iff2: bool,
    /// Which microinstruction is being executed
    pub mpc: usize,
    /// Interruption mode of the processor
    pub interrupt_mode: InterruptMode,
    /// The `MEMPTR` internal register
    pub memptr: u16,
}

impl Default for State {
    fn default() -> Self {
        let mut state = State {
            registers: [[0; 2]; Register16::COUNT],
            iff1: false,
            iff2: false,
            mpc: 0,
            interrupt_mode: InterruptMode::Instruction,
            memptr: 0,
        };

        state.set_register_16(Register16::AF, 0xffff);
        state.set_register_16(Register16::SP, 0xffff);
        state
    }
}

impl State {
    /// Create a new Z80 processor
    pub fn new() -> Self {
        Default::default()
    }

    /// Get the value of an 8-bit register
    pub fn get_register_8(&self, register: Register) -> u8 {
        let index = register as usize;
        self.registers[index / 2][index % 2]
    }

    /// Get the value of a 16-bit register.
    pub fn get_register_16(&self, register: Register16) -> u16 {
        u16::from_le_bytes(self.registers[register as usize])
    }

    /// Get the value of a 16-bit register as two bytes in little endian order.
    pub fn get_register_16_bytes(&self, register: Register16) -> [u8; 2] {
        self.registers[register as usize]
    }

    /// Get a mutable reference to an 8-bit register
    pub fn get_register_mut_8(&mut self, register: Register) -> &mut u8 {
        let index = register as usize;
        &mut self.registers[index / 2][index % 2]
    }

    /// Get a mutable reference to a 16-bit register
    pub fn get_register_mut_16(&mut self, register: Register16) -> &mut [u8; 2] {
        &mut self.registers[register as usize]
    }

    /// Set the value of an 8-bit register
    pub fn set_register_8(&mut self, register: Register, value: u8) {
        let index = register as usize;
        self.registers[index / 2][index % 2] = value;
    }

    /// Set the value of a 16-bit register
    pub fn set_register_16(&mut self, register: Register16, value: u16) {
        self.registers[register as usize] = value.to_le_bytes();
    }

    /// Set the value of a 16-bit register to a 2-byte array
    pub fn set_register_16_bytes(&mut self, register: Register16, value: [u8; 2]) {
        self.registers[register as usize] = value;
    }

    /// Advances the PC by an amount of bytes.
    pub fn advance_pc(&mut self, amount: u16) {
        let pc_addr = self.get_register_16(Register16::PC);
        self.set_register_16(Register16::PC, pc_addr.wrapping_add(amount))
    }

    /// Get the value of the flags.
    pub fn get_flags(&self) -> Flags {
        Flags(self.get_register_8(Register::Flags))
    }

    /// Change the flags register to the new value.
    ///
    /// Didn't use `set` to not confuse with the idea of setting individual flags
    pub fn update_flags(&mut self, flags: Flags) {
        self.set_register_8(Register::Flags, flags.0);
    }

    /// Set or reset flags to the value of a boolean
    pub fn flags_from_bool(&mut self, flags: u8, value: bool) {
        let register = self.get_register_mut_8(Register::Flags);
        if value {
            *register |= flags;
        } else {
            *register &= !flags;
        }
    }

    /// Get the value of the register A
    pub fn a(&self) -> u8 {
        self.get_register_8(Register::A)
    }

    /// Get a mutable reference to the register A
    pub fn a_mut(&mut self) -> &mut u8 {
        self.get_register_mut_8(Register::A)
    }

    /// Get the value of the register B
    pub fn b(&self) -> u8 {
        self.get_register_8(Register::B)
    }

    /// Get a mutable reference to the register B
    pub fn b_mut(&mut self) -> &mut u8 {
        self.get_register_mut_8(Register::B)
    }

    /// Get the value of the register C
    pub fn c(&self) -> u8 {
        self.get_register_8(Register::C)
    }

    /// Get a mutable reference to the register C
    pub fn c_mut(&mut self) -> &mut u8 {
        self.get_register_mut_8(Register::C)
    }

    /// Get the value of the register D
    pub fn d(&self) -> u8 {
        self.get_register_8(Register::D)
    }

    /// Get a mutable reference to the register D
    pub fn d_mut(&mut self) -> &mut u8 {
        self.get_register_mut_8(Register::D)
    }

    /// Get the value of the register E
    pub fn e(&self) -> u8 {
        self.get_register_8(Register::E)
    }

    /// Get a mutable reference to the register E
    pub fn e_mut(&mut self) -> &mut u8 {
        self.get_register_mut_8(Register::E)
    }

    /// Get the value of the register H
    pub fn h(&self) -> u8 {
        self.get_register_8(Register::H)
    }

    /// Get a mutable reference to the register H
    pub fn h_mut(&mut self) -> &mut u8 {
        self.get_register_mut_8(Register::H)
    }

    /// Get the value of the register L
    pub fn l(&self) -> u8 {
        self.get_register_8(Register::L)
    }

    /// Get a mutable reference to the register L
    pub fn l_mut(&mut self) -> &mut u8 {
        self.get_register_mut_8(Register::L)
    }

    /// Get the value of the register I
    pub fn i(&self) -> u8 {
        self.get_register_8(Register::I)
    }

    /// Get a mutable reference to the register I
    pub fn i_mut(&mut self) -> &mut u8 {
        self.get_register_mut_8(Register::I)
    }

    /// Get the value of the register R
    pub fn r(&self) -> u8 {
        self.get_register_8(Register::R)
    }

    /// Get a mutable reference to the register R
    pub fn r_mut(&mut self) -> &mut u8 {
        self.get_register_mut_8(Register::R)
    }

    /// Get the value of the register Z
    pub fn z(&self) -> u8 {
        self.get_register_8(Register::Z)
    }

    /// Get a mutable reference to the register Z
    pub fn z_mut(&mut self) -> &mut u8 {
        self.get_register_mut_8(Register::Z)
    }

    /// Get the value of the register W
    pub fn w(&self) -> u8 {
        self.get_register_8(Register::W)
    }

    /// Get a mutable reference to the register W
    pub fn w_mut(&mut self) -> &mut u8 {
        self.get_register_mut_8(Register::W)
    }

    /// Get the value of the register AF
    pub fn af(&self) -> u16 {
        self.get_register_16(Register16::AF)
    }

    /// Get the value of the register AF as a little endian array of bytes
    pub fn af_bytes(&self) -> [u8; 2] {
        self.get_register_16_bytes(Register16::AF)
    }

    /// Get a mutable reference to the register AF
    pub fn af_mut(&mut self) -> &mut [u8; 2] {
        self.get_register_mut_16(Register16::AF)
    }

    /// Get the value of the register BC
    pub fn bc(&self) -> u16 {
        self.get_register_16(Register16::BC)
    }

    /// Get the value of the register BC as a little endian array of bytes
    pub fn bc_bytes(&self) -> [u8; 2] {
        self.get_register_16_bytes(Register16::BC)
    }

    /// Get a mutable reference to the register
    pub fn bc_mut(&mut self) -> &mut [u8; 2] {
        self.get_register_mut_16(Register16::BC)
    }

    /// Get the value of the register DE
    pub fn de(&self) -> u16 {
        self.get_register_16(Register16::DE)
    }

    /// Get the value of the register DE as a little endian array of bytes
    pub fn de_bytes(&self) -> [u8; 2] {
        self.get_register_16_bytes(Register16::DE)
    }

    /// Get a mutable reference to the register DE
    pub fn de_mut(&mut self) -> &mut [u8; 2] {
        self.get_register_mut_16(Register16::DE)
    }

    /// Get the value of the register HL
    pub fn hl(&self) -> u16 {
        self.get_register_16(Register16::HL)
    }

    /// Get the value of the register HL as a little endian array of bytes
    pub fn hl_bytes(&self) -> [u8; 2] {
        self.get_register_16_bytes(Register16::HL)
    }

    /// Get a mutable reference to the register
    pub fn hl_mut(&mut self) -> &mut [u8; 2] {
        self.get_register_mut_16(Register16::HL)
    }

    /// Get the value of the register SP
    pub fn sp(&self) -> u16 {
        self.get_register_16(Register16::SP)
    }

    /// Get the value of the register SP as a little endian array of bytes
    pub fn sp_bytes(&self) -> [u8; 2] {
        self.get_register_16_bytes(Register16::SP)
    }

    /// Get a mutable reference to the register SP
    pub fn sp_mut(&mut self) -> &mut [u8; 2] {
        self.get_register_mut_16(Register16::SP)
    }

    /// Get the value of the register PC
    pub fn pc(&self) -> u16 {
        self.get_register_16(Register16::PC)
    }

    /// Get the value of the register PC as a little endian array of bytes
    pub fn pc_bytes(&self) -> [u8; 2] {
        self.get_register_16_bytes(Register16::PC)
    }

    /// Get a mutable reference to the register PC
    pub fn pc_mut(&mut self) -> &mut [u8; 2] {
        self.get_register_mut_16(Register16::PC)
    }

    /// Get the value of the register IX
    pub fn ix(&self) -> u16 {
        self.get_register_16(Register16::IX)
    }

    /// Get the value of the register IX as a little endian array of bytes
    pub fn ix_bytes(&self) -> [u8; 2] {
        self.get_register_16_bytes(Register16::IX)
    }

    /// Get a mutable reference to the register IX
    pub fn ix_mut(&mut self) -> &mut [u8; 2] {
        self.get_register_mut_16(Register16::IX)
    }

    /// Get the value of the register IY
    pub fn iy(&self) -> u16 {
        self.get_register_16(Register16::IY)
    }

    /// Get the value of the register IY as a little endian array of bytes
    pub fn iy_bytes(&self) -> [u8; 2] {
        self.get_register_16_bytes(Register16::IY)
    }

    /// Get a mutable reference to the register IY
    pub fn iy_mut(&mut self) -> &mut [u8; 2] {
        self.get_register_mut_16(Register16::IY)
    }

    /// Get the value of the register WZ
    pub fn wz(&self) -> u16 {
        self.get_register_16(Register16::WZ)
    }

    /// Get the value of the register WZ as a little endian array of bytes
    pub fn wz_bytes(&self) -> [u8; 2] {
        self.get_register_16_bytes(Register16::WZ)
    }

    /// Get a mutable reference to the register WZ
    pub fn wz_mut(&mut self) -> &mut [u8; 2] {
        self.get_register_mut_16(Register16::WZ)
    }

    /// Skip the execution of the current instruction and proceed to the next
    pub fn skip_instruction(&mut self) {
        self.mpc = usize::MAX;
    }

    /// Receive a byte from memory or I/O ports
    ///
    /// The value is stored in the `Z` register
    pub fn load_data_8(&mut self, data: u8) {
        self.set_register_8(Register::Z, data);
    }

    /// Receive a word from memory or I/O ports
    ///
    /// The value is stored in the `WZ` register
    pub fn load_data_16(&mut self, data: [u8; 2]) {
        self.set_register_16_bytes(Register16::WZ, data);
    }

    /// Advance the R register to the next value
    pub fn advance_r(&mut self) {
        // Advance the R register
        let r = self.r();
        *self.r_mut() = (r.wrapping_add(1) & !(1 << 7)) | r & 1 << 7;
    }

    /// Move R backwards
    pub fn revert_r(&mut self) {
        // Advance the R register
        let r = self.r();
        *self.r_mut() = (r.wrapping_sub(1) & !(1 << 7)) | r & 1 << 7;
    }

    /// Display the state for debugging.
    ///
    /// Also shows the opcode if it's known.
    pub fn print_debug(&self, opcode: Option<u8>) {
        use Register::{I, R};
        use Register16::*;

        fn print_registers(state: &State, registers: &[Register]) {
            for register in registers {
                print!("{}={:02x}h,", register, state.get_register_8(*register));
            }
        }

        fn print_registers_16(state: &State, registers: &[Register16]) {
            for register in registers {
                print!("{}={:04x}h,", register, state.get_register_16(*register));
            }
        }

        fn print_flag(state: &State, name: &str, flag: Flags) {
            print!("{}={},", name, state.get_flags().is_set(flag) as u8);
        }

        print_registers_16(self, &[PC, SP]);
        if let Some(opcode) = opcode {
            print!("op={:02x}h,", opcode);
        }
        print_registers_16(self, &[AF, BC, DE, HL, IX, IY]);
        print_registers(self, &[I, R]);
        print_registers_16(self, &[AfAlt, BcAlt, DeAlt, HlAlt]);

        for (name, flag) in [
            ("c", Flags::C),
            ("po", Flags::P),
            ("hc", Flags::H),
            ("n", Flags::N),
            ("z", Flags::Z),
            ("s", Flags::S),
        ] {
            print_flag(self, name, flag);
        }
        println!("memptr={:04x}h", self.memptr);
    }
}

/// A structure representing Z80 flags
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Flags(u8);

impl Flags {
    /// Carry flag
    pub const C: Flags = Flags(1 << 0);

    /// Add/Subtract
    pub const N: Flags = Flags(1 << 1);

    /// Parity
    ///
    /// Parity and overflow are two names for the same flag
    pub const P: Flags = Flags(1 << 2);

    /// Overflow
    ///
    /// Parity and overflow are two names for the same flag
    pub const V: Flags = Flags(1 << 2);

    /// Half carry flag
    pub const H: Flags = Flags(1 << 4);

    /// Zero flag
    pub const Z: Flags = Flags(1 << 6);

    /// Sign flag
    pub const S: Flags = Flags(1 << 7);

    /// Undocumented flag X
    pub const X: Flags = Flags(1 << 3);

    /// Undocumented flag Y
    pub const Y: Flags = Flags(1 << 5);

    /// New empty flags with nothing set
    pub fn new() -> Self {
        Flags(0)
    }

    /// Get the sign and zero flags from a number
    pub fn sz(number: u8) -> Self {
        if number == 0 {
            Self::Z
        } else if number > 0b01111111 {
            Self::S
        } else {
            Self(0)
        }
    }

    /// Get the S, Z, X and Y flags from a value
    pub fn from_value(number: u8) -> Self {
        Flags::xy(number)
            | if number == 0 {
                Self::Z
            } else if number > 0b01111111 {
                Self::S
            } else {
                Self::new()
            }
    }

    /// Get the numeric value from a flag
    pub fn as_u8(self) -> u8 {
        self.0
    }

    /// Turn a raw value into flags
    pub fn from_raw(value: u8) -> Self {
        Self(value)
    }

    /// Check if any of the bits of the mask is set on the flags
    pub fn is_set(self, mask: Flags) -> bool {
        self.0 & mask.0 != 0
    }

    /// Get the flags if the condition is true, otherwise they're reset
    pub const fn set_if(self, condition: bool) -> Flags {
        if condition { self } else { Flags(0) }
    }

    /// Get the parity flag for a value
    pub const fn parity(value: u8) -> Self {
        Flags::P.set_if(value.count_ones() & 1 == 0)
    }

    /// Get the X and Y flags for a value
    pub const fn xy(value: u8) -> Self {
        Flags(value & (Flags::X.0 | Flags::Y.0))
    }

    /// Flips the value of the tags selected in the mask
    pub fn flip(self, mask: Self) -> Self {
        Self(self.0 ^ mask.0)
    }

    /// Make a copy of these flags setting all bits selected by the mask
    pub fn set(self, mask: Self) -> Self {
        self | mask
    }

    /// Make a copy of these flags resetting all bits selected by the mask
    pub fn reset(self, mask: Self) -> Self {
        self & !mask
    }

    /// Get the flags specified in the mask
    pub fn select(self, mask: Self) -> Self {
        Self(self.0 & mask.0)
    }
}

impl BitOr for Flags {
    type Output = Flags;

    /// Get all flags that are set in either `self` or `rhs`.
    fn bitor(self, rhs: Self) -> Self::Output {
        Flags(self.0 | rhs.0)
    }
}

impl BitOrAssign for Flags {
    /// Copy all set flags in `rhs` to `self`
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs
    }
}

impl BitAnd for Flags {
    type Output = Flags;

    /// Return all flags that are set in both `self` and `rhs`.
    fn bitand(self, rhs: Self) -> Self::Output {
        Flags(self.0 & rhs.0)
    }
}

impl BitAndAssign for Flags {
    /// Update `self` to contain only flags that are set in both `self` and `rhs`.
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs
    }
}

impl Not for Flags {
    type Output = Self;

    /// Flip the flags that are set
    fn not(self) -> Self::Output {
        Flags(!self.0)
    }
}

impl Sub for Flags {
    type Output = Self;

    /// Return `self` with all the flags set in `rhs` removed.
    fn sub(self, rhs: Self) -> Self::Output {
        Flags(self.0 & !rhs.0)
    }
}

impl SubAssign for Flags {
    /// Clear all flags that are set in `rhs`.
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ensure the indices of Register and Register16 are consistent
    #[test]
    fn register_enums_test() {
        // All registers grouped in trios of 16-bit register, high and low
        let grouped = [
            (Register16::AF, Register::A, Register::Flags),
            (Register16::BC, Register::B, Register::C),
            (Register16::DE, Register::D, Register::E),
            (Register16::HL, Register::H, Register::L),
            (Register16::IX, Register::IXH, Register::IXL),
            (Register16::IY, Register::IYH, Register::IYL),
            (Register16::PC, Register::PCH, Register::PCL),
            (Register16::RI, Register::R, Register::I),
            (Register16::SP, Register::SPH, Register::SPL),
            (Register16::AfAlt, Register::AAlt, Register::FAlt),
            (Register16::BcAlt, Register::BAlt, Register::CAlt),
            (Register16::DeAlt, Register::DAlt, Register::EAlt),
        ];
        for (r16, high, low) in grouped {
            assert_eq!(r16 as usize, high as usize / 2);
            assert_eq!(r16 as usize, low as usize / 2);
            assert_eq!(low as usize + 1, high as usize);
        }
    }
}

/// How the processor handles interruptions
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Default)]
pub enum InterruptMode {
    /// The data received from the interrupting device is executed as an instruction.
    ///
    /// This is the original mode from the Intel 8080 and is the default.
    #[default]
    Instruction,
    /// This mode handles interruptions by jumping to the address `0038h`
    Rst0038,
    /// This mode handles interrupts using the interrupt vector in the `I` register.
    ///
    /// When in this mode the processor performs a jump to an address formed by the value of the `I`
    /// vector as the most significant and the data sent by the device as the least significant byte.
    Vectored,
}
