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

pub const CPU_FREQ: u32 = 4194304;
pub const GPU_FPS: u32 = 60;
pub const MILLIS_PER_FRAME: u32 = 1000 / GPU_FPS;
pub const TICKS_PER_FRAME: u32 = CPU_FREQ / 1000 * MILLIS_PER_FRAME;

pub mod args;
pub mod cpu;
pub(crate) mod gpu;
pub(crate) mod joypad;
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

#[cfg(test)]
mod test {}
