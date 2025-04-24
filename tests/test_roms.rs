use std::path::PathBuf;

// To run integration tests you need to download and unpack
// https://github.com/c-sp/game-boy-test-roms/ to this directory.

use gbemu::{
    cpu::{
        instruction::{Instruction, JumpTest, LoadByteSource, LoadByteTarget, LoadType},
        CPU,
    },
    SCREEN_HEIGHT, SCREEN_WIDTH,
};

const TEST_ROM_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/game-boy-test-roms/");

fn test_rom_screen(rom_path: PathBuf, img_expected: PathBuf, timeout: u64) {
    let img = image::open(&img_expected).unwrap().to_rgb8();
    let rom = gbemu::read_rom(&rom_path).unwrap();

    let mut cpu = CPU::new_without_sound(rom);
    let mut cycles = 0;

    while cycles < timeout {
        let prev_pc = cpu.pc();

        cycles += cpu.cycle();

        if prev_pc == cpu.pc() {
            match cpu.get_current_instruction() {
                Instruction::JR(JumpTest::Always) | Instruction::JP(JumpTest::Always) => break,
                _ => {}
            }
        }
    }

    let mut actual = [0u8; SCREEN_HEIGHT * SCREEN_WIDTH * 3];
    cpu.gpu().to_rgb8(&mut actual);

    let mut save_img = img_expected;
    assert!(save_img.set_extension("actual.png"));

    image::save_buffer(
        &save_img,
        &actual,
        SCREEN_WIDTH as u32,
        SCREEN_HEIGHT as u32,
        image::ColorType::Rgb8,
    )
    .unwrap();

    assert_eq!(actual, img.as_raw().as_slice());
}

macro_rules! test_by_screen {
    ($($test_name:ident($rom_path:expr, $img_expected:expr, $timeout:literal),)*) => {
        $(#[test]
        fn $test_name() {
            crate::test_rom_screen($rom_path, $img_expected, $timeout);
        })*
    };
}

// Writes the Fibonacci numbers 3/5/8/13/21/34 to the registers B/C/D/E/H/L.
// Executes an `LD B, B` opcode.
fn test_rom_fibonacci(rom_path: PathBuf, timeout: u64) {
    let rom = gbemu::read_rom(&rom_path).unwrap();

    let mut cpu = CPU::new_without_sound(rom);
    let mut cycles = 0;

    while cycles < timeout {
        cycles += cpu.cycle();

        if let Instruction::Load(LoadType::Byte(LoadByteTarget::B, LoadByteSource::B)) =
            cpu.get_current_instruction()
        {
            break;
        }
    }

    assert_eq!(cpu.registers().b, 3);
    assert_eq!(cpu.registers().c, 5);
    assert_eq!(cpu.registers().d, 8);
    assert_eq!(cpu.registers().e, 13);
    assert_eq!(cpu.registers().h, 21);
    assert_eq!(cpu.registers().l, 34);
}

macro_rules! test_by_fibonacci {
    ($($test_name:ident($rom_path:expr, $timeout:literal),)*) => {
        $(#[test]
        fn $test_name() {
            crate::test_rom_fibonacci($rom_path, $timeout);
        })*
    };
}

mod blargg {
    macro_rules! path {
        ($path:literal) => {
            std::path::PathBuf::from(crate::TEST_ROM_PATH.to_string() + "blargg/" + $path)
        };
    }

    test_by_screen!(
        cpu_instrs(
            path!("cpu_instrs/cpu_instrs.gb"),
            path!("cpu_instrs/cpu_instrs-dmg-cgb.png"),
            230_000_000
        ),
        instr_timing(
            path!("instr_timing/instr_timing.gb"),
            path!("instr_timing/instr_timing-dmg-cgb.png"),
            5_000_000
        ),
        dmg_sound(
            path!("dmg_sound/dmg_sound.gb"),
            path!("dmg_sound/dmg_sound-dmg.png"),
            500_000_000
        ),
    );
}

mod turtle_tests {
    macro_rules! path {
        ($path:literal) => {
            std::path::PathBuf::from(crate::TEST_ROM_PATH.to_string() + "turtle-tests/" + $path)
        };
    }

    test_by_screen!(
        window_y_trigger_wx_offscreen(
            path!("window_y_trigger_wx_offscreen/window_y_trigger_wx_offscreen.gb"),
            path!("window_y_trigger_wx_offscreen/window_y_trigger_wx_offscreen.png"),
            800_000
        ),
        instr_timing(
            path!("window_y_trigger/window_y_trigger.gb"),
            path!("window_y_trigger/window_y_trigger.png"),
            800_000
        ),
    );
}

mod dmg_acid2 {
    macro_rules! path {
        ($path:literal) => {
            std::path::PathBuf::from(crate::TEST_ROM_PATH.to_string() + "dmg-acid2/" + $path)
        };
    }

    test_by_screen!(dmg_acid2(
        path!("dmg-acid2.gb"),
        path!("dmg-acid2-dmg.png"),
        800_000
    ),);
}

mod mooneye_test_suite {
    macro_rules! path {
        ($path:literal) => {
            std::path::PathBuf::from(
                crate::TEST_ROM_PATH.to_string() + "mooneye-test-suite/" + $path,
            )
        };
    }

    mod acceptance {
        mod bits {
            test_by_fibonacci!(
                mem_oam(path!("acceptance/bits/mem_oam.gb"), 800_000),
                bits(path!("acceptance/bits/reg_f.gb"), 800_000),
                unused_hwio_gs(path!("acceptance/bits/unused_hwio-GS.gb"), 800_000),
            );
        }

        mod instr {
            test_by_fibonacci!(daa(path!("acceptance/instr/daa.gb"), 5_000_000),);
        }

        mod oam_dma {
            test_by_fibonacci!(basic(path!("acceptance/oam_dma/basic.gb"), 1_000_000),);
        }
    }

    mod manual_only {
        test_by_screen!(sprite_priority(
            path!("manual-only/sprite_priority.gb"),
            path!("manual-only/sprite_priority-dmg.png"),
            1_000_000
        ),);
    }

    mod emulator_only {
        mod mbc1 {
            test_by_fibonacci!(
                bits_bank1(path!("emulator-only/mbc1/bits_bank1.gb"), 30_000_000),
                bits_bank2(path!("emulator-only/mbc1/bits_bank2.gb"), 30_000_000),
                bits_mode(path!("emulator-only/mbc1/bits_mode.gb"), 30_000_000),
                bits_ramg(path!("emulator-only/mbc1/bits_ramg.gb"), 30_000_000),
                ram_256kb(path!("emulator-only/mbc1/ram_256kb.gb"), 5_000_000),
                ram_64kb(path!("emulator-only/mbc1/ram_64kb.gb"), 5_000_000),
                rom_16mb(path!("emulator-only/mbc1/rom_16Mb.gb"), 5_000_000),
                rom_1mb(path!("emulator-only/mbc1/rom_1Mb.gb"), 5_000_000),
                rom_2mb(path!("emulator-only/mbc1/rom_2Mb.gb"), 5_000_000),
                rom_4mb(path!("emulator-only/mbc1/rom_4Mb.gb"), 5_000_000),
                rom_512kb(path!("emulator-only/mbc1/rom_512kb.gb"), 5_000_000),
                rom_8mb(path!("emulator-only/mbc1/rom_8Mb.gb"), 5_000_000),
                // multicart_rom_8mb(path!("emulator-only/mbc1/multicart_rom_8Mb.gb"), 5_000_000),
            );
        }
    }
}
