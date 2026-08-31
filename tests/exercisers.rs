//! Test the 8080 architecture

use maz80emu::cpus::z80::Z80;
use maz80emu::emulator::Emulator;
use maz80emu::instructions::ExecResult;
use maz80emu::state::{Register16, State};

/// Create a block of memory with the given program loaded at address 0x100 and the CP/M call stubs
/// set up
fn load_program(program: &[u8]) -> [u8; 0x10000] {
    // We initialize the memory with "HALT" instructions, so whenever we go where we shouldn't the
    // program crashes.
    let mut memory = [0x76; 0x10000];
    memory[0x100..program.len() + 0x100].copy_from_slice(program);

    // Trap CP/M program exit with an "OUT" instruction
    memory[0x0] = 0xD3;
    // CP/M BDOS call is an IN instruction so we can trap and handle it
    memory[0x5] = 0xDB;
    // Return from the BDOS call
    memory[0x7] = 0xC9;
    memory
}

/// Handle CP/M BDOS call 5
fn bdos_call(state: &State, memory: &[u8; 0x10000]) {
    match state.c() {
        2 => {
            // Function 2: Print s character to the screen
            print!("{}", state.e() as char);
        }
        9 => {
            // Function 9: Write a $ terminated string to the screen
            let mut addr = state.de() as usize;
            while memory[addr] != '$' as u8 {
                print!("{}", memory[addr] as char);
                addr += 1;
            }
        }
        _ => panic!("Unknown BDOS call"),
    }
}

/// Run a program in a very limited CP/M environment
fn run_program(program: &[u8]) {
    run_memory(load_program(program));
}

/// Run the program stored in memory
fn run_memory(mut memory: [u8; 0x10000]) {
    //let mut memory = load_program(program);
    let mut emulator = Emulator::new_with_instruction_set(&Z80);
    // Point PC to the start of the program
    emulator.state.set_register_16(Register16::PC, 0x100);
    //emulator.enable_tracing();
    //emulator.enable_state_dump();
    loop {
        match emulator.run_with_full_memory(&mut memory) {
            ExecResult::In { .. } => {
                bdos_call(&emulator.state, &memory);
            }
            ExecResult::Out { .. } => {
                println!();
                println!("Finished execution");
                break;
            }
            ExecResult::Halt => {
                println!();
                println!("Crashed");
                break;
            }
            _ => {}
        }
    }
}

#[test]
fn pre_exerciser_8080() {
    run_program(include_bytes!("8080PRE.COM"));
}

#[test]
fn exerciser_8080() {
    run_program(include_bytes!("8080EXER.COM"));
}

#[test]
fn tst_8080() {
    let mut memory = load_program(include_bytes!("TST8080.COM"));
    // Patch the program to address how the Z80 and the 8080 handle the P flag differently
    memory[0x1f2] = 0xea; // jp po, nn => jp pe,nn
    memory[0x22e] = 0xea; // jp po, nn => jp pe,nn
    memory[0x2af] = 0xe4; // call pe, nn => call po, nn
    memory[0x2d0] = 0xe4; // call pe, nn => call po, nn
    memory[0x2d9] = 0xe8; // ret po => ret pe
    memory[0x2e1] = 0xe0; // ret pe => ret po
    run_memory(memory)
}
