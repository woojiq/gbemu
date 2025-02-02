// https://gbdev.io/pandocs/Memory_Map.html

const BOOT_ROM_START: usize = 0x0000;
const BOOT_ROM_END: usize = 0x00FF;
const BOOT_ROM_SIZE: usize = BOOT_ROM_END - BOOT_ROM_START + 1;

const ROM_BANK_0_START: usize = 0x0000;
const ROM_BANK_0_END: usize = 0x3FFF;
const ROM_BANK_0_SIZE: usize = ROM_BANK_0_END - ROM_BANK_0_START + 1;

const ROM_BANK_N_START: usize = 0x4000;
const ROM_BANK_N_END: usize = 0x7FFF;
const ROM_BANK_N_SIZE: usize = ROM_BANK_N_END - ROM_BANK_N_START + 1;

const VIDEO_RAM_START: usize = 0x8000;
const VIDEO_RAM_END: usize = 0x9FFF;
const VIDEO_RAM_SIZE: usize = VIDEO_RAM_END - VIDEO_RAM_START + 1;

const EXTERNAL_RAM_START: usize = 0xA000;
const EXTERNAL_RAM_END: usize = 0xBFFF;
const EXTERNAL_RAM_SIZE: usize = EXTERNAL_RAM_END - EXTERNAL_RAM_START + 1;

const WORKING_RAM_START: usize = 0xC000;
const WORKING_RAM_END: usize = 0xDFFF;
const WORKING_RAM_SIZE: usize = WORKING_RAM_END - WORKING_RAM_START + 1;

const ECHO_RAM_START: usize = 0xE000;
const ECHO_RAM_END: usize = 0xFDFF;
const ECHO_RAM_SIZE: usize = ECHO_RAM_END - ECHO_RAM_START + 1;

// Object attribute memory (OAM).
const OAM_START: usize = 0xFE00;
const OAM_END: usize = 0xFE9F;
const OAM_SIZE: usize = OAM_END - OAM_START + 1;

const UNUSED_START: usize = 0xFEA0;
const UNUSED_END: usize = 0xFEFF;
const UNUSED_SIZE: usize = UNUSED_END - UNUSED_START + 1;

const IO_REGISTERS_START: usize = 0xFF00;
const IO_REGISTERS_END: usize = 0xFF7F;
const IO_REGISTERS_SIZE: usize = IO_REGISTERS_END - IO_REGISTERS_START + 1;

const HIGH_RAM_AREA_START: usize = 0xFF80;
const HIGH_RAM_AREA_END: usize = 0xFFFE;
const HIGH_RAM_AREA_SIZE: usize = HIGH_RAM_AREA_END - HIGH_RAM_AREA_START + 1;

const INTERRUPT_ENABLED_REGISTER: usize = 0xFFFF;

pub struct MemoryBus {
    boot_rom: [u8; BOOT_ROM_SIZE],
    rom_bank_0: [u8; ROM_BANK_0_SIZE],
    rom_bank_n: [u8; ROM_BANK_N_SIZE],
    vram: [u8; VIDEO_RAM_SIZE],
    external_ram: [u8; EXTERNAL_RAM_SIZE],
    /// Working RAM.
    wram: [u8; WORKING_RAM_SIZE],
    oam: [u8; OAM_SIZE],
    io_registers: IoRegisters,
    /// Hight RAM.
    hram: [u8; HIGH_RAM_AREA_SIZE],
    interrupt_enable: InterruptEnableRegister,
}

pub struct IoRegisters {}

pub struct InterruptEnableRegister {}

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
            io_registers: IoRegisters::new(),
            hram: [0; HIGH_RAM_AREA_SIZE],
            interrupt_enable: InterruptEnableRegister::new(),
        }
    }

    pub fn read_byte<T: Into<u16>>(&self, addr: T) -> u8 {
        fn inner(memory: &MemoryBus, addr: usize) -> u8 {
            match addr {
                ECHO_RAM_START..=ECHO_RAM_END => panic!("Use of \"Echo RAM\" memory section."),
                UNUSED_START..=UNUSED_END => panic!("Use of \"Not Usable\" memory section."),
                _ => todo!(),
            }
        }
        inner(&self, addr.into() as usize)
    }

    pub fn read_high_byte<T: Into<u8>>(&self, addr: T) -> u8 {
        let addr = IO_REGISTERS_START as u16 + addr.into() as u16;
        self.read_byte(addr)
    }

    pub fn write_byte<T: Into<u16>>(&self, addr: T, val: u8) {
        fn inner(memory: &MemoryBus, addr: usize, val: u8) {
            match addr {
                ECHO_RAM_START..=ECHO_RAM_END => panic!("Use of \"Echo RAM\" memory section."),
                UNUSED_START..=UNUSED_END => panic!("Use of \"Not Usable\" memory section."),
                _ => todo!(),
            }
        }
        inner(&self, addr.into() as usize, val)
    }

    pub fn write_high_byte<T: Into<u8>>(&self, addr: T, val: u8) {
        let addr = IO_REGISTERS_START as u16 + addr.into() as u16;
        self.write_byte(addr, val);
    }
}

impl IoRegisters {
    pub fn new() -> Self {
        Self {}
    }
}

impl InterruptEnableRegister {
    pub fn new() -> Self {
        Self {}
    }
}
