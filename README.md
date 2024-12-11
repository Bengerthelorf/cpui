# cpui

A TUI (Terminal User Interface) based replacement for the traditional `cp` command with an interactive progress bar and advanced features.

## Features

- Interactive progress bar showing both total and per-file progress
- Real-time transfer speed display
- Recursive directory copying support
- Graceful handling of Ctrl+C interruption
- Modern terminal UI using ratatui

## Installation

```bash
cargo install cpui
```

## Usage

Basic file copy:

```bash
cpui source.txt destination.txt
```

Recursive directory copy:

```bash
cpui -r source_dir destination_dir
```

## Development

Requirements:

- Rust 1.75 or higher
- Cargo

Build from source:

```bash
git clone https://github.com/zaneleong/cpui
cd modern-cp
cargo build --release
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Author

Zane Leong (2024)
