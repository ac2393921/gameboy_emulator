struct Registers {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
    h: u8,
    l: u8,
}

impl Registers {
    pub fn new() -> Self {
        Self {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            f: 0,
            h: 0,
            l: 0,
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_bc() {
        let mut registers = Registers::new();
        registers.b = 0x1A;
        registers.c = 0x3C;
        assert_eq!(registers.get_bc(), 0x1A3C);
    }

    #[test]
    fn test_set_bc() {
        let mut registers = Registers::new();
        registers.set_bc(0x1A3C);
        assert_eq!(registers.b, 0x1A);
        assert_eq!(registers.c, 0x3C);
    }
}

fn main() {
    println!("Hello, world!");
}
