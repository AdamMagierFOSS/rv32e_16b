#![no_std]
#![no_main]

include!("rv32e_rt.rs");

/// Read N from 0x1000, compute fib(N), print it, and write to 0x1004.
const INPUT_ADDR: *mut u32 = 0x1000 as *mut u32;
const OUTPUT_ADDR: *mut u32 = 0x1004 as *mut u32;

#[no_mangle]
pub extern "C" fn main() {
    let n = unsafe { core::ptr::read_volatile(INPUT_ADDR) };
    let result = fibonacci(n);
    unsafe { core::ptr::write_volatile(OUTPUT_ADDR, result) };

    print!("fib(");
    _print_u32(n);
    print!(") = ");
    _print_u32(result);
    println!();
}

#[inline(never)]
fn fibonacci(n: u32) -> u32 {
    if n <= 1 {
        return n;
    }
    let mut a: u32 = 0;
    let mut b: u32 = 1;
    let mut i: u32 = 2;
    while i <= n {
        let tmp = a.wrapping_add(b);
        a = b;
        b = tmp;
        i += 1;
    }
    b
}
