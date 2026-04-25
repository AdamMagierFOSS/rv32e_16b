#!/bin/bash
set -euo pipefail

LLC="${LLVM_BIN:+${LLVM_BIN}/}llc"
LLD="${LLVM_BIN:+${LLVM_BIN}/}ld.lld"
OBJCOPY="${LLVM_BIN:+${LLVM_BIN}/}llvm-objcopy"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

INPUT="${1:?Usage: build.sh <input.rs> [output.bin]}"
OUTPUT="${2:-${INPUT%.rs}.bin}"
BASENAME="${INPUT%.rs}"

echo "=== Step 1: Rust -> LLVM IR ==="
rustc --edition 2021 \
    --emit=llvm-ir \
    -C opt-level=2 \
    -C panic=abort \
    --crate-type=lib \
    -o "${BASENAME}.ll" \
    "${INPUT}"

echo "=== Step 2: Retarget IR to riscv32 ==="
sed -i \
    -e 's/^target datalayout = .*/target datalayout = "e-m:e-p:32:32-i64:64-n32-S128"/' \
    -e 's/^target triple = .*/target triple = "riscv32-unknown-none-elf"/' \
    -e 's/"target-cpu"="[^"]*"/"target-cpu"="generic-rv32"/g' \
    -e '/"probe-stack"="inline-asm"/d' \
    "${BASENAME}.ll"
# Remove target-features that reference x86
sed -i 's/"target-features"="[^"]*"/"target-features"="+e"/g' "${BASENAME}.ll"
# Strip x86 artifacts from the IR
sed -i \
    -e '/^module asm "\.intel_syntax"/d' \
    -e '/^module asm "\.att_syntax"/d' \
    -e 's/ inteldialect//g' \
    -e 's/~{dirflag},~{fpsr},~{flags},//g' \
    "${BASENAME}.ll"

echo "=== Step 3: LLVM IR -> object file ==="
"${LLC}" \
    -march=riscv32 \
    -mattr=+e \
    -mcpu=generic-rv32 \
    -filetype=obj \
    -o "${BASENAME}.o" \
    "${BASENAME}.ll"

echo "=== Step 4: Link -> ELF ==="
"${LLD}" \
    -T "${SCRIPT_DIR}/linker.ld" \
    -o "${BASENAME}.elf" \
    "${BASENAME}.o"

echo "=== Step 5: ELF -> flat binary ==="
"${OBJCOPY}" -O binary "${BASENAME}.elf" "${OUTPUT}"

SIZE=$(stat -c%s "${OUTPUT}")
echo "=== Done: ${OUTPUT} (${SIZE} bytes) ==="
