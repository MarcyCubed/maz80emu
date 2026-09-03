use maz80emu::emulator::Emulator;
use maz80emu::instructions::ExecResult;
use maz80emu::state::Register16;
use std::fs;

const MIN_PRINT: usize = usize::MAX;
const MAX_PRINT: usize = usize::MAX;

//const MIN_PRINT: usize = 000000;
//const MAX_PRINT: usize = 100000;

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
        // Trap CP/M program exit with an "OUT 1, a" instruction
        memory[0x0] = 0xD3;
        memory[0x1] = 0x00;
        // CP/M BDOS call is an IN a, 0 instruction so we can trap and handle it
        memory[0x5] = 0xDB;
        memory[0x6] = 0x00;
        // Return from the BDOS call
        memory[0x7] = 0xC9;
        let mut runner = Self {
            instruction_counter: 0,
            memory,
            emulator: Emulator::new_z80(),
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
        loop {
            if self.instruction_counter == MAX_PRINT {
                return;
            } else if self.instruction_counter == MIN_PRINT {
                self.emulator.enable_state_dump();
                //self.emulator.enable_tracing();
            }
            match self.emulator.run_with_memory_trap(&mut self.memory, |er| {
                matches!(er, ExecResult::Fetch { .. })
            }) {
                ExecResult::In { .. } => {
                    self.bdos_call();
                    self.emulator.state.load_data_8(0xff);
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
                ExecResult::Fetch { address } => {
                    self.emulator.send_byte(self.memory[address as usize]);
                    self.instruction_counter += 1;
                }
                _ => {}
            }
        }
    }
}

fn main() {
    for file in std::env::args().skip(1) {
        match fs::read(&file) {
            Ok(file) => {
                let mut runner = CpmRunner::new(&file);
                runner.run();
            }
            Err(error) => {
                eprintln!("Can't open file {} : {}", file, error)
            }
        }
    }
}
