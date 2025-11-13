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
