mod instruction;
mod registers;

use crate::memory_bus::MemoryBus;
use instruction::Instruction;
use registers::{CpuRegisters, HALF_CARRY_MASK};

pub struct CPU {
    registers: CpuRegisters,
    memory: MemoryBus,
    /// Program counter.
    pc: u16,
    /// Stack pointer.
    sp: u16,
}

pub struct CpuCyclesCount(usize);

impl CPU {
    const INSTRUCTION_PREFIX: u8 = 0xCB;

    pub fn new() -> Self {
        Self {
            registers: CpuRegisters::new(),
            memory: MemoryBus::new(),
            pc: 0,
            sp: 0,
        }
    }

    pub fn cycle(&mut self) -> CpuCyclesCount {
        let instruction = self.get_current_instruction();
        let res = self.execute(instruction);
        self.pc = res.0;
        res.1
    }

    fn get_current_instruction(&self) -> Instruction {
        let byte = self.read_current_byte();
        if byte == Self::INSTRUCTION_PREFIX {
            let byte = self.read_next_byte();
            Instruction::from_byte(byte, true).unwrap()
        } else {
            Instruction::from_byte(byte, false).unwrap()
        }
    }

    fn read_current_byte(&self) -> u8 {
        self.memory.read_byte(self.pc)
    }

    fn read_next_byte(&self) -> u8 {
        self.memory.read_byte(self.pc + 1)
    }

    fn read_next_word(&self) -> u16 {
        // Little-endian
        let (lo, hi) = (
            self.memory.read_byte(self.pc + 1),
            self.memory.read_byte(self.pc + 2),
        );
        ((hi as u16) << (u8::BITS as u16)) | (lo as u16)
    }

    fn read_hl_byte(&self) -> u8 {
        self.memory.read_byte(self.registers.hl())
    }

    fn execute(&mut self, instruction: Instruction) -> (u16, CpuCyclesCount) {
        macro_rules! arithmetic_instruction {
            ($target:ident; $func:ident) => {{
                let _fake;
                arithmetic_instruction!($target; $func => _fake)
            }};
            ($target:ident; $func:ident => $var:expr) => {
                match $target {
                    // Bytes: 1; Cycles: 1;
                    instruction::ArithmeticTarget::A => {
                        $var = self.$func(self.registers.a);
                        (self.pc + 1, CpuCyclesCount(1))
                    }
                    instruction::ArithmeticTarget::B => {
                        $var = self.$func(self.registers.b);
                        (self.pc + 1, CpuCyclesCount(1))
                    }
                    instruction::ArithmeticTarget::C => {
                        $var = self.$func(self.registers.c);
                        (self.pc + 1, CpuCyclesCount(1))
                    }
                    instruction::ArithmeticTarget::D => {
                        $var = self.$func(self.registers.d);
                        (self.pc + 1, CpuCyclesCount(1))
                    }
                    instruction::ArithmeticTarget::E => {
                        $var = self.$func(self.registers.e);
                        (self.pc + 1, CpuCyclesCount(1))
                    }
                    instruction::ArithmeticTarget::H => {
                        $var = self.$func(self.registers.h);
                        (self.pc + 1, CpuCyclesCount(1))
                    }
                    instruction::ArithmeticTarget::L => {
                        $var = self.$func(self.registers.l);
                        (self.pc + 1, CpuCyclesCount(1))
                    }

                    // Bytes: 1; Cycles: 2;
                    instruction::ArithmeticTarget::HLP => {
                        $var = self.$func(self.read_hl_byte());
                        (self.pc + 1, CpuCyclesCount(2))
                    }

                    // Bytes: 2; Cycles: 2;
                    instruction::ArithmeticTarget::U8 => {
                        $var = self.$func(self.read_next_byte());
                        (self.pc + 2, CpuCyclesCount(2))
                    }
                }
            };
        }

        macro_rules! incdec_instruction {
            ($target:ident; u8: $func_u8:ident, u16: $func_u16:ident) => {
                match $target {
                    // Bytes: 1; Cycles: 1;
                    instruction::IncDecTarget::A => {
                        self.registers.a = self.$func_u8(self.registers.a);
                        (self.pc + 1, CpuCyclesCount(1))
                    }
                    instruction::IncDecTarget::B => {
                        self.registers.b = self.$func_u8(self.registers.b);
                        (self.pc + 1, CpuCyclesCount(1))
                    }
                    instruction::IncDecTarget::C => {
                        self.registers.c = self.$func_u8(self.registers.c);
                        (self.pc + 1, CpuCyclesCount(1))
                    }
                    instruction::IncDecTarget::D => {
                        self.registers.d = self.$func_u8(self.registers.d);
                        (self.pc + 1, CpuCyclesCount(1))
                    }
                    instruction::IncDecTarget::E => {
                        self.registers.e = self.$func_u8(self.registers.e);
                        (self.pc + 1, CpuCyclesCount(1))
                    }
                    instruction::IncDecTarget::H => {
                        self.registers.h = self.$func_u8(self.registers.h);
                        (self.pc + 1, CpuCyclesCount(1))
                    }
                    instruction::IncDecTarget::L => {
                        self.registers.l = self.$func_u8(self.registers.l);
                        (self.pc + 1, CpuCyclesCount(1))
                    }

                    // Bytes: 1; Cycles: 2;
                    instruction::IncDecTarget::BC => {
                        self.registers.set_bc(self.$func_u16(self.registers.bc()));
                        (self.pc + 1, CpuCyclesCount(2))
                    }
                    instruction::IncDecTarget::DE => {
                        self.registers.set_de(self.$func_u16(self.registers.de()));
                        (self.pc + 1, CpuCyclesCount(2))
                    }
                    instruction::IncDecTarget::HL => {
                        self.registers.set_hl(self.$func_u16(self.registers.hl()));
                        (self.pc + 1, CpuCyclesCount(2))
                    }

                    // Bytes: 1; Cycles: 3;
                    instruction::IncDecTarget::HLP => {
                        let new_val = self.$func_u8(self.read_hl_byte());
                        self.memory.write_byte(self.registers.hl(), new_val);
                        (self.pc + 1, CpuCyclesCount(3))
                    }

                    // Bytes: 1; Cycles: 2;
                    instruction::IncDecTarget::SP => {
                        self.sp = self.$func_u16(self.sp);
                        (self.pc + 1, CpuCyclesCount(2))
                    }
                }
            };
        }

        macro_rules! load_byte {
            ($target:ident, $source:expr) => {
                match $target {
                    instruction::LoadByteTarget::A => {
                        self.registers.a = $source;
                        (self.pc + 1, CpuCyclesCount(1))
                    }
                    instruction::LoadByteTarget::B => {
                        self.registers.b = $source;
                        (self.pc + 1, CpuCyclesCount(1))
                    }
                    instruction::LoadByteTarget::C => {
                        self.registers.c = $source;
                        (self.pc + 1, CpuCyclesCount(1))
                    }
                    instruction::LoadByteTarget::D => {
                        self.registers.d = $source;
                        (self.pc + 1, CpuCyclesCount(1))
                    }
                    instruction::LoadByteTarget::E => {
                        self.registers.e = $source;
                        (self.pc + 1, CpuCyclesCount(1))
                    }
                    instruction::LoadByteTarget::H => {
                        self.registers.h = $source;
                        (self.pc + 1, CpuCyclesCount(1))
                    }
                    instruction::LoadByteTarget::L => {
                        self.registers.l = $source;
                        (self.pc + 1, CpuCyclesCount(1))
                    }
                    instruction::LoadByteTarget::HLP => {
                        self.memory.write_byte(self.registers.hl(), $source);
                        (self.pc + 1, CpuCyclesCount(2))
                    }
                }
            };
        }

        match instruction {
            Instruction::ADD(target) => {
                arithmetic_instruction!(target; add_without_carry => self.registers.a)
            }
            Instruction::ADC(target) => {
                arithmetic_instruction!(target; add_with_carry => self.registers.a)
            }
            Instruction::SUB(target) => {
                arithmetic_instruction!(target; sub_without_carry => self.registers.a)
            }
            Instruction::SBC(target) => {
                arithmetic_instruction!(target; sub_with_carry => self.registers.a)
            }
            Instruction::CP(target) => {
                arithmetic_instruction!(target; compare)
            }

            Instruction::ADDHL(target) => match target {
                instruction::ADDHLTarget::BC => {
                    let new_val = self.add_hl(self.registers.bc());
                    self.registers.set_hl(new_val);
                    (self.pc + 1, CpuCyclesCount(2))
                }
                instruction::ADDHLTarget::DE => {
                    let new_val = self.add_hl(self.registers.de());
                    self.registers.set_hl(new_val);
                    (self.pc + 1, CpuCyclesCount(2))
                }
                instruction::ADDHLTarget::HL => {
                    let new_val = self.add_hl(self.registers.hl());
                    self.registers.set_hl(new_val);
                    (self.pc + 1, CpuCyclesCount(2))
                }
                instruction::ADDHLTarget::SP => {
                    let new_val = self.add_hl(self.sp);
                    self.registers.set_hl(new_val);
                    (self.pc + 1, CpuCyclesCount(2))
                }
            },

            Instruction::INC(target) => {
                incdec_instruction!(target; u8: increment_u8, u16: increment_u16)
            }
            Instruction::DEC(target) => {
                incdec_instruction!(target; u8: decrement_u8, u16: decrement_u16)
            }

            Instruction::BIT(pos, target) => match target {
                instruction::PrefixTarget::A => {
                    self.check_bit(self.registers.a, pos as u32);
                    (self.pc + 2, CpuCyclesCount(2))
                }
                instruction::PrefixTarget::B => {
                    self.check_bit(self.registers.b, pos as u32);
                    (self.pc + 2, CpuCyclesCount(2))
                }
                instruction::PrefixTarget::C => {
                    self.check_bit(self.registers.c, pos as u32);
                    (self.pc + 2, CpuCyclesCount(2))
                }
                instruction::PrefixTarget::D => {
                    self.check_bit(self.registers.d, pos as u32);
                    (self.pc + 2, CpuCyclesCount(2))
                }
                instruction::PrefixTarget::E => {
                    self.check_bit(self.registers.e, pos as u32);
                    (self.pc + 2, CpuCyclesCount(2))
                }
                instruction::PrefixTarget::H => {
                    self.check_bit(self.registers.h, pos as u32);
                    (self.pc + 2, CpuCyclesCount(2))
                }
                instruction::PrefixTarget::L => {
                    self.check_bit(self.registers.l, pos as u32);
                    (self.pc + 2, CpuCyclesCount(2))
                }
                instruction::PrefixTarget::HLP => {
                    self.check_bit(self.read_hl_byte(), pos as u32);
                    (self.pc + 2, CpuCyclesCount(3))
                }
            },
            Instruction::RES(pos, target) => match target {
                instruction::PrefixTarget::A => {
                    self.registers.a = self.reset_bit(self.registers.a, pos as u32);
                    (self.pc + 2, CpuCyclesCount(2))
                }
                instruction::PrefixTarget::B => {
                    self.registers.b = self.reset_bit(self.registers.b, pos as u32);
                    (self.pc + 2, CpuCyclesCount(2))
                }
                instruction::PrefixTarget::C => {
                    self.registers.c = self.reset_bit(self.registers.c, pos as u32);
                    (self.pc + 2, CpuCyclesCount(2))
                }
                instruction::PrefixTarget::D => {
                    self.registers.d = self.reset_bit(self.registers.d, pos as u32);
                    (self.pc + 2, CpuCyclesCount(2))
                }
                instruction::PrefixTarget::E => {
                    self.registers.e = self.reset_bit(self.registers.e, pos as u32);
                    (self.pc + 2, CpuCyclesCount(2))
                }
                instruction::PrefixTarget::H => {
                    self.registers.h = self.reset_bit(self.registers.h, pos as u32);
                    (self.pc + 2, CpuCyclesCount(2))
                }
                instruction::PrefixTarget::L => {
                    self.registers.l = self.reset_bit(self.registers.l, pos as u32);
                    (self.pc + 2, CpuCyclesCount(2))
                }
                instruction::PrefixTarget::HLP => {
                    self.memory.write_byte(
                        self.registers.hl(),
                        self.reset_bit(self.read_hl_byte(), pos as u32),
                    );
                    (self.pc + 2, CpuCyclesCount(4))
                }
            },
            Instruction::SET(pos, target) => match target {
                instruction::PrefixTarget::A => {
                    self.registers.a = self.set_bit(self.registers.a, pos as u32);
                    (self.pc + 2, CpuCyclesCount(2))
                }
                instruction::PrefixTarget::B => {
                    self.registers.b = self.set_bit(self.registers.b, pos as u32);
                    (self.pc + 2, CpuCyclesCount(2))
                }
                instruction::PrefixTarget::C => {
                    self.registers.c = self.set_bit(self.registers.c, pos as u32);
                    (self.pc + 2, CpuCyclesCount(2))
                }
                instruction::PrefixTarget::D => {
                    self.registers.d = self.set_bit(self.registers.d, pos as u32);
                    (self.pc + 2, CpuCyclesCount(2))
                }
                instruction::PrefixTarget::E => {
                    self.registers.e = self.set_bit(self.registers.e, pos as u32);
                    (self.pc + 2, CpuCyclesCount(2))
                }
                instruction::PrefixTarget::H => {
                    self.registers.h = self.set_bit(self.registers.h, pos as u32);
                    (self.pc + 2, CpuCyclesCount(2))
                }
                instruction::PrefixTarget::L => {
                    self.registers.l = self.set_bit(self.registers.l, pos as u32);
                    (self.pc + 2, CpuCyclesCount(2))
                }
                instruction::PrefixTarget::HLP => {
                    self.memory.write_byte(
                        self.registers.hl(),
                        self.set_bit(self.read_hl_byte(), pos as u32),
                    );
                    (self.pc + 2, CpuCyclesCount(4))
                }
            },

            Instruction::CPL => {
                self.registers.a = self.complement_accum();
                (self.pc + 1, CpuCyclesCount(1))
            }
            Instruction::AND(target) => {
                arithmetic_instruction!(target; bitwise_and => self.registers.a)
            }
            Instruction::OR(target) => {
                arithmetic_instruction!(target; bitwise_or => self.registers.a)
            }
            Instruction::XOR(target) => {
                arithmetic_instruction!(target; bitwise_xor => self.registers.a)
            }

            Instruction::SCF => {
                self.set_carry_flag(true);
                (self.pc + 1, CpuCyclesCount(1))
            }
            Instruction::CCF => {
                self.set_carry_flag(!self.registers.f.carry);
                (self.pc + 1, CpuCyclesCount(1))
            }

            #[allow(clippy::self_assignment)]
            Instruction::Load(load_type) => match load_type {
                instruction::LoadType::Byte(target, source) => match source {
                    instruction::LoadByteSource::A => load_byte!(target, self.registers.a),
                    instruction::LoadByteSource::B => load_byte!(target, self.registers.b),
                    instruction::LoadByteSource::C => load_byte!(target, self.registers.c),
                    instruction::LoadByteSource::D => load_byte!(target, self.registers.d),
                    instruction::LoadByteSource::E => load_byte!(target, self.registers.e),
                    instruction::LoadByteSource::H => load_byte!(target, self.registers.h),
                    instruction::LoadByteSource::L => load_byte!(target, self.registers.l),

                    instruction::LoadByteSource::HLP => {
                        let res = load_byte!(target, self.read_hl_byte());
                        (res.0, res.1 + CpuCyclesCount(1))
                    }
                    instruction::LoadByteSource::U8 => {
                        let res = load_byte!(target, self.read_next_byte());
                        (res.0 + 1, res.1 + CpuCyclesCount(1))
                    }
                },

                instruction::LoadType::Word(target) => {
                    let value = self.read_next_word();
                    match target {
                        instruction::LoadWordTarget::BC => self.registers.set_bc(value),
                        instruction::LoadWordTarget::DE => self.registers.set_de(value),
                        instruction::LoadWordTarget::HL => self.registers.set_hl(value),
                        instruction::LoadWordTarget::SP => self.sp = value,
                    }
                    (self.pc + 3, CpuCyclesCount(3))
                }

                instruction::LoadType::AFromIndirect(target) => match target {
                    instruction::IndirectTarget::C => {
                        self.registers.a = self.memory.read_high_byte(self.registers.c);
                        (self.pc + 1, CpuCyclesCount(2))
                    }
                    instruction::IndirectTarget::U8 => {
                        self.registers.a = self.memory.read_high_byte(self.read_next_byte());
                        (self.pc + 2, CpuCyclesCount(3))
                    }
                    instruction::IndirectTarget::U16 => {
                        self.registers.a = self.memory.read_byte(self.read_next_word());
                        (self.pc + 3, CpuCyclesCount(4))
                    }
                    instruction::IndirectTarget::BCP => {
                        self.registers.a = self.memory.read_byte(self.registers.bc());
                        (self.pc + 1, CpuCyclesCount(2))
                    }
                    instruction::IndirectTarget::DEP => {
                        self.registers.a = self.memory.read_byte(self.registers.de());
                        (self.pc + 1, CpuCyclesCount(2))
                    }
                    instruction::IndirectTarget::HLI => {
                        self.registers.a = self.memory.read_byte(self.registers.hl());
                        self.registers.set_hl(self.registers.hl() + 1);
                        (self.pc + 1, CpuCyclesCount(2))
                    }
                    instruction::IndirectTarget::HLD => {
                        self.registers.a = self.memory.read_byte(self.registers.hl());
                        self.registers.set_hl(self.registers.hl() - 1);
                        (self.pc + 1, CpuCyclesCount(2))
                    }
                },

                instruction::LoadType::IndirectFromA(target) => match target {
                    instruction::IndirectTarget::C => {
                        self.memory
                            .write_high_byte(self.registers.c, self.registers.a);
                        (self.pc + 1, CpuCyclesCount(2))
                    }
                    instruction::IndirectTarget::U8 => {
                        self.memory
                            .write_high_byte(self.read_next_byte(), self.registers.a);
                        (self.pc + 2, CpuCyclesCount(3))
                    }
                    instruction::IndirectTarget::U16 => {
                        self.memory
                            .write_byte(self.read_next_word(), self.registers.a);
                        (self.pc + 3, CpuCyclesCount(4))
                    }
                    instruction::IndirectTarget::BCP => {
                        self.memory
                            .write_byte(self.registers.bc(), self.registers.a);
                        (self.pc + 1, CpuCyclesCount(2))
                    }
                    instruction::IndirectTarget::DEP => {
                        self.memory
                            .write_byte(self.registers.de(), self.registers.a);
                        (self.pc + 1, CpuCyclesCount(2))
                    }
                    instruction::IndirectTarget::HLI => {
                        self.memory
                            .write_byte(self.registers.hl(), self.registers.a);
                        self.registers.set_hl(self.registers.hl() + 1);
                        (self.pc + 1, CpuCyclesCount(2))
                    }
                    instruction::IndirectTarget::HLD => {
                        self.memory
                            .write_byte(self.registers.hl(), self.registers.a);
                        self.registers.set_hl(self.registers.hl() - 1);
                        (self.pc + 1, CpuCyclesCount(2))
                    }
                },

                instruction::LoadType::IndirectFromSP => {
                    let addr = self.read_next_word();
                    // Little-endian
                    self.memory.write_byte(addr, self.sp as u8);
                    self.memory
                        .write_byte(addr + 1, (self.sp >> u8::BITS) as u8);
                    (self.pc + 3, CpuCyclesCount(5))
                }

                instruction::LoadType::SPFromHL => {
                    self.sp = self.registers.hl();
                    (self.pc + 1, CpuCyclesCount(2))
                }

                instruction::LoadType::HLFromSPN => {
                    // WARN: Maybe read_next_byte() as i8 as i16 as u16?
                    // Because this value is signed.
                    let val = self.read_next_byte() as u16;
                    self.registers.set_hl(val.wrapping_add(self.sp));

                    self.registers.f.zero = false;
                    self.registers.f.subtract = false;
                    // Set if overflow from bit 3.
                    self.registers.f.half_carry = (self.sp & 0xF) + (val & 0xF) > 0xF;
                    // Set if overflow from bit 7.
                    self.registers.f.carry = (self.sp & 0xFF) + (val & 0xFF) > 0xFF;

                    (self.pc + 2, CpuCyclesCount(3))
                }
            },

            // TODO: Bit shift instructions
            _ => todo!(),
        }
    }

    // https://rgbds.gbdev.io/docs/v0.9.0/gbz80.7

    fn add(&mut self, rhs: u8, include_carry: bool) -> u8 {
        let additional = (include_carry & self.registers.f.carry) as u8;

        let (res1, overflow1) = self.registers.a.overflowing_add(rhs);
        let (res2, overflow2) = res1.overflowing_add(additional);

        self.registers.f.zero = res2 == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry =
            (self.registers.a & HALF_CARRY_MASK) + (rhs & HALF_CARRY_MASK) + additional
                > HALF_CARRY_MASK;
        self.registers.f.carry = overflow1 | overflow2;

        res2
    }

    fn add_without_carry(&mut self, rhs: u8) -> u8 {
        self.add(rhs, false)
    }

    fn add_with_carry(&mut self, rhs: u8) -> u8 {
        self.add(rhs, true)
    }

    fn add_hl(&mut self, rhs: u16) -> u16 {
        let (res, overflow) = self.registers.hl().overflowing_add(rhs);

        self.registers.f.subtract = false;
        // Set if overflow from bit 11.
        self.registers.f.half_carry = (self.registers.hl() & 0xFFF) + (rhs & 0xFFF) > 0xFFF;
        self.registers.f.carry = overflow;

        res
    }

    fn sub(&mut self, rhs: u8, include_carry: bool) -> u8 {
        let additional = (include_carry & self.registers.f.carry) as u8;

        let (res1, overflow1) = self.registers.a.overflowing_sub(rhs);
        let (res2, overflow2) = res1.overflowing_sub(additional);

        self.registers.f.zero = res2 == 0;
        self.registers.f.subtract = true;
        self.registers.f.half_carry =
            (self.registers.a & HALF_CARRY_MASK) < (rhs & HALF_CARRY_MASK) + additional;
        self.registers.f.carry = overflow1 | overflow2;

        res2
    }

    fn sub_without_carry(&mut self, rhs: u8) -> u8 {
        self.sub(rhs, false)
    }

    fn sub_with_carry(&mut self, rhs: u8) -> u8 {
        self.sub(rhs, true)
    }

    fn bitwise_and(&mut self, rhs: u8) -> u8 {
        let res = self.registers.a & rhs;

        self.registers.f.zero = res == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = true;
        self.registers.f.carry = false;

        res
    }

    fn bitwise_or(&mut self, rhs: u8) -> u8 {
        let res = self.registers.a | rhs;

        self.registers.f.zero = res == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = false;

        res
    }

    fn bitwise_xor(&mut self, rhs: u8) -> u8 {
        let res = self.registers.a ^ rhs;

        self.registers.f.zero = res == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = false;

        res
    }

    fn compare(&mut self, rhs: u8) {
        self.sub_without_carry(rhs);
    }

    fn complement_accum(&mut self) -> u8 {
        self.registers.f.subtract = true;
        self.registers.f.half_carry = true;

        !self.registers.a
    }

    fn increment_u8(&mut self, val: u8) -> u8 {
        let res = val.overflowing_add(1).0;

        self.registers.f.zero = res == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = val == HALF_CARRY_MASK;

        res
    }

    fn increment_u16(&self, val: u16) -> u16 {
        val.overflowing_add(1).0
    }

    fn decrement_u8(&mut self, val: u8) -> u8 {
        let res = val.overflowing_sub(1).0;

        self.registers.f.zero = res == 0;
        self.registers.f.subtract = true;
        self.registers.f.half_carry = val & HALF_CARRY_MASK == 0;

        res
    }

    fn decrement_u16(&self, val: u16) -> u16 {
        val.overflowing_add(1).0
    }

    fn check_bit(&mut self, val: u8, bit_pos: u32) {
        self.registers.f.zero = val.checked_shr(bit_pos).unwrap() & 1 == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = true;
    }

    fn set_bit(&self, val: u8, bit_pos: u32) -> u8 {
        val | 1u8.checked_shl(bit_pos).unwrap()
    }

    fn reset_bit(&self, val: u8, bit_pos: u32) -> u8 {
        val & !1u8.checked_shl(bit_pos).unwrap()
    }

    fn set_carry_flag(&mut self, val: bool) {
        self.registers.f.subtract = false;
        self.registers.f.carry = val;
        self.registers.f.half_carry = false;
    }
}

impl CpuCyclesCount {
    pub fn get(&self) -> usize {
        self.0
    }
}

impl std::ops::Add for CpuCyclesCount {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        CpuCyclesCount(self.0 + rhs.0)
    }
}
