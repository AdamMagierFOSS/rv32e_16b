#!/bin/bash
set -euo pipefail

# RV32E-16B build script.
# Compiles C programs for the 16-bit cell-addressed RV32E architecture.
#
# Usage: ./build.sh <input.c> [output.elf]
#
# Set LLVM_BIN to point to the modified LLVM toolchain with ilp32e16 support.
# If unset, tools are looked up from PATH.

CLANG="${LLVM_BIN:+${LLVM_BIN}/}clang"
LLD="${LLVM_BIN:+${LLVM_BIN}/}ld.lld"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

INPUT="${1:?Usage: build.sh <input.c> [output.elf]}"
OUTPUT="${2:-${INPUT%.c}.elf}"
BASENAME="${INPUT%.c}"

COMMON_FLAGS="--target=riscv32 -march=rv32e -mabi=ilp32e16 -ffreestanding -nostdlib -O2 -mno-relax"

echo "=== Compile ==="
"${CLANG}" ${COMMON_FLAGS} -c -o "${BASENAME}.o" "${INPUT}"

# Compile startup code if not already built
CRT0="${SCRIPT_DIR}/programs/crt0.o"
if [ ! -f "${CRT0}" ] || [ "${SCRIPT_DIR}/programs/crt0.S" -nt "${CRT0}" ]; then
    "${CLANG}" ${COMMON_FLAGS} -c -o "${CRT0}" "${SCRIPT_DIR}/programs/crt0.S"
fi

echo "=== Link ==="
"${LLD}" \
    -T "${SCRIPT_DIR}/linker.ld" \
    -o "${OUTPUT}" \
    "${CRT0}" "${BASENAME}.o"

echo "=== Done: ${OUTPUT} ==="
