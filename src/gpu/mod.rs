mod lcd_registers;

use crate::{
    bit,
    memory_bus::{OAM_SIZE, OAM_START, VIDEO_RAM_SIZE, VIDEO_RAM_START},
    SCREEN_HEIGHT, SCREEN_WIDTH,
};
use lcd_registers::{LcdControl, LcdStatus};

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

    // https://gbdev.io/pandocs/Scrolling.html#window:
    /// Whether at some point in this frame the value of WY was equal to LY (checked at the start of
    /// Mode 2 only)
    window_y_trigger: bool,
    /// The Y position is selected by an internal counter, which is reset to 0 during VBlank and
    /// only incremented when the Window starts being rendered on a given scanline.
    window_current_y: u8,

    pub bg_colors: BackgroundColors,
    pub obj0_colors: BackgroundColors,
    pub obj1_colors: BackgroundColors,

    cycles: u32,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum PpuMode {
    HBlank,
    VBlank,
    OAMScan,
    DrawingPixels,
}

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
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

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Oam {
    pos: Coordinate<i16>,
    tile_idx: u8,
    attrs: OamAttributes,
    oam_idx: usize,
}

impl Oam {
    fn new(oam_idx: usize, oam_height: u16, val: [u8; 4]) -> Self {
        Self {
            pos: Coordinate::new(val[1] as i16 - 8, val[0] as i16 - 16),
            // In 8×16 mode the least significant bit of the tile index is ignored.
            tile_idx: val[2] & if oam_height == 16 { !1 } else { !0 },
            attrs: OamAttributes::from(val[3]),
            oam_idx,
        }
    }
}

impl std::cmp::PartialOrd for Oam {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for Oam {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.pos.x != other.pos.x {
            self.pos.x.cmp(&other.pos.x)
        } else {
            self.oam_idx.cmp(&other.oam_idx)
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct OamAttributes {
    /// Sprite to Background Priority: If this flag is set to 0 then sprite
    /// is always rendered above the background and the window. However if it
    /// is set to 1 then the sprite is hidden behind the background and window
    /// unless the colour of the background or window is white, then it is still
    /// rendered on top.
    bg_prio: bool,
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

            window_current_y: 0,
            window_y_trigger: false,

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
            // TODO: Why 4? Maybe this:
            // https://gist.github.com/SonoSooS/c0055300670d678b5ae8433e20bea595#fetch-and-stuff
            self.cycles = 4;
        } else if self.lcd_control.lcd_enable && !new.lcd_enable {
            self.cycles = 0;
            if self.lcd_status.set_line(0) {
                inter.lcd = true;
            }
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
                if self
                    .lcd_status
                    .set_line((self.lcd_status.line() + 1) % (LAST_SCANLINE + 1))
                {
                    inter.lcd = true;
                }

                if self.lcd_status.ppu_mode != PpuMode::VBlank
                    && self.lcd_status.line() > LAST_VISIBLE_SCANLINE
                {
                    self.switch_to_mode(PpuMode::VBlank, &mut inter);
                }
            }

            if self.lcd_status.line() <= LAST_VISIBLE_SCANLINE {
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

                self.window_current_y = 0;
                self.window_y_trigger = false;

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
                if self.lcd_control.window_enable && self.lcd_status.line() == self.window.y {
                    self.window_y_trigger = true;
                }
            }
        }
    }

    fn draw_line(&mut self) {
        self.draw_tiles();

        // TODO: Profile this shit: copying w*h*3 before drawing every line is too much.
        let bg_state = self.buffer;

        self.draw_sprites(&bg_state);
    }

    fn draw_tiles(&mut self) {
        // background is 256x256. Each tile is 8x8 pixels x2 (for color) = 16 byte.
        // background is 32x32 tiles. Each tile 16 bytes.

        if !self.lcd_control.bg_and_window_display {
            return;
        }

        for screen_x in 0..(SCREEN_WIDTH as u8) {
            let tile = self.get_tile_addr(screen_x);
            let bg_mem = self.get_bg_mem(screen_x);

            let tile_data = if self.lcd_control.bg_and_window_tile_data_area {
                0x8000u16
            } else {
                0x8800
            };

            let tile_map_idx = (tile.y as u16 / 8) * 32 + tile.x as u16 / 8;

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

            let pixel = 7 - tile.x % 8;
            let color = {
                let color_raw = (((data[1] >> pixel) & 1) << 1) | ((data[0] >> pixel) & 1);
                self.bg_colors.get()[color_raw as usize].rgb()
            };

            self.buffer[screen_x as usize][self.lcd_status.line() as usize] = [color, color, color];
        }

        if self.is_window_visible(SCREEN_WIDTH as u8 - 1) {
            self.window_current_y += 1;
        }
    }

    fn draw_sprites(&mut self, bg_state: &[[[u8; 3]; SCREEN_HEIGHT]; SCREEN_WIDTH]) {
        // The Game Boy PPU can display up to 40 movable objects (or sprites), each 8×8 or
        // 8×16 pixels. Because of a limitation of hardware, only 10 objects can be displayed per
        // scanline.
        const MAX_OBJS_PER_SCANLINE: usize = 10;

        if !self.lcd_control.obj_enable {
            return;
        }

        let obj_height = if self.lcd_control.obj_size { 16u16 } else { 8 };
        let mut objs_to_draw = Vec::with_capacity(40);

        for sprite_attr_addr in ((0xFE00 - OAM_START)..=(0xFE9F - OAM_START)).step_by(4) {
            if objs_to_draw.len() == MAX_OBJS_PER_SCANLINE {
                break;
            }

            let mem: [u8; 4] = self.oam[sprite_attr_addr as usize..(sprite_attr_addr + 4) as usize]
                .try_into()
                .unwrap();
            let obj = Oam::new(sprite_attr_addr as usize / 4, obj_height, mem);

            if !(obj.pos.y <= self.lcd_status.line() as i16
                && (self.lcd_status.line() as i16) < obj.pos.y + obj_height as i16)
            {
                continue;
            }

            objs_to_draw.push(obj);
        }
        objs_to_draw.sort_unstable();
        objs_to_draw.reverse();

        for obj in objs_to_draw {
            let line = if obj.attrs.y_flip {
                obj_height - 1 - (self.lcd_status.line() as i16 - obj.pos.y) as u16
            } else {
                (self.lcd_status.line() as i16 - obj.pos.y) as u16
            };

            let addr = 0x8000 + obj.tile_idx as u16 * 16 + line * 2 - VIDEO_RAM_START;

            let data = [self.vram[addr as usize], self.vram[addr as usize + 1]];

            for pixel_x in (0..8).rev() {
                if !(0 <= obj.pos.x + pixel_x && obj.pos.x + pixel_x < SCREEN_WIDTH as i16) {
                    continue;
                }

                let color_bit = if obj.attrs.x_flip {
                    pixel_x
                } else {
                    7 - pixel_x
                };

                let color = {
                    let color_raw =
                        (((data[1] >> color_bit) & 1) << 1) | ((data[0] >> color_bit) & 1);
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

                if obj.attrs.bg_prio
                    && bg_state[buffer_x as usize][self.lcd_status.line() as usize][0]
                        != Color::White.rgb()
                {
                    continue;
                }

                self.buffer[buffer_x as usize][self.lcd_status.line() as usize] =
                    [color, color, color];
            }
        }
    }

    fn is_window_visible(&self, screen_x: u8) -> bool {
        self.lcd_control.window_enable && self.window_y_trigger && self.window.x <= screen_x + 7
    }

    fn get_tile_addr(&mut self, screen_x: u8) -> Coordinate<u8> {
        if self.is_window_visible(screen_x) {
            Coordinate::new(screen_x + 7 - self.window.x, self.window_current_y)
        } else {
            Coordinate::new(
                self.viewport.x.wrapping_add(screen_x),
                self.viewport.y.wrapping_add(self.lcd_status.line()),
            )
        }
    }

    fn get_bg_mem(&self, screen_x: u8) -> u16 {
        if self.is_window_visible(screen_x) {
            if self.lcd_control.window_tile_map_area {
                0x9C00
            } else {
                0x9800
            }
        } else {
            if self.lcd_control.bg_tile_map_area {
                0x9C00
            } else {
                0x9800
            }
        }
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

impl From<u8> for OamAttributes {
    fn from(val: u8) -> Self {
        Self {
            bg_prio: bit!(val, 7),
            y_flip: bit!(val, 6),
            x_flip: bit!(val, 5),
            dmg_palette: bit!(val, 4),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn viewport_coordinates_are_wrapped() {
        let mut gpu = GPU::new();

        gpu.viewport = Coordinate::new(200, 200);
        assert_eq!(gpu.get_tile_addr(100), Coordinate::new(44, 200));

        let _ = gpu.lcd_status.set_line(100);
        assert_eq!(gpu.get_tile_addr(100), Coordinate::new(44, 44));
    }
}
