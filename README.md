#  RAM Lava Lamp

![Lavalampen-Demo](img/lavalamp2.gif)

A beautiful desktop widget that visualizes your system's RAM usage as an animated lava lamp. The color and animation speed change based on memory consumption, providing an aesthetic and functional way to monitor your system resources.

##  Features

- **Real-time RAM monitoring** - Continuously tracks system memory usage
- **Color-coded visualization**:
  -  **Green** (0-30%): All good, plenty of memory available
  -  **Yellow** (30-50%): Getting busier
  -  **Orange** (50-80%): Memory usage is high
  -  **Red** (80-100%): Critical memory usage
- **Dynamic animation speed** - Animation speeds up as RAM usage increases
- **Scalable window** - 128×128, 256×256, 512×512, or 1024×1024 pixels
- **Normal windowed mode** - Standard window with borders and decorations
- **Smooth 169-frame animation** - Fluid lava lamp effect

##  Controls

- **Ctrl + Up Arrow**: Scale window up
- **Ctrl + Down Arrow**: Scale window down  
- **Esc**: Exit application

## Known Issues

**Shadow rendering bug**: There is currently a visual bug where the shadow in the bottom-left corner of the lava lamp is missing or not rendering correctly. I discovered this issue but haven't been able to fix it yet. If anyone has a solution or suggestions, contributions would be greatly appreciated!

##  Future Plans

The following features are planned for future releases:
- **CPU usage visualization** - Monitor processor load
- **GPU usage visualization** - Track graphics card activity
- **VRAM monitoring** - Display video memory usage
- **Multi-monitor support** - Show different metrics on multiple widgets
- **Customizable thresholds** - User-defined color change points

##  Requirements

- **Rust** (1.70 or newer)
- **Linux** (tested on Ubuntu/Debian-based systems)
- Required system libraries:
  ```bash
  sudo apt install libxcb-shape0-dev libxcb-xfixes0-dev
  ```

##  Quick Start

### Development Mode

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/ram-lavalampe.git
   cd ram-lavalampe
   ```

2. Run in development mode:
   ```bash
   cargo run
   ```

3. Or build and run optimized version:
   ```bash
   cargo run --release
   ```

### System Installation

Install the lava lamp to run from anywhere:

1. Make the install script executable:
   ```bash
   chmod +x install.sh
   ```

2. Run the installer:
   ```bash
   ./install.sh
   ```

3. Make sure `~/.local/bin` is in your PATH. If not, add to `~/.bashrc`:
   ```bash
   export PATH="$HOME/.local/bin:$PATH"
   ```

4. Reload your shell configuration:
   ```bash
   source ~/.bashrc
   ```

5. Run from anywhere:
   ```bash
   ram-lavalampe
   ```

##  Assets

The application uses sprite sheet animations with 169 frames per color. Each PNG file should be:
- **Width**: 21,632 pixels (169 frames × 128 pixels)
- **Height**: 128 pixels

Assets are located in the `assets/` directory:
- `lavalampe_green.png` - Green lava (low RAM usage)
- `lavalampe_yellow.png` - Yellow lava (moderate RAM usage)
- `lavalampe_orange.png` - Orange lava (high RAM usage)
- `lavalampe_red.png` - Red lava (critical RAM usage)

##  Project Structure

```
ram-lavalampe/
├── assets/              # Sprite sheets for animations
│   ├── lavalampe_green.png
│   ├── lavalampe_yellow.png
│   ├── lavalampe_orange.png
│   └── lavalampe_red.png
├── img/                 # Documentation images
│   └── lavalamp2.gif
├── src/
│   └── main.rs         # Main application code
├── Cargo.toml          # Project configuration
├── install.sh          # Installation script
└── README.md           # This file
```

##  Configuration

You can modify these constants in `src/main.rs`:

```rust
const WINDOW_SIZE: usize = 128;        // Base window size
const ANIMATION_FRAMES: usize = 169;    // Number of animation frames

// RAM thresholds for color changes
// Green:  0-30%
// Yellow: 30-50%
// Orange: 50-80%
// Red:    80-100%

// Animation speeds (milliseconds per frame)
// Green:  200ms (slow and relaxed)
// Yellow: 150ms (getting busier)
// Orange: 100ms (pretty busy)
// Red:    60ms (frantic!)
```

##  Building from Source

### Debug Build
```bash
cargo build
```

### Release Build (Optimized)
```bash
cargo build --release
```

The compiled binary will be in `target/release/ram-lavalampe`.

##  Dependencies

- **pixels** (0.13) - Pixel buffer for rendering
- **winit** (0.28) - Window creation and management
- **image** (0.24) - Image loading and processing
- **sysinfo** (0.29) - System information (RAM usage)
- **lazy_static** (1.4) - Static initialization

##  Troubleshooting

### Assets not found
Make sure you're running the application from the project directory, or use the installation script to install it system-wide.

### High CPU usage
The application continuously redraws at ~60 FPS. This is normal for a real-time visualization widget.

### Shadow bug in bottom-left corner
This is a known rendering issue. If you have experience with pixel-based rendering or sprite sheet rendering and can help fix this, please open an issue or submit a pull request!

##  License

This project is open source. Feel free to use and modify it as needed.

##  Contributing

Contributions are welcome! Feel free to:
- Report bugs
- Suggest new features
- Submit pull requests
- Create new lava lamp color schemes
- **Help fix the shadow rendering bug!**

If you have ideas for the planned features (CPU, GPU, VRAM monitoring) or solutions for the current bugs, your input would be greatly appreciated!

##  Acknowledgments

- Built with Rust 🦀
- Uses the excellent `pixels` crate for efficient rendering
- Inspired by classic lava lamp aesthetics
