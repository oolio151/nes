# oolio151-nes
An emulator of the Nintendo Entertainment System, built in Rust. Currently in Development.

## How to Play

At the ROM path prompt, enter a file directly or a directory to browse its `.nes` files. Use Up/Down arrows and Enter to select a ROM, or Esc to return to the path prompt. The list is alphabetical and includes `.NES` files; subdirectories are not searched.

Emulator is still pretty new, so loading games is done via terminal, simply enter the file path relative to the executable.<br>

**CONTROLS**
- Z - A
- X - B
- A - Select
- S - Start
- T - Save State
- Y - Load State

## Development
Currently the emulator is functional with [NROM/mapper 0 games](https://nescartdb.com/search/advanced?ines=0&rows=400&rfa=1+2+11+3+9+20+41+53+10). More mappers will come soon to expand the game library. This emulator is meant to emulate an NTSC-region NES.



## Other info
cpu tests are from [SingleStepTests/65x02](https://github.com/SingleStepTests/65x02)<br>
passed 75/141 tests from [100thCoin/AccuracyCoin](https://github.com/100thCoin/AccuracyCoin) (better than many official Nintendo emulators btw!)