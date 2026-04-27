# RV32E-16B ISA Specification

## A RISC-V RV32E Derivative with 16-Bit Addressable Memory

### 1. Overview

RV32E-16B is a modified RV32E instruction set architecture where the fundamental addressable unit of memory is a **16-bit cell** rather than an 8-bit byte. Every address in the system refers to a 16-bit quantity. This has cascading implications across the entire ISA: instruction encoding, load/store semantics, branch offsets, the program counter, alignment, and the toolchain.

This document specifies every departure from standard RV32E, and explains the rationale for each decision.

### 2. Design Principles

1. **The cell is the byte.** Address N refers to a 16-bit cell. There is no sub-cell addressing. `CHAR_BIT = 16`.
2. **Stay as close to RV32E encoding as possible.** Instruction bit-fields are unchanged from standard RISC-V. What changes is the *interpretation* of address units.
3. **A 32-bit instruction occupies 2 consecutive addresses** (not 4). The PC increments by 2 per instruction.
4. **A 32-bit word occupies 2 consecutive addresses** (not 4). A "halfword" load/store accesses exactly 1 address.
5. **8-bit byte operations are removed.** LB, LBU, SB are either illegal or repurposed.

---

### 3. Registers

Identical to standard RV32E:

| Register | ABI Name | Description |
|----------|----------|-------------|
| x0 | zero | Hardwired to 0 |
| x1 | ra | Return address |
| x2 | sp | Stack pointer |
| x3 | gp | Global pointer |
| x4 | tp | Thread pointer |
| x5–x7 | t0–t2 | Temporaries |
| x8 | s0/fp | Saved register / frame pointer |
| x9 | s1 | Saved register |
| x10–x11 | a0–a1 | Function arguments / return values |
| x12–x15 | a2–a5 | Function arguments |

All registers are 32 bits wide. x0 reads as zero; writes are discarded.

---

### 4. Memory Model

#### 4.1 Addressing

- The address space is **cell-addressed**. Each address refers to one 16-bit cell.
- A 32-bit address can reference up to 2^32 cells (8 GB of 16-bit storage, or 16 GB of equivalent 8-bit data).
- Address 0 is the first cell. Address 1 is the next cell. There are no "half-addresses" or sub-cell offsets.

#### 4.2 Data Sizes

| Name | Size | Addresses Occupied | Standard RV32 Equivalent |
|------|------|--------------------|--------------------------|
| Cell (native unit) | 16 bits | 1 | Halfword |
| Word | 32 bits | 2 | Word |
| Doubleword | 64 bits | 4 | Doubleword |

There is no 8-bit data type at the architectural level. The smallest loadable/storable unit is 16 bits.

#### 4.3 Endianness

Little-endian at the cell level. For a 32-bit word stored at address A:

- Address A contains bits [15:0] (low cell)
- Address A+1 contains bits [31:16] (high cell)

#### 4.4 Alignment

- Cell (16-bit) access: any address (always naturally aligned by definition).
- Word (32-bit) access: should be at even addresses (A mod 2 == 0) for natural alignment. The implementation may choose to support unaligned word access or raise an exception.

---

### 5. Instruction Encoding

#### 5.1 Instruction Size and Fetch

All base instructions are 32 bits wide, occupying **2 consecutive cell addresses**. The instruction at address A is formed by:

    inst[15:0]  = memory[A]
    inst[31:16] = memory[A+1]

The PC holds a cell address and increments by **2** per instruction (not 4 as in standard RV32).

#### 5.2 Encoding Formats

The bit-level encoding of each instruction format (R, I, S, B, U, J) is **identical** to standard RISC-V. The 32-bit instruction word has the same field positions for opcode, rd, rs1, rs2, funct3, funct7, and immediates.

What changes is how the decoded immediates are *interpreted* — specifically for instructions that produce or consume addresses.

#### 5.3 Register Index Width

Standard RV32E uses 4 bits for register specifiers (x0–x15). This is unchanged. The bit positions are:

- rd: bits [10:7] (4 bits, not the usual [11:7] 5-bit field)
- rs1: bits [18:15] (4 bits, not [19:15])
- rs2: bits [23:20] (4 bits, not [24:20])

**Important note on encoding compatibility:** Standard RISC-V RV32E still uses the full 5-bit register field in the encoding; the high bit is simply required to be 0 (registers 0–15 only). A strict RV32E-16B implementation should trap if bit 11 (for rd), bit 19 (for rs1), or bit 24 (for rs2) is set, as those would reference non-existent registers x16–x31.

---

### 6. Instruction Reference

#### 6.1 Unchanged Instructions (Operate on Registers Only)

These instructions are semantically identical to standard RV32E. They operate on 32-bit register values and do not touch memory or addresses:

**R-type ALU:**

| Instruction | Operation |
|------------|-----------|
| ADD rd, rs1, rs2 | rd = rs1 + rs2 |
| SUB rd, rs1, rs2 | rd = rs1 - rs2 |
| SLL rd, rs1, rs2 | rd = rs1 << (rs2 & 0x1F) |
| SLT rd, rs1, rs2 | rd = (rs1 <s rs2) ? 1 : 0 |
| SLTU rd, rs1, rs2 | rd = (rs1 <u rs2) ? 1 : 0 |
| XOR rd, rs1, rs2 | rd = rs1 ^ rs2 |
| SRL rd, rs1, rs2 | rd = rs1 >>u (rs2 & 0x1F) |
| SRA rd, rs1, rs2 | rd = rs1 >>s (rs2 & 0x1F) |
| OR rd, rs1, rs2 | rd = rs1 \| rs2 |
| AND rd, rs1, rs2 | rd = rs1 & rs2 |

**I-type ALU:**

| Instruction | Operation |
|------------|-----------|
| ADDI rd, rs1, imm | rd = rs1 + sext(imm) |
| SLTI rd, rs1, imm | rd = (rs1 <s sext(imm)) ? 1 : 0 |
| SLTIU rd, rs1, imm | rd = (rs1 <u sext(imm)) ? 1 : 0 |
| XORI rd, rs1, imm | rd = rs1 ^ sext(imm) |
| ORI rd, rs1, imm | rd = rs1 \| sext(imm) |
| ANDI rd, rs1, imm | rd = rs1 & sext(imm) |
| SLLI rd, rs1, shamt | rd = rs1 << shamt |
| SRLI rd, rs1, shamt | rd = rs1 >>u shamt |
| SRAI rd, rs1, shamt | rd = rs1 >>s shamt |

**System:**

| Instruction | Operation |
|------------|-----------|
| ECALL | Environment call |
| EBREAK | Debugger breakpoint |

All of the above are bit-for-bit identical to standard RV32E.

---

#### 6.2 Upper Immediate Instructions

##### LUI (Load Upper Immediate)

**Encoding:** U-type, opcode 0110111

    rd = imm[31:12] << 12    (standard: unchanged)

LUI loads a 20-bit immediate into the upper 20 bits of rd, zeroing the lower 12. This is a pure register operation — no address semantics. **Unchanged.**

However, note the implications: in standard RISC-V, LUI + ADDI is used to construct 32-bit byte addresses. In RV32E-16B, addresses are cell addresses. Since the address space is smaller (same 32-bit space covers more data per address), LUI + ADDI still works for constructing any 32-bit cell address. The interpretation of the resulting value changes, not the instruction.

##### AUIPC (Add Upper Immediate to PC)

**Encoding:** U-type, opcode 0010111

    rd = PC + (imm[31:12] << 12)

**Changed semantics.** The PC is now a cell address. AUIPC adds the upper immediate to the current cell-addressed PC. The result is a cell address. This is used by the linker for PC-relative addressing (combined with ADDI or JALR).

The *encoding* is unchanged. The *meaning* of the result shifts because PC is in cell units.

**Toolchain impact:** The linker must generate AUIPC immediates in cell units. If a symbol is 0x2000 cells away, the immediate reflects that directly. In standard RV32, the symbol would be 0x4000 bytes away and the immediate would encode 0x4000. This is a linker/relocation change, not an ISA encoding change.

---

#### 6.3 Jump Instructions

##### JAL (Jump and Link)

**Encoding:** J-type, opcode 1101111

Standard RV32:

    rd = PC + 4          (return address, byte-addressed)
    PC = PC + sext(imm)  (imm is in byte units, bit[0] implicitly 0)

The J-type immediate is a 21-bit signed value with bit[0] always zero, giving a range of ±1 MiB in byte units (±512K instructions).

**RV32E-16B semantics:**

    rd = PC + 2          (return address, cell-addressed; 2 cells = 1 instruction)
    PC = PC + sext(imm)  (imm is in cell units)

**Critical encoding question: What is the implicit LSB?**

In standard RISC-V, J-type and B-type immediates have bit[0] implicitly zero because the minimum instruction alignment is 2 bytes (with the C extension) or 4 bytes (without). The encoding saves one bit by not storing it.

In RV32E-16B, every instruction is 2 cells. If we keep bit[0] implicitly zero, the immediate is always a multiple of 2 cells, meaning we can only jump to even cell addresses — which is exactly where instructions start. This is **correct and convenient**: the implicit zero LSB now means "instruction-aligned" rather than "2-byte-aligned."

**Decision: Keep the standard J-type encoding. The implicit zero bit[0] means multiples of 2 cells = instruction-aligned.**

This gives a jump range of ±2^20 cells = ±1M cells from the current PC. Since each cell is 16 bits, this is a ±2 MiB range in terms of data, compared to ±1 MiB in standard RV32. The effective instruction reach doubles.

##### JALR (Jump and Link Register)

**Encoding:** I-type, opcode 1100111

Standard RV32:

    rd = PC + 4
    PC = (rs1 + sext(imm)) & ~1    (clear bit[0] for alignment)

**RV32E-16B semantics:**

    rd = PC + 2
    PC = (rs1 + sext(imm)) & ~1    (clear bit[0] for instruction alignment)

The `& ~1` ensures the target is at an even cell address (instruction-aligned). rs1 contains a cell address. The immediate is in cell units.

**Unchanged encoding. Changed address interpretation.**

---

#### 6.4 Branch Instructions

##### B-type: BEQ, BNE, BLT, BGE, BLTU, BGEU

**Encoding:** B-type, opcode 1100011

Standard RV32:

    if (condition) PC = PC + sext(imm)    (imm in byte units, bit[0] implicitly 0)

The B-type immediate is a 13-bit signed value with bit[0] always zero, giving a range of ±4 KiB in byte units.

**RV32E-16B semantics:**

    if (condition) PC = PC + sext(imm)    (imm in cell units, bit[0] implicitly 0)

Same analysis as JAL: the implicit zero bit[0] means the branch target is always at an even cell address = instruction-aligned. The branch range becomes ±2^12 cells = ±4K cells from the current PC.

**Unchanged encoding. Changed unit interpretation.**

| Instruction | Condition |
|------------|-----------|
| BEQ rs1, rs2, offset | rs1 == rs2 |
| BNE rs1, rs2, offset | rs1 != rs2 |
| BLT rs1, rs2, offset | rs1 <s rs2 |
| BGE rs1, rs2, offset | rs1 >=s rs2 |
| BLTU rs1, rs2, offset | rs1 <u rs2 |
| BGEU rs1, rs2, offset | rs1 >=u rs2 |

---

#### 6.5 Load Instructions

This is the most significantly changed area.

##### Standard RV32E loads:

| funct3 | Instruction | Width | Sign Extension |
|--------|------------|-------|----------------|
| 000 | LB | 8-bit | Sign-extend to 32 |
| 001 | LH | 16-bit | Sign-extend to 32 |
| 010 | LW | 32-bit | None |
| 100 | LBU | 8-bit | Zero-extend to 32 |
| 101 | LHU | 16-bit | Zero-extend to 32 |

##### RV32E-16B loads:

| funct3 | Instruction | Width | Operation | Notes |
|--------|------------|-------|-----------|-------|
| 000 | **LC** | 16-bit (1 cell) | rd = sext(mem[rs1 + imm]) | Replaces LB. Loads one cell, sign-extends to 32 bits. |
| 001 | **LCU** | 16-bit (1 cell) | rd = zext(mem[rs1 + imm]) | Replaces LH. Loads one cell, zero-extends to 32 bits. |
| 010 | **LW** | 32-bit (2 cells) | rd = mem[rs1 + imm : rs1 + imm + 1] | Unchanged semantics: loads 2 consecutive cells. |
| 100 | *Reserved* | — | — | Was LBU. No 8-bit type exists. |
| 101 | *Reserved* | — | — | Was LHU. Redundant with LCU. |

**Naming rationale:** "LC" = Load Cell. "LCU" = Load Cell Unsigned. These replace LB/LBU and LH/LHU respectively. Since the cell is the native unit, we need exactly two load widths: cell (16-bit, the minimum) and word (32-bit).

**Alternative considered:** Keep funct3 encoding identical (000=cell signed, 100=cell unsigned, 010=word) and declare funct3 001/101 as aliases. This maximizes backward compatibility with the instruction encoding — assembler-generated LB and LH opcodes both work, just with cell semantics. The decision comes down to whether we want a clean break (reserved/trap) or silent compatibility (aliases). **Recommendation: aliases.** This means any standard RV32E binary where LB/LH happen to access cell-aligned data will "just work." Funct3 000 and 001 behave identically (load cell, sign-extend); funct3 100 and 101 behave identically (load cell, zero-extend).

**Effective address calculation:**

    addr = rs1 + sext(imm)

Both rs1 and imm are in **cell units**. The result is a cell address used directly to index memory. No shifting or scaling.

**Sign vs zero extension for LC:**

A 16-bit cell loaded into a 32-bit register needs extension. LC (funct3=000) sign-extends: if bit[15] of the cell is 1, bits [31:16] of rd are set to 1. LCU (funct3=001 or 100) zero-extends: bits [31:16] of rd are 0.

---

#### 6.6 Store Instructions

##### Standard RV32E stores:

| funct3 | Instruction | Width |
|--------|------------|-------|
| 000 | SB | 8-bit |
| 001 | SH | 16-bit |
| 010 | SW | 32-bit |

##### RV32E-16B stores:

| funct3 | Instruction | Width | Operation | Notes |
|--------|------------|-------|-----------|-------|
| 000 | **SC** | 16-bit (1 cell) | mem[rs1 + imm] = rs2[15:0] | Replaces SB. Stores low 16 bits. |
| 001 | *SC alias* | 16-bit (1 cell) | mem[rs1 + imm] = rs2[15:0] | Was SH. Identical to SC. |
| 010 | **SW** | 32-bit (2 cells) | mem[rs1+imm : rs1+imm+1] = rs2 | Unchanged. |

**Effective address:**

    addr = rs1 + sext(imm)

Both in cell units. SC stores the low 16 bits of rs2 to a single cell. SW stores all 32 bits of rs2 to two consecutive cells.

---

### 7. Program Counter

| Property | Standard RV32E | RV32E-16B |
|----------|---------------|-----------|
| Unit | Byte address | Cell address |
| Increment per instruction | +4 | +2 |
| JALR alignment mask | & ~1 (halfword align) | & ~1 (instruction align) |
| Reset value | Implementation-defined | Implementation-defined |

The PC holds a cell address. Each 32-bit instruction occupies 2 cells, so the PC advances by 2 for sequential execution.

The `& ~1` mask in JALR ensures the target is at an even cell address. In standard RISC-V this ensures halfword alignment (for possible C-extension instructions); here it ensures instruction alignment (32-bit instructions must start at even cell addresses).

---

### 8. The C Extension Question

The standard C (compressed) extension provides 16-bit instructions that are exactly 1 cell in RV32E-16B. This is architecturally natural: a compressed instruction occupies 1 address, an uncompressed instruction occupies 2 addresses.

**Detection mechanism:** Standard RISC-V detects compressed instructions when bits [1:0] of the 16-bit parcel ≠ 0b11. In cell-addressed memory, the instruction fetch reads one cell (16 bits) and checks bits [1:0]:

- If `cell[1:0] != 11`: this is a 16-bit compressed instruction occupying 1 cell. PC += 1.
- If `cell[1:0] == 11`: this is a 32-bit instruction. Fetch the next cell. PC += 2.

This is actually **cleaner** than standard RISC-V, where variable-length instruction detection works on byte-level parcels. Here each cell is a natural fetch unit.

**Decision: The C extension is compatible with RV32E-16B with PC += 1 for compressed instructions.** Not included in the base spec but architecturally sound for a future extension.

---

### 9. Immediate Scaling Summary

This is the central table for understanding how RV32E-16B differs from standard RV32E:

| Immediate Type | Encoding (bits) | Implicit LSB | Standard RV32 Unit | RV32E-16B Unit | Effective Range |
|---------------|-----------------|-------------|-------------------|----------------|-----------------|
| I-type (ALU) | 12-bit signed | None | Integer | Integer | ±2048 |
| I-type (Load) | 12-bit signed | None | Byte offset | Cell offset | ±2048 cells |
| S-type (Store) | 12-bit signed | None | Byte offset | Cell offset | ±2048 cells |
| B-type (Branch) | 13-bit signed | bit[0] = 0 | Byte offset | Cell offset | ±4096 cells |
| U-type (LUI) | 20-bit | None (upper) | Integer | Integer | Upper 20 bits |
| U-type (AUIPC) | 20-bit | None (upper) | Byte address | Cell address | Upper 20 bits + PC |
| J-type (JAL) | 21-bit signed | bit[0] = 0 | Byte offset | Cell offset | ±1M cells |

**Key insight:** The encodings are identical. Only the unit interpretation changes. The assembler/linker must generate immediates in cell units rather than byte units.

---

### 10. Impact on Standard Type Sizes

#### 10.1 C Language Types

| Type | Standard RV32E | RV32E-16B | sizeof() |
|------|---------------|-----------|----------|
| char | 8-bit | **16-bit** | 1 (one cell) |
| short | 16-bit | **16-bit** | 1 (one cell) |
| int | 32-bit | 32-bit | 2 (two cells) |
| long | 32-bit | 32-bit | 2 (two cells) |
| void* | 32-bit | 32-bit | 2 (two cells) |
| CHAR_BIT | 8 | **16** |  |

`sizeof` returns counts in units of `char`, which is one cell (16 bits). So `sizeof(char) == 1`, `sizeof(int) == 2`, `sizeof(long) == 2`. These are the same numeric values as standard RV32E, but each unit is 16 bits instead of 8.

#### 10.2 Implications for String Handling

Each character in a string occupies one cell (16 bits). ASCII characters use only the low 8 bits of each cell, wasting the upper 8. This is the same tradeoff as the TI TMS320C2000 DSP family, which also has `CHAR_BIT=16`.

Options for string storage:
1. **One char per cell (natural).** Simple, fast, compatible with the type system. Wastes 50% of storage for ASCII text.
2. **Packed strings (library convention).** Pack two ASCII characters per cell. Requires explicit pack/unpack code. Breaks `char*` pointer arithmetic.

**Recommendation: One char per cell.** This is the only approach consistent with `sizeof(char) == 1` and standard pointer arithmetic. Packed strings can be a library optimization but must not be the default.

#### 10.3 Implications for Pointer Arithmetic

In standard C, `ptr + 1` advances by `sizeof(*ptr)` bytes. In RV32E-16B, `ptr + 1` advances by `sizeof(*ptr)` cells. Since the cell is the addressable unit, this works correctly:

- `char *p; p++;` → advance 1 cell (1 address)
- `int *p; p++;` → advance 2 cells (2 addresses)

No change to the compiler's pointer arithmetic logic — `sizeof` already accounts for the unit size.

---

### 11. Impact on Toolchain

#### 11.1 Assembler

Must interpret all address-related immediates as cell offsets:

- `.word` emits a 32-bit value across 2 cells.
- `.half` emits a 16-bit value in 1 cell.
- `.byte` **must be redefined or removed.** Options:
  - Remove `.byte` directive entirely (no 8-bit type).
  - Redefine `.byte` as `.cell` (alias for `.half`): emits a 16-bit value.
  - Add `.cell` as a new directive.
- Labels produce cell addresses.
- Branch/jump offset calculation: `(target_label - current_label)` in cells, not bytes.

#### 11.2 Linker

Relocation types that currently operate in byte units must operate in cell units:

| Relocation | Standard | RV32E-16B |
|-----------|----------|-----------|
| R_RISCV_BRANCH | Byte offset | Cell offset |
| R_RISCV_JAL | Byte offset | Cell offset |
| R_RISCV_HI20 | Byte address upper 20 | Cell address upper 20 |
| R_RISCV_LO12_I | Byte address lower 12 | Cell address lower 12 |
| R_RISCV_LO12_S | Byte address lower 12 | Cell address lower 12 |
| R_RISCV_PCREL_HI20 | Byte-relative | Cell-relative |
| R_RISCV_PCREL_LO12_I | Byte-relative | Cell-relative |

The linker script's ORIGIN and LENGTH values are in cell units.

#### 11.3 Compiler (LLVM/GCC)

This is the deepest change. The compiler must be taught:

1. **Data layout:** `CHAR_BIT=16`. The smallest addressable type is 16 bits. `i8` in LLVM IR must either be promoted to `i16` everywhere or mapped to a 16-bit cell with the upper 8 bits zeroed.

2. **Address arithmetic:** All pointer offsets computed in cell units. `getelementptr` for a `char*` advances by 1 address, not by 1/2 address.

3. **Stack frame layout:** The stack pointer advances in cell units. Minimum stack slot is 1 cell (16 bits). Word-sized slots are 2 cells.

4. **String literals:** Each character occupies one cell. The compiler must emit `.half` (or equivalent) per character instead of `.byte`.

5. **Load/store selection:** Where standard RV32 would emit LB/SB for `char` access, the compiler must emit LC/SC (or LH/SH in the aliased model).

6. **ABI:** Function arguments that were `char` (8-bit) are now `cell` (16-bit). No sub-cell types exist in the function calling convention.

**This is a custom LLVM target.** It cannot be achieved by sed-rewriting IR. Required changes:
- New `RV32E16B` target triple (e.g., `riscv32e16b-unknown-none-elf`)
- Custom `DataLayout` string with 16-bit minimum addressable unit
- Modified `RISCVISelLowering` to emit cell loads/stores instead of byte loads/stores
- Modified `RISCVAsmPrinter` for label/offset calculation in cell units
- Custom ABI lowering for the 16-bit char type

---

### 12. Comparison Table: Standard RV32E vs RV32E-16B

| Feature | Standard RV32E | RV32E-16B |
|---------|---------------|-----------|
| Addressable unit | 8-bit byte | 16-bit cell |
| CHAR_BIT | 8 | 16 |
| sizeof(char) | 1 | 1 |
| sizeof(int) | 4 | 2 |
| PC increment | +4 per instruction | +2 per instruction |
| Instruction size in addresses | 4 | 2 |
| Word size in addresses | 4 | 2 |
| LB/SB semantics | 8-bit memory access | Cell access (alias of LC/SC) or illegal |
| LH/SH semantics | 16-bit memory access | Cell access (alias of LC/SC) |
| LW/SW semantics | 32-bit memory access | 32-bit access (2 cells) |
| Branch range | ±4 KiB (bytes) | ±4K cells (±8 KiB equivalent) |
| JAL range | ±1 MiB (bytes) | ±1M cells (±2 MiB equivalent) |
| JALR mask | & ~1 (halfword align) | & ~1 (instruction align) |
| Memory capacity (32-bit addr) | 4 GiB | 4G cells = 8 GiB |
| C extension fit | 16-bit = 2 bytes | 16-bit = 1 cell (natural) |

---

### 13. Instruction Encoding Summary (All 32-bit Base Instructions)

All opcodes, funct3, and funct7 values are identical to standard RV32E. The instruction set is **encoding-compatible**. Differences are purely semantic (address unit interpretation).

```
31       25 24  20 19  15 14  12 11   7 6     0
|  funct7  | rs2  | rs1  |funct3|  rd  | opcode |  R-type
|     imm[11:0]   | rs1  |funct3|  rd  | opcode |  I-type
| imm[11:5]| rs2  | rs1  |funct3|imm[4:0]|opcode|  S-type
|imm[12|10:5]|rs2 | rs1  |funct3|imm[4:1|11]|opc|  B-type
|          imm[31:12]           |  rd  | opcode |  U-type
|imm[20|10:1|11|19:12]         |  rd  | opcode |  J-type
```

| Opcode | Type | Instructions |
|--------|------|-------------|
| 0110111 | U | LUI |
| 0010111 | U | AUIPC |
| 1101111 | J | JAL |
| 1100111 | I | JALR |
| 1100011 | B | BEQ, BNE, BLT, BGE, BLTU, BGEU |
| 0000011 | I | LC, LCU, LW (and aliases) |
| 0100011 | S | SC, SW (and aliases) |
| 0010011 | I | ADDI, SLTI, SLTIU, XORI, ORI, ANDI, SLLI, SRLI, SRAI |
| 0110011 | R | ADD, SUB, SLL, SLT, SLTU, XOR, SRL, SRA, OR, AND |
| 1110011 | I | ECALL, EBREAK |

---

### 14. Open Questions and Future Work

#### 14.1 The `fence` and Atomic Instructions

Not specified in this base ISA. If added:
- FENCE operates on cell-addressed regions.
- Atomic operations (A extension) operate on cell or word-sized quantities. LR.W/SC.W would lock 2-cell-aligned word addresses.

#### 14.2 The M Extension (Multiply/Divide)

MUL, MULH, DIV, REM and variants are pure register operations. **No changes needed.** Can be adopted directly from standard RV32M.

#### 14.3 Floating Point (F/D Extensions)

FLW/FSW (float load/store) would store 32-bit floats in 2 cells. FLD/FSD (double load/store) would use 4 cells. Otherwise no changes to the float computation instructions. However, RV32E typically does not include F/D.

#### 14.4 Interaction with 8-Bit I/O Devices

External devices that operate on 8-bit bytes (UART, SPI, I2C) need a bridge layer. The MMIO region should define per-device conventions:
- For a UART: a store-cell instruction writes the low 8 bits of a cell to the TX register. A load-cell instruction returns the received byte zero-extended to 16 bits in the low cell.
- This matches how 16-bit DSP architectures (TMS320C2000) handle 8-bit peripherals.

#### 14.5 Custom LLVM Target

Building a full custom LLVM target for RV32E-16B is a substantial project. A phased approach:

1. **Phase 1 (current):** Retarget standard RV32E LLVM IR with address translation in the simulator. This works for programs that don't depend on 8-bit byte semantics.
2. **Phase 2:** Patch LLVM's RISC-V backend to emit cell-addressed code: modify `RISCVSubtarget` to report `CHAR_BIT=16`, adjust `SelectionDAGISel` to lower `i8` to `i16`, change address computation in `RISCVISelLowering`.
3. **Phase 3:** Upstream or fork a proper target triple and ABI specification.

#### 14.6 Binary Compatibility

RV32E-16B binaries are **not** binary-compatible with standard RV32E. While the instruction encoding is identical at the bit level, the semantic interpretation of addresses, offsets, and memory layout differs. An RV32E-16B binary loaded on a standard RV32E machine (or vice versa) would compute wrong addresses and access wrong memory locations.

An RV32E-16B ELF should use a distinct `e_machine` or `e_flags` value to prevent accidental execution on a standard RISC-V machine.
