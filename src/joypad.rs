use crate::bit;

#[derive(Copy, Clone, Debug)]
pub enum JoypadKey {
    Right,
    Left,
    Up,
    Down,
    A,
    B,
    Select,
    Start,
}

#[derive(Copy, Clone, Default)]
pub struct Joypad {
    // true - pressed
    // false - released
    is_dpad: bool,
    down: bool,
    up: bool,
    left: bool,
    right: bool,

    is_buttons: bool,
    start: bool,
    select: bool,
    b: bool,
    a: bool,
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            is_buttons: false,
            is_dpad: false,
            down: false,
            up: false,
            left: false,
            right: false,
            start: false,
            select: false,
            b: false,
            a: false,
        }
    }

    pub fn key_up(&mut self, key: JoypadKey) -> bool {
        self.key_press(key, false)
    }

    pub fn key_down(&mut self, key: JoypadKey) -> bool {
        self.key_press(key, true)
    }

    /// # Returns
    ///
    /// Whether an interrupt should occur.
    fn key_press(&mut self, key: JoypadKey, is_pressed: bool) -> bool {
        use JoypadKey::*;

        let old = u8::from(*self) & 0xF;

        match key {
            Right => self.right = is_pressed,
            Left => self.left = is_pressed,
            Up => self.up = is_pressed,
            Down => self.down = is_pressed,
            A => self.a = is_pressed,
            B => self.b = is_pressed,
            Select => self.select = is_pressed,
            Start => self.start = is_pressed,
        }

        let new = u8::from(*self) & 0xF;

        // Sets the JP bit in IF any time the low 4 bits of the joypad register go from all 1s to
        // any 0s.
        old == 0xF && new != 0xF
    }

    pub fn set_mode(&mut self, val: u8) {
        self.is_dpad = !bit!(val, 4);
        self.is_buttons = !bit!(val, 5);
    }

    fn bit0(&self) -> bool {
        (self.a && self.is_buttons) || (self.right && self.is_dpad)
    }

    fn bit1(&self) -> bool {
        (self.b && self.is_buttons) || (self.left && self.is_dpad)
    }

    fn bit2(&self) -> bool {
        (self.select && self.is_buttons) || (self.up && self.is_dpad)
    }

    fn bit3(&self) -> bool {
        (self.start && self.is_buttons) || (self.down && self.is_dpad)
    }
}

impl From<Joypad> for u8 {
    fn from(v: Joypad) -> Self {
        (!v.bit0() as u8)
            | ((!v.bit1() as u8) << 1)
            | ((!v.bit2() as u8) << 2)
            | ((!v.bit3() as u8) << 3)
            | ((!v.is_dpad as u8) << 4)
            | ((!v.is_buttons as u8) << 5)
            | (1 << 6)
            | (1 << 7)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_u8() {
        assert_eq!(false as u8, 0);
        assert_eq!(true as u8, 1);

        let joypad = Joypad {
            is_dpad: true,
            ..Default::default()
        };

        assert_eq!(u8::from(joypad), 0b11101111);
    }

    #[test]
    fn set_mode() {
        let mut joypad = Joypad::default();
        assert_eq!(u8::from(joypad), 0xFF);

        joypad.set_mode(32);
        assert_eq!(u8::from(joypad), 0xEF);

        joypad.set_mode(16);
        assert_eq!(u8::from(joypad), 0xDF);
    }
}
