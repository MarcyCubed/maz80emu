use crate::instructions::ExecResult;
use crate::instructions::micro::math;
use crate::state::{Flags, Register, Register16, State};

/// Switch the data of AF and AF'
pub fn ex_af_af(state: &mut State, cycles: u32) -> ExecResult {
    let af = state.af_bytes();
    *state.af_mut() = state.alternate[Register16::AF as usize];
    state.alternate[Register16::AF as usize] = af;
    ExecResult::Done(cycles)
}

/// Switch between register sets
pub fn exx(state: &mut State, cycles: u32) -> ExecResult {
    let dest = &mut state.registers[Register16::BC as usize..=Register16::HL as usize];
    dest.copy_from_slice(&state.alternate[Register16::BC as usize..=Register16::HL as usize]);
    ExecResult::Done(cycles)
}

/// Update the registers for a `LDI` or `LDD` instruction.
///
/// The offset is applied to DE and HL.
///
/// This should be called after the memory transfer is done
fn ldx_registers(state: &mut State, offset: u16) {
    *state.de_mut() = state.de().wrapping_add(offset).to_le_bytes();
    *state.hl_mut() = state.hl().wrapping_add(offset).to_le_bytes();
    let bc = state.bc().wrapping_sub(1);
    *state.bc_mut() = bc.to_le_bytes();
    let flags = state.get_flags() - (Flags::H | Flags::N | Flags::V);
    state.update_flags(flags | Flags::P.set_if(bc != 0));
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
fn loop_if(
    state: &mut State,
    condition: bool,
    length: u16,
    cycles_loop: u32,
    cycles_no_loop: u32,
) -> ExecResult {
    if condition {
        *state.pc_mut() = state.pc().wrapping_sub(length).to_le_bytes();
        ExecResult::Done(cycles_loop)
    } else {
        // End of loop
        ExecResult::Done(cycles_no_loop)
    }
}

/// Updates the registers for a `LDIR` instruction
///
/// This should be called after the memory transfer is done
pub fn ldir_registers(state: &mut State, cycles_loop: u32, cycles_no_loop: u32) -> ExecResult {
    ldx_registers(state, 1);
    loop_if(state, state.bc() != 0, 2, cycles_loop, cycles_no_loop)
}

/// Updates the registers for a `LDD` instruction
///
/// This should be called after the memory transfer is done
pub fn ldd_registers(state: &mut State, cycles: u32) -> ExecResult {
    ldx_registers(state, 0xffff); // offset == -1
    ExecResult::Done(cycles)
}

/// Updates the registers for a `LDDR` instruction
///
/// This should be called after the memory transfer is done
pub fn lddr_registers(state: &mut State, cycles_loop: u32, cycles_no_loop: u32) -> ExecResult {
    ldx_registers(state, 0xffff); // offset == -1
    loop_if(state, state.bc() != 0, 2, cycles_loop, cycles_no_loop)
}

/// Updates the registers for a `CPX(L)` instruction after HL was already loaded.
fn cpx_registers(state: &mut State, offset: u16) {
    let bc = state.bc().wrapping_add(offset);
    *state.bc_mut() = bc.to_le_bytes();
    *state.hl_mut() = state.hl().wrapping_add(1).to_le_bytes();

    let c_flag = state.get_flags() & Flags::C;
    math::cp_r(state, Register::Z, 0);
    state.update_flags(
        (state.get_flags() - Flags::C - Flags::V) | c_flag | Flags::V.set_if(bc != 0),
    );
}

/// Updates the registers for a `CPI` instruction
///
/// This should be called after `(HL)` was already loaded
pub fn cpi_registers(state: &mut State, cycles: u32) -> ExecResult {
    cpx_registers(state, 1);
    ExecResult::Done(cycles)
}

/// Updates the registers for a `CPIR` instruction
///
/// This should be called after `(HL)` was already loaded
pub fn cpir_registers(state: &mut State, cycles_loop: u32, cycles_no_loop: u32) -> ExecResult {
    cpx_registers(state, 1);
    loop_if(
        state,
        state.bc() != 0 && !state.get_flags().is_set(Flags::Z),
        2,
        cycles_loop,
        cycles_no_loop,
    )
}

/// Updates the registers for a `CPD` instruction
///
/// This should be called after `(HL)` was already loaded
pub fn cpd_registers(state: &mut State, cycles: u32) -> ExecResult {
    cpx_registers(state, 0xffff); // Offset == -1
    ExecResult::Done(cycles)
}

/// Updates the registers for a `CPIR` instruction
///
/// This should be called after `(HL)` was already loaded
pub fn cpdr_registers(state: &mut State, cycles_loop: u32, cycles_no_loop: u32) -> ExecResult {
    cpx_registers(state, 0xffff); // Offset == -1
    loop_if(
        state,
        state.bc() != 0 && !state.get_flags().is_set(Flags::Z),
        2,
        cycles_loop,
        cycles_no_loop,
    )
}

/// Updates the registers for a `INX(R)` instruction after input and storage were already done.
fn inx_registers(state: &mut State, offset: u16) {
    *state.b_mut() = state.b().wrapping_sub(1);
    *state.hl_mut() = state.hl().wrapping_add(offset).to_le_bytes();

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
    inx_registers(state, 0xffff); // offset is -1
    ExecResult::Done(cycles)
}

/// Updates the registers for an `INIR` instruction
///
/// This should be called after `(HL)` was already loaded
pub fn inir_registers(state: &mut State, cycles_loop: u32, cycles_no_loop: u32) -> ExecResult {
    inx_registers(state, 1);
    loop_if(state, state.b() != 0, 2, cycles_loop, cycles_no_loop)
}

/// Updates the registers for an `INDR` instruction
///
/// This should be called after `(HL)` was already loaded
pub fn indr_registers(state: &mut State, cycles_loop: u32, cycles_no_loop: u32) -> ExecResult {
    inx_registers(state, 0xffff);
    loop_if(state, state.b() != 0, 2, cycles_loop, cycles_no_loop)
}

/// Updates the registers for an `OUTX(R)` instruction
fn outx_registers(state: &mut State, offset: u16) {
    *state.hl_mut() = state.hl().wrapping_add(offset).to_le_bytes();
    state.update_flags((state.get_flags() - Flags::Z) | Flags::N | Flags::Z.set_if(state.b() == 0));
}

/// Updates the registers for an `OUTI` instruction
pub fn outi_registers(state: &mut State, cycles: u32) -> ExecResult {
    outx_registers(state, 1);
    ExecResult::Done(cycles)
}

/// Updates the registers for an `OUTD` instruction
pub fn outd_registers(state: &mut State, cycles: u32) -> ExecResult {
    outx_registers(state, 0xffff); // offset is -1
    ExecResult::Done(cycles)
}

/// Updates the registers for an `OTIR` instruction
pub fn otir_registers(state: &mut State, cycles_loop: u32, cycles_no_loop: u32) -> ExecResult {
    outx_registers(state, 1);
    loop_if(state, state.b() != 0, 2, cycles_loop, cycles_no_loop)
}

/// Updates the registers for an `OTDR` instruction
pub fn otdr_registers(state: &mut State, cycles_loop: u32, cycles_no_loop: u32) -> ExecResult {
    outx_registers(state, 0xffff);
    loop_if(state, state.b() != 0, 2, cycles_loop, cycles_no_loop)
}
