# Terminal Music Player

A beautiful, vim-inspired terminal-based music player written in Rust. Navigate your music library with keyboard shortcuts and enjoy a clean, distraction-free listening experience.

![Music Player Demo](https://img.shields.io/badge/Platform-macOS%20%7C%20Linux-blue)
![Language](https://img.shields.io/badge/Language-Rust-orange)
![License](https://img.shields.io/badge/License-MIT-green)

## Features

- **Vim-Inspired Controls** - Navigate with `j`/`k`, play with `Space`, and more
- **Accurate Progress Tracking** - Real-time progress bar with actual song durations
- **Shuffle & Repeat** - Multiple playback modes for your listening pleasure
- **Directory Support** - Load entire music directories or single files
- **Multiple Formats** - Supports MP3, WAV, OGG, FLAC, M4A
- **Beautiful Terminal UI** - Clean interface built with ratatui
- **Signal Handling** - Graceful shutdown with Ctrl+C

## 🚀 Quick Start

### Prerequisites
- **Rust** (1.70 or later) - [Install from rustup.rs](https://rustup.rs/)
- **Git** - For cloning the repository

### Download & Build
```bash
# Clone the repository
git clone https://github.com/azcoldflames19/music-player.git
cd music-player

# Build the project
cargo build --release

# Run with your music directory
./target/release/music_player /path/to/your/music

# Or run with the included anime piano collection
./target/release/music_player music
```

### Alternative Installation Methods

**Install from GitHub (requires Rust):**
```bash
cargo install --git https://github.com/azcoldflames19/music-player.git
music_player /path/to/your/music
```

**Download Pre-built Binary:**
- Check the [Releases page](https://github.com/azcoldflames19/music-player/releases) for pre-compiled binaries

## Controls

| Key | Action |
|-----|--------|
| `j` or `↓` | Navigate down in track list |
| `k` or `↑` | Navigate up in track list |
| `Space` | Play selected track or pause/unpause current |
| `Enter` | Play selected track |
| `n` | Next track (changes playback) |
| `p` | Previous track (changes playback) |
| `s` | Toggle shuffle mode |
| `r` | Cycle repeat modes (Off → Track → All) |
| `q` or `Esc` | Quit the application |
| `?` | Show help screen |

## Usage

### Playing Music
```bash
# Play a single file
./target/release/music_player song.mp3

# Play all music in a directory
./target/release/music_player /Users/username/Music

# Play the included anime piano collection
./target/release/music_player music
```

### Navigation Tips
- Use `j`/`k` to browse tracks without changing what's playing
- Press `Space` to play the selected track or pause/unpause
- Use `n`/`p` to change what's actually playing
- The progress bar shows real-time playback with accurate durations

## Supported Formats

- **MP3** - Primary format with metadata extraction
- **WAV** - Uncompressed audio
- **OGG** - Open-source compressed format
- **FLAC** - Lossless compression
- **M4A** - Apple's audio format

**Note:** WebM files are not supported. If you have WebM files, convert them to MP3:
```bash
ffmpeg -i input.webm -acodec mp3 output.mp3
```

## Development

### Running Tests
```bash
# Run the comprehensive test script
./test.sh

# Or run individual Rust tests
cargo test
```

### Project Structure
```
music-player/
├── src/
│   └── main.rs          # Main application code
├── music/               # 161 anime piano tracks included
├── Cargo.toml          # Dependencies and project config
├── README.md           # This file
└── test.sh            # Comprehensive test script
```

### Dependencies
- **rodio** - Audio playback engine
- **ratatui** + **crossterm** - Terminal UI framework
- **mp3-duration** - Accurate MP3 duration extraction
- **anyhow** - Error handling
- **walkdir** - Directory traversal
- **ctrlc** - Signal handling

## Troubleshooting

**"No such file or directory"**
- Ensure the path exists and contains supported audio files
- Check file permissions

**"Failed to decode audio file"**
- File may be corrupted or in an unsupported format
- Try converting to MP3 or another supported format

**"No audio device found"**
- Ensure your system has working audio output
- Check audio drivers and system audio settings

**Build fails**
- Update Rust: `rustup update`
- Clear cache: `cargo clean && cargo build`

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature-name`
3. Make your changes and test: `./test.sh`
4. Commit: `git commit -m "Add feature-name"`
5. Push: `git push origin feature-name`
6. Open a Pull Request

### Development Setup
```bash
git clone https://github.com/azcoldflames19/music-player.git
cd music-player
cargo build
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [rodio](https://github.com/RustAudio/rodio) for audio playback
- UI powered by [ratatui](https://github.com/ratatui-org/ratatui)
- Includes 161 beautiful anime piano arrangements for testing and enjoyment 