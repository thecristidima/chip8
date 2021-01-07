# CHIP 8 Emulator

This is a [CHIP 8](https://en.wikipedia.org/wiki/CHIP-8) emulator written in Rust. The aim of the project is to write more Rust code and to get into writing emulators (end goal: play Pokemon LeafGreen on my emulated GBA).

Implementation is based on [Cowgod's documentation](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#2.1) and other CHIP 8 emulators found online.

### **Implementation goals**

*DISCLAIMER: I will probably not implement all of these, but I can still hope, right?*

- [x] ~~Unit test operations (shouldn't be too hard, just tedious)~~ Test ROMs + unit tests for PC incrementation
- [x] Add sound support
- [ ] Find a way to save state (and resume from it later on)
- [ ] Customisable user settings (e.g. custom colour schemes)

### **Issues I encountered during development**

*This section is meant to document issues I ran into while writing the emulator. I doubt anyone else will read this, but I'm sure it'll help me when I make the same mistake later on when working on other projects.*

- Debugging is a mess, need to work out some way of dumping out data in a readable format
- Could not get screen to render correctly with SDL2 until I checked user input; Didn't bother to read the documentation (of course), but I still think this should've worked.
- I kept incrementing the PC even after a jump; test ROMs didn't help with this, I had to dump registers and compare them with other emulators. Luckily, one of the ROMs had a jump instruction at the beginning.