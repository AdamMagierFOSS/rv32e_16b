# rv32e_16b

A RISC-V RV32E simulator with 16-bit addressable memory.

This is an experimental machine where the smallest addressable unit is a 16-bit cell rather than the standard 8-bit byte. It executes standard RV32E instructions (the embedded subset with 16 registers, x0–x15) and can run programs compiled from Rust via a cross-compilation pipeline.

## Architecture

- **Registers:** 16 × 32-bit (x0 hardwired to zero), per the RV32E spec
- **Memory:** Flat array of 16-bit cells, 65536 cells (128KB) by default
- **PC:** Byte-addressed internally; translated to cell index at the memory boundary (`cell = byte_addr >> 1`), so standard compiler output works unmodified
- **Instruction encoding:** Standard 32-bit RV32E — each instruction occupies 2 consecutive 16-bit cells
- **Sub-cell access:** LB/LBU/SB extract or modify individual 8-bit halves within a 16-bit cell using bit 0 of the byte address to select the low or high byte
- **UART:** Memory-mapped at `0x10000000` — any byte stored there is captured as console output

## Building the simulator

```
cargo build
```

## Running

**Built-in demo** (sums 1 through 5):
```
cargo run
```

**Load a binary:**
```
cargo run -- program.bin
```

**Pre-load memory values** (useful for passing input to programs):
```
cargo run -- fibonacci.bin --set 0x1000=10
```

The simulator accepts both flat binaries and ELF32 files (auto-detected by magic bytes).

## Compiling Rust programs

Programs are `#![no_std]` / `#![no_main]` Rust files. The build pipeline is:

1. `rustc --emit=llvm-ir` (compiles to LLVM IR for the host target)
2. `sed` retargets the IR to `riscv32` (fixes datalayout, triple, strips x86 artifacts)
3. `llc -march=riscv32 -mattr=+e` compiles to an RV32E object file
4. `ld.lld` links with `linker.ld`
5. `llvm-objcopy -O binary` extracts a flat binary

The `build.sh` script wraps this. Run it from the `programs/` directory:

```
cd programs
../build.sh hello.rs hello.bin
cargo run -- hello.bin      # from project root
```

### Prerequisites

The build script requires `llc`, `ld.lld`, and `llvm-objcopy` with **RISC-V target support** (specifically `riscv32` with the `+e` attribute). These must be built from an LLVM installation configured with `-DLLVM_TARGETS_TO_BUILD=RISCV` (or `all`). Many distro-packaged LLVM builds include RISC-V support, but verify with:

```
llc --version | grep riscv32
```

By default `build.sh` looks for tools on `PATH`. If your RISC-V-capable LLVM is installed elsewhere, set the `LLVM_BIN` environment variable:

```
LLVM_BIN=/path/to/llvm/bin ./build.sh hello.rs
```

You also need `rustc` (any recent stable version) for the initial Rust-to-LLVM-IR compilation step.

## Runtime library

`programs/rv32e_rt.rs` is a minimal runtime that provides:

- **`_start` trampoline** — sets up the stack pointer and calls `main()`
- **`print!("...")`** / **`println!("...")`** — write string literals to the UART
- **`eprint!` / `eprintln!`** — same UART (no hardware separation on bare metal, matching real RISC-V behavior)
- **`_print_u32(n)`** — print an unsigned 32-bit integer in decimal
- **`_print_i32(n)`** — print a signed 32-bit integer in decimal
- **`_print_hex(n)`** — print a 32-bit integer in hexadecimal (e.g., `0xdeadbeef`)
- **Panic handler** — prints `PANIC` and halts via `ebreak`

Number formatting uses binary long division to avoid hardware divide instructions (RV32E has no M extension).

Include it at the top of your program:

```rust
#![no_std]
#![no_main]

include!("rv32e_rt.rs");

#[no_mangle]
pub extern "C" fn main() {
    println!("Hello, World!");
    print!("The answer is ");
    _print_u32(42);
    println!();
}
```

### Limitations

- `print!` / `println!` only accept string literals, not format strings with `{}` — `core::fmt` can't be retargeted through the IR rewriting pipeline
- Use the explicit `_print_u32()` / `_print_i32()` / `_print_hex()` functions for numeric output

## Example programs

### hello.rs

Demonstrates `print!`/`println!`, numeric output, and `eprintln!`:

```
cd programs && ../build.sh hello.rs hello.bin
cargo run -- programs/hello.bin
```

Output:
```
Hello, World!
This is stderr (same UART on bare metal)
1 + 2 = 3
Counting: 1, 2, 3, 4, 5
-42 as signed: -42
0xDEADBEEF in hex: 0xdeadbeef
```

### fibonacci.rs

Reads N from memory address `0x1000`, computes fib(N), prints the result, and writes it to `0x1004`:

```
cd programs && ../build.sh fibonacci.rs fibonacci.bin
cargo run -- programs/fibonacci.bin --set 0x1000=10
```

Output:
```
fib(10) = 55
```

## Tests

```
cargo test
```

12 tests covering memory operations (16-bit cells, 32-bit words, sub-cell byte access), register file behavior, instruction decoding, CPU execution sequences, and UART output.

## Project structure

```
src/
  main.rs       — CLI entry point, demo program
  cpu.rs        — Fetch/decode/execute loop, UART MMIO intercept
  decode.rs     — RV32E instruction decoder (all R/I/S/B/U/J formats)
  memory.rs     — 16-bit cell memory with 8/16/32-bit access
  registers.rs  — 16-register file (x0 hardwired to zero)
  loader.rs     — ELF and flat binary loader
programs/
  rv32e_rt.rs   — Runtime library (startup, print macros, panic handler)
  hello.rs      — Hello world demo
  fibonacci.rs  — Fibonacci with UART output
build.sh        — Rust-to-RV32E cross-compilation script
linker.ld       — Linker script (128KB RAM at address 0)
```
