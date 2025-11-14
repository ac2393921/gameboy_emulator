#[derive(Default)]
struct Registers {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: FlagsRegister,
    h: u8,
    l: u8,
}

impl Registers {
    fn get_bc(&self) -> u16 {
        // bを左に8ビットシフトしてcと論理和を取り、u16にキャスト
        // b: 10101010 c: 11001100 -> bc: 1010101011001100
        (self.b as u16) << 8 | self.c as u16
    }

    fn set_bc(&mut self, value: u16) {
        // valueを0xFF00と論理積を取り、8ビット右にシフトしてbにキャスト
        // value: 1010101011001100 -> b: 10101010
        // valueを0xFFと論理積を取り、cにキャスト
        // value: 1010101011001100 -> c: 11001100
        self.b = ((value & 0xFF00) >> 8) as u8;
        self.c = (value & 0xFF) as u8;
    }
}

#[derive(Default, PartialEq, Debug)]
struct FlagsRegister {
    zero: bool,
    subtract: bool,
    half_carry: bool,
    carry: bool,
}

const ZERO_FLAG_BYTE_POSITION: u8 = 7;
const SUBTRACT_FLAG_BYTE_POSITION: u8 = 6;
const HALF_CARRY_FLAG_BYTE_POSITION: u8 = 5;
const CARRY_FLAG_BYTE_POSITION: u8 = 4;

impl std::convert::From<FlagsRegister> for u8 {
    fn from(flag: FlagsRegister) -> u8 {
        (if flag.zero { 1 } else { 0 } << ZERO_FLAG_BYTE_POSITION)
            | (if flag.subtract { 1 } else { 0 } << SUBTRACT_FLAG_BYTE_POSITION)
            | (if flag.half_carry { 1 } else { 0 } << HALF_CARRY_FLAG_BYTE_POSITION)
            | (if flag.carry { 1 } else { 0 } << CARRY_FLAG_BYTE_POSITION)
    }
}

impl std::convert::From<u8> for FlagsRegister {
    fn from(byte: u8) -> FlagsRegister {
        let zero = ((byte >> ZERO_FLAG_BYTE_POSITION) & 0x01) != 0;
        let subtract = ((byte >> SUBTRACT_FLAG_BYTE_POSITION) & 0x01) != 0;
        let half_carry = ((byte >> HALF_CARRY_FLAG_BYTE_POSITION) & 0x01) != 0;
        let carry = ((byte >> CARRY_FLAG_BYTE_POSITION) & 0x01) != 0;

        FlagsRegister {
            zero,
            subtract,
            half_carry,
            carry,
        }
    }
}

// すべての命令が定義される中心的な場所
enum Instruction {
    ADD(ArithmeticTarget),
    JP(JumpTest),
}

impl Instruction {
    fn from_byte(byte: u8, prefixed: bool) -> Option<Instruction> {
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

enum ArithmeticTarget {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

#[derive(Default)]
struct CPU {
    registers: Registers,
    pc: u16,
    bus: MemoryBus,
}

impl CPU {
    fn execute(&mut self, instruction: Instruction) -> u16 {
        match instruction {
            Instruction::JP(test) => {
                let jump_condition = match test {
                    JumpTest::NotZero => !self.registers.f.zero,
                    JumpTest::Zero => self.registers.f.zero,
                    JumpTest::NotCarry => !self.registers.f.carry,
                    JumpTest::Carry => self.registers.f.carry,
                    JumpTest::Always => true,
                };
                self.jump(jump_condition)
            }
            Instruction::ADD(target) => match target {
                ArithmeticTarget::A => self.pc,
                ArithmeticTarget::B => self.pc,
                ArithmeticTarget::C => {
                    let value = self.registers.c;
                    let new_value = self.add(value);
                    self.registers.a = new_value;
                    self.pc.wrapping_add(1)
                }
                ArithmeticTarget::D => self.pc,
                ArithmeticTarget::E => self.pc,
                ArithmeticTarget::H => self.pc,
                ArithmeticTarget::L => self.pc,
            },
        }
    }

    fn add(&mut self, value: u8) -> u8 {
        let (new_value, did_overflow) = self.registers.a.overflowing_add(value);
        self.registers.f.zero = new_value == 0;
        // addなのでsubtractはfalse
        self.registers.f.subtract = false;
        // オーバーフローが発生したらcarryはtrue
        self.registers.f.carry = did_overflow;
        //下位ニブルの和が0xFを超えたらhalf_carryはtrue
        // 上位ニブルをマスクして0xFと論理和を取り、0xFを超えたらtrue
        self.registers.f.half_carry = (self.registers.a & 0xF) + (value & 0xF) > 0xF;
        new_value
    }

    // should_jumpがtrueの場合はジャンプ命令の次と次に飛び先が書いてあるから、飛び先を取得する
    // should_jumpがfalseの場合は２バイトを無視しないといけないので3バイト進める
    // +-------------+-------------- +--------------+
    // | Instruction | Least Signif- | Most Signif- |
    // | Identifier  | icant Byte    | icant Byte   |
    // +-------------+-------------- +--------------+
    fn jump(&self, should_jump: bool) -> u16 {
        if should_jump {
            // 16ビットのアドレスを取得する
            // 最下位バイトはself.pc + 1のアドレスを読み込む
            // 最上位バイトはself.pc + 2のアドレスを読み込む
            // それらを組み合わせて16ビットのアドレスを取得する
            let least_significant_byte = self.bus.read_byte(self.pc + 1) as u16;
            let most_significant_byte = self.bus.read_byte(self.pc + 2) as u16;
            // little endianなので最下位バイトが先に来る
            (most_significant_byte << 8) | least_significant_byte
        } else {
            // ジャンプしない場合は3バイト進める
            self.pc.wrapping_add(3)
        }
    }

    fn step(&mut self) {
        let mut instruction_byte = self.bus.read_byte(self.pc);
        let prefixed = instruction_byte == 0xCB;
        if prefixed {
            instruction_byte = self.bus.read_byte(self.pc + 1);
        }
        let next_pc = if let Some(instruction) = Instruction::from_byte(instruction_byte, prefixed) {
            self.execute(instruction)
        } else {
            let description = format!("0x{}{:02X}", if prefixed { "CB" } else { "" }, instruction_byte);
            panic!("Unkown instruction found for: {}", description)
        };

        self.pc = next_pc;
    }
}

struct MemoryBus {
    memory: [u8; 0xFFFF],
}

impl MemoryBus {
    fn read_byte(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }
}

impl Default for MemoryBus {
    fn default() -> Self {
        Self {
            memory: [0; 0xFFFF],
        }
    }
}

enum JumpTest {
    NotZero,
    Zero,
    NotCarry,
    Carry,
    Always,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_bc() {
        let mut registers = Registers::default();
        registers.b = 0x1A;
        registers.c = 0x3C;
        assert_eq!(registers.get_bc(), 0x1A3C);
    }

    #[test]
    fn test_set_bc() {
        let mut registers = Registers::default();
        registers.set_bc(0x1A3C);
        assert_eq!(registers.b, 0x1A);
        assert_eq!(registers.c, 0x3C);
    }

    #[test]
    fn test_flags_register_from_u8() {
        let flag = FlagsRegister {
            zero: true,
            subtract: false,
            half_carry: true,
            carry: false,
        };
        assert_eq!(u8::from(flag), 0b10100000);
    }

    #[test]
    fn test_u8_from_flags_register() {
        let u8_value = 0b10100000;
        assert_eq!(
            FlagsRegister::from(u8_value),
            FlagsRegister {
                zero: true,
                subtract: false,
                half_carry: true,
                carry: false
            }
        );
    }

    // addでオーバーフローが発生しない場合のテスト
    #[test]
    fn test_add_no_overflow() {
        let mut cpu = CPU::default();
        cpu.registers.a = 0x01;
        cpu.registers.c = 0x02;
        cpu.execute(Instruction::ADD(ArithmeticTarget::C));
        assert_eq!(cpu.registers.a, 0x03);
    }

    // addでオーバーフローが発生する場合のテスト
    #[test]
    fn test_add_overflow() {
        let mut cpu = CPU::default();
        cpu.registers.a = 0xFF;
        cpu.registers.c = 0x01;
        cpu.execute(Instruction::ADD(ArithmeticTarget::C));
        assert_eq!(cpu.registers.a, 0x00);
    }

    // addでhalf_carryが発生する場合のテスト
    #[test]
    fn test_add_half_carry() {
        let mut cpu = CPU::default();
        cpu.registers.a = 0x0F;
        cpu.registers.c = 0x01;
        cpu.execute(Instruction::ADD(ArithmeticTarget::C));
        assert_eq!(cpu.registers.a, 0x10);
    }

    #[test]
    fn test_jump_not_zero_taken() {
        let mut cpu = CPU::default();
        cpu.pc = 0x0100;
        cpu.registers.f.zero = false;
        cpu.bus.memory[0x0101] = 0x34;
        cpu.bus.memory[0x0102] = 0x12;

        let next_pc = cpu.execute(Instruction::JP(JumpTest::NotZero));
        assert_eq!(next_pc, 0x1234);
    }

    #[test]
    fn test_jump_not_zero_not_taken() {
        let mut cpu = CPU::default();
        cpu.pc = 0x0100;
        cpu.registers.f.zero = true;

        let next_pc = cpu.execute(Instruction::JP(JumpTest::NotZero));
        assert_eq!(next_pc, 0x0103);
    }

    #[test]
    fn test_jump_carry_taken() {
        let mut cpu = CPU::default();
        cpu.pc = 0x0200;
        cpu.registers.f.carry = true;
        cpu.bus.memory[0x0201] = 0x78;
        cpu.bus.memory[0x0202] = 0x56;

        let next_pc = cpu.execute(Instruction::JP(JumpTest::Carry));
        assert_eq!(next_pc, 0x5678);
    }

    #[test]
    fn test_jump_carry_not_taken() {
        let mut cpu = CPU::default();
        cpu.pc = 0x0200;
        cpu.registers.f.carry = false;

        let next_pc = cpu.execute(Instruction::JP(JumpTest::Carry));
        assert_eq!(next_pc, 0x0203);
    }

    #[test]
    fn test_jump_always() {
        let mut cpu = CPU::default();
        cpu.pc = 0x0300;
        cpu.bus.memory[0x0301] = 0xAA;
        cpu.bus.memory[0x0302] = 0xBB;

        let next_pc = cpu.execute(Instruction::JP(JumpTest::Always));
        assert_eq!(next_pc, 0xBBAA);
    }

    #[test]
    #[should_panic(expected = "Unkown instruction found for: 0x00")]
    fn test_step_non_prefixed_unknown_instruction() {
        let mut cpu = CPU::default();
        cpu.bus.memory[0] = 0x00; // 未知の非プレフィックス命令
        cpu.step();
    }

    #[test]
    #[should_panic(expected = "Unkown instruction found for: 0xCB00")]
    fn test_step_prefixed_unknown_instruction() {
        let mut cpu = CPU::default();
        cpu.bus.memory[0] = 0xCB;
        cpu.bus.memory[1] = 0x00; // 未知のプレフィックス命令
        cpu.step();
    }
}

fn main() {
    println!("Hello, world!");
}
