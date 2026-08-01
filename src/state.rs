//! State of a Z80 CPU.
//!
//! The state is formed by the normal registers, the alternate register sets and the interruption
//! flip-flops. This is basically the same as described in the Z80 User Manual. Unlike most
//! emulators we don't manage memory, so the user is free to use any memory layout as they please.
//!
//! The registers are implemented as an array of two element arrays of bytes (`[[u8;2];N]`). Since
//! the Z80 can transfer both 8 and 16 bits to memory in a single instruction, this gives us
//! flexibility to pass around references to both 8-bit and 16-bit data types.

/// The list of registers in a Z80 CPU.
///
/// The order may seem a bit strange, but that's because the Z80 is a little endian CPU. This means
/// when we store in memory the value of 16 bit registers, the least significant byte goes first.
/// If we keep our registers pairs also in little endian order we may avoid some format translation.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Register {
    /// The flags
    Flags,
    /// The accumulator
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
}

/// The list of 16 bit registers.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Register16 {
    /// Accumulator and flags
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
}

/// The last register
const LAST_REGISTER: Register16 = Register16::PC;

/// The last register with an alternate
const LAST_ALTERNATE: Register16 = Register16::HL;

/// The state of a Z80 CPU.
#[derive(Debug, Clone, Copy, Default)]
pub struct State {
    /// The set of registers
    pub(crate) registers: [[u8; 2]; LAST_REGISTER as usize + 1],
    /// Alternate register set for AF, BC, DE and HL
    pub(crate) alternate: [[u8; 2]; LAST_ALTERNATE as usize + 1],
    /// Interrupt flip-flop
    pub(crate) iff1: bool,
    /// Temporary storage for `iff1``
    pub(crate) iff2: bool,
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

    /// Set flags in the `FLAGS` register.
    ///
    /// For bits that are 1, the corresponding bit in the `FLAGS` register is set. For bits that are
    /// 0 the register bit remains unchanged.
    ///
    /// ## Example
    /// ```
    /// # use maz80emu::state::{flags, State, Register};
    /// let mut state = State::new();
    /// // Some initial value
    /// state.set_register_8(Register::Flags, flags::C);
    /// assert_eq!(state.get_register_8(Register::Flags) & flags::Z, 0);
    /// state.set_flags(flags::Z);
    /// assert!(state.get_register_8(Register::Flags) & flags::C != 0); // C is still set
    /// assert!(state.get_register_8(Register::Flags) & flags::Z != 0); // Z was newly set
    /// ```
    pub fn set_flags(&mut self, flags: u8) {
        let register = self.get_register_mut_8(Register::Flags);
        *register |= flags
    }

    /// Clear flags in the `FLAGS` register.
    ///
    /// For bits that are 1, the corresponding bit in the `FLAGS` register is cleared. For bits that
    /// are 0 the register bit remains unchanged.
    ///
    /// # Example
    /// ```
    /// # use maz80emu::state::{flags, State, Register};
    /// let mut state = State::new();
    /// // Some initial value
    /// state.set_register_8(Register::Flags, flags::C | flags::Z);
    /// assert_eq!(state.get_register_8(Register::Flags), flags::C | flags::Z);
    /// state.clear_flags(flags::Z);
    /// assert!(state.get_register_8(Register::Flags) & flags::C != 0); // C is still set
    /// assert!(state.get_register_8(Register::Flags) & flags::Z == 0); // Z was cleared
    /// ```
    pub fn clear_flags(&mut self, flags: u8) {
        let register = self.get_register_mut_8(Register::Flags);
        *register &= !flags
    }
}

/// Constants for the flags
pub mod flags {
    /// Carry flag
    pub const C: u8 = 1 << 0;

    /// Add/Subtract
    pub const N: u8 = 1 << 1;

    /// Parity
    ///
    /// Parity and overflow are two names for the same flag
    pub const P: u8 = 1 << 2;

    /// Overflow
    ///
    /// Parity and overflow are two names for the same flag
    pub const V: u8 = 1 << 2;

    /// Half carry flag
    pub const H: u8 = 1 << 4;

    /// Zero flag
    pub const Z: u8 = 1 << 6;

    /// Sign flag
    pub const S: u8 = 1 << 7;
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
        ];
        for (r16, high, low) in grouped {
            assert_eq!(r16 as usize, high as usize / 2);
            assert_eq!(r16 as usize, low as usize / 2);
            assert_eq!(low as usize + 1, high as usize);
        }
    }

    /// Ensure all registers with an alternate fit in the alternate register array.
    #[test]
    fn alternate_indices_test() {
        let alt_regs = [
            Register16::AF,
            Register16::BC,
            Register16::DE,
            Register16::HL,
        ];
        for reg in alt_regs {
            assert!(reg <= LAST_ALTERNATE);
        }
    }
}
