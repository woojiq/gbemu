use crate::bit;

pub const HALF_CARRY_MASK: u8 = 0xF;

// If there's a combination of values of fields which breaks some invariant,
// than make all fields private and provide a getter.
pub struct CpuRegisters {
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
    /// true if there is an overflow from the lower nibble (a.k.a the lower four bits).
    pub half_carry: bool,
    pub carry: bool,
}

impl CpuRegisters {
    pub fn new() -> Self {
        Self {
            a: 0x01,
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xD8,
            f: FlagsRegister::new(),
            h: 0x01,
            l: 0x4D,
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
            zero: true,
            subtract: false,
            half_carry: true,
            carry: true,
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
            zero: bit!(value, Self::ZERO_FLAG_POSITION),
            subtract: bit!(value, Self::SUBTRACT_FLAG_POSITION),
            half_carry: bit!(value, Self::HALF_CARRY_FLAG_POSITION),
            carry: bit!(value, Self::CARRY_FLAG_POSITION),
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
        let mut reg = CpuRegisters::new();
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
