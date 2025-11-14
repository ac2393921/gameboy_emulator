use crate::instruction::{
    ArithmeticTarget, Instruction, JumpTest, LoadByteSource, LoadByteTarget, LoadType,
};
use crate::memory::MemoryBus;
use crate::registers::Registers;

#[derive(Default)]
pub struct CPU {
    pub registers: Registers,
    pub pc: u16,
    pub bus: MemoryBus,
}

impl CPU {
    pub fn execute(&mut self, instruction: Instruction) -> u16 {
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
            Instruction::LD(load_type) => match load_type {
                LoadType::Byte(target, source) => {
                    let source_value = match source {
                        LoadByteSource::A => self.registers.a,
                        LoadByteSource::B => self.registers.b,
                        LoadByteSource::C => self.registers.c,
                        LoadByteSource::D => self.registers.d,
                        LoadByteSource::E => self.registers.e,
                        LoadByteSource::H => self.registers.h,
                        LoadByteSource::L => self.registers.l,
                        LoadByteSource::D8 => self.read_next_byte(),
                        LoadByteSource::HLI => self.bus.read_byte(self.registers.get_hl()),
                    };
                    match target {
                        LoadByteTarget::A => self.registers.a = source_value,
                        LoadByteTarget::B => self.registers.b = source_value,
                        LoadByteTarget::C => self.registers.c = source_value,
                        LoadByteTarget::D => self.registers.d = source_value,
                        LoadByteTarget::E => self.registers.e = source_value,
                        LoadByteTarget::H => self.registers.h = source_value,
                        LoadByteTarget::L => self.registers.l = source_value,
                        LoadByteTarget::HLI => {
                            self.bus.write_byte(self.registers.get_hl(), source_value)
                        }
                    }
                    match source {
                        LoadByteSource::D8 => self.pc.wrapping_add(2),
                        _ => self.pc.wrapping_add(1),
                    }
                }
            },
        }
    }

    fn read_next_byte(&mut self) -> u8 {
        let byte = self.bus.read_byte(self.pc);
        self.pc = self.pc.wrapping_add(1);
        byte
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

    pub fn step(&mut self) {
        let mut instruction_byte = self.bus.read_byte(self.pc);
        let prefixed = instruction_byte == 0xCB;
        if prefixed {
            instruction_byte = self.bus.read_byte(self.pc + 1);
        }
        let next_pc = if let Some(instruction) = Instruction::from_byte(instruction_byte, prefixed)
        {
            self.execute(instruction)
        } else {
            let description = format!(
                "0x{}{:02X}",
                if prefixed { "CB" } else { "" },
                instruction_byte
            );
            panic!("Unkown instruction found for: {}", description)
        };

        self.pc = next_pc;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    // LD命令のテスト: レジスタ間のロード
    #[test]
    fn test_ld_register_to_register() {
        let mut cpu = CPU::default();
        cpu.pc = 0x0100;
        cpu.registers.b = 0x42;
        let next_pc = cpu.execute(Instruction::LD(
            LoadType::Byte(LoadByteTarget::A, LoadByteSource::B),
        ));
        assert_eq!(cpu.registers.a, 0x42);
        assert_eq!(next_pc, 0x0101);
    }

    // LD命令のテスト: 即値（D8）からレジスタへのロード
    // #[test]
    // fn test_ld_immediate_to_register() {
    //     let mut cpu = CPU::default();
    //     cpu.pc = 0x0200;
    //     cpu.bus.memory[0x0200] = 0xAB; // D8の値
    //     let next_pc = cpu.execute(Instruction::LD(
    //         LoadType::Byte(LoadByteTarget::C, LoadByteSource::D8),
    //     ));
    //     assert_eq!(cpu.registers.c, 0xAB);
    //     assert_eq!(next_pc, 0x0202); // D8の場合は2バイト進む
    // }

    // LD命令のテスト: メモリ（HLI）からレジスタへのロード
    #[test]
    fn test_ld_memory_to_register() {
        let mut cpu = CPU::default();
        cpu.pc = 0x0300;
        cpu.registers.set_hl(0x1000);
        cpu.bus.memory[0x1000] = 0xCD;
        let next_pc = cpu.execute(Instruction::LD(
            LoadType::Byte(LoadByteTarget::D, LoadByteSource::HLI),
        ));
        assert_eq!(cpu.registers.d, 0xCD);
        assert_eq!(next_pc, 0x0301);
    }

    // LD命令のテスト: レジスタからメモリ（HLI）へのロード
    #[test]
    fn test_ld_register_to_memory() {
        let mut cpu = CPU::default();
        cpu.pc = 0x0400;
        cpu.registers.set_hl(0x2000);
        cpu.registers.e = 0xEF;
        let next_pc = cpu.execute(Instruction::LD(
            LoadType::Byte(LoadByteTarget::HLI, LoadByteSource::E),
        ));
        assert_eq!(cpu.bus.memory[0x2000], 0xEF);
        assert_eq!(next_pc, 0x0401);
    }

    // LD命令のテスト: 複数のレジスタ間ロード
    #[test]
    fn test_ld_multiple_registers() {
        let mut cpu = CPU::default();
        cpu.pc = 0x0500;
        cpu.registers.h = 0x12;
        cpu.registers.l = 0x34;
        let next_pc = cpu.execute(Instruction::LD(
            LoadType::Byte(LoadByteTarget::A, LoadByteSource::H),
        ));
        assert_eq!(cpu.registers.a, 0x12);
        assert_eq!(next_pc, 0x0501);

        cpu.pc = 0x0501;
        let next_pc = cpu.execute(Instruction::LD(
            LoadType::Byte(LoadByteTarget::B, LoadByteSource::L),
        ));
        assert_eq!(cpu.registers.b, 0x34);
        assert_eq!(next_pc, 0x0502);
    }
}
