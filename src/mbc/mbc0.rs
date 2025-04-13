use crate::memory_bus::{EXTERNAL_RAM_START, ROM_BANK_0_START};

use super::KB;

pub struct MBC0 {
    rom: [u8; 32 * KB],
    ram: [u8; 8 * KB],
}

impl MBC0 {
    pub fn new(data: Vec<u8>) -> Self {
        let mut mbc = Self {
            rom: [0; 32 * KB],
            ram: [0; 8 * KB],
        };
        assert!(data.len() <= mbc.rom.len());

        mbc.rom[..data.len()].copy_from_slice(&data);

        mbc
    }
}

impl super::MBC for MBC0 {
    fn read_rom(&self, addr: u16) -> u8 {
        *self.rom.get((addr - ROM_BANK_0_START) as usize).unwrap()
    }

    #[allow(unused_variables)]
    fn write_rom(&mut self, addr: u16, val: u8) {
        // ROM in MBC0 is read-only
    }

    fn read_ram(&self, addr: u16) -> u8 {
        *self.ram.get((addr - EXTERNAL_RAM_START) as usize).unwrap()
    }

    fn write_ram(&mut self, addr: u16, val: u8) {
        *self
            .ram
            .get_mut((addr - EXTERNAL_RAM_START) as usize)
            .unwrap() = val;
    }
}
