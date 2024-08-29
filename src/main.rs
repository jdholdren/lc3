use std::ops::{Index, IndexMut};

// 2^16 memory

enum Register {
    R0 = 0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    PC, /* program counter */
    COND,
    COUNT,
}

impl Index<Register> for [u16] {
    type Output = u16;

    fn index(&self, index: Register) -> &Self::Output {
        todo!()
    }
}

impl IndexMut<Register> for [u16] {
    fn index_mut(&mut self, index: Register) -> &mut Self::Output {
        todo!()
    }
}

// Flag for the COND register
#[derive(Debug)]
enum Flag {
    P = 1 << 0,
    Z = 1 << 1,
    N = 1 << 2,
}

impl From<Flag> for u16 {
    fn from(value: Flag) -> Self {
        todo!()
    }
}

const MAX_MEMORY: usize = 1 << 16;

struct VM {
    reg: [u16; Register::COUNT as usize],
    mem: [u16; MAX_MEMORY],
}

impl Default for VM {
    fn default() -> Self {
        let mut vm = Self {
            reg: [0; Register::COUNT as usize],
            mem: [0; MAX_MEMORY],
        };

        // since exactly one condition flag should be set at any given time, set the Z flag
        vm.reg[Register::COND] = Flag::Z.into();
        // set the PC to starting position
        // 0x3000 is the default
        vm.reg[Register::PC] = 0x3000;

        vm
    }
}

impl VM {
    // Runs one instruction to completion.
    fn step(&mut self) {}
}

fn main() {}

enum OpCode {
    Br,   /* branch */
    Add,  /* add  */
    Ld,   /* load */
    St,   /* store */
    Jsr,  /* jump register */
    And,  /* bitwise and */
    Ldr,  /* load register */
    Str,  /* store register */
    Rti,  /* unused */
    Not,  /* bitwise not */
    Ldi,  /* load indirect */
    Sti,  /* store indirect */
    Jmp,  /* jump */
    Res,  /* reserved (unused) */
    Lea,  /* load effective address */
    Trap, /* execute trap */
}

impl From<u16> for OpCode {
    fn from(value: u16) -> Self {
        match value {
            0 => Self::Br,
            1 => Self::Add,
            2 => Self::Ld,
            3 => Self::St,
            4 => Self::Jsr,
            5 => Self::And,
            6 => Self::Ldr,
            7 => Self::Str,
            8 => Self::Rti,
            9 => Self::Not,
            10 => Self::Ldi,
            11 => Self::Sti,
            12 => Self::Jmp,
            13 => Self::Res,
            14 => Self::Lea,
            15 => Self::Trap,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, PartialEq)]
enum Operation {
    AddRegMode {
        dr: u16,
        sr1: u16,
        sr2: u16,
    },
    AddImmediateMode {
        dr: u16,
        sr1: u16,
        imm5: u16,
    },
    AndRegMode {
        dr: u16,
        sr1: u16,
        sr2: u16,
    },
    AndImmediateMode {
        dr: u16,
        sr1: u16,
        imm5: u16,
    },
    Br {
        n: bool,
        z: bool,
        p: bool,
        pc_offset_9: u16,
    },
    Jmp {
        base_r: u16,
    },
    Ret,
    Jsr {
        pc_offset11: u16,
    },
    Jsrr {
        base_r: u16,
    },
    Ld {
        dr: u16,
        pc_offset9: u16,
    },
    Ldi {
        dr: u16,
        pc_offset9: u16,
    },
    Ldr {
        dr: u16,
        base_r: u16,
        offset6: u16,
    },
    Lea {
        dr: u16,
        pc_offset9: u16,
    },
    Not {
        dr: u16,
        sr: u16,
    },
    Rti,
    St {
        sr: u16,
        pc_offset9: u16,
    },
    Sti {
        sr: u16,
        pc_offset9: u16,
    },
    Str {
        sr: u16,
        base_r: u16,
        offset6: u16,
    },
    Trap {
        trap_vect8: u16,
    },
}

// Turns a raw 16 bits into an Op.
fn parse_op(instr: u16) -> Operation {
    // Leading 4 bytes are the op code
    let leading = instr >> 12;
    let op_code: OpCode = leading.into();

    match op_code {
        OpCode::Add => {
            // Check if immediate mode
            let immediate_mode = instr >> 5 & 0b1;
            if immediate_mode == 0 {
                Operation::AddRegMode {
                    dr: instr >> 9 & 0b111,
                    sr1: instr >> 6 & 0b111,
                    sr2: instr & 0b111,
                }
            } else {
                Operation::AddImmediateMode {
                    dr: instr >> 9 & 0b111,
                    sr1: instr >> 6 & 0b111,
                    imm5: sign_extend(instr & 0b11111, 5),
                }
            }
        }
        OpCode::And => {
            let immediate_mode = instr >> 5 & 0b1;
            if immediate_mode == 0 {
                Operation::AndRegMode {
                    dr: instr >> 9 & 0b111,
                    sr1: instr >> 6 & 0b111,
                    sr2: instr & 0b111,
                }
            } else {
                Operation::AndImmediateMode {
                    dr: instr >> 9 & 0b111,
                    sr1: instr >> 6 & 0b111,
                    imm5: sign_extend(instr & 0b11111, 5),
                }
            }
        }
        OpCode::Br => Operation::Br {
            n: (instr >> 11 & 1) == 1,
            z: (instr >> 10 & 1) == 1,
            p: (instr >> 9 & 1) == 1,
            pc_offset_9: sign_extend(instr & 0b111111111, 9),
        },
        OpCode::Jmp => {
            let base_r = instr >> 6 & 0b111;
            if base_r == 0b111 {
                Operation::Ret
            } else {
                Operation::Jmp { base_r }
            }
        }
        OpCode::Jsr => {
            let mode = instr >> 11 & 1;
            if mode == 1 {
                Operation::Jsr {
                    pc_offset11: sign_extend(instr & 0b11111111111, 11),
                }
            } else {
                Operation::Jsrr {
                    base_r: instr >> 6 & 0b111,
                }
            }
        }
        OpCode::Ld => Operation::Ld {
            dr: instr >> 9 & 0b111,
            pc_offset9: instr & 0b111111111,
        },
        OpCode::Ldi => Operation::Ldi {
            dr: instr >> 9 & 0b111,
            pc_offset9: sign_extend(instr & 0b111111111, 9),
        },
        OpCode::Ldr => Operation::Ldr {
            dr: instr >> 9 & 0b111,
            base_r: instr >> 6 & 0b111,
            offset6: instr & 0b111111,
        },
        OpCode::Lea => Operation::Lea {
            dr: instr >> 9 & 0b111,
            pc_offset9: sign_extend(instr & 0b111111111, 9),
        },
        OpCode::Not => Operation::Not {
            dr: instr >> 9 & 0b111,
            sr: instr >> 6 & 0b111,
        },
        OpCode::Rti => Operation::Rti,
        OpCode::St => Operation::St {
            sr: instr >> 9 & 0b111,
            pc_offset9: sign_extend(instr & 0b111111111, 9),
        },
        OpCode::Sti => Operation::Sti {
            sr: instr >> 9 & 0b111,
            pc_offset9: sign_extend(instr & 0b111111111, 9),
        },
        OpCode::Str => Operation::Str {
            sr: instr >> 9 & 0b111,
            base_r: instr >> 6 & 0b111,
            offset6: sign_extend(instr & 0b111111, 6),
        },
        OpCode::Trap => Operation::Trap {
            trap_vect8: instr & 0b11111111,
        },
        _ => panic!("bad op code: {}", leading),
    }
}

// Extends a 5 bit integer to 16 bits, preserving the sign.
fn sign_extend(i: u16, bit_count: u16) -> u16 {
    let sign = (i >> (bit_count - 1) & 0b1);
    if sign > 0 {
        let mut ret = i;
        ret |= 0xFFFF << bit_count;
        return ret;
    }

    i
}

#[cfg(test)]
mod parse_test {
    use super::*;

    #[test]
    fn parse_add_reg() {
        let want = Operation::AddRegMode {
            dr: 0b101,
            sr1: 0b111,
            sr2: 0b011,
        };
        assert_eq!(want, parse_op(0b0001_101_111_0_00_011));
    }

    #[test]
    fn parse_add_imm() {
        let want = Operation::AddImmediateMode {
            dr: 0b101,
            sr1: 0b111,
            imm5: 11,
        };
        assert_eq!(want, parse_op(0b0001_101_111_1_01011));
    }

    #[test]
    fn parse_add_imm_sign_extend() {
        let want = Operation::AddImmediateMode {
            dr: 0b101,
            sr1: 0b111,
            imm5: 0b1111111111111111,
        };
        assert_eq!(want, parse_op(0b0001_101_111_1_11111));
    }

    #[test]
    fn parse_and_reg() {
        let want = Operation::AndRegMode {
            dr: 0b101,
            sr1: 0b111,
            sr2: 0b111,
        };
        assert_eq!(want, parse_op(0b0101_101_111_0_00111));
    }

    #[test]
    fn parse_and_imm() {
        let want = Operation::AndImmediateMode {
            dr: 0b101,
            sr1: 0b111,
            imm5: 0b111,
        };
        assert_eq!(want, parse_op(0b0101_101_111_1_00111));
    }

    #[test]
    fn parse_br() {
        let want = Operation::Br {
            n: true,
            z: true,
            p: false,
            pc_offset_9: 0b011111111,
        };
        assert_eq!(want, parse_op(0b0000_1_1_0_011111111));
    }

    #[test]
    fn parse_jmp() {
        let want = Operation::Jmp { base_r: 0b101 };
        assert_eq!(want, parse_op(0b1100_000_101_000000));
    }

    #[test]
    fn parse_ret() {
        let want = Operation::Ret;
        assert_eq!(want, parse_op(0b1100_000_111_000000));
    }

    #[test]
    fn parse_jsr() {
        let want = Operation::Jsr {
            pc_offset11: 0b00001111101,
        };
        assert_eq!(want, parse_op(0b0100_1_00001111101));
    }

    #[test]
    fn parse_jsrr() {
        let want = Operation::Jsrr { base_r: 0b101 };
        assert_eq!(want, parse_op(0b0100_0_00_101_000000));
    }

    #[test]
    fn parse_ld() {
        let want = Operation::Ld {
            dr: 0b101,
            pc_offset9: 0b011111111,
        };
        assert_eq!(want, parse_op(0b0010_101_011111111));
    }

    #[test]
    fn parse_ldi() {
        let want = Operation::Ldi {
            dr: 0b101,
            pc_offset9: 0b011111111,
        };
        assert_eq!(want, parse_op(0b1010_101_011111111));
    }

    #[test]
    fn parse_ldr() {
        let want = Operation::Ldr {
            dr: 0b101,
            base_r: 0b101,
            offset6: 0b010010,
        };
        assert_eq!(want, parse_op(0b0110_101_101_010010));
    }

    #[test]
    fn parse_lea() {
        let want = Operation::Lea {
            dr: 0b101,
            pc_offset9: 0b011111111,
        };
        assert_eq!(want, parse_op(0b1110_101_011111111));
    }

    #[test]
    fn parse_rti() {
        let want = Operation::Rti;
        assert_eq!(want, parse_op(0b1000_101011111111));
    }

    #[test]
    fn parse_st() {
        let want = Operation::St {
            sr: 0b111,
            pc_offset9: 0b001010010,
        };
        assert_eq!(want, parse_op(0b0011_111_001010010));
    }

    #[test]
    fn parse_sti() {
        let want = Operation::Sti {
            sr: 0b111,
            pc_offset9: 0b001010010,
        };
        assert_eq!(want, parse_op(0b1011_111_001010010));
    }

    #[test]
    fn parse_str() {
        let want = Operation::Str {
            sr: 0b111,
            base_r: 0b111,
            offset6: 65517,
        };
        assert_eq!(want, parse_op(0b0111_111_111_101101));
    }

    #[test]
    fn parse_trap() {
        let want = Operation::Trap {
            trap_vect8: 0b11111111,
        };
        assert_eq!(want, parse_op(0b1111_1110_11111111));
    }
}

#[cfg(test)]
mod test {
    use super::*;
}
