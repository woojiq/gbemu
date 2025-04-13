# Game Boy Emulator

## TODO (most important)

* Support different Memory Bank Controllers.
* Automate ROM tests.
* Builtin debugger (in time, lol, after spending hours for eprintln).
* Write one ROM test myself.
* Add a LICENSE (and note that the license only applies to my code, not game/test ROMs).

## List of passed tests

* blargg:
  * cpu_instrs
  * instr_timing
* turtle-tests:
  * window_y_trigger
  * window_y_trigger_wx_offscreen
* dmg-acid2
* mooneye-test-suite
  * acceptance
    * bits
    * instr
  * oam_dma
    * basic
  * manual-only
    * sprite_priority

## Resources

A curated list of awesome Game Boy resources: https://gbdev.io/resources.html

Test roms: https://github.com/c-sp/game-boy-test-roms

PyBoy: https://github.com/Baekalfen/PyBoy/blob/master/extras/PyBoy.pdf

DMG-01 (How to Emulate a Game Boy): https://rylev.github.io/DMG-01/public/book

Rust implementation (DMG-01): https://github.com/rylev/DMG-01/blob/master/lib-dmg-01/src

Opcodes: https://izik1.github.io/gbops/

Opcodes textual explanation: https://rgbds.gbdev.io/docs/v0.9.0/gbz80.7

Gameboy Emulation: http://www.codeslinger.co.uk/pages/projects/gameboy.html

Guide with pictures: https://hacktix.github.io/GBEDG/ppu

A collection of different test suites for your Game Boy emulator: https://github.com/c-sp/game-boy-test-roms/tree/8e1f6d7f3a1d8683f11fdf23008d1b1b26e51b52
