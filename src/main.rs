use std::sync::mpsc::{self, Receiver, SyncSender};

use gbemu::{args::parse_args, cpu::JoypadKey, cpu::CPU, SCREEN_HEIGHT, SCREEN_WIDTH};
use minifb::{Key, Window};

type GuiFrame = [u32; SCREEN_HEIGHT * SCREEN_WIDTH];

#[derive(Copy, Clone, Debug)]
enum GuiEvent {
    KeyUp(JoypadKey),
    KeyDown(JoypadKey),
    // Debug keys:
    ToggleCpuPause,
}

pub fn minifb_key_to_joypad(key: minifb::Key) -> Option<JoypadKey> {
    match key {
        Key::Up => Some(JoypadKey::Up),
        Key::Down => Some(JoypadKey::Down),
        Key::Left => Some(JoypadKey::Left),
        Key::Right => Some(JoypadKey::Right),
        Key::Enter => Some(JoypadKey::Start),
        Key::Space => Some(JoypadKey::Select),
        Key::Z => Some(JoypadKey::A),
        Key::X => Some(JoypadKey::B),
        _ => None,
    }
}

fn main() {
    let args = parse_args().unwrap();

    let content = gbemu::read_rom(&args.rom_path).unwrap();

    let cpu = CPU::new(content);
    let mut window = Window::new(
        "DMG-01",
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        minifb::WindowOptions {
            scale: minifb::Scale::X4,
            ..Default::default()
        },
    )
    .unwrap();

    let key_events = mpsc::channel();
    // sync_channel because we want the previous frame to be drawn before the next frame is
    // transmitted.
    let gui_frame = mpsc::sync_channel(1);

    // At the moment I don't understand why the default stack size of 2MB is not enough: buffer
    // array ~200KB.
    let cpu_run = std::thread::Builder::new()
        .stack_size(1024 * 1024 * 10)
        .spawn(|| run(cpu, gui_frame.0, key_events.1))
        .unwrap();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if window.is_key_pressed(Key::P, minifb::KeyRepeat::No) {
            let _ = key_events.0.send(GuiEvent::ToggleCpuPause);
        }

        for key in window.get_keys_pressed(minifb::KeyRepeat::No) {
            if let Some(ev) = minifb_key_to_joypad(key) {
                // No unwrap because the CPU may already be stopped (channels are closed).
                let _ = key_events.0.send(GuiEvent::KeyDown(ev));
            }
        }
        for key in window.get_keys_released() {
            if let Some(ev) = minifb_key_to_joypad(key) {
                let _ = key_events.0.send(GuiEvent::KeyUp(ev));
            }
        }

        if let Ok(new_frame) = gui_frame.1.recv() {
            window
                .update_with_buffer(&new_frame, SCREEN_WIDTH, SCREEN_HEIGHT)
                .unwrap();
        } else {
            break;
        }
    }

    // Drop, so the CPU will stop because no one is sending/listening for updates.
    drop(gui_frame.1);
    drop(key_events.0);

    cpu_run.join().unwrap();
}

fn run(mut cpu: CPU, gui_frame: SyncSender<GuiFrame>, key_events: Receiver<GuiEvent>) {
    // Inspired by https://github.com/mvdnes/rboy/blob/1e46c6d5fc61140e8e1919dea9f799d9d4e41345/src/main.rs#L317
    let limiter = spawn_limiter(gbemu::MILLIS_PER_FRAME);

    let mut gui_buf = [0u32; SCREEN_HEIGHT * SCREEN_WIDTH];

    let mut ticks = 0;
    let mut cpu_pause = false;

    'main: loop {
        if !cpu_pause {
            while ticks < gbemu::TICKS_PER_FRAME {
                ticks += cpu.cycle();
            }
            ticks -= gbemu::TICKS_PER_FRAME;
        }

        cpu.gpu().to_rgb32(&mut gui_buf);

        if gui_frame.send(gui_buf).is_err() {
            break;
        }

        loop {
            match key_events.try_recv() {
                Ok(ev) => match ev {
                    GuiEvent::KeyUp(joypad_key) => cpu.key_up(joypad_key),
                    GuiEvent::KeyDown(joypad_key) => cpu.key_down(joypad_key),
                    GuiEvent::ToggleCpuPause => cpu_pause = !cpu_pause,
                },
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => break 'main,
            }
        }

        limiter.recv().unwrap();
    }
}

fn spawn_limiter(ms: u64) -> Receiver<()> {
    let (snd, rcv) = mpsc::sync_channel(1);
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_millis(ms));
        snd.send(()).unwrap();
    });
    rcv
}
