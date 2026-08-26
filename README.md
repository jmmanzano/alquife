# Alquife

A terminal-based Subsonic music client written in Rust, powered by FFmpeg. Works on Windows and Linux.

Alquife is a fork of [Ferrosonic](https://github.com/aarondill/ferrosonic) with enhanced FFmpeg audio playback, radio streaming support, and native Windows compilation.

## Features

- **FFmpeg audio playback** - High-quality audio streaming on Windows and Linux
- **Artist/album browser** - Tree-based navigation with artist and album listings
- **Playlists** - Browse and play server playlists with shuffle
- **Internet radio** - Play radio stations configured on your Subsonic server
- **Queue management** - Add, remove, reorder, shuffle, and skip low-rated songs
- **Session persistence** - Restores queue and UI state between runs
- **13 built-in themes** - Multiple dark and light themes (Monokai, Dracula, Nord, Catppuccin, etc.)
- **Custom themes** - Create your own themes as TOML files in `~/.config/alquife/themes/`
- **Mouse support** - Clickable navigation and progress bar seeking
- **Desktop integration** - MPRIS2 media controls on Linux, keyboard media keys on Windows
- **Audio device selection** - Switch between speakers and Bluetooth devices (F8)
- **Keyboard-driven** - Vim-style navigation (j/k) and arrow keys

## Installation

### Dependencies

Alquife requires **FFmpeg** at runtime.

Optional dependencies:
- **PipeWire** / **PulseAudio** - For audio device management (Linux)
- **D-Bus** - MPRIS2 desktop media controls (Linux)

### Quick Install (Linux)

Supports Arch, Fedora, and Debian/Ubuntu:

```bash
curl -sSf https://raw.githubusercontent.com/jmmanzano/alquife/master/install.sh | sh
```

### Windows

1. Download `alquife-windows-x86_64.zip` from [Releases](https://github.com/jmmanzano/alquife/releases)
2. Extract the zip
3. Run `install.bat` (downloads and installs FFmpeg if needed)
4. Launch Alquife from Start Menu or Desktop

### Linux Releases

- **Binary**: `alquife-linux-x86_64`
- **Debian package**: `.deb` with desktop integration

### Build from Source

```bash
git clone https://github.com/jmmanzano/alquife.git
cd alquife
cargo build --release
sudo cp target/release/alquife /usr/local/bin/
```

Requires: Rust toolchain, pkg-config, OpenSSL dev headers, and D-Bus dev headers (Linux).

## Usage

```bash
alquife                    # Run with default config
alquife -c config.toml     # Custom config file
alquife -v                 # Verbose logging
```

## Configuration

Config file: `~/.config/alquife/config.toml`

```toml
BaseURL = "https://your-subsonic-server.com"
Username = "your-username"
Password = "your-password"
Theme = "Default"
NonStopMode = false
AudioOutputDevice = ""
```

Configure server connection directly in Alquife (F5 - Server page).

## Keyboard Shortcuts

### Global

| Key | Action |
|---|---|
| `q` | Quit |
| `p` / `Space` | Play/Pause |
| `l` | Next track |
| `h` | Previous track |
| `Shift+Left/Right` | Seek ±10s |
| `t` | Cycle theme |
| `F1` | Artists |
| `F2` | Queue |
| `F3` | Playlists |
| `F4` | Radio |
| `F5` | Server settings |
| `F6` | Settings |
| `F7` | Equalizer |
| `F8` | Audio devices |

### Artists (F1)

| Key | Action |
|---|---|
| `↑/↓` or `k/j` | Navigate artists/albums |
| `←/→` | Collapse/expand, move panes |
| `e` | Add to queue |
| `n` | Add as next song |
| `0-5` | Rate song |
| `/` | Filter artists |
| `Enter` | Play selected |

### Queue (F2)

| Key | Action |
|---|---|
| `↑/↓` or `k/j` | Navigate |
| `d` | Remove song |
| `J/K` | Move song up/down |
| `r` | Shuffle queue |
| `c` / `C` | Clear history / Clear queue |
| `0-5` | Rate song |
| `Enter` | Play selected |

### Playlists (F3)

| Key | Action |
|---|---|
| `↑/↓` or `←/→` | Navigate |
| `e` | Add to queue |
| `Enter` | Play |

### Radio (F4)

| Key | Action |
|---|---|
| `↑/↓` | Navigate stations |
| `Enter` | Play |
| `s` | Stop |

### Audio Devices (F8)

| Key | Action |
|---|---|
| `↑/↓` | Change filter / Navigate devices |
| `Tab/→` | Switch pane |
| `Enter` | Select device |
| `Del` | Use system default |
| `r` | Refresh device list |

## Supported Servers

Alquife works with any Subsonic-compatible server:
- Subsonic
- Navidrome
- Airsonic
- Gonic
- Ampache

## License

Alquife is released under the GPL-3.0 License.

## Contributing

Issues and pull requests welcome! Check out the [GitHub repository](https://github.com/jmmanzano/alquife).

## Troubleshooting

### No audio output
- Ensure FFmpeg is installed: `ffmpeg -version`
- Check that your Subsonic server URL is correct (F5)
- Try setting `RUST_LOG=debug` for detailed logs

### Audio device not appearing
- Linux: Ensure PipeWire/PulseAudio is running
- Windows: Check Sound settings → Advanced → App volume and device preferences
- Press `r` on the Audio Devices page (F8) to refresh

### Theme not applying
Place custom theme TOML files in:
- Linux: `~/.config/alquife/themes/`
- Windows: `%APPDATA%\Alquife\themes\`
