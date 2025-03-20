// https://gbdev.io/pandocs/Memory_Map.html

const BOOT_ROM_START: u16 = 0x0000;
const BOOT_ROM_END: u16 = 0x00FF;
const BOOT_ROM_SIZE: usize = (BOOT_ROM_END - BOOT_ROM_START + 1) as usize;

const ROM_BANK_0_START: u16 = 0x0000;
const ROM_BANK_0_END: u16 = 0x3FFF;
const ROM_BANK_0_SIZE: usize = (ROM_BANK_0_END - ROM_BANK_0_START + 1) as usize;

const ROM_BANK_N_START: u16 = 0x4000;
const ROM_BANK_N_END: u16 = 0x7FFF;
const ROM_BANK_N_SIZE: usize = (ROM_BANK_N_END - ROM_BANK_N_START + 1) as usize;

const VIDEO_RAM_START: u16 = 0x8000;
const VIDEO_RAM_END: u16 = 0x9FFF;
const VIDEO_RAM_SIZE: usize = (VIDEO_RAM_END - VIDEO_RAM_START + 1) as usize;

const EXTERNAL_RAM_START: u16 = 0xA000;
const EXTERNAL_RAM_END: u16 = 0xBFFF;
const EXTERNAL_RAM_SIZE: usize = (EXTERNAL_RAM_END - EXTERNAL_RAM_START + 1) as usize;

const WORKING_RAM_START: u16 = 0xC000;
const WORKING_RAM_END: u16 = 0xDFFF;
const WORKING_RAM_SIZE: usize = (WORKING_RAM_END - WORKING_RAM_START + 1) as usize;

const ECHO_RAM_START: u16 = 0xE000;
const ECHO_RAM_END: u16 = 0xFDFF;
const ECHO_RAM_SIZE: usize = (ECHO_RAM_END - ECHO_RAM_START + 1) as usize;

// Object attribute memory (OAM).
const OAM_START: u16 = 0xFE00;
const OAM_END: u16 = 0xFE9F;
const OAM_SIZE: usize = (OAM_END - OAM_START + 1) as usize;

const UNUSED_START: u16 = 0xFEA0;
const UNUSED_END: u16 = 0xFEFF;
const UNUSED_SIZE: usize = (UNUSED_END - UNUSED_START + 1) as usize;

const IO_REGISTERS_START: u16 = 0xFF00;
const IO_REGISTERS_END: u16 = 0xFF7F;
const IO_REGISTERS_SIZE: usize = (IO_REGISTERS_END - IO_REGISTERS_START + 1) as usize;

const HIGH_RAM_AREA_START: u16 = 0xFF80;
const HIGH_RAM_AREA_END: u16 = 0xFFFE;
const HIGH_RAM_AREA_SIZE: usize = (HIGH_RAM_AREA_END - HIGH_RAM_AREA_START + 1) as usize;

const INTERRUPT_ENABLED_REGISTER: u16 = 0xFFFF;

pub struct MemoryBus {
    boot_rom: [u8; BOOT_ROM_SIZE],
    rom_bank_0: [u8; ROM_BANK_0_SIZE],
    rom_bank_n: [u8; ROM_BANK_N_SIZE],
    vram: [u8; VIDEO_RAM_SIZE],
    external_ram: [u8; EXTERNAL_RAM_SIZE],
    /// Working RAM.
    wram: [u8; WORKING_RAM_SIZE],
    oam: [u8; OAM_SIZE],

    // IO registers:
    interrupt_enable: InterruptFlags,
    interrupt_flag: InterruptFlags,
    joypad: Joypad,
    // TODO: Increment divider and timer.
    divider: Timer,
    timer: Timer,

    /// Hight RAM.
    hram: [u8; HIGH_RAM_AREA_SIZE],
}

#[derive(Copy, Clone)]
pub struct Joypad {
    mode: JoypadMode,
    down: bool,
    up: bool,
    left: bool,
    right: bool,
    start: bool,
    select: bool,
    b: bool,
    a: bool,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum JoypadMode {
    // Down, Up, Left, Right
    Dpad,
    // Start, Select, B, A
    Buttons,
    // QUESTION: Do we really need `None`?
    None,
}

#[derive(Copy, Clone, Default)]
enum TimerRateHz {
    #[default]
    F4096,
    F262144,
    F65536,
    F16384,
}

// TODO
#[derive(Copy, Clone, Default)]
pub struct Timer {
    freq: TimerRateHz,
    cycles: usize,
    pub val: u8,
    /// When TIMA overflows, it is reset to the value in this register and an
    /// interrupt is requested.
    pub modulo: u8,
    pub enable: bool,
}

#[derive(Copy, Clone)]
pub struct InterruptFlags {
    vbank: bool,
    lcd: bool,
    timer: bool,
    serial: bool,
    joypad: bool,
}

impl MemoryBus {
    pub fn new() -> Self {
        Self {
            boot_rom: [0; BOOT_ROM_SIZE],
            rom_bank_0: [0; ROM_BANK_0_SIZE],
            rom_bank_n: [0; ROM_BANK_N_SIZE],
            vram: [0; VIDEO_RAM_SIZE],
            external_ram: [0; EXTERNAL_RAM_SIZE],
            wram: [0; WORKING_RAM_SIZE],
            oam: [0; OAM_SIZE],

            joypad: Joypad::new(),
            divider: Timer::new(TimerRateHz::F16384),
            timer: Timer::default(),
            interrupt_enable: InterruptFlags::new(),
            interrupt_flag: InterruptFlags::new(),

            hram: [0; HIGH_RAM_AREA_SIZE],
        }
    }

    pub fn step(&mut self, cycles: usize) {
        self.divider.step(cycles);

        if self.timer.step(cycles) {
            self.interrupt_flag.timer = true;
        }
    }

    pub fn has_interrupt(&self) -> bool {
        u8::from(self.interrupt_enable) & u8::from(self.interrupt_flag) != 0
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            BOOT_ROM_START..=BOOT_ROM_END => self.boot_rom[(addr - BOOT_ROM_START) as usize],
            ROM_BANK_0_START..=ROM_BANK_0_END => {
                self.rom_bank_0[(addr - ROM_BANK_0_START) as usize]
            }
            ROM_BANK_N_START..=ROM_BANK_N_END => {
                self.rom_bank_0[(addr - ROM_BANK_N_START) as usize]
            }
            VIDEO_RAM_START..=VIDEO_RAM_END => self.vram[(addr - VIDEO_RAM_START) as usize],
            EXTERNAL_RAM_START..=EXTERNAL_RAM_END => {
                self.external_ram[(addr - EXTERNAL_RAM_START) as usize]
            }
            WORKING_RAM_START..=WORKING_RAM_END => self.wram[(addr - WORKING_RAM_START) as usize],
            ECHO_RAM_START..=ECHO_RAM_END => panic!(r#"Use of "Echo RAM" memory section."#),
            OAM_START..=OAM_END => self.oam[(addr - OAM_START) as usize],
            UNUSED_START..=UNUSED_END => panic!(r#"Use of "Not Usable" memory section."#),
            IO_REGISTERS_START..=IO_REGISTERS_END => self.read_io_register(addr),
            HIGH_RAM_AREA_START..=HIGH_RAM_AREA_END => {
                self.hram[(addr - HIGH_RAM_AREA_START) as usize]
            }
            INTERRUPT_ENABLED_REGISTER => u8::from(self.interrupt_enable),
        }
    }

    pub fn read_high_byte(&self, addr: u8) -> u8 {
        let addr = IO_REGISTERS_START as u16 + addr as u16;
        self.read_byte(addr)
    }

    // TODO
    pub fn write_byte(&mut self, addr: u16, val: u8) {
        match addr {
            BOOT_ROM_START..=BOOT_ROM_END => panic!("Boot ROM cannot be overwritten."),
            ROM_BANK_0_START..=ROM_BANK_0_END => {
                self.rom_bank_0[(addr - ROM_BANK_0_START) as usize] = val
            }
            ROM_BANK_N_START..=ROM_BANK_N_END => {
                self.rom_bank_0[(addr - ROM_BANK_N_START) as usize] = val
            }
            VIDEO_RAM_START..=VIDEO_RAM_END => self.vram[(addr - VIDEO_RAM_START) as usize] = val,
            EXTERNAL_RAM_START..=EXTERNAL_RAM_END => {
                self.external_ram[(addr - EXTERNAL_RAM_START) as usize] = val
            }
            WORKING_RAM_START..=WORKING_RAM_END => {
                self.wram[(addr - WORKING_RAM_START) as usize] = val
            }
            ECHO_RAM_START..=ECHO_RAM_END => panic!(r#"Use of "Echo RAM" memory section."#),
            OAM_START..=OAM_END => self.oam[(addr - OAM_START) as usize] = val,
            UNUSED_START..=UNUSED_END => panic!(r#"Use of "Not Usable" memory section."#),
            IO_REGISTERS_START..=IO_REGISTERS_END => self.write_io_register(addr, val),
            HIGH_RAM_AREA_START..=HIGH_RAM_AREA_END => {
                self.hram[(addr - HIGH_RAM_AREA_START) as usize] = val
            }
            INTERRUPT_ENABLED_REGISTER => self.interrupt_enable = InterruptFlags::from(val),
        }
    }

    pub fn write_high_byte(&mut self, addr: u8, val: u8) {
        let addr = IO_REGISTERS_START as u16 + addr as u16;
        self.write_byte(addr, val);
    }

    // https://gbdev.io/pandocs/Memory_Map.html#io-ranges
    // TODO
    fn read_io_register(&self, addr: u16) -> u8 {
        assert!((IO_REGISTERS_START..=IO_REGISTERS_END).contains(&addr));

        match addr {
            0xFF00 => u8::from(self.joypad),
            0xFF01..=0xFF02 => unimplemented!("Serial Transfer"),
            0xFF04 => self.divider.val,
            0xFF05 => self.timer.val,
            0xFF06 => self.timer.modulo,
            0xFF07 => {
                (match self.timer.freq {
                    TimerRateHz::F4096 => 0,
                    TimerRateHz::F262144 => 1,
                    TimerRateHz::F65536 => 2,
                    TimerRateHz::F16384 => 3,
                }) | ((self.timer.enable as u8) << 2)
            }
            0xFF0F => u8::from(self.interrupt_flag),
            0xFF10..=0xFF26 => unimplemented!("Audio registers are not supported yet."),
            0xFF30..=0xFF3F => unimplemented!("Wave pattern registers are not supported yet."),
            _ => unimplemented!(),
        }
    }

    // TODO
    fn write_io_register(&mut self, addr: u16, val: u8) {
        assert!((IO_REGISTERS_START..=IO_REGISTERS_END).contains(&addr));

        match addr {
            0xFF00 => self.joypad.set_mode(val),
            0xFF01..=0xFF02 => unimplemented!("Serial Transfer"),
            0xFF04 => self.divider.val = 0,
            0xFF05 => self.timer.val = val,
            0xFF06 => self.timer.modulo = val,
            0xFF07 => {
                self.timer.freq = match val & 0b11 {
                    0 => TimerRateHz::F4096,
                    1 => TimerRateHz::F262144,
                    2 => TimerRateHz::F65536,
                    3 => TimerRateHz::F16384,
                    _ => panic!("Unknown timer frequency rate {}", val & 0b11),
                };
                self.timer.enable = val & (1 << 2) != 0;
            }
            0xFF0F => self.interrupt_flag = InterruptFlags::from(val),
            _ => panic!("Cannot write to memory location 0x{addr:X}"),
        }
    }
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            mode: JoypadMode::None,
            down: false,
            up: false,
            left: false,
            right: false,
            start: false,
            select: false,
            b: false,
            a: false,
        }
    }

    pub fn set_mode(&mut self, val: u8) {
        if val == 0x20 {
            self.mode = JoypadMode::Dpad;
        } else if val == 0x30 {
            self.mode = JoypadMode::Buttons;
        } else {
            panic!("Only 4-5th bits can be written to joypad register.");
        }
    }

    fn is_dpad(&self) -> bool {
        self.mode == JoypadMode::Dpad
    }

    fn is_buttons(&self) -> bool {
        self.mode == JoypadMode::Buttons
    }

    fn bit0(&self) -> bool {
        (self.a && self.is_buttons()) || (self.right && self.is_dpad())
    }

    fn bit1(&self) -> bool {
        (self.b && self.is_buttons()) || (self.left && self.is_dpad())
    }

    fn bit2(&self) -> bool {
        (self.select && self.is_buttons()) || (self.up && self.is_dpad())
    }

    fn bit3(&self) -> bool {
        (self.start && self.is_buttons()) || (self.down && self.is_dpad())
    }
}

impl From<Joypad> for u8 {
    fn from(v: Joypad) -> Self {
        (v.bit0() as u8)
            | ((v.bit1() as u8) << 1)
            | ((v.bit2() as u8) << 2)
            | ((v.bit3() as u8) << 3)
            | ((v.is_dpad() as u8) << 4)
            | ((v.is_buttons() as u8) << 5)
    }
}

impl TimerRateHz {
    pub const fn per_cpu_cycle(&self) -> usize {
        use crate::cpu::CPU_FREQ;
        match self {
            TimerRateHz::F4096 => CPU_FREQ / 4096,
            TimerRateHz::F262144 => CPU_FREQ / 262144,
            TimerRateHz::F65536 => CPU_FREQ / 65536,
            TimerRateHz::F16384 => CPU_FREQ / 16384,
        }
    }
}

impl Timer {
    pub fn new(freq: TimerRateHz) -> Self {
        Self {
            freq,
            cycles: 0,
            val: 0,
            modulo: 0,
            enable: true,
        }
    }

    /// # Returns
    ///
    /// Whether overflow occurs.
    pub fn step(&mut self, cpu_cycles: usize) -> bool {
        if !self.enable {
            return false;
        }

        self.cycles += cpu_cycles;

        let overflow = if self.cycles > self.freq.per_cpu_cycle() {
            let (new_val, overflow) = self
                .val
                .overflowing_add(u8::try_from(self.cycles / self.freq.per_cpu_cycle()).unwrap());

            self.cycles %= self.freq.per_cpu_cycle();
            self.val = new_val;

            overflow
        } else {
            false
        };

        if overflow {
            self.val = self.modulo;
        }

        overflow
    }
}

impl InterruptFlags {
    pub fn new() -> Self {
        Self {
            vbank: false,
            lcd: false,
            timer: false,
            serial: false,
            joypad: false,
        }
    }
}

impl From<InterruptFlags> for u8 {
    fn from(v: InterruptFlags) -> Self {
        ((v.joypad as u8) << 4)
            | ((v.serial as u8) << 3)
            | ((v.timer as u8) << 2)
            | ((v.lcd as u8) << 1)
            | (v.vbank as u8)
    }
}

impl From<u8> for InterruptFlags {
    fn from(v: u8) -> Self {
        Self {
            vbank: (v >> 0) & 1 == 1,
            lcd: (v >> 1) & 1 == 1,
            timer: (v >> 2) & 1 == 1,
            serial: (v >> 3) & 1 == 1,
            joypad: (v >> 4) & 1 == 1,
        }
    }
}

impl std::ops::BitAnd for InterruptFlags {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self::Output {
            vbank: self.vbank & rhs.vbank,
            lcd: self.lcd & rhs.lcd,
            timer: self.timer & rhs.timer,
            serial: self.serial & rhs.serial,
            joypad: self.joypad & rhs.joypad,
        }
    }
}
