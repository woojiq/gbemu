use crate::{
    bit,
    memory_bus::{OAM_SIZE, VIDEO_RAM_SIZE, VIDEO_RAM_START},
};

pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

pub struct GPU {
    // 4: RGBA
    pub buffer: [[[u8; 4]; SCREEN_HEIGHT]; SCREEN_WIDTH],
    pub vram: [u8; VIDEO_RAM_SIZE],
    pub oam: [u8; OAM_SIZE],
    pub lcd_control: LcdControl,
    pub lcd_status: LcdStatus,
    /// Specify the top-left coordinates of the visible 160×144 pixel area
    /// within the 256×256 pixels BG map.
    /// SCY, SCX.
    pub viewport: Coordinate,
    /// Specify the on-screen coordinates of the Window’s top-left pixel.
    /// The X Position -7.
    pub window: Coordinate,

    pub bg_colors: BackgroundColors,
    pub obj0_colors: BackgroundColors,
    pub obj1_colors: BackgroundColors,

    cycles: usize,
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

#[derive(Copy, Clone)]
pub struct LcdStatus {
    // FF41 — STAT: LCD status
    pub lyc_int_select: bool,
    pub oam_scan_interrupt: bool,
    pub vblank_interrupt: bool,
    pub hblank_interrupt: bool,
    // read-only
    same_line_check: bool,
    ppu_mode: PpuMode,

    // FF44 — LY: LCD Y coordinate [read-only]
    ly: u8,

    // FF45 — LYC: LY compare
    pub lyc: u8,
}

#[derive(Copy, Clone)]
pub enum PpuMode {
    HBlank,
    VBlank,
    SearchingSprites,
    TransferDataToLCD,
}

#[derive(Copy, Clone, Default)]
pub struct Coordinate {
    pub x: u8,
    pub y: u8,
}

// Starts from ID 0.
#[derive(Copy, Clone)]
pub struct BackgroundColors(Color, Color, Color, Color);

#[derive(Copy, Clone)]
pub enum Color {
    White = 0,
    LightGray = 1,
    DarkGray = 2,
    Black = 3,
}

#[derive(Copy, Clone, Default)]
pub struct GpuInterrupts {
    pub vblank: bool,
    pub lcd: bool,
}

#[derive(Copy, Clone)]
pub struct Oam {
    pos: Coordinate,
    tile_idx: u8,
    attrs: OamAttributes,
}

#[derive(Copy, Clone)]
pub struct OamAttributes {
    prio: bool,
    y_flip: bool,
    x_flip: bool,
    /// If the palette property is 1 then OBP1 is used, otherwise OBP0 is used.
    dmg_palette: bool,
    // Bank and CGB palette are not used in Gameboy.
}

impl GPU {
    pub fn new() -> Self {
        Self {
            buffer: [[[0; 4]; SCREEN_HEIGHT]; SCREEN_WIDTH],
            vram: [0; VIDEO_RAM_SIZE],
            oam: [0; OAM_SIZE],
            lcd_control: LcdControl::new(),
            lcd_status: LcdStatus::new(),
            viewport: Coordinate::default(),
            window: Coordinate::default(),

            bg_colors: BackgroundColors::new(),
            obj0_colors: BackgroundColors::new(),
            obj1_colors: BackgroundColors::new(),

            cycles: 0,
        }
    }

    pub fn step(&mut self, cycles: usize) -> GpuInterrupts {
        const ALL_MODES_DOTS: usize = 456;
        const MODE2_DOTS: usize = 80;
        const MODE3_DOTS: usize = 172;
        const MODE0_DOTS: usize = ALL_MODES_DOTS - MODE2_DOTS - MODE3_DOTS;

        let mut inter = GpuInterrupts::default();

        if !self.lcd_control.lcd_enable {
            return inter;
        }

        self.cycles += cycles;

        // ref: http://www.codeslinger.co.uk/pages/projects/gameboy/lcd.html
        /* When starting a new scanline the lcd status is set to 2, it then
        moves on to 3 and then to 0. It then goes back to and continues then
        pattern until the v-blank period starts where it stays on mode 1. When
        the vblank period ends it goes back to 2 and continues this pattern
        over and over. As previously mentioned it takes 456 clock cycles to
        draw one scanline before moving onto the next. This can be split down
        into different sections which will represent the different modes. Mode 2
        (Searching Sprites Atts) will take the first 80 of the 456 clock cycles.
        Mode 3 (Transfering to LCD Driver) will take 172 clock cycles of the 456
        and the remaining clock cycles of the 456 is for Mode 0 (H-Blank). */
        match self.lcd_status.ppu_mode {
            PpuMode::HBlank => {
                if self.cycles >= MODE0_DOTS {
                    self.cycles -= MODE0_DOTS;
                    self.lcd_status.ly += 1;

                    if self.lcd_status.ly <= SCREEN_HEIGHT as u8 {
                        self.lcd_status.ppu_mode = PpuMode::SearchingSprites;
                        if self.lcd_status.oam_scan_interrupt {
                            inter.lcd = true;
                        }
                    } else {
                        self.lcd_status.ppu_mode = PpuMode::VBlank;
                        if self.lcd_status.vblank_interrupt {
                            inter.lcd = true;
                        }
                    }
                }
            }
            PpuMode::VBlank => {
                // 10 scanlines x 456 dots
                if self.cycles >= ALL_MODES_DOTS {
                    self.cycles -= ALL_MODES_DOTS;
                    self.lcd_status.ly += 1;

                    if self.lcd_status.ly > 153 {
                        self.lcd_status.ly = 0;
                        self.lcd_status.ppu_mode = PpuMode::SearchingSprites;
                        if self.lcd_status.oam_scan_interrupt {
                            inter.lcd = true;
                        }
                    }
                }
            }
            PpuMode::SearchingSprites => {
                if self.cycles >= MODE2_DOTS {
                    self.cycles -= MODE2_DOTS;
                    self.lcd_status.ppu_mode = PpuMode::TransferDataToLCD;
                }
            }
            PpuMode::TransferDataToLCD => {
                if self.cycles >= MODE3_DOTS {
                    self.cycles -= MODE3_DOTS;

                    self.lcd_status.ppu_mode = PpuMode::HBlank;
                    if self.lcd_status.hblank_interrupt {
                        inter.lcd = true;
                    }
                    self.draw_line();
                }
            }
        }

        if self.lcd_status.compare_lines() {
            inter.lcd = true;
        }

        inter
    }

    fn draw_line(&mut self) {
        if self.lcd_control.bg_and_window_display {
            self.draw_tiles();
        }

        if self.lcd_control.obj_enable {
            self.draw_sprites();
        }
    }

    fn draw_tiles(&mut self) {
        // background is 256x256. Each tile is 8x8 pixels x2 (for color) = 16 byte.
        // background is 32x32 tiles. Each tile 16 bytes.

        let use_window = self.lcd_control.window_enable && self.window.y <= self.lcd_status.ly;

        for screen_x in 0..(SCREEN_WIDTH as u8) {
            let tile = {
                let x = self.viewport.x + screen_x;
                let tile_x = if use_window && self.window.x <= x + 7 {
                    x + 7 - self.window.x
                } else {
                    x
                };

                let y = self.viewport.y + self.lcd_status.ly;
                let tile_y = if use_window && self.window.y <= y {
                    y - self.window.y
                } else {
                    y
                };

                Coordinate::new(tile_x, tile_y)
            };

            let bg_mem = if use_window {
                if self.lcd_control.window_tile_map_area {
                    0x9C00u16
                } else {
                    0x9800
                }
            } else {
                if self.lcd_control.bg_tile_map_area {
                    0x9C00
                } else {
                    0x9800
                }
            };

            let tile_data = if self.lcd_control.bg_and_window_tile_data_area {
                0x8000u16
            } else {
                0x8800
            };

            let tile_map_idx = (tile.y / 8) * 32 + tile.x / 8;

            let tile_idx = {
                let addr = bg_mem + tile_map_idx as u16;
                self.vram[(addr - VIDEO_RAM_START) as usize] + 128
            };

            let tile_addr = tile_data + tile_idx as u16 * 16;
            let line = (tile.y % 8) as u16 * 2;

            let data = [
                self.vram[(tile_addr + line - VIDEO_RAM_START) as usize],
                self.vram[(tile_addr + line + 1 - VIDEO_RAM_START) as usize],
            ];

            let pixel = 7 - screen_x % 8;
            let color = {
                let color_raw = (((data[0] >> pixel) & 1) << 1) | ((data[1] >> pixel) & 1);
                self.bg_colors.get()[color_raw as usize].rgb()
            };

            self.buffer[screen_x as usize][self.lcd_status.ly as usize] = [color, color, color, 0];
        }
    }

    fn draw_sprites(&mut self) {
        if !self.lcd_control.obj_enable {
            return;
        }

        let obj_height = if self.lcd_control.obj_size { 16 } else { 8 };

        for sprite_attr_addr in ((0xFE00 - VIDEO_RAM_START)..=(0xFE9F - VIDEO_RAM_START)).step_by(4)
        {
            let mem: [u8; 4] = self.vram
                [sprite_attr_addr as usize..(sprite_attr_addr + 4) as usize]
                .try_into()
                .unwrap();
            let obj = Oam::from(mem);

            if !(obj.pos.y <= self.lcd_status.ly && self.lcd_status.ly < obj.pos.y + obj_height) {
                continue;
            }

            let mut line = self.lcd_status.ly - obj.pos.y;
            if obj.attrs.y_flip {
                line = obj_height - line;
            }

            let addr = 0x8000 + obj.tile_idx as u16 * 16 + line as u16 * 2 - VIDEO_RAM_START;

            let data = [self.vram[addr as usize], self.vram[addr as usize + 1]];

            for pixel_x in (0..8).rev() {
                let color_bit = if obj.attrs.x_flip {
                    7 - pixel_x
                } else {
                    pixel_x
                };

                let color = {
                    let color_raw =
                        (((data[0] >> color_bit) & 1) << 1) | ((data[1] >> color_bit) & 1);
                    if obj.attrs.dmg_palette {
                        self.obj1_colors.get()[color_raw as usize].rgb()
                    } else {
                        self.obj0_colors.get()[color_raw as usize].rgb()
                    }
                };

                // Note that while 4 colors are stored per OBJ palette, color #0
                // is never used, as it’s always transparent.
                if color == 255 {
                    continue;
                }

                let buffer_x = 7 - pixel_x + obj.pos.x;

                self.buffer[buffer_x as usize][self.lcd_status.ly as usize] =
                    [color, color, color, 0];
            }
        }
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

impl From<u8> for LcdControl {
    fn from(val: u8) -> Self {
        use crate::bit;

        Self {
            lcd_enable: bit!(val, 7),
            window_tile_map_area: bit!(val, 6),
            window_enable: bit!(val, 5),
            bg_and_window_tile_data_area: bit!(val, 4),
            bg_tile_map_area: bit!(val, 3),
            obj_size: bit!(val, 2),
            obj_enable: bit!(val, 1),
            bg_and_window_display: bit!(val, 0),
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

    fn compare_lines(&mut self) -> bool {
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

impl From<PpuMode> for u8 {
    fn from(val: PpuMode) -> Self {
        match val {
            PpuMode::HBlank => 0,
            PpuMode::VBlank => 1,
            PpuMode::SearchingSprites => 2,
            PpuMode::TransferDataToLCD => 3,
        }
    }
}

impl Coordinate {
    pub fn new(x: u8, y: u8) -> Self {
        Self { x, y }
    }
}

impl BackgroundColors {
    pub fn new() -> Self {
        Self(
            Color::White,
            Color::LightGray,
            Color::DarkGray,
            Color::Black,
        )
    }

    pub fn get(&self) -> [Color; 4] {
        [self.0, self.1, self.2, self.3]
    }
}

impl From<u8> for BackgroundColors {
    fn from(v: u8) -> Self {
        Self(
            Color::from(v & 0b11),
            Color::from((v >> 2) & 0b11),
            Color::from((v >> 4) & 0b11),
            Color::from((v >> 6) & 0b11),
        )
    }
}

impl From<BackgroundColors> for u8 {
    fn from(val: BackgroundColors) -> Self {
        ((val.3 as u8) << 6) | ((val.2 as u8) << 4) | ((val.1 as u8) << 2) | ((val.0 as u8) << 0)
    }
}

impl Color {
    pub fn rgb(&self) -> u8 {
        match self {
            Color::White => 255,
            Color::LightGray => 211,
            Color::DarkGray => 68,
            Color::Black => 0,
        }
    }
}

impl From<u8> for Color {
    fn from(val: u8) -> Self {
        match val {
            0 => Self::White,
            1 => Self::LightGray,
            2 => Self::DarkGray,
            3 => Self::Black,
            _ => panic!("{val} is invalid color."),
        }
    }
}

impl From<[u8; 4]> for Oam {
    fn from(val: [u8; 4]) -> Self {
        Self {
            pos: Coordinate::new(val[1] - 8, val[0] - 16),
            tile_idx: val[2],
            attrs: OamAttributes::from(val[3]),
        }
    }
}

impl From<u8> for OamAttributes {
    fn from(val: u8) -> Self {
        Self {
            prio: bit!(val, 7),
            y_flip: bit!(val, 6),
            x_flip: bit!(val, 5),
            dmg_palette: bit!(val, 4),
        }
    }
}
