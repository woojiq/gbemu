use super::{RAM_SIZE_ADDR, ROM_SIZE_ADDR};

pub struct MBC1 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    rom_banks: usize,
    ram_banks: usize,
    current_rom_bank: usize,
    current_ram_bank: usize,
    ram_enabled: bool,
    advanced_mode: bool,
}

impl MBC1 {
    pub fn new(data: Vec<u8>) -> Self {
        let (rom_banks, rom_size) = super::rom_info_reg(data[ROM_SIZE_ADDR]);
        let (ram_banks, ram_size) = super::ram_info_reg(data[RAM_SIZE_ADDR]);
        assert!(
            data.len() <= rom_size,
            "ROM size detected 0x{rom_size:X}, but cartridge size 0x{:X}.",
            data.len()
        );

        Self {
            rom: data,
            ram: vec![0; ram_size],
            rom_banks,
            ram_banks,
            current_rom_bank: 1,
            current_ram_bank: 0,
            ram_enabled: false,
            advanced_mode: false,
        }
    }
}

impl super::MBC for MBC1 {
    fn read_rom(&self, addr: u16) -> u8 {
        let bank = if addr <= 0x3FFF {
            if self.advanced_mode {
                self.current_rom_bank & !0b11111
            } else {
                0
            }
        } else {
            self.current_rom_bank
        };

        let addr = (bank * 0x4000) | (addr as usize & 0x3FFF);
        *self.rom.get(addr).unwrap_or(&0xFF)
    }

    fn write_rom(&mut self, addr: u16, val: u8) {
        if addr <= 0x1FFF {
            self.ram_enabled = val & 0xF == 0xA;
        } else if addr <= 0x3FFF {
            // > If this register is set to $00, it behaves as if it is set to $01.
            let bank = std::cmp::max(val & 0b11111, 1);
            self.current_rom_bank =
                ((self.current_rom_bank & !0b11111) | (bank as usize)) % self.rom_banks;
        } else if addr <= 0x5FFF {
            if self.rom_banks > 32 {
                self.current_rom_bank = ((self.current_rom_bank & 0b11111)
                    | ((val as usize & 0b11) << 5))
                    % self.rom_banks;
            }
            if self.ram_banks == 4 {
                self.current_ram_bank = (val & 0b11) as usize;
            }
        } else if addr <= 0x7FFF {
            self.advanced_mode = val & 1 == 1;
        }
    }

    fn read_ram(&self, addr: u16) -> u8 {
        if !self.ram_enabled {
            return 0xFF;
        }
        let bank = if self.advanced_mode {
            self.current_ram_bank
        } else {
            0
        };
        let addr = (bank * 0x2000) | (addr as usize & 0x1FFF);
        *self.ram.get(addr).unwrap()
    }

    fn write_ram(&mut self, addr: u16, val: u8) {
        if !self.ram_enabled {
            return;
        }
        let bank = if self.advanced_mode {
            self.current_ram_bank
        } else {
            0
        };
        let addr = (bank * 0x2000) | (addr as usize & 0x1FFF);
        if let Some(mem) = self.ram.get_mut(addr) {
            *mem = val;
        }
    }
}
