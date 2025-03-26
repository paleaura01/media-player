use log::info;

pub fn register_all_codecs() {
    // Symphonia automatically registers codecs based on enabled features
    // We don't need to manually register anything - the features in Cargo.toml handle this
    
    // Log the supported formats
    info!("Audio system initialized");
    info!("Player supports: MP3, AAC, FLAC, WAV, M4A, and OGG files");
    
    // Note about Opus: Limited support currently available through OGG container
    info!("Note: Opus support is limited to files that Symphonia can automatically detect");
}