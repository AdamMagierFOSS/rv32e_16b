use crate::decode::{self, Instruction};
use crate::memory::Memory;
use crate::registers::RegisterFile;

pub const UART_BASE: u32 = 0x10000000;

pub enum StepResult {
    Continue,
    Ecall,
    Ebreak,
    IllegalInstruction(u32),
}

pub struct Cpu {
    pub pc: u32,
    pub regs: RegisterFile,
    pub mem: Memory,
    /// Raw 16-bit cell values written to the UART.
    pub uart_output: Vec<u16>,
}

impl Cpu {
    pub fn new(mem_cells: usize) -> Self {
        Self {
            pc: 0,
            regs: RegisterFile::new(),
            mem: Memory::new(mem_cells),
            uart_output: Vec::new(),
        }
    }

    /// Load 32-bit instruction words. `base_addr` is a cell address.
    pub fn load_program(&mut self, base_addr: u32, program: &[u32]) {
        for (i, &inst) in program.iter().enumerate() {
            let addr = base_addr + (i as u32) * 2;
            self.mem.store32(addr, inst);
        }
    }

    /// Load raw bytes (e.g. a flat binary) into memory at `base_addr` (cell address).
    /// Packs pairs of bytes into 16-bit cells.
    pub fn load_binary(&mut self, base_addr: u32, data: &[u8]) {
        for i in (0..data.len()).step_by(2) {
            let lo = data[i] as u16;
            let hi = if i + 1 < data.len() { data[i + 1] as u16 } else { 0 };
            self.mem.store16(base_addr + (i / 2) as u32, lo | (hi << 8));
        }
    }

    /// Fetch: PC is a cell address.
    fn fetch(&self) -> u32 {
        self.mem.load32(self.pc)
    }

    /// Effective cell address for all loads/stores.
    fn addr(&self, rs1: u8, imm: i32) -> u32 {
        self.regs.read(rs1).wrapping_add(imm as u32)
    }

    fn store_cell(&mut self, cell_addr: u32, val: u16) {
        if cell_addr == UART_BASE {
            self.uart_output.push(val);
        } else {
            self.mem.store16(cell_addr, val);
        }
    }

    pub fn step(&mut self) -> StepResult {
        let raw = self.fetch();
        let inst = match decode::decode(raw) {
            Some(i) => i,
            None => return StepResult::IllegalInstruction(raw),
        };

        // Each 32-bit instruction = 2 cells
        let mut next_pc = self.pc.wrapping_add(2);

        match inst {
            Instruction::Lui { rd, imm } => {
                self.regs.write(rd, imm);
            }
            Instruction::Auipc { rd, imm } => {
                self.regs.write(rd, self.pc.wrapping_add(imm));
            }

            Instruction::Jal { rd, imm } => {
                self.regs.write(rd, next_pc);
                next_pc = self.pc.wrapping_add(imm as u32);
            }
            Instruction::Jalr { rd, rs1, imm } => {
                let target = self.regs.read(rs1).wrapping_add(imm as u32) & !1;
                self.regs.write(rd, next_pc);
                next_pc = target;
            }

            Instruction::Beq { rs1, rs2, imm } => {
                if self.regs.read(rs1) == self.regs.read(rs2) {
                    next_pc = self.pc.wrapping_add(imm as u32);
                }
            }
            Instruction::Bne { rs1, rs2, imm } => {
                if self.regs.read(rs1) != self.regs.read(rs2) {
                    next_pc = self.pc.wrapping_add(imm as u32);
                }
            }
            Instruction::Blt { rs1, rs2, imm } => {
                if (self.regs.read(rs1) as i32) < (self.regs.read(rs2) as i32) {
                    next_pc = self.pc.wrapping_add(imm as u32);
                }
            }
            Instruction::Bge { rs1, rs2, imm } => {
                if (self.regs.read(rs1) as i32) >= (self.regs.read(rs2) as i32) {
                    next_pc = self.pc.wrapping_add(imm as u32);
                }
            }
            Instruction::Bltu { rs1, rs2, imm } => {
                if self.regs.read(rs1) < self.regs.read(rs2) {
                    next_pc = self.pc.wrapping_add(imm as u32);
                }
            }
            Instruction::Bgeu { rs1, rs2, imm } => {
                if self.regs.read(rs1) >= self.regs.read(rs2) {
                    next_pc = self.pc.wrapping_add(imm as u32);
                }
            }

            // LC: Load Cell (sign-extended). Replaces LB and LH — both load one cell.
            Instruction::Lb { rd, rs1, imm } |
            Instruction::Lh { rd, rs1, imm } => {
                let val = self.mem.load16(self.addr(rs1, imm)) as i16 as i32 as u32;
                self.regs.write(rd, val);
            }
            // LCU: Load Cell Unsigned. Replaces LBU and LHU.
            Instruction::Lbu { rd, rs1, imm } |
            Instruction::Lhu { rd, rs1, imm } => {
                let val = self.mem.load16(self.addr(rs1, imm)) as u32;
                self.regs.write(rd, val);
            }
            Instruction::Lw { rd, rs1, imm } => {
                let val = self.mem.load32(self.addr(rs1, imm));
                self.regs.write(rd, val);
            }

            // SC: Store Cell. Replaces SB and SH — both store one cell.
            Instruction::Sb { rs1, rs2, imm } |
            Instruction::Sh { rs1, rs2, imm } => {
                let val = self.regs.read(rs2) as u16;
                self.store_cell(self.addr(rs1, imm), val);
            }
            Instruction::Sw { rs1, rs2, imm } => {
                let val = self.regs.read(rs2);
                self.mem.store32(self.addr(rs1, imm), val);
            }

            Instruction::Addi { rd, rs1, imm } => {
                self.regs.write(rd, self.regs.read(rs1).wrapping_add(imm as u32));
            }
            Instruction::Slti { rd, rs1, imm } => {
                let val = if (self.regs.read(rs1) as i32) < imm { 1 } else { 0 };
                self.regs.write(rd, val);
            }
            Instruction::Sltiu { rd, rs1, imm } => {
                let val = if self.regs.read(rs1) < (imm as u32) { 1 } else { 0 };
                self.regs.write(rd, val);
            }
            Instruction::Xori { rd, rs1, imm } => {
                self.regs.write(rd, self.regs.read(rs1) ^ imm as u32);
            }
            Instruction::Ori { rd, rs1, imm } => {
                self.regs.write(rd, self.regs.read(rs1) | imm as u32);
            }
            Instruction::Andi { rd, rs1, imm } => {
                self.regs.write(rd, self.regs.read(rs1) & imm as u32);
            }
            Instruction::Slli { rd, rs1, shamt } => {
                self.regs.write(rd, self.regs.read(rs1) << shamt);
            }
            Instruction::Srli { rd, rs1, shamt } => {
                self.regs.write(rd, self.regs.read(rs1) >> shamt);
            }
            Instruction::Srai { rd, rs1, shamt } => {
                self.regs.write(rd, ((self.regs.read(rs1) as i32) >> shamt) as u32);
            }

            Instruction::Add { rd, rs1, rs2 } => {
                self.regs.write(rd, self.regs.read(rs1).wrapping_add(self.regs.read(rs2)));
            }
            Instruction::Sub { rd, rs1, rs2 } => {
                self.regs.write(rd, self.regs.read(rs1).wrapping_sub(self.regs.read(rs2)));
            }
            Instruction::Sll { rd, rs1, rs2 } => {
                self.regs.write(rd, self.regs.read(rs1) << (self.regs.read(rs2) & 0x1F));
            }
            Instruction::Slt { rd, rs1, rs2 } => {
                let val = if (self.regs.read(rs1) as i32) < (self.regs.read(rs2) as i32) { 1 } else { 0 };
                self.regs.write(rd, val);
            }
            Instruction::Sltu { rd, rs1, rs2 } => {
                let val = if self.regs.read(rs1) < self.regs.read(rs2) { 1 } else { 0 };
                self.regs.write(rd, val);
            }
            Instruction::Xor { rd, rs1, rs2 } => {
                self.regs.write(rd, self.regs.read(rs1) ^ self.regs.read(rs2));
            }
            Instruction::Srl { rd, rs1, rs2 } => {
                self.regs.write(rd, self.regs.read(rs1) >> (self.regs.read(rs2) & 0x1F));
            }
            Instruction::Sra { rd, rs1, rs2 } => {
                self.regs.write(rd, ((self.regs.read(rs1) as i32) >> (self.regs.read(rs2) & 0x1F)) as u32);
            }
            Instruction::Or { rd, rs1, rs2 } => {
                self.regs.write(rd, self.regs.read(rs1) | self.regs.read(rs2));
            }
            Instruction::And { rd, rs1, rs2 } => {
                self.regs.write(rd, self.regs.read(rs1) & self.regs.read(rs2));
            }

            Instruction::Ecall => {
                self.pc = next_pc;
                return StepResult::Ecall;
            }
            Instruction::Ebreak => {
                self.pc = next_pc;
                return StepResult::Ebreak;
            }
        }

        self.pc = next_pc;
        StepResult::Continue
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(program: &[u32]) -> Cpu {
        let mut cpu = Cpu::new(1024);
        cpu.load_program(0, program);
        for _ in 0..10000 {
            match cpu.step() {
                StepResult::Ebreak | StepResult::Ecall => break,
                StepResult::IllegalInstruction(raw) =>
                    panic!("illegal instruction 0x{:08X} at pc=0x{:X}", raw, cpu.pc),
                StepResult::Continue => {}
            }
        }
        cpu
    }

    // --- I-type ALU ---

    #[test]
    fn addi() {
        let cpu = run(&[
            0x00A00093, // addi x1, x0, 10
            0x01408113, // addi x2, x1, 20
            0xFFF08193, // addi x3, x1, -1
            0x00100073, // ebreak
        ]);
        assert_eq!(cpu.regs.read(1), 10);
        assert_eq!(cpu.regs.read(2), 30);
        assert_eq!(cpu.regs.read(3), 9);
    }

    #[test]
    fn slti_sltiu() {
        let cpu = run(&[
            0x00500093, // addi x1, x0, 5
            0x00A0A113, // slti x2, x1, 10     -> 1 (5 < 10)
            0x0010A193, // slti x3, x1, 1      -> 0 (5 >= 1)
            0xFFF0B213, // sltiu x4, x1, -1    -> 1 (5 < 0xFFFFFFFF unsigned)
            0x00100073, // ebreak
        ]);
        assert_eq!(cpu.regs.read(2), 1);
        assert_eq!(cpu.regs.read(3), 0);
        assert_eq!(cpu.regs.read(4), 1);
    }

    #[test]
    fn logical_imm() {
        let cpu = run(&[
            0x0FF00093, // addi x1, x0, 0xFF
            0x0F00C113, // xori x2, x1, 0xF0  -> 0x0F
            0x0F00E193, // ori  x3, x1, 0xF0  -> 0xFF
            0x0F00F213, // andi x4, x1, 0xF0  -> 0xF0
            0x00100073, // ebreak
        ]);
        assert_eq!(cpu.regs.read(2), 0x0F);
        assert_eq!(cpu.regs.read(3), 0xFF);
        assert_eq!(cpu.regs.read(4), 0xF0);
    }

    #[test]
    fn shifts_imm() {
        let cpu = run(&[
            0x00100093, // addi x1, x0, 1
            0x01009113, // slli x2, x1, 16    -> 0x10000
            0x01015193, // srli x3, x2, 16    -> 1
            0xFFF00213, // addi x4, x0, -1    (0xFFFFFFFF)
            0x41F25293, // srai x5, x4, 31    -> 0xFFFFFFFF (sign extend)
            0x01F2D313, // srli x6, x5, 31    -> 1
            0x00100073, // ebreak
        ]);
        assert_eq!(cpu.regs.read(2), 0x10000);
        assert_eq!(cpu.regs.read(3), 1);
        assert_eq!(cpu.regs.read(5), 0xFFFFFFFF);
        assert_eq!(cpu.regs.read(6), 1);
    }

    // --- R-type ALU ---

    #[test]
    fn add_sub() {
        let cpu = run(&[
            0x01400093, // addi x1, x0, 20
            0x00A00113, // addi x2, x0, 10
            0x002081B3, // add x3, x1, x2     -> 30
            0x40208233, // sub x4, x1, x2     -> 10
            0x40110293, // addi x5, x2, 0x401... wait that's wrong
            0x00100073, // ebreak
        ]);
        assert_eq!(cpu.regs.read(3), 30);
        assert_eq!(cpu.regs.read(4), 10);
    }

    #[test]
    fn slt_sltu() {
        let cpu = run(&[
            0xFFF00093, // addi x1, x0, -1    (0xFFFFFFFF)
            0x00100113, // addi x2, x0, 1
            0x0020A1B3, // slt x3, x1, x2     -> 1 (-1 < 1 signed)
            0x0020B233, // sltu x4, x1, x2    -> 0 (0xFFFFFFFF > 1 unsigned)
            0x00100073, // ebreak
        ]);
        assert_eq!(cpu.regs.read(3), 1);
        assert_eq!(cpu.regs.read(4), 0);
    }

    #[test]
    fn logical_reg() {
        let cpu = run(&[
            0x0AA00093, // addi x1, x0, 0xAA
            0x05500113, // addi x2, x0, 0x55
            0x0020C1B3, // xor x3, x1, x2     -> 0xFF
            0x0020E233, // or  x4, x1, x2     -> 0xFF
            0x0020F2B3, // and x5, x1, x2     -> 0x00
            0x00100073, // ebreak
        ]);
        assert_eq!(cpu.regs.read(3), 0xFF);
        assert_eq!(cpu.regs.read(4), 0xFF);
        assert_eq!(cpu.regs.read(5), 0x00);
    }

    // --- LUI / AUIPC ---

    #[test]
    fn lui() {
        let cpu = run(&[
            0xDEADC0B7, // lui x1, 0xDEADC  -> 0xDEADC000
            0x00100073, // ebreak
        ]);
        assert_eq!(cpu.regs.read(1), 0xDEADC000);
    }

    #[test]
    fn auipc() {
        let cpu = run(&[
            0x00001097, // auipc x1, 1       -> PC(0) + 0x1000 = 0x1000
            0x00100073, // ebreak
        ]);
        assert_eq!(cpu.regs.read(1), 0x1000);
    }

    // --- Branches ---

    #[test]
    fn beq_taken() {
        // beq x1, x2, +8: from cell 4, skip 4 cells to cell 8 (ebreak)
        let cpu = run(&[
            0x00500093, // addi x1, x0, 5
            0x00500113, // addi x2, x0, 5
            0x00208263, // beq x1, x2, +4
            0x00000093, // addi x1, x0, 0     (skipped)
            0x00100073, // ebreak              (cell 8)
        ]);
        assert_eq!(cpu.regs.read(1), 5);
    }

    #[test]
    fn beq_not_taken() {
        let cpu = run(&[
            0x00500093, // addi x1, x0, 5
            0x00A00113, // addi x2, x0, 10
            0x00208263, // beq x1, x2, +4     -> not taken (5 != 10)
            0x00000093, // addi x1, x0, 0     (executed)
            0x00100073, // ebreak
        ]);
        assert_eq!(cpu.regs.read(1), 0);
    }

    #[test]
    fn bne() {
        let cpu = run(&[
            0x00500093, // addi x1, x0, 5
            0x00A00113, // addi x2, x0, 10
            0x00209263, // bne x1, x2, +4     -> taken (5 != 10)
            0x00000093, // addi x1, x0, 0     (skipped)
            0x00100073, // ebreak
        ]);
        assert_eq!(cpu.regs.read(1), 5);
    }

    #[test]
    fn blt_bge() {
        let cpu = run(&[
            0x00500093, // addi x1, x0, 5
            0x00A00113, // addi x2, x0, 10
            0x0020C263, // blt x1, x2, +4     -> taken (5 < 10)
            0x00000093, // addi x1, x0, 0     (skipped)
            0x00100073, // ebreak
        ]);
        assert_eq!(cpu.regs.read(1), 5);

        let cpu2 = run(&[
            0x00A00093, // addi x1, x0, 10
            0x00500113, // addi x2, x0, 5
            0x0020D263, // bge x1, x2, +4     -> taken (10 >= 5)
            0x00000093, // addi x1, x0, 0     (skipped)
            0x00100073, // ebreak
        ]);
        assert_eq!(cpu2.regs.read(1), 10);
    }

    #[test]
    fn backward_branch_loop() {
        // Sum 1..=5 using backward branch
        let cpu = run(&[
            0x00000093, // addi x1, x0, 0      (acc)
            0x00100113, // addi x2, x0, 1      (i)
            0x00600193, // addi x3, x0, 6      (bound)
            0x00310463, // beq  x2, x3, +8     (skip to ebreak)
            0x002080B3, // add  x1, x1, x2
            0x00110113, // addi x2, x2, 1
            0xFE000DE3, // beq  x0, x0, -6     (back to beq x2,x3)
            0x00100073, // ebreak
        ]);
        assert_eq!(cpu.regs.read(1), 15); // 1+2+3+4+5
    }

    // --- JAL / JALR ---

    #[test]
    fn jal_forward() {
        // jal x1, +8: from cell 0, jump to cell 8. x1 = cell 2 (return).
        let cpu = run(&[
            0x008000EF, // jal x1, +8          (cell 0 -> cell 8)
            0x00000013, // nop                  (cell 2, skipped)
            0x00000013, // nop                  (cell 4, skipped)
            0x00000013, // nop                  (cell 6, skipped)
            0x00100073, // ebreak               (cell 8)
        ]);
        assert_eq!(cpu.regs.read(1), 2); // ra = cell 2
    }

    #[test]
    fn jal_and_jalr() {
        // cell 0: jal x1, +6 -> cell 6.  x1 = 2.
        // cell 2: addi x3, x0, 99        (reached on return)
        // cell 4: ebreak
        // cell 6: addi x2, x0, 42
        // cell 8: jalr x0, x1, 0         (jump to cell 2)
        let cpu = run(&[
            0x006000EF, // jal x1, +6          (cell 0 -> cell 6)
            0x06300193, // addi x3, x0, 99     (cell 2, executed on return)
            0x00100073, // ebreak               (cell 4)
            0x02A00113, // addi x2, x0, 42     (cell 6)
            0x000080E7, // jalr x1, x1, 0      (cell 8 -> cell 2, x1=10)
        ]);
        assert_eq!(cpu.regs.read(2), 42);
        assert_eq!(cpu.regs.read(3), 99);
    }

    // --- Load / Store ---

    #[test]
    fn lw_sw_word() {
        // lui 0xDEADC + addi 0xEEF = 0xDEADC000 + sign_ext(0xEEF) = 0xDEADBEEF
        let cpu = run(&[
            0x06400093, // addi x1, x0, 100       (addr)
            0xDEADC137, // lui x2, 0xDEADC         -> 0xDEADC000
            0xEEF10113, // addi x2, x2, 0xEEF      -> 0xDEADBEEF
            0x0020A023, // sw x2, 0(x1)
            0x0000A183, // lw x3, 0(x1)
            0x00100073, // ebreak
        ]);
        assert_eq!(cpu.regs.read(2), 0xDEADBEEF);
        assert_eq!(cpu.regs.read(3), 0xDEADBEEF);
    }

    #[test]
    fn lh_sign_extension() {
        let cpu = run(&[
            0x06400093, // addi x1, x0, 100
            0xFFF00113, // addi x2, x0, -1   (0xFFFFFFFF; low 16 = 0xFFFF)
            0x00209023, // sh x2, 0(x1)       (store cell: 0xFFFF)
            0x00009183, // lh x3, 0(x1)       (sign-extend: 0xFFFFFFFF)
            0x0000D203, // lhu x4, 0(x1)      (zero-extend: 0x0000FFFF)
            0x00100073, // ebreak
        ]);
        assert_eq!(cpu.regs.read(3), 0xFFFFFFFF); // sign-extended
        assert_eq!(cpu.regs.read(4), 0x0000FFFF); // zero-extended
    }

    #[test]
    fn lb_sb_alias_to_cell() {
        // LB/SB operate on full cells (16-bit), same as LH/SH
        let cpu = run(&[
            0x06400093, // addi x1, x0, 100
            0x04200113, // addi x2, x0, 0x42
            0x00208023, // sb x2, 0(x1)       (stores cell = 0x0042)
            0x00008183, // lb x3, 0(x1)       (loads cell, sign-extends)
            0x0000C203, // lbu x4, 0(x1)      (loads cell, zero-extends)
            0x00100073, // ebreak
        ]);
        assert_eq!(cpu.regs.read(3), 0x42);
        assert_eq!(cpu.regs.read(4), 0x42);
    }

    #[test]
    fn store_load_with_offset() {
        let cpu = run(&[
            0x06400093, // addi x1, x0, 100
            0x00A00113, // addi x2, x0, 10
            0x01400193, // addi x3, x0, 20
            0x00209023, // sh x2, 0(x1)       (cell 100 = 10)
            0x003090A3, // sh x3, 1(x1)       (cell 101 = 20)
            0x00109203, // lh x4, 1(x1)       (load cell 101)
            0x00100073, // ebreak
        ]);
        assert_eq!(cpu.regs.read(4), 20);
    }

    // --- x0 register ---

    #[test]
    fn x0_always_zero() {
        let cpu = run(&[
            0x02A00013, // addi x0, x0, 42     (write to x0, should be discarded)
            0x00100073, // ebreak
        ]);
        assert_eq!(cpu.regs.read(0), 0);
    }

    // --- UART ---

    #[test]
    fn uart_captures_cells() {
        let cpu = run(&[
            0x10000137, // lui x2, 0x10000     (x2 = UART_BASE)
            0x04800093, // addi x1, x0, 0x48   ('H')
            0x00111023, // sh x1, 0(x2)
            0x06900093, // addi x1, x0, 0x69   ('i')
            0x00111023, // sh x1, 0(x2)
            0x00100073, // ebreak
        ]);
        assert_eq!(cpu.uart_output, &[0x48u16, 0x69u16]);
    }

    // --- PC increment ---

    #[test]
    fn pc_increments_by_2() {
        let mut cpu = Cpu::new(256);
        cpu.load_program(0, &[
            0x00000013, // nop
            0x00000013, // nop
            0x00100073, // ebreak
        ]);
        assert_eq!(cpu.pc, 0);
        cpu.step();
        assert_eq!(cpu.pc, 2);
        cpu.step();
        assert_eq!(cpu.pc, 4);
    }

    // --- ECALL ---

    #[test]
    fn ecall() {
        let mut cpu = Cpu::new(256);
        cpu.load_program(0, &[0x00000073]); // ecall
        assert!(matches!(cpu.step(), StepResult::Ecall));
    }

    // --- Load binary ---

    #[test]
    fn load_binary_packs_bytes() {
        let mut cpu = Cpu::new(256);
        let binary: [u8; 8] = [
            0x93, 0x00, 0xA0, 0x02, // addi x1, x0, 42 (LE)
            0x73, 0x00, 0x10, 0x00, // ebreak
        ];
        cpu.load_binary(0, &binary);
        // Cell 0 should be 0x0093, cell 1 should be 0x02A0
        assert_eq!(cpu.mem.load16(0), 0x0093);
        assert_eq!(cpu.mem.load16(1), 0x02A0);
        cpu.step();
        assert_eq!(cpu.regs.read(1), 42);
        assert!(matches!(cpu.step(), StepResult::Ebreak));
    }
}
