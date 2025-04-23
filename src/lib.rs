#![allow(clippy::new_without_default)]
#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::identity_op)]
#![allow(non_camel_case_types)]
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::needless_range_loop)]

pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

// TCycles - CPU
// MCycles - Hardware

pub const CPU_FREQ: u64 = 4194304;
pub const GPU_FPS: u64 = 60;
pub const MILLIS_PER_FRAME: u64 = 1000 / GPU_FPS;
pub const TICKS_PER_FRAME: u64 = CPU_FREQ / 1000 * MILLIS_PER_FRAME;

pub mod args;
pub mod cpu;
pub(crate) mod gpu;
pub(crate) mod joypad;
pub(crate) mod mbc;
pub(crate) mod memory_bus;

#[macro_export]
macro_rules! bit {
    ($val:expr, $ith:expr) => {
        (($val >> $ith) & 1 == 1)
    };
}

#[macro_export]
macro_rules! hex {
    ($val:expr) => {
        format!("0x{:X}", $val)
    };
}

pub fn read_rom(path: &std::path::Path) -> std::io::Result<Vec<u8>> {
    let mut f = std::fs::File::open(path)?;
    let mut content = vec![];

    use std::io::Read;
    f.read_to_end(&mut content)?;

    // Remove EOF.
    content.resize(content.len() - 1, 0);
    Ok(content)
}

#[cfg(test)]
mod test {}
