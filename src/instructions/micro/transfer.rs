use crate::instructions::ExecResult;
use crate::instructions::micro::math;
use crate::state::{Flags, Register16, State};

/// Switch the data of AF and AF'
pub fn ex_af_af(state: &mut State, cycles: u32) -> ExecResult {
    let af = state.af();
    *state.af_mut() = state.get_register_16_bytes(Register16::AfAlt);
    state.set_register_16(Register16::AfAlt, af);
    ExecResult::Done(cycles)
}

/// Switch between register sets
pub fn exx(state: &mut State, cycles: u32) -> ExecResult {
    let offset = Register16::HlAlt as usize - Register16::HL as usize;
    for register in Register16::BC as usize..=Register16::HL as usize {
        let value = state.registers[register];
        state.registers[register] = state.registers[register + offset];
        state.registers[register + offset] = value;
    }
    ExecResult::Done(cycles)
}

/// Update the registers for a `LDI` or `LDD` instruction.
///
/// The offset is applied to DE and HL.
///
/// This should be called after the memory transfer is done
fn ldx_registers(state: &mut State, offset: i16) {
    *state.de_mut() = state.de().wrapping_add_signed(offset).to_le_bytes();
    *state.hl_mut() = state.hl().wrapping_add_signed(offset).to_le_bytes();
    let bc = state.bc().wrapping_sub(1);
    *state.bc_mut() = bc.to_le_bytes();
    let flags = state.get_flags() - (Flags::H | Flags::N | Flags::V | Flags::X | Flags::Y);
    // The XY flags on ldi and friends are weird
    let za = state.z().wrapping_add(state.a());
    let xy = Flags::X.set_if(za & 1 << 3 != 0) | Flags::Y.set_if(za & 1 << 1 != 0);
    state.update_flags(flags | Flags::P.set_if(bc != 0) | xy);
}

/// Updates the registers for a `LDI` instruction
///
/// This should be called after the memory transfer is done
pub fn ldi_registers(state: &mut State, cycles: u32) -> ExecResult {
    ldx_registers(state, 1);
    ExecResult::Done(cycles)
}

/// Loop an instruction if the condition is true
///
/// Perform a loop for an instruction of length `length`, returning an [[ExecResult::Done]] with
/// the right value if a loop was performed or not.
///
/// If the loop is performed, it'll set `state.memptr` using `memptr_update`.
fn loop_if(
    state: &mut State,
    condition: bool,
    length: u16,
    memptr_update: fn(&State) -> u16,
    cycles_loop: u32,
    cycles_no_loop: u32,
) -> ExecResult {
    if condition {
        state.memptr = memptr_update(state);
        *state.pc_mut() = state.pc().wrapping_sub(length).to_le_bytes();
        ExecResult::Done(cycles_loop)
    } else {
        // End of loop
        ExecResult::Done(cycles_no_loop)
    }
}

/// Updates the registers for a `LDIR` or `LDDR` instruction
///
/// This should be called after the memory transfer is done
fn ldxr_registers(
    state: &mut State,
    offset: i16,
    cycles_loop: u32,
    cycles_no_loop: u32,
) -> ExecResult {
    ldx_registers(state, offset);
    loop_if(
        state,
        state.bc() != 0,
        2,
        |state| state.pc().wrapping_sub(1),
        cycles_loop,
        cycles_no_loop,
    )
}

/// Updates the registers for a `LDIR` instruction
///
/// This should be called after the memory transfer is done
pub fn ldir_registers(state: &mut State, cycles_loop: u32, cycles_no_loop: u32) -> ExecResult {
    ldxr_registers(state, 1, cycles_loop, cycles_no_loop)
}

/// Updates the registers for a `LDD` instruction
///
/// This should be called after the memory transfer is done
pub fn ldd_registers(state: &mut State, cycles: u32) -> ExecResult {
    ldx_registers(state, -1);
    ExecResult::Done(cycles)
}

/// Updates the registers for a `LDDR` instruction
///
/// This should be called after the memory transfer is done
pub fn lddr_registers(state: &mut State, cycles_loop: u32, cycles_no_loop: u32) -> ExecResult {
    ldxr_registers(state, -1, cycles_loop, cycles_no_loop)
}

/// Updates the registers for a `CPX(L)` instruction after HL was already loaded.
fn cpx_registers(state: &mut State, offset: i16) {
    state.memptr = state.memptr.wrapping_add_signed(offset);
    let bc = state.bc().wrapping_sub(1);
    *state.bc_mut() = bc.to_le_bytes();
    *state.hl_mut() = state.hl().wrapping_add_signed(offset).to_le_bytes();

    let (diff, flags) = math::sub_flags(state.a(), state.z(), false);
    // The strange source of X and Y
    let xy_diff = diff.wrapping_sub(flags.is_set(Flags::H) as u8);

    let flags = flags.select(Flags::S | Flags::Z | Flags::H | Flags::N)
        | state.get_flags().select(Flags::C)
        | Flags::P.set_if(bc != 0)
        | Flags::X.set_if(xy_diff & (1 << 3) != 0)
        | Flags::Y.set_if(xy_diff & (1 << 1) != 0);

    state.update_flags(flags);
}

/// Updates the registers for a `CPI` instruction
///
/// This should be called after `(HL)` was already loaded
pub fn cpi_registers(state: &mut State, cycles: u32) -> ExecResult {
    cpx_registers(state, 1);
    ExecResult::Done(cycles)
}

/// Updates the registers for a `CPD` instruction
///
/// This should be called after `(HL)` was already loaded
pub fn cpd_registers(state: &mut State, cycles: u32) -> ExecResult {
    cpx_registers(state, -1);
    ExecResult::Done(cycles)
}

/// Updates the registers for a `CPIR` instruction
///
/// This should be called after `(HL)` was already loaded
pub fn cpxr_registers(
    state: &mut State,
    offset: i16,
    cycles_loop: u32,
    cycles_no_loop: u32,
) -> ExecResult {
    cpx_registers(state, offset);
    loop_if(
        state,
        state.bc() != 0 && !state.get_flags().is_set(Flags::Z),
        2,
        |state| state.pc(),
        cycles_loop,
        cycles_no_loop,
    )
}

/// Updates the registers for a `CPIR` instruction
///
/// This should be called after `(HL)` was already loaded
pub fn cpir_registers(state: &mut State, cycles_loop: u32, cycles_no_loop: u32) -> ExecResult {
    cpxr_registers(state, 1, cycles_loop, cycles_no_loop)
}

/// Updates the registers for a `CPIR` instruction
///
/// This should be called after `(HL)` was already loaded
pub fn cpdr_registers(state: &mut State, cycles_loop: u32, cycles_no_loop: u32) -> ExecResult {
    cpxr_registers(state, -1, cycles_loop, cycles_no_loop)
}

/// Updates the registers for a `INX(R)` instruction after input and storage were already done.
fn inx_registers(state: &mut State, offset: i16) {
    state.memptr = state.bc().wrapping_add_signed(offset);
    *state.b_mut() = state.b().wrapping_sub(1);
    *state.hl_mut() = state.hl().wrapping_add_signed(offset).to_le_bytes();

    state.update_flags((state.get_flags() - Flags::Z) | Flags::N | Flags::Z.set_if(state.b() == 0));
}

/// Updates the registers for an `INI` instruction
///
/// This should be called after `(HL)` was already loaded
pub fn ini_registers(state: &mut State, cycles: u32) -> ExecResult {
    inx_registers(state, 1);
    ExecResult::Done(cycles)
}

/// Updates the registers for an `IND` instruction
///
/// This should be called after `(HL)` was already loaded
pub fn ind_registers(state: &mut State, cycles: u32) -> ExecResult {
    inx_registers(state, -1);
    ExecResult::Done(cycles)
}

/// Updates the registers for an `INIR` or `INDR` instruction
///
/// This should be called after `(HL)` was already loaded
pub fn inxr_registers(
    state: &mut State,
    offset: i16,
    cycles_loop: u32,
    cycles_no_loop: u32,
) -> ExecResult {
    inx_registers(state, offset);
    loop_if(
        state,
        state.b() != 0,
        2,
        |state| state.memptr,
        cycles_loop,
        cycles_no_loop,
    )
}

/// Updates the registers for an `INIR` instruction
///
/// This should be called after `(HL)` was already loaded
pub fn inir_registers(state: &mut State, cycles_loop: u32, cycles_no_loop: u32) -> ExecResult {
    inxr_registers(state, 1, cycles_loop, cycles_no_loop)
}

/// Updates the registers for an `INDR` instruction
///
/// This should be called after `(HL)` was already loaded
pub fn indr_registers(state: &mut State, cycles_loop: u32, cycles_no_loop: u32) -> ExecResult {
    inxr_registers(state, -1, cycles_loop, cycles_no_loop)
}

/// Updates the registers for an `OUTX(R)` instruction
fn outx_registers(state: &mut State, offset: i16) {
    *state.b_mut() = state.b().wrapping_sub(1);
    state.memptr = state.bc().wrapping_add_signed(offset);

    *state.hl_mut() = state.hl().wrapping_add_signed(offset).to_le_bytes();
    state.update_flags((state.get_flags() - Flags::Z) | Flags::N | Flags::Z.set_if(state.b() == 0));
}

/// Updates the registers for an `OUTI` instruction
pub fn outi_registers(state: &mut State, cycles: u32) -> ExecResult {
    outx_registers(state, 1);
    ExecResult::Done(cycles)
}

/// Updates the registers for an `OUTD` instruction
pub fn outd_registers(state: &mut State, cycles: u32) -> ExecResult {
    outx_registers(state, -1);
    ExecResult::Done(cycles)
}

/// Updates the registers for an `OTIR` or `OTDR` instruction
pub fn otxr_registers(
    state: &mut State,
    offset: i16,
    cycles_loop: u32,
    cycles_no_loop: u32,
) -> ExecResult {
    outx_registers(state, offset);
    loop_if(
        state,
        state.b() != 0,
        2,
        |state| state.memptr,
        cycles_loop,
        cycles_no_loop,
    )
}

/// Updates the registers for an `OTIR` instruction
pub fn otir_registers(state: &mut State, cycles_loop: u32, cycles_no_loop: u32) -> ExecResult {
    otxr_registers(state, 1, cycles_loop, cycles_no_loop)
}

/// Updates the registers for an `OTDR` instruction
pub fn otdr_registers(state: &mut State, cycles_loop: u32, cycles_no_loop: u32) -> ExecResult {
    otxr_registers(state, -1, cycles_loop, cycles_no_loop)
}
