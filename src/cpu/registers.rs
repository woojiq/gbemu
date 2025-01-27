// If there's a combination of values of fields which breaks some invariant,
// than make all fields private and provide a getter.
pub struct Registers {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub f: FlagsRegister,
    pub h: u8,
    pub l: u8,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct FlagsRegister {
    pub zero: bool,
    pub subtract: bool,
    pub half_carry: bool,
    pub carry: bool,
}

impl Registers {
    pub fn new() -> Self {
        Self {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            f: FlagsRegister::new(),
            h: 0,
            l: 0,
        }
    }

    pub fn af(&self) -> u16 {
        (self.a as u16) << (u8::BITS as u16) | (u8::from(self.f) as u16)
    }
    pub fn set_af(&mut self, val: u16) {
        self.a = (val >> (u8::BITS as u16)) as u8;
        self.f = ((val & u8::MAX as u16) as u8).into();
    }

    pub fn bc(&self) -> u16 {
        (self.b as u16) << (u8::BITS as u16) | (self.c as u16)
    }
    pub fn set_bc(&mut self, val: u16) {
        self.b = (val >> (u8::BITS as u16)) as u8;
        self.c = (val & u8::MAX as u16) as u8;
    }

    pub fn de(&self) -> u16 {
        (self.d as u16) << (u8::BITS as u16) | (self.e as u16)
    }
    pub fn set_de(&mut self, val: u16) {
        self.d = (val >> (u8::BITS as u16)) as u8;
        self.e = (val & u8::MAX as u16) as u8;
    }

    pub fn hl(&self) -> u16 {
        (self.h as u16) << (u8::BITS as u16) | (self.l as u16)
    }
    pub fn set_hl(&mut self, val: u16) {
        self.h = (val >> (u8::BITS as u16)) as u8;
        self.l = (val & u8::MAX as u16) as u8;
    }
}

impl FlagsRegister {
    const ZERO_FLAG_POSITION: u8 = 7;
    const SUBTRACT_FLAG_POSITION: u8 = 6;
    const HALF_CARRY_FLAG_POSITION: u8 = 5;
    const CARRY_FLAG_POSITION: u8 = 4;

    pub fn new() -> Self {
        FlagsRegister {
            zero: false,
            subtract: false,
            half_carry: false,
            carry: false,
        }
    }
}

impl From<FlagsRegister> for u8 {
    fn from(flag: FlagsRegister) -> Self {
        ((flag.zero as u8) << FlagsRegister::ZERO_FLAG_POSITION)
            | ((flag.subtract as u8) << FlagsRegister::SUBTRACT_FLAG_POSITION)
            | ((flag.half_carry as u8) << FlagsRegister::HALF_CARRY_FLAG_POSITION)
            | ((flag.carry as u8) << FlagsRegister::CARRY_FLAG_POSITION)
    }
}

impl From<u8> for FlagsRegister {
    fn from(value: u8) -> Self {
        FlagsRegister {
            zero: (value >> Self::ZERO_FLAG_POSITION) & 1 == 1,
            subtract: (value >> Self::SUBTRACT_FLAG_POSITION) & 1 == 1,
            half_carry: (value >> Self::HALF_CARRY_FLAG_POSITION) & 1 == 1,
            carry: (value >> Self::CARRY_FLAG_POSITION) & 1 == 1,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[ignore]
    #[test]
    fn set_af_test() {
        unimplemented!();
    }

    #[test]
    fn set_bc_test() {
        let mut reg = Registers::new();
        reg.set_bc(0xab13);
        assert_eq!(reg.b, 0xab);
        assert_eq!(reg.c, 0x13);
        assert_eq!(reg.bc(), 0xab13);
    }

    #[test]
    fn flags_register_from_u8_test() {
        assert_eq!(
            FlagsRegister::from(0b10110000),
            FlagsRegister {
                zero: true,
                subtract: false,
                half_carry: true,
                carry: true
            }
        );
    }

    #[test]
    fn flags_register_to_u8() {
        assert_eq!(
            u8::from(FlagsRegister {
                zero: true,
                subtract: false,
                half_carry: false,
                carry: true
            }),
            0b10010000
        );
    }
}
