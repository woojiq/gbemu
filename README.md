# Game Boy Emulator

## TODO (most important)

* Save game state.
* MBC2, MBC3, MBC5 support.
* Support serial console.
* Write one ROM test myself.
* Builtin debugger (in time, lol, after spending hours with eprintln).

## List of passed tests

See [tests/test_roms.rs](./tests/test_roms.rs)

## Play

To quickly test the emulator, you can use Tetris cartridge included in this repo:
```sh
cargo run -- roms/Tetris.gb
```

## License

The software is licensed under the MIT License.

Note: The MIT license does not apply to the prebuilt ROMs in this repository. They are covered by their own licenses.

## Resources

Pandocs: https://gbdev.io/pandocs/About.html

Gameboy Development Wiki: https://gbdev.gg8.se/wiki/articles/Main_Page

A collection of different test suites for your Game Boy emulator: https://github.com/c-sp/game-boy-test-roms/tree/8e1f6d7f3a1d8683f11fdf23008d1b1b26e51b52

Opcodes: https://izik1.github.io/gbops/

Opcodes textual explanation: https://rgbds.gbdev.io/docs/v0.9.0/gbz80.7

Gameboy Emulation: http://www.codeslinger.co.uk/pages/projects/gameboy.html

Game Boy: Complete Technical Reference: https://github.com/Gekkio/gb-ctr

Reference implementations:
* https://github.com/smparsons/retroboy
* https://github.com/mvdnes/rboy
