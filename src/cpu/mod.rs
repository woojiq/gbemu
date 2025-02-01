mod instruction;
mod registers;

use crate::memory_bus::MemoryBus;
use instruction::Instruction;
use registers::{CpuRegisters, HALF_CARRY_MASK};

pub struct CPU {
    registers: CpuRegisters,
    memory: MemoryBus,
    pc: ProgramCounter,
    // TODO: Custom type too (as ProgramCounter).
    /// Stack pointer.
    sp: usize,
}

pub struct CpuCyclesCount(usize);

#[derive(Copy, Clone)]
struct ProgramCounter(u16);

impl CPU {
    const INSTRUCTION_PREFIX: u8 = 0xCB;

    pub fn new() -> Self {
        Self {
            registers: CpuRegisters::new(),
            memory: MemoryBus::new(),
            pc: ProgramCounter::new(),
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

    fn execute(&mut self, instruction: Instruction) -> (ProgramCounter, CpuCyclesCount) {
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
                        $var = self.$func(self.memory.read_byte(self.registers.hl()));
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
                        let new_val = self.$func_u8(self.memory.read_byte(self.registers.hl()));
                        self.memory.write_byte(self.registers.hl(), new_val);
                        (self.pc + 1, CpuCyclesCount(3))
                    }

                    // Bytes: 1; Cycles: 2;
                    instruction::IncDecTarget::SP => {
                        self.sp = self.$func_u16(self.sp as u16) as usize;
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
                    let new_val = self.add_hl(self.sp as u16);
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

            // TODO: Bitflag instructions
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

            _ => unimplemented!(),
        }
    }

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
}

impl CpuCyclesCount {
    pub fn get(&self) -> usize {
        self.0
    }
}

impl ProgramCounter {
    pub fn new() -> Self {
        Self(0)
    }
}

impl From<u16> for ProgramCounter {
    fn from(pc: u16) -> Self {
        Self(pc)
    }
}

impl From<ProgramCounter> for u16 {
    fn from(pc: ProgramCounter) -> Self {
        pc.0
    }
}

impl std::ops::Add<u16> for ProgramCounter {
    type Output = ProgramCounter;

    fn add(self, rhs: u16) -> Self::Output {
        ProgramCounter(self.0.checked_add(rhs).unwrap())
    }
}
