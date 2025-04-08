mod instruction;
mod registers;

pub use crate::joypad::JoypadKey;
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
    is_halted: bool,
    interrupts_enabled: bool,
    // Counters to schedule enable/disable IME.
    di_timer: u8,
    ei_timer: u8,
}

impl CPU {
    const INSTRUCTION_PREFIX: u8 = 0xCB;

    pub fn new(game_rom: &[u8]) -> Self {
        Self {
            registers: CpuRegisters::new(),
            memory: MemoryBus::new(game_rom),
            pc: 0x100,
            sp: 0xFFFE,
            is_halted: false,
            interrupts_enabled: true,
            di_timer: 0,
            ei_timer: 0,
        }
    }

    pub fn cycle(&mut self) -> u32 {
        // eprintln!(
        //     "PC 0x{:X} SP 0x{:X}, INS 0x{:X}, NX 0x{:X}: {} {} {} {} {} {} {}, INTF {:b}, LINE {}, {}",
        //     self.pc,
        //     self.sp,
        //     self.read_current_byte(),
        //     self.read_next_byte(),
        //     self.registers.a,
        //     self.registers.b,
        //     self.registers.c,
        //     self.registers.d,
        //     self.registers.e,
        //     u8::from(self.registers.f),
        //     self.registers.hl(),
        //     u8::from(self.memory.interrupt_flag),
        //     self.memory.gpu.lcd_status.ly(),
        //     self.memory.gpu.cycles,
        // );

        self.update_ime();

        let cycles = self.process_interrupts();
        if cycles != 0 {
            return self.memory.step(cycles);
        }

        let instruction = self.get_current_instruction();

        // log::trace!("Parsed instruction {instruction:?}.");

        let (new_pc, cycles) = self.execute(instruction);

        // eprintln!(
        //     "Instruction {instruction:?} executed, cycles = {cycles}, new_pc = 0x{new_pc:X}."
        // );

        self.pc = new_pc;

        self.memory.step(cycles)
    }

    pub fn key_up(&mut self, key: JoypadKey) {
        self.memory.key_up(key);
    }

    pub fn key_down(&mut self, key: JoypadKey) {
        self.memory.key_down(key);
    }

    pub fn gpu(&self) -> &crate::gpu::GPU {
        &self.memory.gpu
    }

    // https://gbdev.io/pandocs/Interrupts.html#ime-interrupt-master-enable-flag-write-only
    // The effect of ei is delayed by one instruction. This means that ei followed immediately
    // by di does not allow any interrupts between them. This interacts with the halt bug in an
    // interesting way.
    fn update_ime(&mut self) {
        if self.di_timer == 1 {
            self.interrupts_enabled = false;
        }
        self.di_timer = self.di_timer.saturating_sub(1);

        if self.ei_timer == 1 {
            self.interrupts_enabled = true;
        }
        self.ei_timer = self.ei_timer.saturating_sub(1);
    }

    fn process_interrupts(&mut self) -> u32 {
        // dbg!(self.interrupts_enabled);
        if !self.interrupts_enabled {
            return 0;
        }

        if self.memory.vbank_interrupt() {
            self.memory.reset_vbank_interrupt();
            self.interrupt(0x40);
        } else if self.memory.lcd_interrupt() {
            self.memory.reset_lcd_interrupt();
            self.interrupt(0x48);
        } else if self.memory.timer_interrupt() {
            self.memory.reset_timer_interrupt();
            self.interrupt(0x50);
        } else if self.memory.serial_interrupt() {
            self.memory.reset_serial_interrupt();
            self.interrupt(0x58);
        } else if self.memory.joypad_interrupt() {
            self.memory.reset_joypad_interrupt();
            self.interrupt(0x60);
        } else {
            return 0;
        }

        // TODO: Change to 5: https://gbdev.io/pandocs/Interrupts.html#interrupt-handling
        4 * 4
    }

    fn interrupt(&mut self, addr: u16) {
        self.interrupts_enabled = false;
        // dbg!(addr);
        self.push_stack(self.pc);
        self.pc = addr;
    }

    fn get_current_instruction(&self) -> Instruction {
        let byte = self.read_current_byte();
        if byte == Self::INSTRUCTION_PREFIX {
            let byte = self.read_next_byte();
            Instruction::from_byte(byte, true)
                .unwrap_or_else(|| panic!("Prefixed instruction 0x{byte:X} exists"))
        } else {
            Instruction::from_byte(byte, false)
                .unwrap_or_else(|| panic!("Not prefixed instruction 0x{byte:X} exists"))
        }
    }

    fn read_current_byte(&self) -> u8 {
        self.memory.read_byte(self.pc)
    }

    fn read_next_byte(&self) -> u8 {
        self.memory.read_byte(self.pc.wrapping_add(1))
    }

    fn read_next_word(&self) -> u16 {
        // Little-endian
        let (lo, hi) = (
            self.memory.read_byte(self.pc.wrapping_add(1)),
            self.memory.read_byte(self.pc.wrapping_add(2)),
        );
        ((hi as u16) << (u8::BITS as u16)) | (lo as u16)
    }

    fn read_hl_byte(&self) -> u8 {
        self.memory.read_byte(self.registers.hl())
    }

    fn execute(&mut self, instruction: Instruction) -> (u16, u32) {
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
                        (self.pc.wrapping_add(1), 1)
                    }
                    instruction::ArithmeticTarget::B => {
                        $var = self.$func(self.registers.b);
                        (self.pc.wrapping_add(1), 1)
                    }
                    instruction::ArithmeticTarget::C => {
                        $var = self.$func(self.registers.c);
                        (self.pc.wrapping_add(1), 1)
                    }
                    instruction::ArithmeticTarget::D => {
                        $var = self.$func(self.registers.d);
                        (self.pc.wrapping_add(1), 1)
                    }
                    instruction::ArithmeticTarget::E => {
                        $var = self.$func(self.registers.e);
                        (self.pc.wrapping_add(1), 1)
                    }
                    instruction::ArithmeticTarget::H => {
                        $var = self.$func(self.registers.h);
                        (self.pc.wrapping_add(1), 1)
                    }
                    instruction::ArithmeticTarget::L => {
                        $var = self.$func(self.registers.l);
                        (self.pc.wrapping_add(1), 1)
                    }

                    // Bytes: 1; Cycles: 2;
                    instruction::ArithmeticTarget::HLP => {
                        $var = self.$func(self.read_hl_byte());
                        (self.pc.wrapping_add(1), 2)
                    }

                    // Bytes: 2; Cycles: 2;
                    instruction::ArithmeticTarget::U8 => {
                        $var = self.$func(self.read_next_byte());
                        (self.pc.wrapping_add(2), 2)
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
                        (self.pc.wrapping_add(1), 1)
                    }
                    instruction::IncDecTarget::B => {
                        self.registers.b = self.$func_u8(self.registers.b);
                        (self.pc.wrapping_add(1), 1)
                    }
                    instruction::IncDecTarget::C => {
                        self.registers.c = self.$func_u8(self.registers.c);
                        (self.pc.wrapping_add(1), 1)
                    }
                    instruction::IncDecTarget::D => {
                        self.registers.d = self.$func_u8(self.registers.d);
                        (self.pc.wrapping_add(1), 1)
                    }
                    instruction::IncDecTarget::E => {
                        self.registers.e = self.$func_u8(self.registers.e);
                        (self.pc.wrapping_add(1), 1)
                    }
                    instruction::IncDecTarget::H => {
                        self.registers.h = self.$func_u8(self.registers.h);
                        (self.pc.wrapping_add(1), 1)
                    }
                    instruction::IncDecTarget::L => {
                        self.registers.l = self.$func_u8(self.registers.l);
                        (self.pc.wrapping_add(1), 1)
                    }

                    // Bytes: 1; Cycles: 2;
                    instruction::IncDecTarget::BC => {
                        self.registers.set_bc(self.$func_u16(self.registers.bc()));
                        (self.pc.wrapping_add(1), 2)
                    }
                    instruction::IncDecTarget::DE => {
                        self.registers.set_de(self.$func_u16(self.registers.de()));
                        (self.pc.wrapping_add(1), 2)
                    }
                    instruction::IncDecTarget::HL => {
                        self.registers.set_hl(self.$func_u16(self.registers.hl()));
                        (self.pc.wrapping_add(1), 2)
                    }

                    // Bytes: 1; Cycles: 3;
                    instruction::IncDecTarget::HLP => {
                        let new_val = self.$func_u8(self.read_hl_byte());
                        self.memory.write_byte(self.registers.hl(), new_val);
                        (self.pc.wrapping_add(1), 3)
                    }

                    // Bytes: 1; Cycles: 2;
                    instruction::IncDecTarget::SP => {
                        self.sp = self.$func_u16(self.sp);
                        (self.pc.wrapping_add(1), 2)
                    }
                }
            };
        }

        macro_rules! load_byte {
            ($target:ident, $source:expr) => {
                match $target {
                    instruction::LoadByteTarget::A => {
                        self.registers.a = $source;
                        (self.pc.wrapping_add(1), 1)
                    }
                    instruction::LoadByteTarget::B => {
                        self.registers.b = $source;
                        (self.pc.wrapping_add(1), 1)
                    }
                    instruction::LoadByteTarget::C => {
                        self.registers.c = $source;
                        (self.pc.wrapping_add(1), 1)
                    }
                    instruction::LoadByteTarget::D => {
                        self.registers.d = $source;
                        (self.pc.wrapping_add(1), 1)
                    }
                    instruction::LoadByteTarget::E => {
                        self.registers.e = $source;
                        (self.pc.wrapping_add(1), 1)
                    }
                    instruction::LoadByteTarget::H => {
                        self.registers.h = $source;
                        (self.pc.wrapping_add(1), 1)
                    }
                    instruction::LoadByteTarget::L => {
                        self.registers.l = $source;
                        (self.pc.wrapping_add(1), 1)
                    }
                    instruction::LoadByteTarget::HLP => {
                        self.memory.write_byte(self.registers.hl(), $source);
                        (self.pc.wrapping_add(1), 2)
                    }
                }
            };
        }

        macro_rules! bit_shift_instruction {
            ($target:ident; $func:ident: $($opt:expr),*) => {
                match $target {
                    instruction::PrefixTarget::A => {
                        self.registers.a = self.$func(self.registers.a, $($opt),*);
                        (self.pc.wrapping_add( 2), 2)
                    }
                    instruction::PrefixTarget::B => {
                        self.registers.b = self.$func(self.registers.b, $($opt),*);
                        (self.pc.wrapping_add( 2), 2)
                    }
                    instruction::PrefixTarget::C => {
                        self.registers.c = self.$func(self.registers.c, $($opt),*);
                        (self.pc.wrapping_add( 2), 2)
                    }
                    instruction::PrefixTarget::D => {
                        self.registers.d = self.$func(self.registers.d, $($opt),*);
                        (self.pc.wrapping_add( 2), 2)
                    }
                    instruction::PrefixTarget::E => {
                        self.registers.e = self.$func(self.registers.e, $($opt),*);
                        (self.pc.wrapping_add( 2), 2)
                    }
                    instruction::PrefixTarget::H => {
                        self.registers.h = self.$func(self.registers.h, $($opt),*);
                        (self.pc.wrapping_add( 2), 2)
                    }
                    instruction::PrefixTarget::L => {
                        self.registers.l = self.$func(self.registers.l, $($opt),*);
                        (self.pc.wrapping_add( 2), 2)
                    }
                    instruction::PrefixTarget::HLP => {
                        let new_val = self.$func(self.read_hl_byte(), $($opt),*);
                        self.memory.write_byte(self.registers.hl(), new_val);
                        (self.pc.wrapping_add( 2), 4)
                    }
                }
            };
        }

        let res = match instruction {
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
                    (self.pc.wrapping_add(1), 2)
                }
                instruction::ADDHLTarget::DE => {
                    let new_val = self.add_hl(self.registers.de());
                    self.registers.set_hl(new_val);
                    (self.pc.wrapping_add(1), 2)
                }
                instruction::ADDHLTarget::HL => {
                    let new_val = self.add_hl(self.registers.hl());
                    self.registers.set_hl(new_val);
                    (self.pc.wrapping_add(1), 2)
                }
                instruction::ADDHLTarget::SP => {
                    let new_val = self.add_hl(self.sp);
                    self.registers.set_hl(new_val);
                    (self.pc.wrapping_add(1), 2)
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
                    (self.pc.wrapping_add(2), 2)
                }
                instruction::PrefixTarget::B => {
                    self.check_bit(self.registers.b, pos as u32);
                    (self.pc.wrapping_add(2), 2)
                }
                instruction::PrefixTarget::C => {
                    self.check_bit(self.registers.c, pos as u32);
                    (self.pc.wrapping_add(2), 2)
                }
                instruction::PrefixTarget::D => {
                    self.check_bit(self.registers.d, pos as u32);
                    (self.pc.wrapping_add(2), 2)
                }
                instruction::PrefixTarget::E => {
                    self.check_bit(self.registers.e, pos as u32);
                    (self.pc.wrapping_add(2), 2)
                }
                instruction::PrefixTarget::H => {
                    self.check_bit(self.registers.h, pos as u32);
                    (self.pc.wrapping_add(2), 2)
                }
                instruction::PrefixTarget::L => {
                    self.check_bit(self.registers.l, pos as u32);
                    (self.pc.wrapping_add(2), 2)
                }
                instruction::PrefixTarget::HLP => {
                    self.check_bit(self.read_hl_byte(), pos as u32);
                    (self.pc.wrapping_add(2), 3)
                }
            },
            Instruction::RES(pos, target) => match target {
                instruction::PrefixTarget::A => {
                    self.registers.a = self.reset_bit(self.registers.a, pos as u32);
                    (self.pc.wrapping_add(2), 2)
                }
                instruction::PrefixTarget::B => {
                    self.registers.b = self.reset_bit(self.registers.b, pos as u32);
                    (self.pc.wrapping_add(2), 2)
                }
                instruction::PrefixTarget::C => {
                    self.registers.c = self.reset_bit(self.registers.c, pos as u32);
                    (self.pc.wrapping_add(2), 2)
                }
                instruction::PrefixTarget::D => {
                    self.registers.d = self.reset_bit(self.registers.d, pos as u32);
                    (self.pc.wrapping_add(2), 2)
                }
                instruction::PrefixTarget::E => {
                    self.registers.e = self.reset_bit(self.registers.e, pos as u32);
                    (self.pc.wrapping_add(2), 2)
                }
                instruction::PrefixTarget::H => {
                    self.registers.h = self.reset_bit(self.registers.h, pos as u32);
                    (self.pc.wrapping_add(2), 2)
                }
                instruction::PrefixTarget::L => {
                    self.registers.l = self.reset_bit(self.registers.l, pos as u32);
                    (self.pc.wrapping_add(2), 2)
                }
                instruction::PrefixTarget::HLP => {
                    self.memory.write_byte(
                        self.registers.hl(),
                        self.reset_bit(self.read_hl_byte(), pos as u32),
                    );
                    (self.pc.wrapping_add(2), 4)
                }
            },
            Instruction::SET(pos, target) => match target {
                instruction::PrefixTarget::A => {
                    self.registers.a = self.set_bit(self.registers.a, pos as u32);
                    (self.pc.wrapping_add(2), 2)
                }
                instruction::PrefixTarget::B => {
                    self.registers.b = self.set_bit(self.registers.b, pos as u32);
                    (self.pc.wrapping_add(2), 2)
                }
                instruction::PrefixTarget::C => {
                    self.registers.c = self.set_bit(self.registers.c, pos as u32);
                    (self.pc.wrapping_add(2), 2)
                }
                instruction::PrefixTarget::D => {
                    self.registers.d = self.set_bit(self.registers.d, pos as u32);
                    (self.pc.wrapping_add(2), 2)
                }
                instruction::PrefixTarget::E => {
                    self.registers.e = self.set_bit(self.registers.e, pos as u32);
                    (self.pc.wrapping_add(2), 2)
                }
                instruction::PrefixTarget::H => {
                    self.registers.h = self.set_bit(self.registers.h, pos as u32);
                    (self.pc.wrapping_add(2), 2)
                }
                instruction::PrefixTarget::L => {
                    self.registers.l = self.set_bit(self.registers.l, pos as u32);
                    (self.pc.wrapping_add(2), 2)
                }
                instruction::PrefixTarget::HLP => {
                    self.memory.write_byte(
                        self.registers.hl(),
                        self.set_bit(self.read_hl_byte(), pos as u32),
                    );
                    (self.pc.wrapping_add(2), 4)
                }
            },

            Instruction::CPL => {
                self.registers.a = self.complement_accum();
                (self.pc.wrapping_add(1), 1)
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
                (self.pc.wrapping_add(1), 1)
            }
            Instruction::CCF => {
                self.set_carry_flag(!self.registers.f.carry);
                (self.pc.wrapping_add(1), 1)
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
                        (res.0, res.1 + 1)
                    }
                    instruction::LoadByteSource::U8 => {
                        let res = load_byte!(target, self.read_next_byte());
                        (res.0 + 1, res.1 + 1)
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
                    (self.pc.wrapping_add(3), 3)
                }

                instruction::LoadType::AFromIndirect(target) => match target {
                    instruction::IndirectTarget::C => {
                        self.registers.a = self.memory.read_high_byte(self.registers.c);
                        (self.pc.wrapping_add(1), 2)
                    }
                    instruction::IndirectTarget::U8 => {
                        self.registers.a = self.memory.read_high_byte(self.read_next_byte());
                        (self.pc.wrapping_add(2), 3)
                    }
                    instruction::IndirectTarget::U16 => {
                        self.registers.a = self.memory.read_byte(self.read_next_word());
                        (self.pc.wrapping_add(3), 4)
                    }
                    instruction::IndirectTarget::BCP => {
                        self.registers.a = self.memory.read_byte(self.registers.bc());
                        (self.pc.wrapping_add(1), 2)
                    }
                    instruction::IndirectTarget::DEP => {
                        self.registers.a = self.memory.read_byte(self.registers.de());
                        (self.pc.wrapping_add(1), 2)
                    }
                    instruction::IndirectTarget::HLI => {
                        self.registers.a = self.memory.read_byte(self.registers.hl());
                        self.registers.set_hl(self.registers.hl() + 1);
                        (self.pc.wrapping_add(1), 2)
                    }
                    instruction::IndirectTarget::HLD => {
                        self.registers.a = self.memory.read_byte(self.registers.hl());
                        self.registers.set_hl(self.registers.hl() - 1);
                        (self.pc.wrapping_add(1), 2)
                    }
                },

                instruction::LoadType::IndirectFromA(target) => match target {
                    instruction::IndirectTarget::C => {
                        self.memory
                            .write_high_byte(self.registers.c, self.registers.a);
                        (self.pc.wrapping_add(1), 2)
                    }
                    instruction::IndirectTarget::U8 => {
                        self.memory
                            .write_high_byte(self.read_next_byte(), self.registers.a);
                        (self.pc.wrapping_add(2), 3)
                    }
                    instruction::IndirectTarget::U16 => {
                        self.memory
                            .write_byte(self.read_next_word(), self.registers.a);
                        (self.pc.wrapping_add(3), 4)
                    }
                    instruction::IndirectTarget::BCP => {
                        self.memory
                            .write_byte(self.registers.bc(), self.registers.a);
                        (self.pc.wrapping_add(1), 2)
                    }
                    instruction::IndirectTarget::DEP => {
                        self.memory
                            .write_byte(self.registers.de(), self.registers.a);
                        (self.pc.wrapping_add(1), 2)
                    }
                    instruction::IndirectTarget::HLI => {
                        self.memory
                            .write_byte(self.registers.hl(), self.registers.a);
                        self.registers.set_hl(self.registers.hl() + 1);
                        (self.pc.wrapping_add(1), 2)
                    }
                    instruction::IndirectTarget::HLD => {
                        // dbg!(crate::hex!(self.registers.hl()));
                        self.memory
                            .write_byte(self.registers.hl(), self.registers.a);
                        self.registers.set_hl(self.registers.hl() - 1);
                        (self.pc.wrapping_add(1), 2)
                    }
                },

                instruction::LoadType::IndirectFromSP => {
                    let addr = self.read_next_word();
                    // Little-endian
                    self.memory.write_byte(addr, self.sp as u8);
                    self.memory
                        .write_byte(addr + 1, (self.sp >> u8::BITS) as u8);
                    (self.pc.wrapping_add(3), 5)
                }

                instruction::LoadType::SPFromHL => {
                    self.sp = self.registers.hl();
                    (self.pc.wrapping_add(1), 2)
                }

                instruction::LoadType::HLFromSPN => {
                    let val = self.read_next_byte() as i8 as i16 as u16;
                    self.registers.set_hl(val.wrapping_add(self.sp));

                    self.registers.f.zero = false;
                    self.registers.f.subtract = false;
                    // Set if overflow from bit 3.
                    self.registers.f.half_carry = (self.sp & 0xF) + (val & 0xF) > 0xF;
                    // Set if overflow from bit 7.
                    self.registers.f.carry = (self.sp & 0xFF) + (val & 0xFF) > 0xFF;

                    (self.pc.wrapping_add(2), 3)
                }
            },

            Instruction::RL(target) => bit_shift_instruction!(target; rotate_left: true, true),
            Instruction::RLA => {
                self.registers.a = self.rotate_left(self.registers.a, true, false);
                (self.pc.wrapping_add(1), 1)
            }
            Instruction::RLC(target) => bit_shift_instruction!(target; rotate_left: false, true),
            Instruction::RLCA => {
                self.registers.a = self.rotate_left(self.registers.a, false, false);
                (self.pc.wrapping_add(1), 1)
            }
            Instruction::SLA(target) => bit_shift_instruction!(target; shift_left_arith:),

            Instruction::RR(target) => bit_shift_instruction!(target; rotate_right: true, true),
            Instruction::RRA => {
                self.registers.a = self.rotate_right(self.registers.a, true, false);
                (self.pc.wrapping_add(1), 1)
            }
            Instruction::RRC(target) => bit_shift_instruction!(target; rotate_right: false, true),
            Instruction::RRCA => {
                self.registers.a = self.rotate_right(self.registers.a, false, false);
                (self.pc.wrapping_add(1), 1)
            }
            Instruction::SRA(target) => bit_shift_instruction!(target; shift_right: true),
            Instruction::SRL(target) => bit_shift_instruction!(target; shift_right: false),

            Instruction::SWAP(target) => bit_shift_instruction!(target; swap_bits:),

            Instruction::JR(test) => {
                let addr = self.read_next_byte() as i8 as i16 as u16;
                let jump = self.jump_test_res(test);
                self.jump_relative(addr, jump)
            }
            Instruction::JP(test) => {
                let addr = self.read_next_word();
                let jump = self.jump_test_res(test);
                self.jump_absolute(addr, jump)
            }
            Instruction::JPHLP => (self.registers.hl(), 1),

            Instruction::CALL(test) => {
                let jump_addr = self.read_next_word();
                let jump_test = self.jump_test_res(test);
                self.call(jump_addr, jump_test)
            }

            Instruction::RET(test) => {
                let jump_test = self.jump_test_res(test);
                let next_pc = self.ret(jump_test);
                let cycles = if let instruction::JumpTest::Always = test {
                    4
                } else if jump_test {
                    5
                } else {
                    2
                };
                (next_pc, cycles)
            }

            Instruction::RETI => {
                self.interrupts_enabled = true;
                (self.ret(true), 4)
            }

            Instruction::RST(vec_) => {
                self.push_stack(self.pc.wrapping_add(1));
                (vec_.to_addr(), 4)
            }

            Instruction::ADDSP => {
                // The reason for such complex conversion is that we want to
                // convert i8 to u16 as two's complement, so when `wrapping_add`
                // it will subtract if i8 is negative.
                let val = self.read_next_byte() as i8 as i16 as u16;
                self.move_sp_relative(val);
                (self.pc.wrapping_add(2), 4)
            }

            Instruction::PUSH(target) => match target {
                instruction::StackTarget::AF => {
                    self.push_stack(self.registers.af());
                    (self.pc.wrapping_add(1), 4)
                }
                instruction::StackTarget::BC => {
                    self.push_stack(self.registers.bc());
                    (self.pc.wrapping_add(1), 4)
                }
                instruction::StackTarget::DE => {
                    self.push_stack(self.registers.de());
                    (self.pc.wrapping_add(1), 4)
                }
                instruction::StackTarget::HL => {
                    self.push_stack(self.registers.hl());
                    (self.pc.wrapping_add(1), 4)
                }
            },
            Instruction::POP(target) => match target {
                instruction::StackTarget::AF => {
                    let val = self.pop_stack();
                    self.registers.set_af(val);
                    (self.pc.wrapping_add(1), 3)
                }
                instruction::StackTarget::BC => {
                    let val = self.pop_stack();
                    self.registers.set_bc(val);
                    (self.pc.wrapping_add(1), 3)
                }
                instruction::StackTarget::DE => {
                    let val = self.pop_stack();
                    self.registers.set_de(val);
                    (self.pc.wrapping_add(1), 3)
                }
                instruction::StackTarget::HL => {
                    let val = self.pop_stack();
                    self.registers.set_hl(val);
                    (self.pc.wrapping_add(1), 3)
                }
            },

            Instruction::DI => {
                self.di_timer = 2;
                (self.pc.wrapping_add(1), 1)
            }
            Instruction::EI => {
                self.ei_timer = 2;
                (self.pc.wrapping_add(1), 1)
            }

            Instruction::HALT => {
                self.is_halted = true;
                (self.pc.wrapping_add(1), 1)
            }

            Instruction::DAA => {
                self.registers.a = self.decimal_adjust_accum(self.registers.a);
                (self.pc.wrapping_add(1), 1)
            }

            Instruction::NOP => (self.pc.wrapping_add(1), 1),

            // https://gbdev.io/pandocs/Reducing_Power_Consumption.html?highlight=stop#using-the-stop-instruction
            Instruction::STOP => unimplemented!("STOP instruction is not supported currently."),
        };
        // Convert MCycles to TCycles.
        (res.0, res.1 * 4)
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

    // https://forums.nesdev.org/viewtopic.php?t=15944
    fn decimal_adjust_accum(&mut self, mut val: u8) -> u8 {
        let mut carry = false;

        if !self.registers.f.subtract {
            if self.registers.f.carry || self.registers.a > 0x99 {
                val = val.wrapping_add(0x60);
                carry = true;
            }
            if self.registers.f.half_carry || (self.registers.a & 0x0f) > 0x09 {
                val = val.wrapping_add(0x06);
            }
        } else {
            if self.registers.f.carry {
                val = val.wrapping_sub(0x60);
            }
            if self.registers.f.half_carry {
                val = val.wrapping_sub(0x06);
            }
        }

        self.registers.f.zero = val == 0;
        self.registers.f.half_carry = false;
        self.registers.f.carry = carry;

        val
    }

    fn increment_u8(&mut self, val: u8) -> u8 {
        let res = val.overflowing_add(1).0;

        self.registers.f.zero = res == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = (val & HALF_CARRY_MASK) + 1 > HALF_CARRY_MASK;

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
        val.overflowing_sub(1).0
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

    fn rotate_left(&mut self, val: u8, through_carry: bool, set_zero: bool) -> u8 {
        let res = val.wrapping_shl(1)
            + if through_carry {
                val >> (u8::BITS - 1)
            } else {
                self.registers.f.carry as u8
            };

        self.registers.f.zero = set_zero && (res == 0);
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = val.overflowing_shl(1).1;

        res
    }

    fn shift_left_arith(&mut self, val: u8) -> u8 {
        let res = val.wrapping_shl(1);

        self.registers.f.zero = res == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = val.overflowing_shl(1).1;

        res
    }

    fn rotate_right(&mut self, val: u8, through_carry: bool, set_zero: bool) -> u8 {
        let res = (val >> 1)
            | if through_carry {
                (self.registers.f.carry as u8) << (u8::BITS - 1)
            } else {
                0
            };

        self.registers.f.zero = set_zero && (res == 0);
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = val & 1 == 1;

        res
    }

    /// Shift Right Arithmetically.
    fn shift_right(&mut self, val: u8, save_msb: bool) -> u8 {
        let res = (val >> 1)
            | if save_msb {
                val & (1 << (u8::BITS - 1))
            } else {
                0
            };

        self.registers.f.zero = res == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = val & 1 == 1;

        res
    }

    fn swap_bits(&mut self, val: u8) -> u8 {
        let res = val.rotate_right(4);
        self.registers.f.zero = res == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = false;

        res
    }

    fn jump_test_res(&self, test: instruction::JumpTest) -> bool {
        match test {
            instruction::JumpTest::Zero => self.registers.f.zero,
            instruction::JumpTest::NotZero => !self.registers.f.zero,
            instruction::JumpTest::Carry => self.registers.f.carry,
            instruction::JumpTest::NotCarry => !self.registers.f.carry,
            instruction::JumpTest::Always => true,
        }
    }

    #[must_use]
    fn jump_relative(&mut self, addr: u16, jump: bool) -> (u16, u32) {
        if jump {
            (self.pc.wrapping_add(2).wrapping_add(addr), 3)
        } else {
            (self.pc.wrapping_add(2), 2)
        }
    }

    #[must_use]
    fn jump_absolute(&mut self, addr: u16, jump: bool) -> (u16, u32) {
        if jump {
            (addr, 4)
        } else {
            (self.pc.wrapping_add(3), 3)
        }
    }

    #[must_use]
    fn call(&mut self, addr: u16, jump: bool) -> (u16, u32) {
        if jump {
            self.push_stack(self.pc.wrapping_add(3));
            (addr, 6)
        } else {
            (self.pc.wrapping_add(3), 3)
        }
    }

    #[must_use]
    fn ret(&mut self, jump: bool) -> u16 {
        if jump {
            self.pop_stack()
        } else {
            self.pc.wrapping_add(1)
        }
    }

    fn move_sp_relative(&mut self, addr: u16) {
        let val = self.sp.wrapping_add(addr);

        self.registers.f.zero = false;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = (self.sp & 0xF) + (addr & 0xF) > 0xF;
        self.registers.f.carry = (self.sp & 0xFF) + (addr & 0xFF) > 0xFF;

        self.sp = val;
    }

    fn push_stack(&mut self, val: u16) {
        self.memory.write_byte(self.sp.wrapping_sub(2), val as u8);
        self.memory
            .write_byte(self.sp.wrapping_sub(1), (val >> u8::BITS) as u8);

        self.sp = self.sp.wrapping_sub(2);
    }

    #[must_use]
    fn pop_stack(&mut self) -> u16 {
        let val = self.memory.read_byte(self.sp) as u16
            | ((self.memory.read_byte(self.sp.wrapping_add(1)) as u16) << u8::BITS);

        self.sp = self.sp.wrapping_add(2);

        val
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn instruction_swap_bits() {
        env_logger::try_init().unwrap();

        let mut cpu = CPU::new(&[]);
        let mut flag = registers::FlagsRegister {
            zero: false,
            subtract: false,
            half_carry: false,
            carry: false,
        };

        assert_eq!(cpu.swap_bits(0xFD), 0xDF);
        assert_eq!(cpu.registers.f, flag);

        assert_eq!(cpu.swap_bits(0x00), 0x00);
        flag.zero = true;
        assert_eq!(cpu.registers.f, flag);
    }

    #[test]
    fn different_n8_cast() {
        let a = -10i8;
        let b = a as u8;
        assert_eq!(b.wrapping_add(10), 0);

        let addr = -32i8 as u8;
        assert_eq!(addr as i8, -32);
        assert_eq!(addr as i8 as i16, -32);
        assert_eq!((addr as i8 as i16 as u16).wrapping_add(32), 0);

        assert_eq!(126i8 as u8, 126);
        assert_eq!(-126i8 as u8, 130);
        assert_eq!(130u8 as i8, -126);
    }
}
