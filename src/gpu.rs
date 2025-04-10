use crate::{
    bit,
    memory_bus::{OAM_END, OAM_SIZE, OAM_START, VIDEO_RAM_SIZE, VIDEO_RAM_START},
    SCREEN_HEIGHT, SCREEN_WIDTH,
};

pub struct GPU {
    // 3: RGB
    pub buffer: [[[u8; 3]; SCREEN_HEIGHT]; SCREEN_WIDTH],
    pub vram: [u8; VIDEO_RAM_SIZE],
    pub oam: [u8; OAM_SIZE],
    pub lcd_control: LcdControl,
    pub lcd_status: LcdStatus,
    /// Specify the top-left coordinates of the visible 160×144 pixel area
    /// within the 256×256 pixels BG map.
    /// SCY, SCX.
    pub viewport: Coordinate<u8>,
    /// Specify the on-screen coordinates of the Window’s top-left pixel.
    /// The X Position -7.
    pub window: Coordinate<u8>,

    pub bg_colors: BackgroundColors,
    pub obj0_colors: BackgroundColors,
    pub obj1_colors: BackgroundColors,

    // TODO: Remove pub.
    pub cycles: u32,
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
    /// Mode 2
    pub oam_scan_interrupt: bool,
    /// Mode 1
    pub vblank_interrupt: bool,
    /// Mode 0
    pub hblank_interrupt: bool,
    // read-only
    same_line_check: bool,
    ppu_mode: PpuMode,

    // FF44 — LY: LCD Y coordinate [read-only]
    ly: u8,

    // FF45 — LYC: LY compare
    pub lyc: u8,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum PpuMode {
    HBlank,
    VBlank,
    OAMScan,
    DrawingPixels,
}

#[derive(Copy, Clone, Default, Debug)]
pub struct Coordinate<T> {
    pub x: T,
    pub y: T,
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
    pos: Coordinate<i16>,
    tile_idx: u8,
    attrs: OamAttributes,
}

#[derive(Copy, Clone)]
pub struct OamAttributes {
    // FIXME: prio is not used.
    /// Sprite to Background Priority: If this flag is set to 0 then sprite
    /// is always rendered above the background and the window. However if it
    /// is set to 1 then the sprite is hidden behind the background and window
    /// unless the colour of the background or window is white, then it is still
    /// rendered on top.
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
            buffer: [[[0; 3]; SCREEN_HEIGHT]; SCREEN_WIDTH],
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

    pub fn set_lcd_control(&mut self, val: u8) -> GpuInterrupts {
        use crate::bit;

        let new = LcdControl {
            lcd_enable: bit!(val, 7),
            window_tile_map_area: bit!(val, 6),
            window_enable: bit!(val, 5),
            bg_and_window_tile_data_area: bit!(val, 4),
            bg_tile_map_area: bit!(val, 3),
            obj_size: bit!(val, 2),
            obj_enable: bit!(val, 1),
            bg_and_window_display: bit!(val, 0),
        };

        let mut inter = GpuInterrupts::default();
        if !self.lcd_control.lcd_enable && new.lcd_enable {
            self.switch_to_mode(PpuMode::OAMScan, &mut inter);
            // TODO: Why 4?
            self.cycles = 4;
        } else if self.lcd_control.lcd_enable && !new.lcd_enable {
            self.cycles = 0;
            self.lcd_status.ly = 0;
            self.lcd_status.ppu_mode = PpuMode::HBlank;
            self.clear_screen();
        }

        self.lcd_control = new;

        inter
    }

    fn clear_screen(&mut self) {
        self.buffer.fill([[Color::White.rgb(); 3]; SCREEN_HEIGHT]);
    }

    pub fn step(&mut self, mut cycles: u32) -> GpuInterrupts {
        const SCANLINE_DOTS: u32 = 456;
        const LAST_SCANLINE: u8 = 153;
        const LAST_VISIBLE_SCANLINE: u8 = 143;

        const OAM_SCAN_DOTS: u32 = 80;
        const DRAWING_PIXELS_DOTS: u32 = 172;

        let mut inter = GpuInterrupts::default();

        // dbg!(self.lcd_control.lcd_enable);
        if !self.lcd_control.lcd_enable {
            return inter;
        }

        // http://www.codeslinger.co.uk/pages/projects/gameboy/lcd.html
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
        while cycles > 0 {
            // The shortest mode is OAM scan (80 dots).
            let cycles_now = std::cmp::min(cycles, 80);
            cycles -= cycles_now;

            self.cycles += cycles_now;

            if self.cycles >= SCANLINE_DOTS {
                self.cycles -= SCANLINE_DOTS;
                self.lcd_status.ly = (self.lcd_status.ly + 1) % (LAST_SCANLINE + 1);

                if self.lcd_status.compare_lines() {
                    inter.lcd = true;
                }

                if self.lcd_status.ppu_mode != PpuMode::VBlank
                    && self.lcd_status.ly > LAST_VISIBLE_SCANLINE
                {
                    self.switch_to_mode(PpuMode::VBlank, &mut inter);
                }
            }

            if self.lcd_status.ly <= LAST_VISIBLE_SCANLINE {
                if self.cycles <= OAM_SCAN_DOTS {
                    if self.lcd_status.ppu_mode != PpuMode::OAMScan {
                        self.switch_to_mode(PpuMode::OAMScan, &mut inter);
                    }
                } else if self.cycles <= OAM_SCAN_DOTS + DRAWING_PIXELS_DOTS {
                    if self.lcd_status.ppu_mode != PpuMode::DrawingPixels {
                        self.switch_to_mode(PpuMode::DrawingPixels, &mut inter);
                    }
                } else {
                    if self.lcd_status.ppu_mode != PpuMode::HBlank {
                        self.switch_to_mode(PpuMode::HBlank, &mut inter);
                    }
                }
            }
        }

        inter
    }

    fn switch_to_mode(&mut self, new_mode: PpuMode, inter: &mut GpuInterrupts) {
        self.lcd_status.ppu_mode = new_mode;

        match new_mode {
            PpuMode::HBlank => {
                self.draw_line();
                if self.lcd_status.hblank_interrupt {
                    inter.lcd = true;
                }
            }
            PpuMode::VBlank => {
                inter.vblank = true;
                if self.lcd_status.vblank_interrupt {
                    inter.lcd = true;
                }
            }
            PpuMode::OAMScan => {
                if self.lcd_status.oam_scan_interrupt {
                    inter.lcd = true;
                }
            }
            PpuMode::DrawingPixels => {
                // TODO
            }
        }
    }

    fn draw_line(&mut self) {
        if self.lcd_control.bg_and_window_display {
            self.draw_tiles();
        }

        // if self.lcd_control.obj_enable {
        //     self.draw_sprites();
        // }
    }

    fn draw_tiles(&mut self) {
        // background is 256x256. Each tile is 8x8 pixels x2 (for color) = 16 byte.
        // background is 32x32 tiles. Each tile 16 bytes.

        let use_window = self.lcd_control.window_enable && self.window.y <= self.lcd_status.ly;

        if !self.lcd_control.bg_and_window_display {
            return;
        }

        for screen_x in 0..(SCREEN_WIDTH as u8) {
            let tile = {
                let x = self.viewport.x + screen_x;
                let tile_x = if use_window && self.window.x <= x + 7 {
                    x + 7 - self.window.x
                } else {
                    x
                };

                let y = self.viewport.y.wrapping_add(self.lcd_status.ly);
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

            let tile_map_idx = (tile.y as u16 / 8) * 32 + tile.x as u16 / 8;

            // TODO: Tests for this.
            let tile_addr = {
                let addr = bg_mem + tile_map_idx;
                // https://gbdev.io/pandocs/Tile_Data.html#vram-tile-data
                let v = self.vram[(addr - VIDEO_RAM_START) as usize];
                tile_data
                    + (if tile_data == 0x8000 {
                        v as u16
                    } else {
                        (v as i8 as i16 + 128) as u16
                    }) * 16
            };

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

            self.buffer[screen_x as usize][self.lcd_status.ly as usize] = [color, color, color];
        }
    }

    fn draw_sprites(&mut self) {
        // TODO: The Game Boy PPU can display up to 40 movable objects (or sprites), each 8×8 or
        // 8×16 pixels. Because of a limitation of hardware, only 10 objects can be displayed per
        // scanline.
        if !self.lcd_control.obj_enable {
            return;
        }

        let obj_height = if self.lcd_control.obj_size { 16u16 } else { 8 };

        for sprite_attr_addr in (0xFE00 - OAM_START..=(0xFE9F - OAM_END)).step_by(4) {
            let mem: [u8; 4] = self.oam[sprite_attr_addr as usize..(sprite_attr_addr + 4) as usize]
                .try_into()
                .unwrap();
            let obj = Oam::from(mem);

            if !(obj.pos.y <= self.lcd_status.ly as i16
                && (self.lcd_status.ly as i16) < obj.pos.y + obj_height as i16)
            {
                continue;
            }

            let mut line = (self.lcd_status.ly as i16 - obj.pos.y) as u16;
            if obj.attrs.y_flip {
                line = obj_height - line;
            }

            let addr = 0x8000 + obj.tile_idx as u16 * 16 + line * 2 - VIDEO_RAM_START;

            let data = [self.vram[addr as usize], self.vram[addr as usize + 1]];

            for pixel_x in (0..8).rev() {
                if !(0 <= obj.pos.x + pixel_x && obj.pos.x + pixel_x < SCREEN_WIDTH as i16) {
                    continue;
                }

                let color_bit = if obj.attrs.x_flip {
                    7 - pixel_x
                } else {
                    pixel_x
                };

                let color = {
                    let color_raw =
                        (((data[0] >> color_bit) & 1) << 1) | ((data[1] >> color_bit) & 1);
                    // Note that while 4 colors are stored per OBJ palette, color #0
                    // is never used, as it’s always transparent.
                    if color_raw == 0 {
                        continue;
                    }
                    if obj.attrs.dmg_palette {
                        self.obj1_colors.get()[color_raw as usize].rgb()
                    } else {
                        self.obj0_colors.get()[color_raw as usize].rgb()
                    }
                };

                let buffer_x = pixel_x + obj.pos.x;

                self.buffer[buffer_x as usize][self.lcd_status.ly as usize] = [color, color, color];
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

impl From<PpuMode> for u8 {
    fn from(val: PpuMode) -> Self {
        match val {
            PpuMode::HBlank => 0,
            PpuMode::VBlank => 1,
            PpuMode::OAMScan => 2,
            PpuMode::DrawingPixels => 3,
        }
    }
}

impl<T> Coordinate<T> {
    pub fn new(x: T, y: T) -> Self {
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
            pos: Coordinate::new(val[1] as i16 - 8, val[0] as i16 - 16),
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
