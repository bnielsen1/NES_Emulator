

# R-NES 

<div style="display: flex; align-items: flex-start;">
  <div style="flex: 1; padding-right: 20px;">
    <ul style="padding-left: 20px;">
      <li style="margin-bottom: 12px;">
        <b>NROM</b> and <b>MMC1</b> mapper support with future plans to support more mapper types.
      </li>
      <li style="margin-bottom: 12px;">
        100% accurate implementation of <b>all 151</b> official 6502 microprocessor instructions.
      </li>
      <li style="margin-bottom: 12px;">
        Support for both <i>vertical</i> and <i>horizontal</i> scrolling modes.
      </li>
      <li style="margin-bottom: 12px;">
        Tested to play a variety of games including but not limited to: "Super Mario Bros", "Donkey Kong", and "Tetris". See <a href="#support"><b>Support</b></a> for details on supported games.
      </li>
    </ul>
  </div>
  <div>
    <img src="https://upload.wikimedia.org/wikipedia/commons/thumb/b/b2/NES-Console-Set.png/1280px-NES-Console-Set.png" alt="NES console" width="200"/>
  </div>
</div>




## Controls
Pressing **escape** will close the emulator at any time!

Controller 1 (Keyboard):
<img src="https://upload.wikimedia.org/wikipedia/commons/3/30/Nes_controller.svg" alt="Nes controller" width="100"/> D-Pad -> "Arrow keys", Start -> "Return", Select -> "Space", A -> "A key", B -> "S key"

Controller 2 (Disabled by default):
<img src="https://upload.wikimedia.org/wikipedia/commons/3/30/Nes_controller.svg" alt="Nes controller" width="100"/> *(Future support planned)*

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

*Example debug log*:
```
AA37  A5 33     LDA $33 = 00                    A:00 X:00 Y:00 P:27 SP:F9 | PPU: L: 223 CYC: 311
MORE PPU DATA: VBLANK: false CTRL: 10010000, STATUS: 00000000
```



⚠️ **A note about debug mode!** ⚠️

Due to the large amount of console output, most computers will see a large drop in emulator performance when the mode is enabled. It is highly suggested to just use the mode for testing/development purposes only.
## Support

The following games have been tested on the emulator with different degrees of success. Many other games likely work but haven't been tested. Games with **mapping modes other than 0 and 1 will not run!**

<table cellspacing="0" cellpadding="0" dir="ltr" border="1" style="" data-sheets-root="1" data-sheets-baot="1">
  <thead>
    <tr style="height:21px;">
      <th>Game</th>
      <th>Mapping Mode</th>
      <th>Compatibility</th>
      <th>Notes?</th>
    </tr>
  </thead><colgroup><col width="169"><col width="100"><col width="100"><col width="437"></colgroup>
  <tbody>
    <tr style="height:21px;">
      <td style="overflow:hidden;padding:2px 3px 2px 3px;vertical-align:bottom;text-align:center;">Pacman</td>
      <td style="overflow:hidden;padding:2px 3px 2px 3px;vertical-align:bottom;text-align:center;">0</td>
      <td style="overflow:hidden;padding:2px 3px 2px 3px;vertical-align:bottom;background-color:#b7e1cd;text-align:center;">Y</td>
      <td style="overflow:hidden;padding:2px 3px 2px 3px;vertical-align:bottom;"></td>
    </tr>
    <tr style="height:21px;">
      <td style="overflow:hidden;padding:2px 3px 2px 3px;vertical-align:bottom;text-align:center;">Zelda 1</td>
      <td style="overflow:hidden;padding:2px 3px 2px 3px;vertical-align:bottom;text-align:center;">1</td>
      <td style="overflow:hidden;padding:2px 3px 2px 3px;vertical-align:bottom;background-color:#fce8b2;text-align:center;">P</td>
      <td style="overflow:hidden;padding:2px 3px 2px 3px;vertical-align:bottom;">Upward and downward function correctly but display wrong</td>
    </tr>
    <tr style="height:21px;">
      <td style="overflow:hidden;padding:2px 3px 2px 3px;vertical-align:bottom;text-align:center;">Super Mario Bros</td>
      <td style="overflow:hidden;padding:2px 3px 2px 3px;vertical-align:bottom;text-align:center;">0</td>
      <td style="overflow:hidden;padding:2px 3px 2px 3px;vertical-align:bottom;background-color:#b7e1cd;text-align:center;">Y</td>
      <td style="overflow:hidden;padding:2px 3px 2px 3px;vertical-align:bottom;"></td>
    </tr>
    <tr style="height:21px;">
      <td style="overflow:hidden;padding:2px 3px 2px 3px;vertical-align:bottom;text-align:center;">Megaman 2</td>
      <td style="overflow:hidden;padding:2px 3px 2px 3px;vertical-align:bottom;text-align:center;">1</td>
      <td style="overflow:hidden;padding:2px 3px 2px 3px;vertical-align:bottom;background-color:#b7e1cd;text-align:center;">Y</td>
      <td style="overflow:hidden;padding:2px 3px 2px 3px;vertical-align:bottom;"></td>
    </tr>
    <tr style="height:21px;">
      <td style="overflow:hidden;padding:2px 3px 2px 3px;vertical-align:bottom;text-align:center;">Pinball</td>
      <td style="overflow:hidden;padding:2px 3px 2px 3px;vertical-align:bottom;text-align:center;">0</td>
      <td style="overflow:hidden;padding:2px 3px 2px 3px;vertical-align:bottom;background-color:#b7e1cd;text-align:center;">Y</td>
      <td style="overflow:hidden;padding:2px 3px 2px 3px;vertical-align:bottom;"></td>
    </tr>
    <tr style="height:21px;">
      <td style="overflow:hidden;padding:2px 3px 2px 3px;vertical-align:bottom;text-align:center;">NES Tetris</td>
      <td style="overflow:hidden;padding:2px 3px 2px 3px;vertical-align:bottom;text-align:center;">1</td>
      <td style="overflow:hidden;padding:2px 3px 2px 3px;vertical-align:bottom;background-color:#b7e1cd;text-align:center;">Y</td>
      <td style="overflow:hidden;padding:2px 3px 2px 3px;vertical-align:bottom;"></td>
    </tr>
    <tr style="height:21px;">
      <td style="overflow:hidden;padding:2px 3px 2px 3px;vertical-align:bottom;text-align:center;">Ice Climbers</td>
      <td style="overflow:hidden;padding:2px 3px 2px 3px;vertical-align:bottom;text-align:center;">0</td>
      <td style="overflow:hidden;padding:2px 3px 2px 3px;vertical-align:bottom;background-color:#b7e1cd;text-align:center;">Y</td>
      <td style="overflow:hidden;padding:2px 3px 2px 3px;vertical-align:bottom;"></td>
    </tr>
    <tr style="height:21px;">
      <td style="overflow:hidden;padding:2px 3px 2px 3px;vertical-align:bottom;text-align:center;">Galaxia</td>
      <td style="overflow:hidden;padding:2px 3px 2px 3px;vertical-align:bottom;text-align:center;">0</td>
      <td style="overflow:hidden;padding:2px 3px 2px 3px;vertical-align:bottom;background-color:#f4c7c3;text-align:center;">N</td>
      <td style="overflow:hidden;padding:2px 3px 2px 3px;vertical-align:bottom;">Does not launch (known to be hard to emulate)</td>
    </tr>
  </tbody>
</table>

A constantly updating version of this table can be found here:
https://docs.google.com/spreadsheets/d/19lmfgJi0hDuFKWADfGg4XUymzAMxmPnBF0Reb8eU9G4/edit?usp=sharing

