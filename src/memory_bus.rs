// https://gbdev.io/pandocs/Memory_Map.html

use crate::{
    bit,
    gpu::GPU,
    joypad::{Joypad, JoypadKey},
};

pub const ROM_BANK_0_START: u16 = 0x0000;
pub const ROM_BANK_0_END: u16 = 0x3FFF;
pub const ROM_BANK_0_SIZE: usize = (ROM_BANK_0_END - ROM_BANK_0_START + 1) as usize;

pub const ROM_BANK_N_START: u16 = 0x4000;
pub const ROM_BANK_N_END: u16 = 0x7FFF;
pub const ROM_BANK_N_SIZE: usize = (ROM_BANK_N_END - ROM_BANK_N_START + 1) as usize;

pub const VIDEO_RAM_START: u16 = 0x8000;
pub const VIDEO_RAM_END: u16 = 0x9FFF;
pub const VIDEO_RAM_SIZE: usize = (VIDEO_RAM_END - VIDEO_RAM_START + 1) as usize;

pub const EXTERNAL_RAM_START: u16 = 0xA000;
pub const EXTERNAL_RAM_END: u16 = 0xBFFF;
pub const EXTERNAL_RAM_SIZE: usize = (EXTERNAL_RAM_END - EXTERNAL_RAM_START + 1) as usize;

pub const WORKING_RAM_START: u16 = 0xC000;
pub const WORKING_RAM_END: u16 = 0xDFFF;
pub const WORKING_RAM_SIZE: usize = (WORKING_RAM_END - WORKING_RAM_START + 1) as usize;

pub const ECHO_RAM_START: u16 = 0xE000;
pub const ECHO_RAM_END: u16 = 0xFDFF;
#[allow(dead_code)]
pub const ECHO_RAM_SIZE: usize = (ECHO_RAM_END - ECHO_RAM_START + 1) as usize;

// Object attribute memory (OAM).
pub const OAM_START: u16 = 0xFE00;
pub const OAM_END: u16 = 0xFE9F;
pub const OAM_SIZE: usize = (OAM_END - OAM_START + 1) as usize;

pub const UNUSED_START: u16 = 0xFEA0;
pub const UNUSED_END: u16 = 0xFEFF;
#[allow(dead_code)]
pub const UNUSED_SIZE: usize = (UNUSED_END - UNUSED_START + 1) as usize;

pub const IO_REGISTERS_START: u16 = 0xFF00;
pub const IO_REGISTERS_END: u16 = 0xFF7F;
#[allow(dead_code)]
pub const IO_REGISTERS_SIZE: usize = (IO_REGISTERS_END - IO_REGISTERS_START + 1) as usize;

pub const HIGH_RAM_AREA_START: u16 = 0xFF80;
pub const HIGH_RAM_AREA_END: u16 = 0xFFFE;
pub const HIGH_RAM_AREA_SIZE: usize = (HIGH_RAM_AREA_END - HIGH_RAM_AREA_START + 1) as usize;

pub const INTERRUPT_ENABLED_REGISTER: u16 = 0xFFFF;

pub struct MemoryBus {
    rom_bank_0: [u8; ROM_BANK_0_SIZE],
    rom_bank_n: [u8; ROM_BANK_N_SIZE],
    external_ram: [u8; EXTERNAL_RAM_SIZE],
    /// Working RAM.
    wram: [u8; WORKING_RAM_SIZE],

    pub gpu: GPU,

    // IO registers:
    interrupt_enable: InterruptFlags,
    interrupt_flag: InterruptFlags,
    joypad: Joypad,
    divider: Timer,
    timer: Timer,

    /// Hight RAM.
    hram: [u8; HIGH_RAM_AREA_SIZE],
}

#[derive(Copy, Clone, Default)]
pub enum TimerRateHz {
    #[default]
    F4096,
    F262144,
    F65536,
    F16384,
}

#[derive(Copy, Clone, Default)]
pub struct Timer {
    freq: TimerRateHz,
    cycles: u32,
    pub val: u8,
    /// When TIMA overflows, it is reset to the value in this register and an
    /// interrupt is requested.
    pub modulo: u8,
    pub enable: bool,
}

#[derive(Copy, Clone, Debug)]
pub struct InterruptFlags {
    vblank: bool,
    lcd: bool,
    timer: bool,
    serial: bool,
    joypad: bool,
}

impl MemoryBus {
    pub fn new(game_rom: &[u8]) -> Self {
        let mut bus = Self {
            rom_bank_0: [0; ROM_BANK_0_SIZE],
            rom_bank_n: [0; ROM_BANK_N_SIZE],
            external_ram: [0; EXTERNAL_RAM_SIZE],
            wram: [0; WORKING_RAM_SIZE],

            gpu: GPU::new(),

            joypad: Joypad::new(),
            divider: Timer::new_enabled(TimerRateHz::F16384),
            timer: Timer::new_disabled(TimerRateHz::F4096),
            interrupt_enable: InterruptFlags::new(),
            interrupt_flag: InterruptFlags::new(),

            hram: [0; HIGH_RAM_AREA_SIZE],
        };

        bus.divider.enable = true;

        use std::cmp::min;

        let bank0_len = min(bus.rom_bank_0.len(), game_rom.len());
        bus.rom_bank_0[..bank0_len].copy_from_slice(&game_rom[..bank0_len]);

        if game_rom.len() > ROM_BANK_0_SIZE {
            assert!(
                game_rom.len() <= ROM_BANK_N_END as usize,
                "Max supported size is {}, got {}.",
                ROM_BANK_N_END,
                game_rom.len()
            );

            let bankn_len = game_rom.len() - bank0_len;
            bus.rom_bank_n[..bankn_len].copy_from_slice(&game_rom[bank0_len..]);
        }

        bus.set_init_values();

        bus
    }

    fn set_init_values(&mut self) {
        self.write_byte(0xFF05, 0);
        self.write_byte(0xFF06, 0);
        self.write_byte(0xFF07, 0);
        self.write_byte(0xFF10, 0x80);
        self.write_byte(0xFF11, 0xBF);
        self.write_byte(0xFF12, 0xF3);
        self.write_byte(0xFF14, 0xBF);
        self.write_byte(0xFF16, 0x3F);
        self.write_byte(0xFF16, 0x3F);
        self.write_byte(0xFF17, 0);
        self.write_byte(0xFF19, 0xBF);
        self.write_byte(0xFF1A, 0x7F);
        self.write_byte(0xFF1B, 0xFF);
        self.write_byte(0xFF1C, 0x9F);
        self.write_byte(0xFF1E, 0xFF);
        self.write_byte(0xFF20, 0xFF);
        self.write_byte(0xFF21, 0);
        self.write_byte(0xFF22, 0);
        self.write_byte(0xFF23, 0xBF);
        self.write_byte(0xFF24, 0x77);
        self.write_byte(0xFF25, 0xF3);
        self.write_byte(0xFF26, 0xF1);
        self.write_byte(0xFF40, 0x91);
        self.write_byte(0xFF42, 0);
        self.write_byte(0xFF43, 0);
        self.write_byte(0xFF45, 0);
        self.write_byte(0xFF47, 0xFC);
        self.write_byte(0xFF48, 0xFF);
        self.write_byte(0xFF49, 0xFF);
        self.write_byte(0xFF4A, 0);
        self.write_byte(0xFF4B, 0);
    }

    pub fn key_up(&mut self, key: JoypadKey) {
        if self.joypad.key_up(key) {
            self.interrupt_flag.joypad = true;
        }
    }

    pub fn key_down(&mut self, key: JoypadKey) {
        if self.joypad.key_down(key) {
            self.interrupt_flag.joypad = true;
        }
    }

    pub fn step(&mut self, cycles: u32) -> u32 {
        self.divider.step(cycles);

        if self.timer.step(cycles) {
            self.interrupt_flag.timer = true;
        }

        let inter = self.gpu.step(cycles);
        self.interrupt_flag.vblank |= inter.vblank;
        self.interrupt_flag.lcd |= inter.lcd;

        cycles
    }

    pub fn pending_interrupt(&self) -> bool {
        u8::from(self.interrupt_enable) & u8::from(self.interrupt_flag) != 0
    }

    pub fn vbank_interrupt(&self) -> bool {
        // dbg!(self.interrupt_enable.vblank, self.interrupt_flag.vblank);
        self.interrupt_enable.vblank && self.interrupt_flag.vblank
    }
    pub fn reset_vbank_interrupt(&mut self) {
        self.interrupt_flag.vblank = false;
    }

    pub fn lcd_interrupt(&self) -> bool {
        self.interrupt_enable.lcd && self.interrupt_flag.lcd
    }
    pub fn reset_lcd_interrupt(&mut self) {
        self.interrupt_flag.lcd = false;
    }

    pub fn timer_interrupt(&self) -> bool {
        self.interrupt_enable.timer && self.interrupt_flag.timer
    }
    pub fn reset_timer_interrupt(&mut self) {
        self.interrupt_flag.timer = false;
    }

    pub fn serial_interrupt(&self) -> bool {
        self.interrupt_enable.serial && self.interrupt_flag.serial
    }
    pub fn reset_serial_interrupt(&mut self) {
        self.interrupt_flag.serial = false;
    }

    pub fn joypad_interrupt(&self) -> bool {
        self.interrupt_enable.joypad && self.interrupt_flag.joypad
    }
    pub fn reset_joypad_interrupt(&mut self) {
        self.interrupt_flag.joypad = false;
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            ROM_BANK_0_START..=ROM_BANK_0_END => {
                self.rom_bank_0[(addr - ROM_BANK_0_START) as usize]
            }
            ROM_BANK_N_START..=ROM_BANK_N_END => {
                self.rom_bank_n[(addr - ROM_BANK_N_START) as usize]
            }
            VIDEO_RAM_START..=VIDEO_RAM_END => self.gpu.vram[(addr - VIDEO_RAM_START) as usize],
            EXTERNAL_RAM_START..=EXTERNAL_RAM_END => {
                self.external_ram[(addr - EXTERNAL_RAM_START) as usize]
            }
            WORKING_RAM_START..=WORKING_RAM_END => self.wram[(addr - WORKING_RAM_START) as usize],
            ECHO_RAM_START..=ECHO_RAM_END => self.wram[(addr - ECHO_RAM_START) as usize],
            OAM_START..=OAM_END => self.gpu.oam[(addr - OAM_START) as usize],
            UNUSED_START..=UNUSED_END => 0,
            IO_REGISTERS_START..=IO_REGISTERS_END => self.read_io_register(addr),
            HIGH_RAM_AREA_START..=HIGH_RAM_AREA_END => {
                self.hram[(addr - HIGH_RAM_AREA_START) as usize]
            }
            INTERRUPT_ENABLED_REGISTER => u8::from(self.interrupt_enable),
        }
    }

    pub fn read_high_byte(&self, addr: u8) -> u8 {
        let addr = IO_REGISTERS_START | addr as u16;
        self.read_byte(addr)
    }

    pub fn write_byte(&mut self, addr: u16, val: u8) {
        // eprintln!("0x{addr:X} = {val}");
        match addr {
            ROM_BANK_0_START..=ROM_BANK_0_END => {
                self.rom_bank_0[(addr - ROM_BANK_0_START) as usize] = val
            }
            ROM_BANK_N_START..=ROM_BANK_N_END => {
                panic!(
                    "Changing ROM Bank memory is forbidden: addr = 0x{:X}, val = 0x{:X}",
                    addr, val
                );
                // self.rom_bank_n[(addr - ROM_BANK_N_START) as usize] = val
            }
            VIDEO_RAM_START..=VIDEO_RAM_END => {
                self.gpu.vram[(addr - VIDEO_RAM_START) as usize] = val
            }
            EXTERNAL_RAM_START..=EXTERNAL_RAM_END => {
                self.external_ram[(addr - EXTERNAL_RAM_START) as usize] = val
            }
            WORKING_RAM_START..=WORKING_RAM_END => {
                self.wram[(addr - WORKING_RAM_START) as usize] = val
            }
            ECHO_RAM_START..=ECHO_RAM_END => self.wram[(addr - ECHO_RAM_START) as usize] = val,
            OAM_START..=OAM_END => self.gpu.oam[(addr - OAM_START) as usize] = val,
            UNUSED_START..=UNUSED_END => {
                // Writing here does nothing.
            }
            IO_REGISTERS_START..=IO_REGISTERS_END => self.write_io_register(addr, val),
            HIGH_RAM_AREA_START..=HIGH_RAM_AREA_END => {
                self.hram[(addr - HIGH_RAM_AREA_START) as usize] = val
            }
            INTERRUPT_ENABLED_REGISTER => self.interrupt_enable = InterruptFlags::from(val),
        }
    }

    pub fn write_high_byte(&mut self, addr: u8, val: u8) {
        let addr = IO_REGISTERS_START + addr as u16;
        self.write_byte(addr, val);
    }

    // https://gbdev.io/pandocs/Memory_Map.html#io-ranges
    fn read_io_register(&self, addr: u16) -> u8 {
        assert!((IO_REGISTERS_START..=IO_REGISTERS_END).contains(&addr));

        match addr {
            0xFF00 => u8::from(self.joypad),
            0xFF01..=0xFF02 => {
                // TODO: Serial transfer read.
                0
            }
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
            0xFF10..=0xFF26 => {
                0
                // unimplemented!("Reading from Audio registers is not supported yet."),
            }
            0xFF30..=0xFF3F => {
                unimplemented!("Reading from Wave pattern registers is not supported yet.")
            }
            0xFF40 => u8::from(self.gpu.lcd_control),
            0xFF41 => self.gpu.lcd_status.get_status_byte(),
            0xFF42 => self.gpu.viewport.y,
            0xFF43 => self.gpu.viewport.x,
            0xFF44 => self.gpu.lcd_status.ly(),
            0xFF45 => self.gpu.lcd_status.lyc(),
            0xFF47 => u8::from(self.gpu.bg_colors),
            0xFF48 => u8::from(self.gpu.obj0_colors),
            0xFF49 => u8::from(self.gpu.obj1_colors),
            0xFF4A => self.gpu.window.y,
            0xFF4B => self.gpu.window.x,
            _ => panic!("Reading from addr 0x{addr:X} is forbidden."),
        }
    }

    fn write_io_register(&mut self, addr: u16, val: u8) {
        assert!((IO_REGISTERS_START..=IO_REGISTERS_END).contains(&addr));

        match addr {
            0xFF00 => self.joypad.set_mode(val),
            0xFF01..=0xFF02 => {
                // TODO: Serial transfer write.
            }
            0xFF04 => self.divider.val = 0,
            0xFF05 => self.timer.val = val,
            0xFF06 => self.timer.modulo = val,
            0xFF07 => {
                self.timer.freq = match val & 0b11 {
                    0 => TimerRateHz::F4096,
                    1 => TimerRateHz::F262144,
                    2 => TimerRateHz::F65536,
                    3 => TimerRateHz::F16384,
                    _ => unreachable!("Unknown timer frequency rate {}", val & 0b11),
                };
                self.timer.enable = val & (1 << 2) != 0;
            }
            0xFF0F => self.interrupt_flag = InterruptFlags::from(val),
            0xFF10..=0xFF26 => {
                // TODO: Audio.
            }
            0xFF30..=0xFF3F => {
                // TODO: Wave pattern.
            }
            0xFF40 => {
                let inter = self.gpu.set_lcd_control(val);
                self.interrupt_flag.vblank |= inter.vblank;
                self.interrupt_flag.lcd |= inter.lcd;
            }
            0xFF41 => self.gpu.lcd_status.write_byte_to_status(val),
            0xFF42 => self.gpu.viewport.y = val,
            0xFF43 => self.gpu.viewport.x = val,
            0xFF44 => panic!("LCD Y coordinate is read-only."),
            0xFF45 => {
                if self.gpu.lcd_status.set_lyc(val) {
                    self.interrupt_flag.lcd = true;
                }
            }
            0xFF46 => {
                // Writing to this register starts a DMA transfer from ROM or
                // RAM to OAM (Object Attribute Memory). The transfer takes 160
                // M-cycles: 640 dots (1.4 lines) in normal speed.
                self.dma_transfer((val as u16) * 0x100);
            }
            0xFF47 => self.gpu.bg_colors = super::gpu::BackgroundColors::from(val),
            // Lower two bits are ignored because color index 0 is transparent for OBJs.
            0xFF48 => self.gpu.obj0_colors = super::gpu::BackgroundColors::from(val & !0b11),
            0xFF49 => self.gpu.obj1_colors = super::gpu::BackgroundColors::from(val & !0b11),
            0xFF4A => self.gpu.window.y = val,
            0xFF4B => self.gpu.window.x = val,
            0xFF7F..=0xFF7F => {
                // Writing here does nothing.
            }
            _ => panic!("Cannot write to memory location 0x{addr:X}"),
        }
    }

    fn dma_transfer(&mut self, addr: u16) {
        // TODO: Use OAM_START/END.
        const DMA_DEST_START: u16 = 0xFE00;
        const DMA_DEST_END: u16 = 0xFE9F;

        for dest_addr in DMA_DEST_START..=DMA_DEST_END {
            self.write_byte(
                dest_addr,
                self.read_byte(addr + (dest_addr - DMA_DEST_START)),
            );
        }
    }
}

impl TimerRateHz {
    pub const fn per_cpu_cycle(&self) -> u32 {
        use crate::CPU_FREQ;
        match self {
            TimerRateHz::F4096 => CPU_FREQ / 4096,
            TimerRateHz::F262144 => CPU_FREQ / 262144,
            TimerRateHz::F65536 => CPU_FREQ / 65536,
            TimerRateHz::F16384 => CPU_FREQ / 16384,
        }
    }
}

impl Timer {
    pub fn new_disabled(freq: TimerRateHz) -> Self {
        Self {
            freq,
            ..Default::default()
        }
    }

    pub fn new_enabled(freq: TimerRateHz) -> Self {
        Self {
            enable: true,
            freq,
            ..Default::default()
        }
    }

    /// # Returns
    ///
    /// Whether overflow occurs.
    pub fn step(&mut self, cpu_cycles: u32) -> bool {
        if !self.enable {
            return false;
        }

        self.cycles += cpu_cycles;

        let mut overflow = false;

        while self.cycles >= self.freq.per_cpu_cycle() {
            let (new_val, overflow_cur) = self.val.overflowing_add(1);

            overflow |= overflow_cur;

            self.cycles -= self.freq.per_cpu_cycle();

            self.val = if overflow_cur { self.modulo } else { new_val };
        }

        overflow
    }
}

impl InterruptFlags {
    pub fn new() -> Self {
        Self {
            vblank: false,
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
            | (v.vblank as u8)
    }
}

impl From<u8> for InterruptFlags {
    fn from(v: u8) -> Self {
        Self {
            vblank: bit!(v, 0),
            lcd: bit!(v, 1),
            timer: bit!(v, 2),
            serial: bit!(v, 3),
            joypad: bit!(v, 4),
        }
    }
}

impl std::ops::BitAnd for InterruptFlags {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self::Output {
            vblank: self.vblank & rhs.vblank,
            lcd: self.lcd & rhs.lcd,
            timer: self.timer & rhs.timer,
            serial: self.serial & rhs.serial,
            joypad: self.joypad & rhs.joypad,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn multiple_overflows_in_one_timer_cycle() {
        let mut timer = Timer::new_enabled(TimerRateHz::F262144);
        timer.step(36);

        assert_eq!(timer.val, 2);
        assert_eq!(timer.cycles, 4);
    }

    #[test]
    fn timer_overflow() {
        let freq = TimerRateHz::F262144;

        let mut timer = Timer::new_enabled(freq);
        assert!(timer.step(freq.per_cpu_cycle() * (u8::MAX as u32 + 1)));
        assert_eq!((timer.val, timer.cycles), (0, 0));

        let mut timer = Timer::new_enabled(freq);
        assert!(!timer.step(freq.per_cpu_cycle() * (u8::MAX as u32) + freq.per_cpu_cycle() - 1));
        assert_eq!(
            (timer.val, timer.cycles),
            (u8::MAX, freq.per_cpu_cycle() - 1)
        );
    }
}
