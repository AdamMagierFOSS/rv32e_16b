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
    pub uart_output: Vec<u8>,
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

    pub fn load_program(&mut self, base_byte_addr: u32, program: &[u32]) {
        let base_cell = base_byte_addr >> 1;
        for (i, &inst) in program.iter().enumerate() {
            let cell = base_cell + (i as u32) * 2;
            self.mem.store32(cell, inst);
        }
    }

    pub fn load_binary(&mut self, base_byte_addr: u32, data: &[u8]) {
        let base_cell = (base_byte_addr >> 1) as usize;
        for i in (0..data.len()).step_by(2) {
            let lo = data[i] as u16;
            let hi = if i + 1 < data.len() { data[i + 1] as u16 } else { 0 };
            self.mem.store16((base_cell + i / 2) as u32, lo | (hi << 8));
        }
    }

    fn fetch(&self) -> u32 {
        self.mem.load32(self.pc >> 1)
    }

    /// Effective byte address for loads/stores.
    fn byte_addr(&self, rs1: u8, imm: i32) -> u32 {
        self.regs.read(rs1).wrapping_add(imm as u32)
    }

    /// Effective cell address for 16-bit and 32-bit loads/stores.
    fn cell_addr(&self, rs1: u8, imm: i32) -> u32 {
        self.byte_addr(rs1, imm) >> 1
    }

    fn store_byte(&mut self, byte_addr: u32, val: u8) {
        if byte_addr == UART_BASE {
            self.uart_output.push(val);
        } else {
            self.mem.store8(byte_addr, val);
        }
    }

    pub fn step(&mut self) -> StepResult {
        let raw = self.fetch();
        let inst = match decode::decode(raw) {
            Some(i) => i,
            None => return StepResult::IllegalInstruction(raw),
        };

        let mut next_pc = self.pc.wrapping_add(4);

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

            // Sub-cell 8-bit loads: extract one byte from a 16-bit cell.
            Instruction::Lb { rd, rs1, imm } => {
                let val = self.mem.load8(self.byte_addr(rs1, imm)) as i8 as i32 as u32;
                self.regs.write(rd, val);
            }
            Instruction::Lbu { rd, rs1, imm } => {
                let val = self.mem.load8(self.byte_addr(rs1, imm)) as u32;
                self.regs.write(rd, val);
            }
            Instruction::Lh { rd, rs1, imm } => {
                let val = self.mem.load16(self.cell_addr(rs1, imm)) as i16 as i32 as u32;
                self.regs.write(rd, val);
            }
            Instruction::Lhu { rd, rs1, imm } => {
                let val = self.mem.load16(self.cell_addr(rs1, imm)) as u32;
                self.regs.write(rd, val);
            }
            Instruction::Lw { rd, rs1, imm } => {
                let val = self.mem.load32(self.cell_addr(rs1, imm));
                self.regs.write(rd, val);
            }

            // Sub-cell 8-bit store: modify one byte within a 16-bit cell.
            Instruction::Sb { rs1, rs2, imm } => {
                let val = self.regs.read(rs2) as u8;
                self.store_byte(self.byte_addr(rs1, imm), val);
            }
            Instruction::Sh { rs1, rs2, imm } => {
                let val = self.regs.read(rs2) as u16;
                self.mem.store16(self.cell_addr(rs1, imm), val);
            }
            Instruction::Sw { rs1, rs2, imm } => {
                let val = self.regs.read(rs2);
                self.mem.store32(self.cell_addr(rs1, imm), val);
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

    #[test]
    fn addi_sequence() {
        let mut cpu = Cpu::new(256);
        cpu.load_program(0, &[0x00A00093, 0x01408113, 0x00100073]);
        cpu.step();
        assert_eq!(cpu.regs.read(1), 10);
        cpu.step();
        assert_eq!(cpu.regs.read(2), 30);
        assert!(matches!(cpu.step(), StepResult::Ebreak));
    }

    #[test]
    fn store_load_round_trip() {
        let mut cpu = Cpu::new(256);
        cpu.load_program(0, &[
            0x04200093, // addi x1, x0, 0x42
            0x06400113, // addi x2, x0, 100
            0x00111023, // sh x1, 0(x2)
            0x00011183, // lh x3, 0(x2)
            0x00100073, // ebreak
        ]);
        cpu.step();
        cpu.step();
        cpu.step();
        cpu.step();
        assert_eq!(cpu.regs.read(3), 0x42);
    }

    #[test]
    fn load_binary_and_run() {
        let mut cpu = Cpu::new(256);
        let binary: [u8; 8] = [
            0x93, 0x00, 0xA0, 0x02, // addi x1, x0, 42
            0x73, 0x00, 0x10, 0x00, // ebreak
        ];
        cpu.load_binary(0, &binary);
        cpu.step();
        assert_eq!(cpu.regs.read(1), 42);
        assert!(matches!(cpu.step(), StepResult::Ebreak));
    }

    #[test]
    fn uart_output() {
        let mut cpu = Cpu::new(256);
        // addi x1, x0, 0x48      (x1 = 'H')
        // lui  x2, 0x10000       (x2 = 0x10000000 = UART_BASE)
        // sb   x1, 0(x2)         (UART <- 'H')
        // ebreak
        cpu.load_program(0, &[
            0x04800093, // addi x1, x0, 0x48
            0x10000137, // lui x2, 0x10000
            0x00110023, // sb x1, 0(x2)
            0x00100073, // ebreak
        ]);
        cpu.step();
        cpu.step();
        cpu.step();
        assert_eq!(cpu.uart_output, b"H");
    }

    #[test]
    fn sub_cell_load_store() {
        let mut cpu = Cpu::new(256);
        // Store 0xBEEF at cell 50 (byte addr 100), then load individual bytes
        // addi x1, x0, 100      (byte addr)
        // lui  x2, 0xBF          (x2 = 0x000BF000)
        // addi x2, x2, -273      (x2 = 0x000BEEF -- wait, let's use a simpler approach)
        //
        // Actually, just store two bytes separately and read them back:
        // addi x1, x0, 100       (x1 = byte addr 100)
        // addi x2, x0, 0x41      (x2 = 'A' = 0x41)
        // sb   x2, 0(x1)         (store 'A' at byte 100)
        // addi x2, x0, 0x42      (x2 = 'B' = 0x42)
        // sb   x2, 1(x1)         (store 'B' at byte 101)
        // lb   x3, 0(x1)         (x3 = byte at addr 100)
        // lb   x4, 1(x1)         (x4 = byte at addr 101)
        // ebreak
        cpu.load_program(0, &[
            0x06400093, // addi x1, x0, 100
            0x04100113, // addi x2, x0, 0x41
            0x00208023, // sb x2, 0(x1)
            0x04200113, // addi x2, x0, 0x42
            0x002080A3, // sb x2, 1(x1)
            0x00008183, // lb x3, 0(x1)
            0x00108203, // lb x4, 1(x1)
            0x00100073, // ebreak
        ]);
        for _ in 0..7 {
            cpu.step();
        }
        assert_eq!(cpu.regs.read(3), 0x41);
        assert_eq!(cpu.regs.read(4), 0x42);
        // The underlying cell should have both bytes packed
        assert_eq!(cpu.mem.load16(50), 0x4241);
    }
}
