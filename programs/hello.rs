#![no_std]
#![no_main]

include!("rv32e_rt.rs");

#[no_mangle]
pub extern "C" fn main() {
    println!("Hello, World!");
    eprintln!("This is stderr (same UART on bare metal)");

    print!("1 + 2 = ");
    _print_u32(1 + 2);
    println!();

    print!("Counting: ");
    let mut i: u32 = 1;
    while i <= 5 {
        if i > 1 {
            print!(", ");
        }
        _print_u32(i);
        i += 1;
    }
    println!();

    print!("-42 as signed: ");
    _print_i32(-42);
    println!();

    print!("0xDEADBEEF in hex: ");
    _print_hex(0xDEADBEEF);
    println!();
}
