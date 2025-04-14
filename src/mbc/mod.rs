mod mbc0;
mod mbc1;
// mod mbc2;
// mod mbc5;

pub const KB: usize = 1024;
pub const MB: usize = 1024 * KB;

pub const CARTRIDGE_TYPE_ADDR: usize = 0x147;
pub const ROM_SIZE_ADDR: usize = 0x148;
pub const RAM_SIZE_ADDR: usize = 0x149;

pub trait MBC: Send {
    fn read_rom(&self, addr: u16) -> u8;
    fn write_rom(&mut self, addr: u16, val: u8);

    fn read_ram(&self, addr: u16) -> u8;
    fn write_ram(&mut self, addr: u16, val: u8);
}

pub fn init(cartridge: Vec<u8>) -> Box<dyn MBC> {
    assert!(cartridge.len() >= RAM_SIZE_ADDR);

    match cartridge[CARTRIDGE_TYPE_ADDR] {
        0x00 => Box::new(mbc0::MBC0::new(cartridge)),
        0x01..=0x03 => Box::new(mbc1::MBC1::new(cartridge)),
        // 0x05..=0x06 => Box::new(mbc2::MBC2::new(cartridge)),
        // 0x19..=0x1E => Box::new(mbc5::MBC5::new(cartridge)),
        code => unimplemented!("Cartridge type with code 0x{:X} is not supported.", code),
    }
}

/// # Returns
///
/// Number of ROM banks and ROM size.
pub fn rom_info_reg(value: u8) -> (usize, usize) {
    assert!(value <= 0x8);
    (1 << (value + 1), (1 << value) * 32 * KB)
}

/// # Returns
///
/// Number of ROM banks and ROM size.
pub fn ram_info_reg(value: u8) -> (usize, usize) {
    match value {
        0x0 => (0, 0),
        0x1 => unimplemented!("https://gbdev.io/pandocs/The_Cartridge_Header.html#2kib_sram"),
        0x2 => (1, 8 * KB),
        0x3 => (4, 32 * KB),
        0x4 => (16, 128 * KB),
        0x5 => (8, 64 * KB),
        _ => panic!("Cartridge RAM size value 0x{value:X} does not exist."),
    }
}
