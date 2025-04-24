// Fix tests   : 10
// Passed tests: 1, 2, 3, 4, 5, 6, 7, 8, 9, 11, 12
use crate::{audio_player::AudioPlayer, bit};

// Namings: https://gbdev.gg8.se/wiki/articles/Gameboy_sound_hardware

// > The frame sequencer generates low frequency clocks for the modulation units. It is clocked by a
// 512 Hz timer.
const CPU_CYCLES_PER_FRAME_SEQ: u64 = crate::CPU_FREQ / 512;

// How often to generate audio samples to get 44.100 Hz.
const AUDIO_SAMPLE_FREQ: u64 = crate::CPU_FREQ / crate::SAMPLE_RATE;

pub struct Sound {
    enabled: bool,
    // > A channel is turned off when any of the following occurs:
    //    * The channel’s length timer is enabled in NRx4 and expires, or
    //    * For CH1 only: when the period sweep overflows, or
    //    * The channel’s DAC is turned off. The envelope reaching a volume of 0 does NOT turn the
    //      channel off!
    channel1: SquareChannel,
    channel2: SquareChannel,
    channel3: WaveChannel,
    channel4: NoiseChannel,
    /// Each channel can be panned hard left, center, hard right, or ignored entirely.
    panning: u8,

    // u4
    left_volume: u8,
    right_volume: u8,

    vin_l_enable: bool,
    vin_r_enable: bool,

    frame_seq_clock: u64,
    frame_seq: u8,

    audio_buffer_clock: u64,
    left_buf: [f32; crate::AUDIO_BUF_LEN],
    right_buf: [f32; crate::AUDIO_BUF_LEN],
    buf_filled: usize,

    player: Box<dyn AudioPlayer>,
}

// CH1, CH2
struct SquareChannel {
    enabled: bool,

    sweep: Option<Sweep>,
    length: LengthTimer,
    envelope: Envelope,

    duty_idx: u8,
    duty_iter: usize,

    period: Period,

    dac: bool,
}

// CH3
struct WaveChannel {
    enabled: bool,
    dac: bool,
    period: Period,
    length: LengthTimer,

    wave_idx: u8,
    waves: [u8; 16],
    // 00	Mute (No sound)
    // 01	100% volume (use samples read from Wave RAM as-is)
    // 10	50% volume (shift samples read from Wave RAM right once)
    // 11	25% volume (shift samples read from Wave RAM right twice)
    output_lvl: u8,
}

// CH4
struct NoiseChannel {
    enabled: bool,
    dac: bool,
    length: LengthTimer,
    envelope: Envelope,

    ff22: u8,
    lfsr: u16,

    cycles: u64,
    period: u64,
}

struct Sweep {
    enabled: bool,
    period: u8,
    timer: u8,
    negate: bool,
    negate_done: bool,
    shift: u8,
    shadow_freq: u16,

    disable_channel: bool,
}

struct Envelope {
    timer: u8,
    volume: u8,
    init_volume: u8,
    dir_up: bool,
    init_timer: u8,
}

#[derive(Debug)]
struct LengthTimer {
    enabled: bool,
    max_len: u16,
    timer: u16,
}

struct Period {
    period: u16,
    timer: u16,
    multiplier: u16,
    reloaded: bool,
}

impl Sound {
    pub fn new(player: Box<dyn AudioPlayer>) -> Self {
        Self {
            enabled: false,
            channel1: SquareChannel::new(true),
            channel2: SquareChannel::new(false),
            channel3: WaveChannel::new(),
            channel4: NoiseChannel::new(),
            panning: 0,
            left_volume: 7,
            right_volume: 7,
            vin_l_enable: false,
            vin_r_enable: false,

            frame_seq: 0,
            frame_seq_clock: 0,

            audio_buffer_clock: 0,
            left_buf: [0.0; crate::AUDIO_BUF_LEN],
            right_buf: [0.0; crate::AUDIO_BUF_LEN],
            buf_filled: 0,

            player,
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0xFF10..=0xFF14 => self.channel1.read_byte(addr),
            0xFF16..=0xFF19 => self.channel2.read_byte(addr),
            0xFF1A..=0xFF1E => self.channel3.read_byte(addr),
            0xFF20..=0xFF23 => self.channel4.read_byte(addr),
            0xFF24 => {
                ((self.vin_l_enable as u8) << 7)
                    | ((self.left_volume & 7) << 4)
                    | ((self.vin_r_enable as u8) << 3)
                    | (self.right_volume & 7)
            }
            0xFF25 => self.panning,
            0xFF26 => {
                ((self.enabled as u8) << 7)
                    | ((self.channel4.enabled as u8) << 3)
                    | ((self.channel3.enabled as u8) << 2)
                    | ((self.channel2.enabled as u8) << 1)
                    | ((self.channel1.enabled as u8) << 0)
                    | 0b01110000
            }
            0xFF30..=0xFF3F => self.channel3.read_byte(addr),
            _ => 0xFF,
        }
    }

    pub fn write_byte(&mut self, addr: u16, val: u8) {
        if !self.enabled {
            // Turning the APU off, however, does not affect Wave RAM, which can always be
            // read/written, nor the DIV-APU counter.
            match addr {
                0xFF11 => self
                    .channel1
                    .write_byte(addr, val & 0b111111, self.frame_seq),
                0xFF16 => self
                    .channel2
                    .write_byte(addr, val & 0b111111, self.frame_seq),
                0xFF1B => self.channel3.write_byte(addr, val, self.frame_seq),
                0xFF20 => self
                    .channel4
                    .write_byte(addr, val & 0b111111, self.frame_seq),
                _ => (),
            }
            if addr != 0xFF26 {
                return;
            }
        }

        match addr {
            0xFF10..=0xFF14 => self.channel1.write_byte(addr, val, self.frame_seq),
            0xFF16..=0xFF19 => self.channel2.write_byte(addr, val, self.frame_seq),
            0xFF1A..=0xFF1E => self.channel3.write_byte(addr, val, self.frame_seq),
            0xFF20..=0xFF23 => self.channel4.write_byte(addr, val, self.frame_seq),
            0xFF24 => {
                self.left_volume = (val >> 4) & 7;
                self.right_volume = (val >> 0) & 7;
                self.vin_l_enable = bit!(val, 7);
                self.vin_r_enable = bit!(val, 3);
            }
            0xFF25 => self.panning = val,
            0xFF26 => {
                let new_enabled = bit!(val, 7);

                if self.enabled && !new_enabled {
                    // Reset all registers when turning off
                    for i in 0xFF10..=0xFF25 {
                        self.write_byte(i, 0);
                    }
                }

                if !self.enabled && new_enabled {
                    self.frame_seq = 0;
                }

                self.enabled = new_enabled;
            }
            0xFF30..=0xFF3F => self.channel3.write_byte(addr, val, self.frame_seq),
            _ => (),
        }
    }

    pub fn cycle(&mut self, cpu_ticks: u64) {
        if !self.enabled {
            return;
        }

        self.cycle_all_channels(cpu_ticks);

        self.frame_seq_clock += cpu_ticks;
        if self.frame_seq_clock >= CPU_CYCLES_PER_FRAME_SEQ {
            self.frame_seq_clock -= CPU_CYCLES_PER_FRAME_SEQ;
            self.cycle_frame_seq();
        }

        self.audio_buffer_clock += cpu_ticks;
        if self.audio_buffer_clock >= AUDIO_SAMPLE_FREQ {
            self.audio_buffer_clock -= AUDIO_SAMPLE_FREQ;
            self.enqueue_sample();
        }

        if self.buf_filled == self.left_buf.len() {
            self.play();
        }
    }

    fn play(&mut self) {
        assert_eq!(self.buf_filled, self.left_buf.len());

        self.player.play((self.left_buf, self.right_buf));

        self.left_buf.fill(0.0);
        self.right_buf.fill(0.0);
        self.buf_filled = 0;
    }

    fn cycle_frame_seq(&mut self) {
        if self.frame_seq % 2 == 0 {
            self.channel1.step_length();
            self.channel2.step_length();
            self.channel3.step_length();
            self.channel4.step_length();
        }

        if self.frame_seq % 4 == 2 {
            self.channel1.step_sweep();
        }

        if self.frame_seq == 7 {
            self.channel1.step_envelope();
            self.channel2.step_envelope();
            self.channel4.step_envelope();
        }

        self.frame_seq = (self.frame_seq + 1) % 8;
    }

    fn cycle_all_channels(&mut self, cpu_ticks: u64) {
        self.channel1.cycle(cpu_ticks);
        self.channel2.cycle(cpu_ticks);
        self.channel3.cycle(cpu_ticks);
        self.channel4.cycle(cpu_ticks);
    }

    fn enqueue_sample(&mut self) {
        // > A value of 0 is treated as a volume of 1 (very quiet), and a value of 7 is treated as a
        // volume of 8 (no volume reduction).
        // 0.25 to split volume between 4 channels.
        // 1 / 15 because of envelope volume.
        let left_vol = self.left_volume as f32 / 7.0 * 0.25 * 1.0 / 15.0;
        let right_vol = self.right_volume as f32 / 7.0 * 0.25 * 1.0 / 15.0;

        self.left_buf[self.buf_filled] = 0.0;
        self.right_buf[self.buf_filled] = 0.0;

        if self.panning & 0b00010000 != 0 {
            self.left_buf[self.buf_filled] += left_vol * self.channel1.sample();
        }
        if self.panning & 0b00000001 != 0 {
            self.right_buf[self.buf_filled] += right_vol * self.channel1.sample();
        }

        if self.panning & 0b00100000 != 0 {
            self.left_buf[self.buf_filled] += left_vol * self.channel2.sample();
        }
        if self.panning & 0b00000010 != 0 {
            self.right_buf[self.buf_filled] += right_vol * self.channel2.sample();
        }

        if self.panning & 0b01000000 != 0 {
            self.left_buf[self.buf_filled] += left_vol * self.channel3.sample();
        }
        if self.panning & 0b00000100 != 0 {
            self.right_buf[self.buf_filled] += right_vol * self.channel3.sample();
        }

        if self.panning & 0b10000000 != 0 {
            self.left_buf[self.buf_filled] += left_vol * self.channel4.sample();
        }
        if self.panning & 0b00001000 != 0 {
            self.right_buf[self.buf_filled] += right_vol * self.channel4.sample();
        }

        self.buf_filled += 1;
    }
}

impl SquareChannel {
    const WAVEFORMS_TABLE: [[u8; 8]; 4] = [
        [0, 0, 0, 0, 0, 0, 0, 1],
        [1, 0, 0, 0, 0, 0, 0, 1],
        [1, 0, 0, 0, 0, 1, 1, 1],
        [0, 1, 1, 1, 1, 1, 1, 0],
    ];

    pub fn new(sweep_enabled: bool) -> Self {
        Self {
            enabled: false,

            sweep: if sweep_enabled {
                Some(Sweep::new())
            } else {
                None
            },
            length: LengthTimer::new(64),
            envelope: Envelope::new(),

            duty_idx: 0,
            duty_iter: 0,

            period: Period::new(4),

            dac: false,
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0xFF10 => self
                .sweep
                .as_ref()
                .map(|s| s.read_byte(addr))
                .unwrap_or(0xFF),
            0xFF11 | 0xFF16 => ((self.duty_idx & 3) << 6) | 0b111111,
            0xFF12 | 0xFF17 => self.envelope.read_byte(addr),
            0xFF13 | 0xFF18 => 0xFF,
            0xFF14 | 0xFF19 => 0b10111111 | ((self.length.enabled as u8) << 6),
            _ => panic!("Trying to read 0x{addr:X} - not Square Channel register."),
        }
    }

    pub fn write_byte(&mut self, addr: u16, val: u8, frame_seq: u8) {
        match addr {
            0xFF10 => {
                if let Some(s) = self.sweep.as_mut() {
                    s.write_byte(addr, val);
                    self.enabled &= !s.disable_channel;
                }
            }
            0xFF11 | 0xFF16 => {
                self.duty_idx = val >> 6;
                self.length.set_current(val & 0b111111);
            }
            0xFF12 | 0xFF17 => {
                self.envelope.write_byte(addr, val);
                // > Setting bits 3-7 of this register all to 0 (initial volume = 0, envelope =
                // decreasing) turns the DAC off (and thus, the channel as well).
                self.dac = val & 0b11111000 != 0;
                self.enabled &= self.dac;
            }
            0xFF13 | 0xFF18 => self.period.set_low(val),
            0xFF14 | 0xFF19 => {
                self.period.set_high(val & 0b111);

                self.length.set_enabled(bit!(val, 6), frame_seq);
                self.enabled &= !self.length.is_expired();

                if bit!(val, 7) {
                    self.trigger(frame_seq);
                }
            }
            _ => panic!("Trying to read 0x{addr:X} - not Square Channel register."),
        }
    }

    pub fn sample(&self) -> f32 {
        if self.enabled {
            Self::WAVEFORMS_TABLE[self.duty_idx as usize][self.duty_iter] as f32
                * self.envelope.volume as f32
        } else {
            0.0
        }
    }

    pub fn cycle(&mut self, cycles: u64) {
        if !self.enabled {
            return;
        }

        self.period.step(cycles, || {
            self.duty_iter = (self.duty_iter + 1) % 8;
        });
    }

    pub fn step_envelope(&mut self) {
        if self.enabled {
            self.envelope.step();
        }
    }

    pub fn step_length(&mut self) {
        self.length.step();
        self.enabled &= !self.length.is_expired();
    }

    pub fn step_sweep(&mut self) {
        if self.enabled {
            if let Some(s) = &mut self.sweep {
                s.step(&mut self.period);
                self.enabled &= !s.disable_channel;
            }
        }
    }

    fn trigger(&mut self, frame_seq: u8) {
        // If the channel’s DAC is off, the channel will not turn on.
        if self.dac {
            self.enabled = true;
        }

        self.period.trigger();
        self.length.trigger(frame_seq);
        self.envelope.trigger();

        if let Some(s) = &mut self.sweep {
            s.trigger(&self.period);
            self.enabled &= !s.disable_channel;
        }
    }
}

impl WaveChannel {
    // idk why it's 5 or 6 (it doesn't work smaller delay).
    // https://github.com/LIJI32/SameSuite/blob/master/apu/channel_3/channel_3_delay.asm
    const WAVE_CHANNEL_TRIGGER_DELAY: u16 = 5;

    pub fn new() -> Self {
        Self {
            enabled: false,
            dac: false,
            period: Period::new(2),
            length: LengthTimer::new(256),

            // > When CH3 is started, the first sample read is the one at index 1, i.e. the lower
            // nibble of the first byte, NOT the upper nibble.
            wave_idx: 1,
            waves: [0; 16],
            output_lvl: 0,
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0xFF1A => 0b1111111 | ((self.dac as u8) << 7),
            0xFF1B => 0xFF,
            0xFF1C => 0b10011111 | (self.output_lvl << 5),
            0xFF1D => 0xFF,
            0xFF1E => 0b10111111 | ((self.length.enabled as u8) << 6),
            0xFF30..=0xFF3F => {
                if !self.enabled {
                    self.waves[addr as usize - 0xFF30]
                } else if self.period.reloaded {
                    self.waves[self.wave_idx as usize >> 1]
                } else {
                    0xFF
                }
            }
            _ => panic!("0x{addr:X} is not Wave Channel register."),
        }
    }

    pub fn write_byte(&mut self, addr: u16, val: u8, frame_seq: u8) {
        match addr {
            0xFF1A => {
                self.dac = bit!(val, 7);
                self.enabled &= self.dac;
            }
            0xFF1B => self.length.set_current(val),
            0xFF1C => self.output_lvl = (val >> 5) & 0b11,
            0xFF1D => self.period.set_low(val),
            0xFF1E => {
                self.period.set_high(val & 0b111);

                self.length.set_enabled(bit!(val, 6), frame_seq);
                self.enabled &= !self.length.is_expired();

                if bit!(val, 7) {
                    self.trigger(frame_seq);
                }
            }
            0xFF30..=0xFF3F => {
                if !self.enabled {
                    self.waves[addr as usize - 0xFF30] = val;
                } else if self.period.reloaded {
                    self.waves[self.wave_idx as usize >> 1] = val;
                }
            }
            _ => panic!("0x{addr:X} is not Wave Channel register."),
        }
    }

    pub fn cycle(&mut self, cycles: u64) {
        if !self.enabled {
            return;
        }

        self.period.step(cycles, || {
            self.wave_idx = (self.wave_idx + 1) % (self.waves.len() as u8 * 2);
        });
    }

    pub fn step_length(&mut self) {
        self.length.step();
        self.enabled &= !self.length.is_expired();
    }

    pub fn sample(&self) -> f32 {
        if self.enabled {
            let (idx, hi_lo) = (self.wave_idx / 2, self.wave_idx % 2);
            let sample = if hi_lo == 0 {
                self.waves[idx as usize] >> 4
            } else {
                self.waves[idx as usize] & 0xF
            } as f32;

            match self.output_lvl {
                0 => 0.0,
                1 => sample,
                2 => sample / 2.0,
                3 => sample / 4.0,
                _ => unreachable!("output level is 2 bits length"),
            }
        } else {
            0.0
        }
    }

    fn trigger(&mut self, freq_seq: u8) {
        if self.enabled && self.period.timer == 1 {
            self.corrupt_wave_ram();
        }

        self.wave_idx = 0;

        if self.dac {
            self.enabled = true;
        }

        self.period.trigger();
        self.period.timer += Self::WAVE_CHANNEL_TRIGGER_DELAY;
        self.length.trigger(freq_seq);
    }

    fn corrupt_wave_ram(&mut self) {
        let idx = (((self.wave_idx + 1) >> 1) & 0xF) as usize;

        if idx < 4 {
            self.waves[0] = self.waves[idx];
        } else {
            // > The first FOUR bytes of wave RAM will be rewritten with the four aligned bytes that
            // the read was from (bytes 4-7, 8-11, or 12-15)
            let idx = (idx / 4) * 4;
            self.waves[0] = self.waves[idx];
            self.waves[1] = self.waves[idx + 1];
            self.waves[2] = self.waves[idx + 2];
            self.waves[3] = self.waves[idx + 3];
        }
    }
}

impl NoiseChannel {
    pub fn new() -> Self {
        Self {
            enabled: false,
            dac: false,
            length: LengthTimer::new(64),
            envelope: Envelope::new(),

            ff22: 0,
            lfsr: 0,

            period: 0,
            cycles: 0,
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0xFF20 => 0xFF,
            0xFF21 => self.envelope.read_byte(addr),
            0xFF22 => self.ff22,
            0xFF23 => 0b10111111 | ((self.length.enabled as u8) << 6),
            _ => panic!("0x{addr:X} is not Noise Channel register."),
        }
    }

    pub fn write_byte(&mut self, addr: u16, val: u8, frame_seq: u8) {
        match addr {
            0xFF20 => self.length.set_current(val & 0b111111),
            0xFF21 => {
                self.envelope.write_byte(addr, val);
                self.dac = val & 0b11111000 != 0;
                self.enabled &= self.dac;
            }
            0xFF22 => self.ff22 = val,
            0xFF23 => {
                self.length.set_enabled(bit!(val, 6), frame_seq);
                self.enabled &= !self.length.is_expired();

                if bit!(val, 7) {
                    self.trigger(frame_seq);
                }
            }
            _ => panic!("0x{addr:X} is not Noise Channel register."),
        }
    }

    pub fn step_envelope(&mut self) {
        if self.enabled {
            self.envelope.step();
        }
    }

    pub fn step_length(&mut self) {
        self.length.step();
        self.enabled &= !self.length.is_expired();
    }

    pub fn cycle(&mut self, cycles: u64) {
        self.cycles += cycles;
        if self.cycles >= self.period {
            self.cycles -= self.period;
            self.period = self.calculate_period();
            self.lfsr = self.calculate_lfsr();
        }
    }

    pub fn sample(&self) -> f32 {
        if self.enabled {
            (if bit!(self.lfsr, 0) { 1.0 } else { 0.0 }) * self.envelope.volume as f32
        } else {
            0.0
        }
    }

    fn calculate_period(&self) -> u64 {
        let shift = self.ff22 >> 4;
        let divider_code = self.ff22 & 0b111;
        let divider = if divider_code == 0 {
            8
        } else {
            16 * divider_code
        };
        (divider as u64) << (shift as u64)
    }

    fn calculate_lfsr(&self) -> u16 {
        let xor = !(bit!(self.lfsr, 0) as u16 ^ bit!(self.lfsr, 1) as u16) & 0b1;

        let next = self.lfsr | (xor << 15) | if bit!(self.ff22, 3) { xor << 7 } else { 0 };

        next >> 1
    }

    fn trigger(&mut self, frame_seq: u8) {
        if self.dac {
            self.enabled = true;
        }

        self.length.trigger(frame_seq);
        self.envelope.trigger();
        self.lfsr = 0;
    }
}

impl Sweep {
    const PERIOD_ZERO: u8 = 8;

    pub fn new() -> Self {
        Self {
            enabled: false,
            period: 0,
            negate: false,
            negate_done: false,
            shift: 0,
            timer: 0,
            shadow_freq: 0,
            disable_channel: false,
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0xFF10 => (1 << 7) | (self.period << 4) | ((self.negate as u8) << 3) | (self.shift & 7),
            _ => unreachable!(),
        }
    }

    pub fn write_byte(&mut self, addr: u16, val: u8) {
        match addr {
            0xFF10 => {
                let old_negate = self.negate;

                self.period = (val >> 4) & 7;
                self.negate = bit!(val, 3);
                self.shift = val & 7;

                // Obscure Behavior:
                // Clearing the sweep negate mode bit in NR10 after at least one sweep calculation
                // has been made using the negate mode since the last trigger causes the channel to
                // be immediately disabled.
                if old_negate && !self.negate && self.negate_done {
                    self.disable_channel = true;
                }
                self.negate_done = false;
            }
            _ => unreachable!(),
        }
    }

    pub fn trigger(&mut self, period: &Period) {
        self.shadow_freq = period.period;

        self.reload_timer();

        self.enabled = self.period != 0 || self.shift != 0;
        self.disable_channel = false;

        if self.shift != 0 {
            self.calculate_freq();
        }
    }

    pub fn step(&mut self, period: &mut Period) {
        if self.timer > 0 {
            self.timer -= 1;
        }

        if self.timer == 0 {
            self.reload_timer();

            if self.period > 0 && self.enabled {
                let new_freq = self.calculate_freq();

                if new_freq <= 2047 && self.shift != 0 {
                    self.shadow_freq = new_freq;

                    period.set(new_freq);

                    self.calculate_freq();
                }
            }
        }
    }

    fn reload_timer(&mut self) {
        if self.period > 0 {
            self.timer = self.period;
        } else {
            self.timer = Self::PERIOD_ZERO;
        }
    }

    #[allow(clippy::assign_op_pattern)]
    fn calculate_freq(&mut self) -> u16 {
        let mut new_freq = self.shadow_freq >> self.shift;

        if self.negate {
            new_freq = self.shadow_freq - new_freq;
            self.negate_done = true;
        } else {
            new_freq = self.shadow_freq + new_freq;
        }

        if new_freq > 2047 {
            self.disable_channel = true;
        }

        new_freq
    }
}

impl Envelope {
    pub fn new() -> Self {
        Self {
            timer: 0,
            volume: 0,
            init_volume: 0,
            dir_up: false,
            init_timer: 0,
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0xFF12 | 0xFF17 | 0xFF21 => {
                ((self.init_volume & 0b1111) << 4)
                    | ((self.dir_up as u8) << 3)
                    | (self.init_timer & 0b111)
            }
            _ => unreachable!(),
        }
    }

    pub fn write_byte(&mut self, addr: u16, val: u8) {
        match addr {
            0xFF12 | 0xFF17 | 0xFF21 => {
                self.init_volume = val >> 4;
                self.volume = self.init_volume;
                self.dir_up = bit!(val, 3);
                self.init_timer = val & 7;
            }
            _ => unreachable!(),
        }
    }

    pub fn trigger(&mut self) {
        self.volume = self.init_volume;
        self.timer = self.init_timer;
    }

    pub fn step(&mut self) {
        if self.init_timer == 0 {
            return;
        }

        self.timer = self.timer.saturating_sub(1);

        if self.timer == 0 {
            self.timer = self.init_timer;

            if self.volume < 0xF && self.dir_up {
                self.volume += 1;
            }
            if self.volume > 0x0 && !self.dir_up {
                self.volume -= 1;
            }
        }
    }
}

fn first_half(frame_seq: u8) -> bool {
    frame_seq % 2 == 1
}

impl LengthTimer {
    pub fn new(len: u16) -> Self {
        Self {
            enabled: false,
            max_len: len,
            timer: 0,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.timer == 0
    }

    pub fn set_current(&mut self, initial_len: u8) {
        self.timer = self.max_len - initial_len as u16;
    }

    pub fn set_enabled(&mut self, new_enabled: bool, frame_seq: u8) {
        let old = self.enabled;
        self.enabled = new_enabled;

        // See https://gbdev.io/pandocs/Audio_details.html#obscure-behavior
        if !old && first_half(frame_seq) {
            self.step();
        }
    }

    pub fn trigger(&mut self, frame_seq: u8) {
        if self.timer == 0 {
            self.timer = self.max_len;
            // See https://gbdev.io/pandocs/Audio_details.html#obscure-behavior
            // > If a channel is triggered when the DIV-APU next step is one that doesn’t clock the
            // length timer and the length timer is now enabled and length is being set to 64 (256
            // for wave channel) because it was previously zero, it is set to 63 instead (255 for
            // wave channel).
            if first_half(frame_seq) {
                self.step();
            }
        }
    }

    fn step(&mut self) {
        if self.enabled {
            self.timer = self.timer.saturating_sub(1);
        }
    }
}

impl Period {
    pub fn new(multiplier: u16) -> Self {
        Self {
            period: 0,
            timer: 0,
            multiplier,
            reloaded: false,
        }
    }

    pub fn set_high(&mut self, val: u8) {
        self.period = (self.period & 0xFF) | (((val as u16) & 0b111) << 8);
    }

    pub fn set_low(&mut self, val: u8) {
        self.period = (self.period & 0xFF00) | (val as u16);
    }

    pub fn set(&mut self, val: u16) {
        self.period = val & 0x7FF;
    }

    pub fn step(&mut self, mut cpu_cycles: u64, mut timer_reload_handler: impl FnMut()) {
        self.reloaded = false;
        while cpu_cycles > 0 {
            cpu_cycles -= 1;
            self.timer = self.timer.saturating_sub(1);

            if self.timer == 0 {
                self.timer = self.calculate_timer();
                self.reloaded = true;
                timer_reload_handler();
            }
        }

        if self.timer < self.calculate_timer() - 2 {
            self.reloaded = false;
        }
    }

    pub fn trigger(&mut self) {
        self.timer = self.calculate_timer();
    }

    fn calculate_timer(&self) -> u16 {
        (2048 - self.period) * self.multiplier
    }
}
