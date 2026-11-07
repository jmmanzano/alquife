# Alquife

A terminal-based Subsonic music client written in Rust, powered by FFmpeg. Works on Windows and Linux.

Alquife is a fork of [Ferrosonic](https://github.com/aarondill/ferrosonic) with enhanced FFmpeg audio playback, radio streaming support, and native Windows compilation.

## Features

- **FFmpeg audio playback** - High-quality audio streaming on Windows and Linux
- **Artist/album browser** - Tree-based navigation with artist and album listings
- **Playlists** - Browse and play server playlists with shuffle
- **Internet radio** - Play radio stations configured on your Subsonic server
- **Queue management** - Add, remove, reorder, shuffle, and skip low-rated songs
- **Offline song cache** - Automatically cache played songs or download selected songs and albums for offline playback
- **Album covers** - Display the currently playing album's artwork alongside the queue
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
ShowCoverArt = true
CacheSizeGB = 1
```

Configure server connection directly in Alquife (F5 - Server page).

### Album covers

The queue page (F2) displays the current song's album cover, fetched from the Subsonic server. Alquife detects supported terminal graphics protocols (Kitty, Sixel, or iTerm2) and falls back to Unicode color blocks when native graphics are unavailable. Image quality therefore depends on your terminal and its settings.

To disable artwork, open **F6 → Album covers** and use `←`, `→`, `Enter`, or `Space` to switch it off. The setting is saved automatically. You can also set `ShowCoverArt = false` in the configuration file. When disabled, Alquife does not request cover images and the queue uses the full page width.

### Offline playback and cache

Alquife starts when the server is unavailable and restores the saved queue. The footer displays **Connected** or **Offline**, based on periodic authenticated server checks. When offline, **F1 Artists** groups cached tracks by album: select an album to play its cached tracks, or a track to start there, and press Enter (or double-click). This replaces the queue only when playback is requested. Other songs require the server. In **F2 Queue**, press `O` to replace the queue with the available cached songs.

Songs played online are downloaded in the background when the cache is available. To download manually, press `D` on a selected song in F2, or on a selected song or album in F1. Downloads require a server connection. The cache has a shared size limit, and older entries may be removed to make room, including manually downloaded songs. Only completed downloads are available offline.

Set **F6 → Cache size** to an integer from 0 to 5 GB (default 1 GB). The setting is saved as `CacheSizeGB`; 0 disables caching and removes cached entries where possible. Cache data is separated by server and user, under `%LOCALAPPDATA%\Alquife\cache\songs` on Windows and `$XDG_CACHE_HOME/alquife/songs` (normally `~/.cache/alquife/songs`) on Linux. Cache content is disposable and is not a backup of your music.

To remove cached songs without changing the size limit, choose **F6 → Clear cache** and press Enter twice to confirm. This clears cached songs for all configured servers and users and stops playback if a cached file is playing.

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
| `F9` | Hide to notification area (Windows, dedicated classic console only) |

On Windows, run Alquife in its own classic console window to use F9. The tray
icon offers Restore and Exit; playback continues while hidden. Windows Terminal
and consoles shared with a shell or other processes are not supported, to avoid
hiding other programs. In unsupported environments F9 reports an error.
For example, select Windows Console Host as the default terminal in Windows
settings and launch `alquife.exe` directly, rather than from an existing shell.

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
| `D` | Download the selected song or album to the cache (online) |

When offline, F1 shows cached albums and tracks. Use `↑/↓` and `Enter`, or double-click, to play the selected album or track.

### Queue (F2)

The left pane contains the queue; the right pane shows the currently playing album's cover when artwork is enabled.

| Key | Action |
|---|---|
| `↑/↓` or `k/j` | Navigate |
| `d` | Remove song |
| `J/K` | Move song up/down |
| `r` | Shuffle queue |
| `c` / `C` | Clear history / Clear queue |
| `0-5` | Rate song |
| `Enter` | Play selected |
| `D` | Download the selected song to the cache |
| `O` | Replace the queue with cached songs |

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

### Album cover looks pixelated

Your terminal may not support a native graphics protocol, causing Alquife to use Unicode color blocks. Try a terminal that supports Kitty graphics or Sixel, or reduce the terminal font size to give the fallback more cells to work with.
