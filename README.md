# 🎵 Terminal Music Player

A beautiful, vim-inspired terminal-based music player written in Rust. Navigate your music library with keyboard shortcuts and enjoy a clean, distraction-free listening experience.

![Music Player Demo](https://img.shields.io/badge/Platform-macOS%20%7C%20Linux-blue)
![Language](https://img.shields.io/badge/Language-Rust-orange)
![License](https://img.shields.io/badge/License-MIT-green)

## ✨ Features

- **🎹 Vim-Inspired Controls** - Navigate with `j`/`k`, play with `Space`, and more
- **🎯 Accurate Progress Tracking** - Real-time progress bar with actual song durations
- **🔀 Shuffle & Repeat** - Multiple playback modes for your listening pleasure
- **📁 Directory Support** - Load entire music directories or single files
- **🎼 Multiple Formats** - Supports MP3, WAV, OGG, FLAC, and M4A files
- **⚡ Fast & Lightweight** - Built in Rust for performance and reliability
- **🎨 Beautiful UI** - Clean terminal interface with track listing and controls

## 🚀 Quick Start

### Prerequisites

- **Rust** (1.70 or later) - [Install Rust](https://rustup.rs/)
- **Git** - For cloning the repository

### Download & Build

```bash
# Clone the repository
git clone https://github.com/yourusername/music-player.git
cd music-player

# Build the application
cargo build --release

# The executable will be at ./target/release/music_player
```

### Basic Usage

```bash
# Play all music files in a directory
./target/release/music_player /path/to/your/music

# Play a specific file
./target/release/music_player /path/to/song.mp3

# Play music from the included sample directory
./target/release/music_player music
```

## 🎮 Controls

| Key | Action |
|-----|--------|
| `j` / `↓` | Navigate down in track list |
| `k` / `↑` | Navigate up in track list |
| `Space` | Play selected track / Pause current track |
| `Enter` | Play selected track |
| `n` | Next track (changes playback) |
| `p` | Previous track (changes playback) |
| `s` | Toggle shuffle mode |
| `r` | Cycle repeat modes (None → One → All) |
| `?` | Show/hide help |
| `q` / `Esc` | Quit |

## 🎵 Supported Audio Formats

- **MP3** - Most common format, full metadata support
- **WAV** - Uncompressed audio
- **OGG** - Open-source alternative to MP3
- **FLAC** - Lossless compression
- **M4A** - Apple's audio format

> **Note**: WebM files are not supported. If you have WebM files, convert them to MP3 using ffmpeg:
> ```bash
> ffmpeg -i input.webm -acodec mp3 output.mp3
> ```

## 🛠️ Installation Options

### Option 1: Build from Source (Recommended)

```bash
git clone https://github.com/yourusername/music-player.git
cd music-player
cargo build --release
```

### Option 2: Install with Cargo

```bash
# Install directly from GitHub
cargo install --git https://github.com/yourusername/music-player.git

# Or install from crates.io (if published)
cargo install music-player
```

### Option 3: Download Pre-built Binary

Visit the [Releases](https://github.com/yourusername/music-player/releases) page to download pre-built binaries for your platform.

## 📖 Detailed Usage

### Playing Music

The music player can handle both individual files and entire directories:

```bash
# Play all supported audio files in a directory (recursive)
./target/release/music_player ~/Music

# Play a specific playlist folder
./target/release/music_player ~/Music/Anime\ Piano

# Play a single file
./target/release/music_player "~/Music/favorite-song.mp3"
```

### Navigation & Playback

1. **Track Selection**: Use `j`/`k` or arrow keys to highlight tracks
2. **Playback Control**: Press `Space` to play the selected track or pause/resume
3. **Track Switching**: Use `n`/`p` to change what's currently playing
4. **Modes**: Toggle shuffle (`s`) and cycle repeat modes (`r`)

### Progress Tracking

The progress bar shows:
- **Real-time progress** based on actual song duration
- **Accurate timing** for MP3 files with metadata
- **Estimated timing** for other formats (5-minute default)

## 🔧 Development

### Running Tests

```bash
# Run the comprehensive test script
./test.sh

# Or run individual cargo tests
cargo test
cargo clippy
cargo fmt --check
```

### Project Structure

```
music-player/
├── src/
│   └── main.rs          # Main application code
├── music/               # Sample music files
├── Cargo.toml          # Rust dependencies
├── test.sh             # Test script
└── README.md           # This file
```

### Dependencies

- **rodio** - Audio playback and decoding
- **ratatui** - Terminal UI framework
- **crossterm** - Cross-platform terminal manipulation
- **mp3-duration** - Accurate MP3 duration extraction
- **anyhow** - Error handling
- **log** - Logging framework

## 🐛 Troubleshooting

### Common Issues

**"No supported audio files found"**
- Ensure your directory contains MP3, WAV, OGG, FLAC, or M4A files
- Check file permissions

**"Failed to decode audio file"**
- File may be corrupted or in an unsupported format
- Try converting to MP3 with ffmpeg

**Audio playback issues**
- Ensure your system has audio output devices available
- Check volume settings

### Getting Help

1. Run with `--help` for usage information
2. Check the [Issues](https://github.com/yourusername/music-player/issues) page
3. Enable debug logging: `RUST_LOG=info ./target/release/music_player`

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

### Development Setup

```bash
git clone https://github.com/yourusername/music-player.git
cd music-player
cargo build
cargo test
```

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built with [Rust](https://www.rust-lang.org/) 🦀
- UI powered by [ratatui](https://github.com/ratatui-org/ratatui)
- Audio playback via [rodio](https://github.com/RustAudio/rodio)
- Inspired by vim's keyboard navigation philosophy

---

**Made with ❤️ and Rust** - Enjoy your music! 🎵 