use maz80emu::cpus::z80::Z80;
use maz80emu::emulator::Emulator;
use maz80emu::instructions::ExecResult;
use maz80emu::state::Register16;

const MIN_PRINT: usize = 0;
const MAX_PRINT: usize = 10000;

/// Run a program in a very limited CP/M emulation
struct CpmRunner {
    /// The number of instructions executed
    instruction_counter: usize,
    /// The memory
    memory: [u8; 1 << 16],
    /// The emulator that will run the program
    emulator: Emulator,
}

impl CpmRunner {
    /// Create a new runner
    fn new(program: &[u8]) -> Self {
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
        let mut runner = Self {
            instruction_counter: 0,
            memory,
            emulator: Emulator::new_with_instruction_set(&Z80),
        };
        // Point PC to the start of the program
        runner.emulator.state.set_register_16(Register16::PC, 0x100);
        runner
    }

    /// Handle CP/M BDOS call 5
    fn bdos_call(&self) {
        match self.emulator.state.c() {
            2 => {
                // Function 2: Print s character to the screen
                print!("{}", self.emulator.state.e() as char);
            }
            9 => {
                // Function 9: Write a $ terminated string to the screen
                let mut addr = self.emulator.state.de() as usize;
                while self.memory[addr] != '$' as u8 {
                    print!("{}", self.memory[addr] as char);
                    addr += 1;
                }
            }
            _ => panic!("Unknown BDOS call"),
        }
    }

    /// Run the program stored in memory
    fn run(&mut self) {
        if self.instruction_counter == MAX_PRINT {
            return;
        } else if self.instruction_counter == MIN_PRINT {
            self.emulator.enable_state_dump();
        }

        loop {
            match self.emulator.run_with_full_memory(&mut self.memory) {
                ExecResult::In { .. } => {
                    self.bdos_call();
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
                _ => {
                    self.instruction_counter += 1;
                }
            }
        }
    }
}

fn main() {
    let mut runner = CpmRunner::new(include_bytes!("prelim.com"));
    runner.run();
}
