# Rust Media Player with FFmpeg

A media player built in Rust using FFmpeg for audio decoding and hot-reloadable UI.

## Setup Instructions for Windows

1. **Install Rust**
   - Download and install from [rustup.rs](https://rustup.rs/)

2. **Install Visual Studio Build Tools**
   - Download from [Visual Studio Downloads](https://visualstudio.microsoft.com/downloads/)
   - Under "Tools for Visual Studio", select "Build Tools for Visual Studio 2022" and choose "Desktop development with C++"

3. **Install FFmpeg Dependencies**
   - Make sure vcpkg is properly installed
   - Install FFmpeg libraries:
     ```
     cd C:\vcpkg
     .\vcpkg install ffmpeg:x64-windows
     ```
   - Set environment variable:
     ```
     setx VCPKG_ROOT C:\vcpkg
     ```

4. **Build and Run the Media Player**
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
     ```
     cargo build --release
     ```

## Supported Formats

With FFmpeg integration, the player now supports a wide range of audio formats including:
- MP3, WAV, FLAC, OGG, M4A, AAC, OPUS
- WMA, APE, MKA, TTA, WV
- MIDI, AU, AIFF
- And many more formats supported by FFmpeg!

## Features

- Play, pause, and stop functionality
- Automatic sample rate conversion
- Multi-channel audio support
- Hot-reloadable UI components