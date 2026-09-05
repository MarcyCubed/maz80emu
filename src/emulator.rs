//! The core of the emulator

use crate::cpus::z80::Z80;
use crate::instructions::decoder::Decoder;
use crate::instructions::micro::{Microinstruction, jump};
use crate::instructions::{ExecResult, InstructionSet};
use crate::state::{InterruptMode, State};

/// The Z80 emulator
#[derive(Debug, Clone)]
pub struct Emulator {
    /// Internal state of the processor
    pub state: State,
    /// The instruction decoder
    decoder: Decoder,
    /// The microinstructions being executed
    micros: &'static [Microinstruction],
    /// Is a Nonmaskable Interrupt waiting to be handled?
    nmi_pending: bool,
    /// Is a regular interrupt waiting to be handled?
    int_pending: bool,
    /// Data for the interrupt
    interrupt_data: u8,
    /// The result of the last time `run` was executed
    last_result: ExecResult,
}

impl Emulator {
    /// Create an emulator to run programs for the instruction set
    pub fn new_with_instruction_set(instruction_set: &'static InstructionSet) -> Emulator {
        Emulator {
            state: Default::default(),
            decoder: Decoder::new(instruction_set),
            micros: &[],
            nmi_pending: false,
            int_pending: false,
            interrupt_data: 0,
            last_result: ExecResult::Done(0),
        }
    }

    /// Create a Z80 emulator
    pub fn new_z80() -> Self {
        Self::new_with_instruction_set(&Z80)
    }

    /// Check if the processor is halted
    pub fn is_halted(&self) -> bool {
        self.last_result == ExecResult::Halt
    }

    /// Try to service a pending interrupt
    ///
    /// if an interrupt can be serviced, return a list of microinstructions to run the interrupt
    /// handler and an `ExecResult` marking the time it took.
    /// Otherwise, return `None` if there is no interrupt to service, if they are disabled or if the
    /// processor is busy processing.
    fn service_interrupt(&mut self) -> Option<(&'static [Microinstruction], ExecResult)> {
        // Check if the processor is in the middle of an instruction
        if !self.last_result.is_finished() {
            None
        } else if self.nmi_pending {
            // Service a nonmaskable interrupt
            self.nmi_pending = false;
            self.state.iff1 = false;
            self.state.advance_r();
            Some((
                &[jump::push_pc, |state| jump::jump_to(state, 0x66, 0)],
                ExecResult::Int(5),
            ))
        } else if !self.last_result.can_interrupt() {
            None
        } else if self.int_pending && self.state.iff1 {
            self.int_pending = false;
            match self.state.interrupt_mode {
                InterruptMode::Instruction => {
                    // Dispatch the instruction
                    let injected = self.decoder.inject_opcode(self.interrupt_data);
                    assert!(
                        injected,
                        "Failed to inject instruction {:02x}h",
                        self.interrupt_data
                    );
                    Some((&[], ExecResult::Int(6)))
                }
                InterruptMode::Rst0038 => {
                    let injected = self.decoder.inject_opcode(0xff);
                    assert!(injected, "Failed to inject rst 0x38 instruction");
                    Some((&[], ExecResult::Int(6)))
                }
                InterruptMode::Vectored => {
                    // Put the address in WZ
                    *self.state.wz_mut() = [self.interrupt_data, self.state.i()];
                    Some((
                        &[
                            // Save the PC
                            jump::push_pc,
                            // Load the address of the interrupt handler
                            |state| ExecResult::load16(state.wz()),
                            // Finish the call to the interrupt handler
                            |state| jump::jr_mm(state, 0),
                        ],
                        ExecResult::Int(7),
                    ))
                }
            }
        } else {
            None
        }
    }

    /// Run the emulator
    pub fn run(&mut self) -> ExecResult {
        // If we're running microinstructions
        let result = if self.state.mpc < self.micros.len() {
            let micro = self.micros[self.state.mpc];
            self.state.mpc += 1;
            micro(&mut self.state)
        } else if let Some((micros, result)) = self.service_interrupt() {
            self.micros = micros;
            self.state.mpc = 0;
            result
        } else {
            // Need to decode the next instruction
            self.micros = self.decoder.decode(&mut self.state);
            self.state.mpc = 1; // We'll run the [0] now.
            self.micros[0](&mut self.state)
        };
        self.last_result = result;
        result
    }

    /// Interrupt request
    pub fn interrupt(&mut self, data: u8) {
        self.int_pending = true;
        self.interrupt_data = data;
    }

    /// Non-masking Interrupt request
    pub fn non_masking_interrupt(&mut self) {
        self.nmi_pending = true;
    }

    /// If the [ExecResult] is related to loading or storing data in the memory, try to perform it.
    ///
    /// If the request is a memory access and its address is within the memory's bounds, the
    /// requested action will be performed and the function will return `true`.
    /// Otherwise, nothing will be done and `false` will be returned instead.
    pub fn access_memory(&mut self, request: ExecResult, memory: &mut [u8]) -> bool {
        // Get the address for the memory access if it's valid
        fn get_address(memory: &[u8], address: u16) -> Option<usize> {
            let address = address as usize;
            if address < memory.len() {
                Some(address)
            } else {
                None
            }
        }

        // Get the addresses for the first and second index
        fn get_address_16(memory: &[u8], address: u16) -> Option<(usize, usize)> {
            let address_0 = get_address(memory, address)?;
            let address_1 = get_address(memory, address.wrapping_add(1))?;
            Some((address_0, address_1))
        }

        match request {
            ExecResult::Fetch { address } | ExecResult::Load { address } => {
                if let Some(address) = get_address(memory, address) {
                    self.send_byte(memory[address]);
                    true
                } else {
                    false
                }
            }
            ExecResult::Load16 { address } => {
                if let Some((address_0, address_1)) = get_address_16(memory, address) {
                    self.send_word([memory[address_0], memory[address_1]]);
                    true
                } else {
                    false
                }
            }
            ExecResult::Store { address, data } => {
                if let Some(address) = get_address(memory, address) {
                    memory[address] = data;
                    true
                } else {
                    false
                }
            }
            ExecResult::Store16 { address, data } => {
                if let Some((address_0, address_1)) = get_address_16(memory, address) {
                    memory[address_0] = data[0];
                    memory[address_1] = data[1];
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Run the emulator with a given memory, optionally disabling automatic handling of any
    /// operation.
    ///
    /// If the requested address is within the memory slice and `trap` returns `false`, the memory
    /// request will be processed automatically.
    /// Otherwise, the load or store request will be returned as in [[Emulator::run]].
    pub fn run_with_memory_trap<F: Fn(ExecResult) -> bool>(
        &mut self,
        memory: &mut [u8],
        trap: F,
    ) -> ExecResult {
        loop {
            let result = self.run();
            if trap(result) {
                return result;
            }
            match result {
                ExecResult::Done(_) => {}
                result if self.access_memory(result, memory) => {}
                _ => return result,
            }
        }
    }

    /// Run the emulator with memory.
    ///
    /// Handle memory access for the emulator. If the requested address is out of bounds, the
    /// load or store request will be returned as in [[Emulator::run]].
    pub fn run_with_memory(&mut self, memory: &mut [u8]) -> ExecResult {
        self.run_with_memory_trap(memory, |_| false)
    }

    /// Pass a byte to the emulator to complete a memory or input instruction
    pub fn send_byte(&mut self, byte: u8) {
        self.state.load_data_8(byte)
    }

    /// Pass a word to the emulator to complete a memory load
    pub fn send_word(&mut self, word: [u8; 2]) {
        self.state.load_data_16(word)
    }

    /// Show the instructions as they are executed
    pub fn enable_tracing(&mut self) {
        self.decoder.enable_tracing();
    }

    /// Stop showing the instructions
    pub fn disable_tracing(&mut self) {
        self.decoder.disable_tracing();
    }

    /// Show the state before each instruction
    pub fn enable_state_dump(&mut self) {
        self.decoder.enable_state_dump();
    }

    /// Don't show the state before each instruction
    pub fn disable_state_dump(&mut self) {
        self.decoder.disable_state_dump();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::Register16;
    use std::assert_matches;

    /// Make a program to test interrupts
    ///
    ///  We can track where we are in the program
    fn make_program() -> [u8; 256] {
        [
            0xd3, 0x01, // 00h: out (1), a
            // Put SP within the memory
            0x31, 0xff, 0x00, // 02h: ld sp, 0xff
            0xfb, // 05h: ei
            0xd3, 0x02, // 06h: out (2), a
            0xd3, 0x03, // 08h: out (3), a
            // Block with interrupts disabled
            0xf3, // 0ah: di
            0xd3, 0x04, // 0bh: out (4), a
            0xfb, // 0dh: ei
            // An out immediately after ei
            0xd3, 0x05, // 0eh: out (5), a
            0xd3, 0x06, // 10h: out (6), a
            0x76, // 12h: halt
            0x00, 0x00, 0x00, 0x00, 0x00, // 13h: nop *  5
            // an interruption handler at 18h
            0xf3, // 18h: di
            0xd3, 0x07, // 19h: out (7), a
            0xfb, // 1bh: ei
            0xed, 0x4d, // 1ch: reti
            0xd3, 0x06, // 1eh: out (6), a
            // an interruption handler with 2 outs at 20h
            0xf3, // 20h: di
            0xd3, 0x20, // 21h: out (20), a
            0xd3, 0x21, // 23h: out (21), a
            0xfb, // 25h: ei
            0xed, 0x4d, // 26h: reti
            // A different interruption handler at 28h
            0xf3, // 28h: di
            0x00, // 29h: nop
            0xd3, 0x28, // 2ah: out (0x28), a
            0x00, // 2ch: nop
            0xfb, // 2dh: ei
            0xed, 0x4d, // 2eh: reti
            // Filler
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 30h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 38h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 40h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 48h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 50h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 58h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 60h: nop * 6
            // NMI handler
            0xd3, 0x66, // 66h: out (66h), a
            0xed, 0x45, // 68h: retn
            // Should be unreachable
            0xd3, 0xbb, // 6ah: out (bbh), a
            0x76, // 6ch: halt
            0x00, 0x00, 0x00, // 6dh: nop
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 70h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 78h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 80h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 88h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 90h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 98h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // a0h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // a8h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // b0h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // b8h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // c0h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // c8h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // d0h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // d8h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // e0h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // e8h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // f0h: nop *  8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // f8h: nop *  8
        ]
    }

    macro_rules! assert_port {
        ($number:literal, $result:expr, $text: literal) => {
            assert_matches!($result, ExecResult::Out { .. });
            if let ExecResult::Out { port, .. } = $result {
                assert_eq!($number, port & 0xff, $text);
            }
        };
    }

    #[test]
    fn check_if_interrupts_trigger_when_they_should() {
        let mut emulator = Emulator::new_z80();
        let mut program = make_program();
        // Start running
        let result = emulator.run();
        // It should start by fetching an instruction, otherwise the emulator isn't working the same
        // way as when this test was first written
        assert!(matches!(result, ExecResult::Fetch { address: 0 }));
        // Can't service an interrupt when an instruction is running
        assert_eq!(emulator.service_interrupt(), None);
        // Request an interrupt that runs rst 18h
        emulator.interrupt(0xdf);
        // Still can't  service the interrupt
        assert_eq!(emulator.service_interrupt(), None);
        emulator.access_memory(result, &mut program);
        // Should run into the first out before the ei because it was already running when the
        // interruption was requested
        let result = emulator.run_with_memory(&mut program);
        assert_port!(
            1,
            result,
            "Interruption handled during instruction execution"
        );
        // ei finished. The next instruction should be still not interrupted
        let result = emulator.run_with_memory(&mut program);
        assert_matches!(result, ExecResult::Ei(_));
        // Should run into the out after the ei
        let result = emulator.run_with_memory(&mut program);
        assert_port!(
            2,
            result,
            "ei didn't block interruptions for the next instruction"
        );
        // Should start handling the interruption
        let result = emulator.run_with_memory(&mut program);
        assert_matches!(result, ExecResult::Int(_));
        // Should perform a rst 18h and run into the out inside the handler
        let result = emulator.run_with_memory(&mut program);
        assert_port!(7, result, "Didn't start handling interruption");
        // EI at the end of the handler
        let result = emulator.run_with_memory(&mut program);
        assert_matches!(result, ExecResult::Ei(_));
        // RETI returning from the handler
        let result = emulator.run_with_memory(&mut program);
        assert_matches!(result, ExecResult::Reti(_));
        // Should return to where we were before
        let result = emulator.run_with_memory(&mut program);
        assert_port!(3, result, "interruption didn't return properly");
    }

    #[test]
    fn check_if_interrupts_trigger_in_the_middle_of_instructions_and_break_stuff() {
        let mut emulator = Emulator::new_z80();
        let mut program = make_program();
        // Put SP within our memory by hand since we'll skip that instruction in this test
        emulator.state.set_register_16(Register16::SP, 0xff);
        // Start executing the first instruction to put the processor in a state it shouldn't be
        // able to run instructions
        let result = emulator.run();
        assert!(matches!(result, ExecResult::Fetch { address: 0 }));
        emulator.access_memory(result, &mut program);
        // Trigger a NMI
        emulator.non_masking_interrupt();
        // Continue running the first instruction. We should get out as a result
        let result = emulator.run_with_memory(&mut program);
        assert_port!(
            1,
            result,
            "Interruption handled during instruction execution"
        );
        let result =
            emulator.run_with_memory_trap(&mut program, |e| matches!(e, ExecResult::Done(_)));
        assert_matches!(
            result,
            ExecResult::Done(_),
            "The instruction didn't finish as it should"
        );
        // Continue execution, we should go get a result indicating an interruption was triggered
        assert_matches!(emulator.run_with_memory(&mut program), ExecResult::Int(_));
        // Keep running. We should be inside the NMI handler at address 66h
        let result = emulator.run_with_memory(&mut program);
        assert_port!(0x66, result, "NMI didn't trigger");
        // The program now should return from the handler and execute the EI instruction
        assert_matches!(emulator.run_with_memory(&mut program), ExecResult::Ei(_));
    }

    #[test]
    fn check_if_nmis_have_precedence_over_regular_interrupts() {
        let mut emulator = Emulator::new_z80();
        let mut program = make_program();
        // Loop until interrupts are enabled
        loop {
            if let ExecResult::Out { port, .. } = emulator.run_with_memory(&mut program) {
                // Check if it's the out after the ei
                if port & 0xff == 0x02 {
                    break;
                }
            }
        }
        // Trigger an NMI and an interrupt
        emulator.non_masking_interrupt();
        emulator.interrupt(0xe7); // rst 20h
        // Run until an out
        loop {
            if let ExecResult::Out { port, .. } = emulator.run_with_memory(&mut program) {
                assert_eq!(0x66, port & 0xff, "Didn't go to the NMI handler");
                break;
            }
        }
        // Run until we're in the middle of the interrupt handler
        loop {
            if let ExecResult::Out { port, .. } = emulator.run_with_memory(&mut program) {
                if port & 0xff == 0x20 {
                    break;
                }
            }
        }
        // Interrupts are disabled, and we're in the middle of a handler: Trigger a NMI.
        emulator.non_masking_interrupt();
        // Run until an out
        loop {
            if let ExecResult::Out { port, .. } = emulator.run_with_memory(&mut program) {
                // We should have jumped straight into the NMI handler
                assert_eq!(0x66, port & 0xff, "Didn't go to the NMI handler");
                break;
            }
        }
        // After all is said and done we should go back to the interruption handler that was running
        // before the NMI
        loop {
            if let ExecResult::Out { port, .. } = emulator.run_with_memory(&mut program) {
                // We should have jumped straight into the NMI handler
                assert_eq!(
                    0x21,
                    port & 0xff,
                    "Didn't return to the interruption handler at 20h"
                );
                break;
            }
        }
    }
}
