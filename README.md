# rv32e_16b

A RISC-V RV32E simulator and custom LLVM toolchain for a 16-bit cell-addressed architecture.

Every address in this machine points to a 16-bit cell (`CHAR_BIT = 16`). The compiler, assembler, linker, and simulator all understand this natively — no address translation hacks.

## Architecture

- **Registers:** 16 × 32-bit (x0 hardwired to zero), per the RV32E spec
- **Memory:** Flat array of 16-bit cells, 131072 cells (128K) by default
- **PC:** Cell-addressed. Each 32-bit instruction occupies 2 cells, so PC increments by 2
- **Instruction encoding:** Standard RV32E 32-bit encodings. Branch/jump immediates are in cell units
- **Load/store:** `LH`/`SH` load/store one cell (16 bits). `LW`/`SW` load/store two cells (32 bits). `LB`/`SB` are aliased to cell operations (no sub-cell access)
- **UART:** Memory-mapped at cell address `0x10000000`. Stores write the full 16-bit cell; the simulator extracts the low 8 bits as a printable character
- **Types:** `sizeof(char) = 1` (one cell = 16 bits), `sizeof(short) = 1`, `sizeof(int) = 2` (two cells = 32 bits), `sizeof(void*) = 2`

See `RV32E-16B-ISA.md` for the full ISA specification.

## Building the simulator

```
cargo build
```

## Running

**Built-in demo** (sums 1 through 5):
```
cargo run
```

**Load an ELF:**
```
cargo run -- program.elf
```

**Pre-load memory values** (for passing input):
```
cargo run -- fibonacci.elf --set 0x1000=10
```

The simulator accepts both ELF32 files (auto-detected) and flat binaries.

## Compiling C programs

Programs are compiled with a modified LLVM/clang toolchain that supports the `ilp32e16` ABI. A separate `crt0.S` provides the startup stub (sets stack pointer, calls `main`, halts with `ebreak`).

```
cd programs
../build.sh hello.c hello.elf
cargo run -- hello.elf      # from project root
```

The `build.sh` script runs:
1. `clang --target=riscv32 -march=rv32e -mabi=ilp32e16` — compiles C to RV32E with 16-bit chars
2. `clang` — assembles `crt0.S` (startup stub)
3. `ld.lld` — links with cell-addressed layout (auto-detected from RISC-V ELF attributes)

### Prerequisites

You need a modified LLVM toolchain with these patches:
- `DataLayout` `B16` specifier for configurable byte width throughout LLVM IR
- `ilp32e16` ABI in clang: `CHAR_BIT=16`, `char`/`short` = 16-bit `i16` in IR, `BoolWidth=16`
- BPAU-aware assembler: fixup values converted to cell units, `Tag_bytes_per_addr_unit` ELF attribute
- BPAU-aware ELF writer: symbol values and sizes in cell units for allocatable sections
- BPAU-aware lld: auto-detects BPAU from ELF attributes, cell-addressed section layout, correct `p_filesz`/`p_memsz` in program headers, RISC-V relocation `P` computation with `/bpau`

Build it with:
```
cmake -G Ninja -DLLVM_TARGETS_TO_BUILD=RISCV -DLLVM_ENABLE_PROJECTS="clang;lld" \
      -DLLVM_USE_LINKER=mold -DCMAKE_BUILD_TYPE=Release ../llvm
ninja clang lld
```

Set `LLVM_BIN` to point to the built tools:
```
LLVM_BIN=/path/to/build/bin ./build.sh hello.c
```

## Standard I/O header

`programs/stdio.h` provides a mock standard I/O library:

- `putchar(c)`, `puts(s)`, `print_str(s)` — character and string output
- `print_unsigned(n)`, `print_int(n)`, `print_hex(n)` — number formatting (binary long division, no hardware divide)
- `memset`, `memcpy`, `strlen`, `strcmp` — basic memory/string operations

No varargs support (`va_arg` generates unsupported 2-byte data relocations on this target). Use the explicit `print_*` functions instead of `printf`.

```c
#include "stdio.h"

void main(void) {
    puts("Hello from RV32E-16B!");
    print_str("answer = ");
    print_int(42);
    putchar('\n');
}
```

Programs define `main()` (not `_start`). The `crt0.S` startup stub sets up the stack and calls `main`, then halts with `ebreak`.

## Example programs

### hello.c

```
cd programs && ../build.sh hello.c hello.elf
cargo run -- programs/hello.elf
```

Output:
```
Hello from RV32E-16B!
CHAR_BIT = 16
sizeof(char)  = 1
sizeof(short) = 1
sizeof(int)   = 2
sizeof(void*) = 2
hex: 0xdeadbeef
negative: -42
strlen("Hi") = 2
```

### fibonacci.c

Reads N from cell address `0x1000`, computes fib(N), prints the result:

```
cd programs && ../build.sh fibonacci.c fibonacci.elf
cargo run -- programs/fibonacci.elf --set 0x1000=10
```

Output:
```
fib(10) = 55
```

## Tests

```
cargo test
```

31 tests covering:
- **I-type ALU:** `addi`, `slti`/`sltiu`, `xori`/`ori`/`andi`, `slli`/`srli`/`srai`
- **R-type ALU:** `add`/`sub`, `slt`/`sltu`, `xor`/`or`/`and`
- **Upper immediate:** `lui`, `auipc`
- **Branches:** `beq` taken/not-taken, `bne`, `blt`/`bge`, backward branch loop
- **Jumps:** `jal` forward, `jal`+`jalr` call/return round-trip
- **Load/store:** `lw`/`sw` word, `lh`/`lhu` sign/zero extension, `lb`/`sb` cell aliasing, offset addressing
- **UART:** 16-bit cell capture
- **System:** `ecall`, `ebreak`
- **Architecture:** PC increments by 2, x0 hardwired to 0, binary loading packs bytes into cells
- **Memory:** 16-bit cell read/write, 32-bit word spanning two cells

## Known limitations

- `-mno-relax` required on clang (linker relaxation disabled for BPAU > 1)
- No varargs / `printf` (va_arg generates unsupported relocations)
- Inline asm string literals crash clang (`getCharByteWidth() == 1` assertion) — use separate `.S` files
- Section name `__attribute__` with string literals may garble names — use linker script ordering
- DWARF debug info untested
- Freestanding only (no standard C library beyond `stdio.h` mock)

## Project structure

```
src/
  main.rs       — CLI entry point, built-in demo
  cpu.rs        — Fetch/decode/execute loop, UART MMIO, 31 ISA tests
  decode.rs     — RV32E instruction decoder (R/I/S/B/U/J formats)
  memory.rs     — 16-bit cell memory (load/store 16 and 32)
  registers.rs  — 16-register file (x0 = 0)
  loader.rs     — ELF (cell-addressed) and flat binary loader
programs/
  crt0.S        — Startup stub (stack init, call main, ebreak)
  stdio.h       — Mock standard I/O (putchar, puts, print_int, memcpy, strlen, ...)
  rt.h          — Minimal runtime (subset of stdio.h, kept for compatibility)
  hello.c       — Hello world with type size info and hex/signed output
  fibonacci.c   — Fibonacci with UART output
build.sh        — C-to-RV32E-16B compilation and linking script
linker.ld       — Linker script (128K cells at address 0)
RV32E-16B-ISA.md — Full ISA specification
```
