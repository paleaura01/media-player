# Rust Media Player

A media player built in Rust using Symphonia for audio decoding and hot-lib-reloader for hot-reloadable UI.

## Setup Instructions for Windows

1. **Install Rust**
   - Download and install from [rustup.rs](https://rustup.rs/)

2. **Install Visual Studio Build Tools**
   - Download from [Visual Studio Downloads](https://visualstudio.microsoft.com/downloads/)
   - Under "Tools for Visual Studio", select "Build Tools for Visual Studio 2022" and choose "Desktop development with C++"

3. **Build and Run the Media Player**
   - Navigate to the project directory.
   - For development with hot reloading, run:
     ```
     cargo clean && cargo run --bin dev
     ```
   - For production build:
     ```
    cargo run --release --bin dev    
     ```
     For distribution build:
     cargo build --release

## Supported Formats

- MP3
- AAC
- ALAC
- FLAC
- WAV
- OGG/Vorbis
- OGG/Opus
- M4A (AAC in MP4 container)

## Features

- Play, pause, and stop functionality
- Automatic sample rate conversion
- Multi-channel audio support
- Hot-reloadable UI components
