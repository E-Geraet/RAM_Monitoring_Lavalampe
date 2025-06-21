# RAM Lava Lamp

A tiny desktop lava lamp that visualizes your RAM usage. Why? Why not!

## What does it do?

The program shows a small window with an animated lava lamp. The color and speed of the animation changes based on your RAM usage:

- **Green** (≤30% RAM): Everything chill, slow bubbles
- **Yellow** (31-50% RAM): Getting a bit more lively  
- **Orange** (51-80% RAM): Now we're cooking
- **Red** (>80% RAM): PANIC MODE! Bubbles going crazy

## Installation

You need Rust. If you don't have it yet:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Then clone and build:
```bash
git clone <your-repo-url>
cd ram-lavalampe
cargo build --release
```

## Assets

The sprite files need to go in the `assets/` folder:
- `lavalampe_green.png` 
- `lavalampe_yellow.png`
- `lavalampe_orange.png` 
- `lavalampe_red.png`

Each file should be 128 pixels tall and the width should be a multiple of 128 (for the individual frames). Default expects 90 frames per animation, but the program adapts automatically.

## Running

```bash
cargo run --release
```

Or run the compiled version directly:
```bash
./target/release/ram-lavalampe
```

The window is borderless and 128x128 pixels - perfect as a small desktop widget.

## Controls  

- **ESC**: Quit the program
- That's it. No more controls needed, it's a lava lamp after all

## Technical stuff

- Runs at ~60 FPS (with smart frame limiting)
- Uses `minifb` for windowing
- RAM monitoring via `sysinfo`
- Sprites are loaded as horizontal sprite sheets
- Auto-fallback to green if other colors fail to load

## Known issues

- Sometimes the window hangs when switching window managers (Linux)
- Minor flickering on some systems
- Asset paths are hardcoded (should make this configurable)

## TODO

- [ ] Config file for paths and settings
- [ ] More color options
- [ ] Maybe CPU monitoring as an alternative?
- [ ] Better error handling (currently crashes silently sometimes)
- [ ] Save window position

## Dependencies

Check `Cargo.toml`, but the main ones:
- `minifb` - for windowing
- `image` - for loading sprites  
- `sysinfo` - for system monitoring
- `lazy_static` - for the print cache

## License

Do whatever you want with it.

---

*If you like this, feel free to drop a star. If not, well....*
