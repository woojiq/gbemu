use crate::bit;

use super::PpuMode;

#[derive(Copy, Clone)]
pub struct LcdStatus {
    // FF41 — STAT: LCD status
    pub lyc_int_select: bool,
    /// Mode 2
    pub oam_scan_interrupt: bool,
    /// Mode 1
    pub vblank_interrupt: bool,
    /// Mode 0
    pub hblank_interrupt: bool,
    // read-only for operations
    same_line_check: bool,
    pub ppu_mode: PpuMode,

    // FF44 — LY: LCD Y coordinate [read-only]
    ly: u8,

    // FF45 — LYC: LY compare
    lyc: u8,
}

#[derive(Copy, Clone)]
pub struct LcdControl {
    // starting from bit 7:
    /// This bit controls whether the LCD is on and the PPU is active. Setting
    /// it to 0 turns both off, which grants immediate and full access to VRAM,
    /// OAM, etc.
    pub lcd_enable: bool,
    /// This bit controls which background map the Window uses for rendering.
    /// When it’s clear (0), the $9800 tilemap is used, otherwise it’s the $9C00
    /// one.
    pub window_tile_map_area: bool,
    /// This bit controls whether the window shall be displayed or not.
    pub window_enable: bool,
    /// This bit controls which addressing mode the BG and Window use to pick
    /// tiles.
    pub bg_and_window_tile_data_area: bool,
    /// If the bit is clear (0), the BG uses tilemap $9800, otherwise tilemap
    /// $9C00.
    pub bg_tile_map_area: bool,
    /// This bit controls the size of all objects (1 tile or 2 stacked
    /// vertically).
    pub obj_size: bool,
    /// This bit toggles whether objects are displayed or not.
    pub obj_enable: bool,
    /// When Bit 0 is cleared, both background and window become blank (white),
    /// and the Window Display Bit is ignored in that case. Only objects may
    /// still be displayed (if enabled in Bit 1).
    pub bg_and_window_display: bool,
}

impl LcdStatus {
    pub fn new() -> Self {
        Self {
            lyc_int_select: false,
            oam_scan_interrupt: false,
            vblank_interrupt: false,
            hblank_interrupt: false,
            same_line_check: false,
            ppu_mode: PpuMode::HBlank,
            ly: 0,
            lyc: 0,
        }
    }

    pub fn line(&self) -> u8 {
        self.ly
    }

    #[must_use]
    pub fn set_line(&mut self, new_line: u8) -> bool {
        self.ly = new_line;

        self.compare_lines()
    }

    pub fn lyc(&self) -> u8 {
        self.lyc
    }

    #[must_use]
    pub fn set_lyc(&mut self, new_val: u8) -> bool {
        self.lyc = new_val;

        self.compare_lines()
    }

    pub fn compare_lines(&mut self) -> bool {
        self.same_line_check = self.ly == self.lyc;

        self.lyc_int_select && self.same_line_check
    }

    pub fn write_byte_to_status(&mut self, val: u8) {
        self.lyc_int_select = bit!(val, 6);
        self.oam_scan_interrupt = bit!(val, 5);
        self.vblank_interrupt = bit!(val, 4);
        self.hblank_interrupt = bit!(val, 3);
        // Other fields are read-only.
    }

    pub fn get_status_byte(&self) -> u8 {
        ((self.lyc_int_select as u8) << 6)
            | ((self.oam_scan_interrupt as u8) << 5)
            | ((self.vblank_interrupt as u8) << 4)
            | ((self.hblank_interrupt as u8) << 3)
            | ((self.same_line_check as u8) << 2)
            | u8::from(self.ppu_mode)
    }

    pub fn ly(&self) -> u8 {
        self.ly
    }
}

impl LcdControl {
    pub fn new() -> Self {
        Self {
            lcd_enable: false,
            window_tile_map_area: false,
            window_enable: false,
            bg_and_window_tile_data_area: false,
            bg_tile_map_area: false,
            obj_size: false,
            obj_enable: false,
            bg_and_window_display: false,
        }
    }
}

impl From<LcdControl> for u8 {
    fn from(val: LcdControl) -> Self {
        ((val.lcd_enable as u8) << 7)
            | ((val.window_tile_map_area as u8) << 6)
            | ((val.window_enable as u8) << 5)
            | ((val.bg_and_window_tile_data_area as u8) << 4)
            | ((val.bg_tile_map_area as u8) << 3)
            | ((val.obj_size as u8) << 2)
            | ((val.obj_enable as u8) << 1)
            | ((val.bg_and_window_display as u8) << 0)
    }
}
