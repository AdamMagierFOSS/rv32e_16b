mod cpu;
mod decode;
mod loader;
mod memory;
mod registers;

use cpu::{Cpu, StepResult};
use std::env;
use std::process;

const MEM_CELLS: usize = 65536; // 128KB byte-addressable
const MAX_STEPS: u64 = 10_000_000;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        run_binary(&args[1..]);
    } else {
        run_demo();
    }
}

fn parse_u32(s: &str) -> u32 {
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u32::from_str_radix(hex, 16).unwrap()
    } else {
        s.parse().unwrap()
    }
}

fn run_binary(args: &[String]) {
    let path = &args[0];
    let loaded = loader::load(path).unwrap_or_else(|e| {
        eprintln!("Error loading {}: {}", path, e);
        process::exit(1);
    });

    let mut cpu = Cpu::new(MEM_CELLS);
    cpu.load_binary(0, &loaded.data);
    cpu.pc = loaded.entry_point;

    // --set ADDR=VALUE to poke words into memory before execution
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--set" && i + 1 < args.len() {
            let parts: Vec<&str> = args[i + 1].splitn(2, '=').collect();
            if parts.len() == 2 {
                let addr = parse_u32(parts[0]);
                let val = parse_u32(parts[1]);
                cpu.mem.store32(addr >> 1, val);
                println!("  set [0x{:04X}] = {}", addr, val);
            }
            i += 2;
        } else {
            i += 1;
        }
    }

    println!("=== RV32E 16-bit-byte simulator ===");
    println!("Loaded {} ({} bytes, entry=0x{:08X})\n", path, loaded.data.len(), loaded.entry_point);

    let mut steps: u64 = 0;
    loop {
        let result = cpu.step();
        steps += 1;

        match result {
            StepResult::Continue => {}
            StepResult::Ecall => {
                println!("ECALL at pc=0x{:08X} after {} steps", cpu.pc, steps);
                break;
            }
            StepResult::Ebreak => {
                println!("EBREAK at pc=0x{:08X} after {} steps", cpu.pc, steps);
                break;
            }
            StepResult::IllegalInstruction(raw) => {
                println!("Illegal instruction 0x{:08X} at pc=0x{:08X} after {} steps",
                    raw, cpu.pc, steps);
                break;
            }
        }

        if steps >= MAX_STEPS {
            println!("Execution limit ({} steps) reached at pc=0x{:08X}", MAX_STEPS, cpu.pc);
            break;
        }
    }

    if !cpu.uart_output.is_empty() {
        println!("\n--- UART output ---");
        print!("{}", String::from_utf8_lossy(&cpu.uart_output));
        if cpu.uart_output.last() != Some(&b'\n') {
            println!();
        }
        println!("--- end UART ---");
    }

    println!("\nRegister dump:");
    for (i, &val) in cpu.regs.dump().iter().enumerate() {
        if val != 0 || i == 0 {
            println!("  x{:2} = 0x{:08X} ({val})", i, val);
        }
    }
}

fn run_demo() {
    let mut cpu = Cpu::new(1024);

    // Sum 1..=5. PC is now byte-addressed, so standard RV32 encoding works.
    //   0x00: addi x1, x0, 0
    //   0x04: addi x2, x0, 1
    //   0x08: addi x3, x0, 6
    //   0x0C: beq  x2, x3, +16  -> 0x1C (ebreak)
    //   0x10: add  x1, x1, x2
    //   0x14: addi x2, x2, 1
    //   0x18: beq  x0, x0, -12  -> 0x0C
    //   0x1C: ebreak
    let program: &[u32] = &[
        0x00000093, // addi x1, x0, 0
        0x00100113, // addi x2, x0, 1
        0x00600193, // addi x3, x0, 6
        0x00310863, // beq  x2, x3, +16
        0x002080B3, // add  x1, x1, x2
        0x00110113, // addi x2, x2, 1
        0xFE000AE3, // beq  x0, x0, -12
        0x00100073, // ebreak
    ];
    cpu.load_program(0, program);

    println!("=== RV32E 16-bit-byte simulator ===");
    println!("Program: sum 1..=5\n");

    let mut steps = 0u64;
    loop {
        let pc = cpu.pc;
        let raw = cpu.mem.load32(pc >> 1);
        let result = cpu.step();
        steps += 1;

        println!(
            "  step {:2}: pc=0x{:02X}  inst=0x{:08X}  x1(acc)={:3}  x2(i)={:3}",
            steps, pc, raw, cpu.regs.read(1), cpu.regs.read(2),
        );

        match result {
            StepResult::Continue => {}
            StepResult::Ebreak => {
                println!("\nEBREAK at pc=0x{:02X}", cpu.pc);
                break;
            }
            StepResult::Ecall => {
                println!("\nECALL at pc=0x{:02X}", cpu.pc);
                break;
            }
            StepResult::IllegalInstruction(raw) => {
                println!("\nIllegal instruction 0x{:08X} at pc=0x{:02X}", raw, pc);
                break;
            }
        }
    }

    println!("\nResult: x1 = {} (expected 15)", cpu.regs.read(1));
    println!("Total steps: {steps}");
}
