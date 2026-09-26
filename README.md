# mb_adv Firmware

Firmware for the mb_adv which distributes GBA multiboot ROMs.

This project is provided as-is, without any warranty.

## Running

1. Copy your multiboot rom to `mb.gba` in the repo directory
2. Hold the pico reset button
3. Connect it to the computer
4. Run `cargo run --release`
5. Eject and reset the pico
6. Plug the pico into the GBA
7. Turn on the GBA

## Credits

Original project by [Zaksabeast](https://github.com/zaksabeast/Portable-Pico-Multibooter).

Created from the [rp2040 project template](https://github.com/rp-rs/rp2040-project-template).

References:
- https://github.com/tangrs/usb-gba-multiboot
- https://problemkaputt.de/gbatek.htm
