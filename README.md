# oolio151-nes
An emulator of the Nintendo Entertainment System, built in Rust.

## How to Play

At the ROM path prompt, enter a file directly or a directory to browse its `.nes` files. Use Up/Down arrows and Enter to select a ROM, or Esc to return to the path prompt. The list is alphabetical and includes `.NES` files; subdirectories are not searched.

Emulator is still pretty new, so loading games is done via terminal, simply enter the file path relative to the executable.<br>

**CONTROLS**
These will be customizable later.
- Z - A
- X - B
- A - Select
- S - Start
- T - Save State
- Y - Load State

## Development
This emulator is meant to emulate an NTSC-region NES. Check the latest release to see what mappers are supported.

## Saves
Pretty lazy for now, but savestates (.ss0) and battery saves (.sav) are just saved to the same location as the rom file.

## Other info
cpu tests are from [SingleStepTests/65x02](https://github.com/SingleStepTests/65x02)<br>
passed 80/141 tests from [100thCoin/AccuracyCoin](https://github.com/100thCoin/AccuracyCoin) (better than many official Nintendo emulators btw!)