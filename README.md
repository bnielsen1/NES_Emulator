
# R-NES

R-NES is a fully functioning emulator built entirely in Rust for the Nintendo Entertainment System, a gaming console from 1983. 

- **NROM** and **MMC1** mapper support with future plans to support more mapper types.

- 100% accurate implementation of **all 151** official 6502 microprocessor instructions.

- Support for both *vertical* and *horizontal* scrolling modes.

- Tested to play a variety of games including but not limited to: "Super Mario Bros", "Donkey Kong", and "Tetris". See **FILL HERE** for details on supported games.
## Usage

### Pre-requisites

- Rust
R-NES is built in Rust and requires the software to compile and run. If you don't already have Rust installed, it can be found here [**here**](https://www.rust-lang.org/tools/install) for your operating system.

- SDL2.0
R-NES utilizes the [Simple DirectMedia Layer Library](https://www.libsdl.org/) to render graphical information to the screen. Instructions to install these binaries on any platform can be found [**here**](https://github.com/Rust-SDL2/rust-sdl2?tab=readme-ov-file#sdl20-development-libraries). All other dependencies will automatically be handled by the rust package manager!


### Launching the emulator

After cloning the repository and entering the directory, launch the emulator with the following command:

`cargo run -- -rom FULL_PATH_TO_ROM`

Failing to provide a rom file will not allow the emulator to start

### Additional parameters

For development purposes, the emulator also comes with toggleable debug mode. Enabling debug mode is as easy as launching R-NES with the flag `-debug`. An example can be found below:

`cargo run -- -rom /home/user/Desktop/smb.nes -debug`

When enabled, every completed instruction gets logged to the console alongside the state of the console before that instruction was run. 

<details>
<summary><b>
Click to view example log


</b></summary>
<p>
```text
AA37  A5 33     LDA $33 = 00                    A:00 X:00 Y:00 P:27 SP:F9 | PPU: L: 223 CYC: 311
MORE PPU DATA: VBLANK: false CTRL: 10010000, STATUS: 00000000
text```
</p>
</details>


⚠️ **A note about debug mode!** ⚠️

Due to the large amount of console output, most computers will see a large drop in emulator performance when the mode is enabled. It is highly suggested to just use the mode for testing/development purposes only.
## Documentation

TODO

