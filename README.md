# Rust Media Player

A simple audio player using Rust with Symphonia and cpal.

## Setup Instructions for Windows

1. **Install Rust**
   - If not already installed, download and install from [rustup.rs](https://rustup.rs/)

2. **Install Visual Studio Build Tools**
   - Download from [Visual Studio Downloads](https://visualstudio.microsoft.com/downloads/)
   - Under "Tools for Visual Studio", download "Build Tools for Visual Studio 2022"
   - Run the installer and select the "Desktop development with C++" workload

3. **Build and Run the Media Player**
   - Navigate to the project directory
   - Run `cargo build`
   - To play an audio file: `cargo run -- path/to/audio/file.mp3`

## Supported Formats

- MP3
- AAC
- ALAC
- FLAC
- WAV
- OGG/Vorbis

## Features

- Play, pause, and stop functionality
- Automatic sample rate conversion
- Multi-channel audio support