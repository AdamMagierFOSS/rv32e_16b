// Runtime support for RV32E 16-bit-byte simulator programs.
//
// Usage: include this at the top of your program:
//   include!("rv32e_rt.rs");
//
// Provides: print!, println!, eprint!, eprintln! (string literals only),
//           and _print_u32/_print_i32/_print_hex for numeric output.

use core::panic::PanicInfo;

core::arch::global_asm!(
    ".section .text._start",
    ".global _start",
    "_start:",
    "    lui sp, %hi(__stack_top)",
    "    addi sp, sp, %lo(__stack_top)",
    "    jal ra, main",
    "    ecall",
);

const _UART_ADDR: *mut u8 = 0x10000000 as *mut u8;

fn _putc(c: u8) {
    unsafe { core::ptr::write_volatile(_UART_ADDR, c) };
}

fn _print_str(s: &str) {
    for &b in s.as_bytes() {
        _putc(b);
    }
}

// Binary long division by 10, no hardware div needed.
fn _divmod10(n: u32) -> (u32, u32) {
    let mut q: u32 = 0;
    let mut r: u32 = 0;
    let mut i: i32 = 31;
    while i >= 0 {
        r = (r << 1) | ((n >> (i as u32)) & 1);
        if r >= 10 {
            r = r.wrapping_sub(10);
            q |= 1 << (i as u32);
        }
        i -= 1;
    }
    (q, r)
}

fn _print_u32_inner(n: u32) {
    if n >= 10 {
        let (q, r) = _divmod10(n);
        _print_u32_inner(q);
        _putc(b'0' + r as u8);
    } else {
        _putc(b'0' + n as u8);
    }
}

fn _print_u32(n: u32) {
    _print_u32_inner(n);
}

fn _print_i32(n: i32) {
    if n < 0 {
        _putc(b'-');
        _print_u32((!n as u32).wrapping_add(1));
    } else {
        _print_u32(n as u32);
    }
}

fn _print_hex(n: u32) {
    _print_str("0x");
    if n == 0 {
        _putc(b'0');
        return;
    }
    let mut started = false;
    let mut shift: i32 = 28;
    while shift >= 0 {
        let nibble = ((n >> shift) & 0xF) as u8;
        if nibble != 0 || started {
            _putc(if nibble < 10 { b'0' + nibble } else { b'a' + nibble - 10 });
            started = true;
        }
        shift -= 4;
    }
}

macro_rules! print {
    ($s:expr) => { _print_str($s) };
}

macro_rules! println {
    () => { _putc(b'\n') };
    ($s:expr) => { _print_str($s); _putc(b'\n') };
}

macro_rules! eprint {
    ($s:expr) => { _print_str($s) };
}

macro_rules! eprintln {
    () => { _putc(b'\n') };
    ($s:expr) => { _print_str($s); _putc(b'\n') };
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    _print_str("PANIC\n");
    unsafe { core::arch::asm!("ebreak", options(noreturn)) }
}
