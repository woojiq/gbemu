use crate::bit;

// Namings: https://gbdev.gg8.se/wiki/articles/Gameboy_sound_hardware

pub struct Sound {
    on: bool,
    channel1: SquareChannel,
    channel2: SquareChannel,
    channel3: WaveChannel,
    channel4: NoiseChannel,
    /// Each channel can be panned hard left, center, hard right, or ignored entirely.
    panning: u8,
}

struct SquareChannel {
    on: bool,
    sweep_on: bool,
    sweep_period: u8,
    sweep_negate: bool,
    sweep_shift: u8,
}

struct WaveChannel {
    on: bool,
}

struct NoiseChannel {
    on: bool,
}

impl Sound {
    pub fn new() -> Self {
        Self {
            on: false,
            channel1: SquareChannel::new(true),
            channel2: SquareChannel::new(false),
            channel3: WaveChannel::new(),
            channel4: NoiseChannel::new(),
            panning: 0,
        }
    }

    pub fn write_byte(&mut self, addr: u16, val: u8) {
        match addr {
            0xFF10..=0xFF14 => self.channel1.write_byte(addr, val),
            0xFF16..=0xFF19 => self.channel2.write_byte(addr, val),
            0xFF1A..=0xFF1E => self.channel3.write_byte(addr, val),
            0xFF20..=0xFF23 => self.channel4.write_byte(addr, val),
            0xFF24 => todo!(),
            0xFF25 => self.panning = val,
            0xFF26 => {
                let new_on = bit!(val, 7);

                if self.on && !new_on {
                    // Reset all registers when turning off
                    for i in 0xFF10..=0xFF25 {
                        self.write_byte(i, 0);
                    }
                }

                self.on = new_on;
            }
            0xFF30..=0xFF3F => todo!(),
            _ => panic!("0x{addr:X} is not sound register."),
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0xFF10..=0xFF14 => self.channel1.read_byte(addr),
            0xFF16..=0xFF19 => self.channel2.read_byte(addr),
            0xFF1A..=0xFF1E => self.channel3.read_byte(addr),
            0xFF20..=0xFF23 => self.channel4.read_byte(addr),
            0xFF24 => todo!(),
            0xFF25 => self.panning,
            0xFF26 => {
                ((self.on as u8) << 7)
                    | ((self.channel4.on as u8) << 3)
                    | ((self.channel3.on as u8) << 2)
                    | ((self.channel2.on as u8) << 1)
                    | ((self.channel1.on as u8) << 0)
            }
            0xFF30..=0xFF3F => todo!(),
            _ => panic!("0x{addr:X} is not sound register."),
        }
    }
}

impl SquareChannel {
    pub fn new(sweep_on: bool) -> Self {
        Self {
            on: false,
            sweep_on,
            sweep_period: 0,
            sweep_negate: false,
            sweep_shift: 0,
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            // Pulse (channel 2) sound channel works exactly like channel 1, except that it lacks a
            // period sweep.
            0xFF10 => {
                (1 << 7)
                    | (self.sweep_period << 4)
                    | ((self.sweep_negate as u8) << 3)
                    | (self.sweep_shift)
            }
            0xFF11 | 0xFF16 => {}
            0xFF12 | 0xFF17 => {}
            0xFF13 | 0xFF18 => {}
            0xFF14 | 0xFF19 => {}
            _ => panic!("Trying to read 0x{addr:X} - not Square Channel register."),
        }
    }

    pub fn write_byte(&mut self, addr: u16, val: u8) {
        match addr {
            0xFF10 => {
                // TODO In addition mode, if the period value would overflow (i.e. is strictly
                // more than $7FF), the channel is turned off instead. This occurs even if sweep
                // iterations are disabled by the pace being 0.
                self.sweep_period = (val >> 4) & 7;
                self.sweep_negate = (val >> 3) == 1;
                self.sweep_shift = val & 7;
            }
            0xFF11 | 0xFF16 => {}
            0xFF12 | 0xFF17 => {}
            0xFF13 | 0xFF18 => {}
            0xFF14 | 0xFF19 => {}
            _ => panic!("Trying to read 0x{addr:X} - not Square Channel register."),
        }
    }
}

impl WaveChannel {
    pub fn new() -> Self {
        Self { on: false }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            _ => panic!("0x{addr:X} is not Wave Channel register."),
        }
    }

    pub fn write_byte(&mut self, addr: u16, val: u8) {
        match addr {
            _ => panic!("0x{addr:X} is not Wave Channel register."),
        }
    }
}

impl NoiseChannel {
    pub fn new() -> Self {
        Self { on: false }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            _ => panic!("0x{addr:X} is not Noise Channel register."),
        }
    }

    pub fn write_byte(&mut self, addr: u16, val: u8) {
        match addr {
            _ => panic!("0x{addr:X} is not Noise Channel register."),
        }
    }
}
