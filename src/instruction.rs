// すべての命令が定義される中心的な場所
pub enum Instruction {
    ADD(ArithmeticTarget),
    JP(JumpTest),
}

impl Instruction {
    pub fn from_byte(byte: u8, prefixed: bool) -> Option<Instruction> {
        if prefixed {
            Instruction::from_byte_prefixed(byte)
        } else {
            Instruction::from_byte_not_prefixed(byte)
        }
    }

    fn from_byte_prefixed(byte: u8) -> Option<Instruction> {
        match byte {
            _ => None,
        }
    }

    fn from_byte_not_prefixed(byte: u8) -> Option<Instruction> {
        match byte {
            _ => None,
        }
    }
}

pub enum ArithmeticTarget {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

pub enum JumpTest {
    NotZero,
    Zero,
    NotCarry,
    Carry,
    Always,
}
