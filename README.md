# Marcy's Amazing Z80 Emulator!

This is the amazing Marcy's amazing Z80 emulator of amazingness!!

Seriously now, this is just an emulator for the Z80 8-bit processor written in Rust. It tries to be as simple as
possible, so most it does is processor stuff. No I/O management, just a little support for memory. Everything else is up
to the user. No memory? No problem! You can handle the processor's load and store requests by hand if you want.

This shouldn't be too hard to use and people can implement whatever memory and I/O weirdness they want.

## Why another Z80 emulator?

In college, I learned about processors and did a little programming in machine language. That was only with toy
emulators and a tiny bit of MIPS, though, so I recently realized I don't really know how a messy CISC processor works.

Well, best time to learn stuff is right now! Since there's a lot of software for the Z80 I chose it (in retrospect, I
should have started with the simpler 8080).

In the end I'm writing this mostly for myself, to learn how a real processor works. Hopefully it'll be useful for 
other people too.

## How do I use this thing?

For starters, you want to add it as a dependency to your project's `Cargo.toml`.

```toml
[dependencies]
maz80emu = { git = "https://github.com/MarcyCubed/maz80emu" }
```

Then, in your Rust program, you'll want to import some stuff:

```rust
use maz80emu::emulator::Emulator;
use maz80emu::instructions::ExecResult;
```

This is the minimum you need to run the emulator.

`Emulator` is obvious. It's the emulator. It emulates.

```rust
let mut emulator = Emulator::new_z80();

loop {
    let exec_result = emulator.run();
    match exec_result {
        ...
    }
    ...
}
```

`ExecResult` is a bit more complicated. I said before the emulator has "a little support for memory". That support 
comes from helper functions. At it's core, the emulator doesn't know anything about memory or I/O. Instead, it'll
execute an instruction until it reaches a dead end, then it'll return an `ExecResult` reporting where it stopped.

### Let's see how `ExecResult` works

```rust
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ExecResult {
    Done(u32),
    Fetch { address: u16 },
    Load { address: u16 },
    Load16 { address: u16 },
    Store { address: u16, data: u8 },
    Store16 { address: u16, data: [u8; 2] },
    In { port: u16 },
    Out { port: u16, data: u8 },
    Halt,
    Reti(u32),
    Ei(u32),
    Int(u32),
}
```

Usually a `match` is used on an `ExecResult` to handle each variant. Here is what they mean:

#### Done
This reports a regular instruction with no magical effects finished running. The next time `Emulator::run` is called,
it'll start executing the next instruction.

#### Loading data from memory: `Fetch`, `Load` and `Load16`
When the emulated processor needs to load data from memory it'll return one of these. `Fetch` and `Load` are pretty much
the same. The only difference is that `Fetch` is used to read opcodes and `Load` to read data and instruction arguments.  

`Load16` is for when the processor needs 2 bytes from memory instead of 1.

You can send data to the emulator using `Emulator::send_byte(&mut self, u8)` and
`Emulator.send_word(&mut self, [u8;2])`. 

Usage:

```rust
match emulator.run() {
    ExecResult::Fetch { address } | ExecResult::Load { address } => {
        emulator.send_byte(memory[address as usize]);
    }
    ExecResult::Load16 { address } => {
        let address = address as usize;        
        emulator.send_word([memory[address], memory[address + 1]);
    }
    _ => {}
}
```

#### Storing data in memory: `Store` and `Store16`
These are the counterparts for `Load` and friends. When the processor wants to store data in memory it'll return one of
these. `Store` to store a byte and `Store16` to store two.

```rust
match emulator.run() {
    ExecResult::Store { address, data } => {
        memory[address as usize] = data;
    }
    ExecResult::Store16 { address, data } => {
        let address = address as usize;
        memory[address] = data[0];
        memory[address + 1] = data[1];
    }
    _ => {}
}
```

#### Input and Output: `In` and `Out`
`Emulator::run` returns this when the Z80 program wants to perform I/O. They are analogue to `Load` and `Store`. The
data requested by `In` should be provided with the same `Emulator::send_byte` function as `Load`.

```rust
match emulator.run() {
    ExecResult::In { port } => {
        let byte = read_from_io_device(port);
        emulator.send_byte(byte);
    }
    ExecResult::Out { port, data } => {
        write_to_io_device(port, data);
    }
    _ => {}
}

```

#### Special instruction ends: `Halt`, `Reti` and `Ei`
Not all instructions finish with `Done`. A few return special values marking their end. `Halt` not only reports the
'HALT' instruction was executed, but also that the processor is now in a halted state. `Reti` reports the interrupt
was handled. And `Ei` that interruptions were enabled.

You can watch for them if your program needs it. Otherwise, they can be treated the same as `Done`.

#### Interrupt accepted: `Int`
This doesn't really correspond to a specific instruction, but it's returned when the processor starts handling an
interrupt.

### I want the speed of a real Z80

Easy. You can get the number of clock cycles an `ExecResult` takes using `ExecResult::t_states`. Then just pause for
the amount of time that matches your clock speed.

## This is all fine and dandy, but I just want to run my program and that memory stuff sounds like a chore

Don't worry! The emulator can take care of that for you. The function `Emulator::run_with_memory(&mut self, memory)`
will do that. Just pass it a mutable reference to a slice or array, and it'll perform all memory access it can. It'll
also eat all the `Done` results, so it'll only stop when it reaches a roadblock.

```rust
fn run_memory_image(memory: &mut [u8]) {
    let mut emulator = Emulator::new_z80();
    loop {
        match emulator.run_with_memory(memory) {
            (ExecResult::In { port }, _) => emulator.send_byte(read_from_io_device(port)),
            (ExecResult::Out { port, data }, _) => write_to_io_device(port, data),
            _ => {} // Ignore the rest
        }
    }
}
```

### I want the emulator to handle memory and `Done` for me, but not all the time

You can use `Emulator::run_with_memory_trap(memory, trap_func)`. `trap_func` is a closure of type 
`Fn(ExecResult) -> bool`. If it returns `true`, the `ExecResult` isn't processed and is just returned instead.

If you want to still perform the memory access on results caught by the trap, use
`Emulator::access_memory(&mut self, ExecResult, memory)`. You can check if a result is a memory access with
`ExecResult::is_memory` and get an address with `ExecResult::get_address`.

```rust
fn print_accessed_addresses(memory: &mut [u8]) {
    let mut emulator = Emulator::new_z80();
    loop {
        match emulator.run_with_memory_trap(memory, ExecResult::is_memory).0 { // .0 is the ExecResult
            result if result.is_memory() => {
                if let Some(address) = result.get_address() {
                    println("addr: 0x{:04x}", )
                }
            }
            _ => {} // Ignore the rest
        }
    }
}
```

### I can't wait until it finishes to do something else

You can set a limit in T-steps with `Emulator::run_with_memory_limit`.
```rust
let mut infinite_loop: [u8; 2] = [0x18, 0xfe]; // jr -2
let mut emulator = Emulator::new_z80();
emulator.run_with_memory_limit(&mut infinite_loop, 1000);
// It will reach this point without issues
```

### Can I do both?

Yeah! `Emulator::run_with_memory_trap_limit(memory, trap_func, limit)`

### What if my memory is more complicated than an array?

The `run_with_memory` family of functions actually uses a trait to represent memory. It's already implemented for arrays
and slices of bytes, but you can use your own types. It's very straightforward:

```rust
pub trait Memory {
    /// Load a byte from memory
    fn load(&self, address: u16) -> u8;

    /// Store a byte to memory
    fn store(&mut self, address: u16, data: u8);

    /// Check if the address is within the memory bounds
    fn contains(&self, address: u16) -> bool;
}
```
`load` and `store` do exactly what is expected of them. `contains` should check if the address is valid for this memory.
If it isn't, `run_with_memory` and friends will return the `ExecResult`. They will never try to access illegal 
addresses.

## I'd like some interrupts, please.

Interrupts are very simple. Call `Emulator::interrupt(&mut self, data: u8)` to cause a regular interrupt, or 
`Emulator::non_masking_interrupt` to cause a non-masking one. They'll stay pending until the processor is in a state to
handle them (not in the middle of an instruction, interrupts enabled for regular ones.)

-----
And that's all for now. I hope you find this emulator useful.