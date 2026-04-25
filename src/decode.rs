/// Decoded instruction — every RV32E instruction maps to one of these.
#[derive(Debug)]
pub enum Instruction {
    Lui { rd: u8, imm: u32 },
    Auipc { rd: u8, imm: u32 },
    Jal { rd: u8, imm: i32 },
    Jalr { rd: u8, rs1: u8, imm: i32 },

    Beq { rs1: u8, rs2: u8, imm: i32 },
    Bne { rs1: u8, rs2: u8, imm: i32 },
    Blt { rs1: u8, rs2: u8, imm: i32 },
    Bge { rs1: u8, rs2: u8, imm: i32 },
    Bltu { rs1: u8, rs2: u8, imm: i32 },
    Bgeu { rs1: u8, rs2: u8, imm: i32 },

    Lb { rd: u8, rs1: u8, imm: i32 },
    Lbu { rd: u8, rs1: u8, imm: i32 },
    Lh { rd: u8, rs1: u8, imm: i32 },
    Lhu { rd: u8, rs1: u8, imm: i32 },
    Lw { rd: u8, rs1: u8, imm: i32 },

    Sb { rs1: u8, rs2: u8, imm: i32 },
    Sh { rs1: u8, rs2: u8, imm: i32 },
    Sw { rs1: u8, rs2: u8, imm: i32 },

    Addi { rd: u8, rs1: u8, imm: i32 },
    Slti { rd: u8, rs1: u8, imm: i32 },
    Sltiu { rd: u8, rs1: u8, imm: i32 },
    Xori { rd: u8, rs1: u8, imm: i32 },
    Ori { rd: u8, rs1: u8, imm: i32 },
    Andi { rd: u8, rs1: u8, imm: i32 },
    Slli { rd: u8, rs1: u8, shamt: u8 },
    Srli { rd: u8, rs1: u8, shamt: u8 },
    Srai { rd: u8, rs1: u8, shamt: u8 },

    Add { rd: u8, rs1: u8, rs2: u8 },
    Sub { rd: u8, rs1: u8, rs2: u8 },
    Sll { rd: u8, rs1: u8, rs2: u8 },
    Slt { rd: u8, rs1: u8, rs2: u8 },
    Sltu { rd: u8, rs1: u8, rs2: u8 },
    Xor { rd: u8, rs1: u8, rs2: u8 },
    Srl { rd: u8, rs1: u8, rs2: u8 },
    Sra { rd: u8, rs1: u8, rs2: u8 },
    Or { rd: u8, rs1: u8, rs2: u8 },
    And { rd: u8, rs1: u8, rs2: u8 },

    Ecall,
    Ebreak,
}

fn rd(inst: u32) -> u8 {
    ((inst >> 7) & 0xF) as u8
}

fn rs1(inst: u32) -> u8 {
    ((inst >> 15) & 0xF) as u8
}

fn rs2(inst: u32) -> u8 {
    ((inst >> 20) & 0xF) as u8
}

fn funct3(inst: u32) -> u32 {
    (inst >> 12) & 0x7
}

fn funct7(inst: u32) -> u32 {
    (inst >> 25) & 0x7F
}

fn imm_i(inst: u32) -> i32 {
    (inst as i32) >> 20
}

fn imm_s(inst: u32) -> i32 {
    let lo = (inst >> 7) & 0x1F;
    let hi = (inst >> 25) & 0x7F;
    ((hi << 5) | lo) as i32 - if inst & 0x8000_0000 != 0 { 4096 } else { 0 }
}

fn imm_b(inst: u32) -> i32 {
    let b11 = (inst >> 7) & 1;
    let b4_1 = (inst >> 8) & 0xF;
    let b10_5 = (inst >> 25) & 0x3F;
    let b12 = (inst >> 31) & 1;
    let imm = (b12 << 12) | (b11 << 11) | (b10_5 << 5) | (b4_1 << 1);
    if b12 != 0 {
        imm as i32 - (1 << 13)
    } else {
        imm as i32
    }
}

fn imm_u(inst: u32) -> u32 {
    inst & 0xFFFFF000
}

fn imm_j(inst: u32) -> i32 {
    let b19_12 = (inst >> 12) & 0xFF;
    let b11 = (inst >> 20) & 1;
    let b10_1 = (inst >> 21) & 0x3FF;
    let b20 = (inst >> 31) & 1;
    let imm = (b20 << 20) | (b19_12 << 12) | (b11 << 11) | (b10_1 << 1);
    if b20 != 0 {
        imm as i32 - (1 << 21)
    } else {
        imm as i32
    }
}

pub fn decode(inst: u32) -> Option<Instruction> {
    let opcode = inst & 0x7F;
    match opcode {
        0b0110111 => Some(Instruction::Lui { rd: rd(inst), imm: imm_u(inst) }),
        0b0010111 => Some(Instruction::Auipc { rd: rd(inst), imm: imm_u(inst) }),
        0b1101111 => Some(Instruction::Jal { rd: rd(inst), imm: imm_j(inst) }),
        0b1100111 => Some(Instruction::Jalr { rd: rd(inst), rs1: rs1(inst), imm: imm_i(inst) }),

        0b1100011 => {
            let (r1, r2, imm) = (rs1(inst), rs2(inst), imm_b(inst));
            match funct3(inst) {
                0b000 => Some(Instruction::Beq { rs1: r1, rs2: r2, imm }),
                0b001 => Some(Instruction::Bne { rs1: r1, rs2: r2, imm }),
                0b100 => Some(Instruction::Blt { rs1: r1, rs2: r2, imm }),
                0b101 => Some(Instruction::Bge { rs1: r1, rs2: r2, imm }),
                0b110 => Some(Instruction::Bltu { rs1: r1, rs2: r2, imm }),
                0b111 => Some(Instruction::Bgeu { rs1: r1, rs2: r2, imm }),
                _ => None,
            }
        }

        0b0000011 => {
            let (d, r1, imm) = (rd(inst), rs1(inst), imm_i(inst));
            match funct3(inst) {
                0b000 => Some(Instruction::Lb { rd: d, rs1: r1, imm }),
                0b001 => Some(Instruction::Lh { rd: d, rs1: r1, imm }),
                0b010 => Some(Instruction::Lw { rd: d, rs1: r1, imm }),
                0b100 => Some(Instruction::Lbu { rd: d, rs1: r1, imm }),
                0b101 => Some(Instruction::Lhu { rd: d, rs1: r1, imm }),
                _ => None,
            }
        }

        0b0100011 => {
            let (r1, r2, imm) = (rs1(inst), rs2(inst), imm_s(inst));
            match funct3(inst) {
                0b000 => Some(Instruction::Sb { rs1: r1, rs2: r2, imm }),
                0b001 => Some(Instruction::Sh { rs1: r1, rs2: r2, imm }),
                0b010 => Some(Instruction::Sw { rs1: r1, rs2: r2, imm }),
                _ => None,
            }
        }

        // I-type ALU
        0b0010011 => {
            let (d, r1, imm) = (rd(inst), rs1(inst), imm_i(inst));
            match funct3(inst) {
                0b000 => Some(Instruction::Addi { rd: d, rs1: r1, imm }),
                0b010 => Some(Instruction::Slti { rd: d, rs1: r1, imm }),
                0b011 => Some(Instruction::Sltiu { rd: d, rs1: r1, imm }),
                0b100 => Some(Instruction::Xori { rd: d, rs1: r1, imm }),
                0b110 => Some(Instruction::Ori { rd: d, rs1: r1, imm }),
                0b111 => Some(Instruction::Andi { rd: d, rs1: r1, imm }),
                0b001 => Some(Instruction::Slli { rd: d, rs1: r1, shamt: (imm & 0x1F) as u8 }),
                0b101 => {
                    if funct7(inst) & 0x20 != 0 {
                        Some(Instruction::Srai { rd: d, rs1: r1, shamt: (imm & 0x1F) as u8 })
                    } else {
                        Some(Instruction::Srli { rd: d, rs1: r1, shamt: (imm & 0x1F) as u8 })
                    }
                }
                _ => None,
            }
        }

        // R-type ALU
        0b0110011 => {
            let (d, r1, r2) = (rd(inst), rs1(inst), rs2(inst));
            match (funct3(inst), funct7(inst)) {
                (0b000, 0b0000000) => Some(Instruction::Add { rd: d, rs1: r1, rs2: r2 }),
                (0b000, 0b0100000) => Some(Instruction::Sub { rd: d, rs1: r1, rs2: r2 }),
                (0b001, 0b0000000) => Some(Instruction::Sll { rd: d, rs1: r1, rs2: r2 }),
                (0b010, 0b0000000) => Some(Instruction::Slt { rd: d, rs1: r1, rs2: r2 }),
                (0b011, 0b0000000) => Some(Instruction::Sltu { rd: d, rs1: r1, rs2: r2 }),
                (0b100, 0b0000000) => Some(Instruction::Xor { rd: d, rs1: r1, rs2: r2 }),
                (0b101, 0b0000000) => Some(Instruction::Srl { rd: d, rs1: r1, rs2: r2 }),
                (0b101, 0b0100000) => Some(Instruction::Sra { rd: d, rs1: r1, rs2: r2 }),
                (0b110, 0b0000000) => Some(Instruction::Or { rd: d, rs1: r1, rs2: r2 }),
                (0b111, 0b0000000) => Some(Instruction::And { rd: d, rs1: r1, rs2: r2 }),
                _ => None,
            }
        }

        // SYSTEM
        0b1110011 => {
            if inst == 0x00000073 {
                Some(Instruction::Ecall)
            } else if inst == 0x00100073 {
                Some(Instruction::Ebreak)
            } else {
                None
            }
        }

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_addi() {
        // addi x1, x0, 5  =>  imm=5, rs1=0, funct3=000, rd=1, opcode=0010011
        let inst: u32 = 0x00500093;
        match decode(inst) {
            Some(Instruction::Addi { rd, rs1, imm }) => {
                assert_eq!(rd, 1);
                assert_eq!(rs1, 0);
                assert_eq!(imm, 5);
            }
            other => panic!("expected Addi, got {:?}", other),
        }
    }

    #[test]
    fn decode_sw() {
        // sw x1, 0(x2)  =>  imm=0, rs2=1, rs1=2, funct3=010, opcode=0100011
        let inst: u32 = 0x00112023;
        match decode(inst) {
            Some(Instruction::Sw { rs1, rs2, imm }) => {
                assert_eq!(rs1, 2);
                assert_eq!(rs2, 1);
                assert_eq!(imm, 0);
            }
            other => panic!("expected Sw, got {:?}", other),
        }
    }
}
