// app/src/ui/icons.rs
use iced::widget::image;

// Embed SVG bytes directly in the binary
const PLAY_ICON: &[u8] = include_bytes!("../../assets/icons/ph--play-circle-thin.svg");
const PAUSE_ICON: &[u8] = include_bytes!("../../assets/icons/ph--pause-circle-thin.svg");
const SKIP_BACK_ICON: &[u8] = include_bytes!("../../assets/icons/ph--skip-back-thin.svg");
const SKIP_FORWARD_ICON: &[u8] = include_bytes!("../../assets/icons/ph--skip-forward-thin.svg");
const REWIND_ICON: &[u8] = include_bytes!("../../assets/icons/ph--rewind-thin.svg");
const FAST_FORWARD_ICON: &[u8] = include_bytes!("../../assets/icons/ph--fast-forward-thin.svg");
const VOLUME_ICON: &[u8] = include_bytes!("../../assets/icons/ph--speaker-simple-high-thin.svg");
const X_ICON: &[u8] = include_bytes!("../../assets/icons/ph--x-square-thin.svg");
const FILE_PLUS_ICON: &[u8] = include_bytes!("../../assets/icons/ph--file-plus-thin.svg");
const FOLDER_PLUS_ICON: &[u8] = include_bytes!("../../assets/icons/ph--folder-plus-thin.svg");
const GRID_ICON: &[u8] = include_bytes!("../../assets/icons/ph--grid-nine-thin.svg");
const LIST_ICON: &[u8] = include_bytes!("../../assets/icons/ph--list-bullets-thin.svg");

// Functions to get svg images
pub fn play_icon() -> image::Handle {
    image::Handle::from_bytes(PLAY_ICON.to_vec())
}

pub fn pause_icon() -> image::Handle {
    image::Handle::from_bytes(PAUSE_ICON.to_vec())
}

pub fn skip_back_icon() -> image::Handle {
    image::Handle::from_bytes(SKIP_BACK_ICON.to_vec())
}

pub fn skip_forward_icon() -> image::Handle {
    image::Handle::from_bytes(SKIP_FORWARD_ICON.to_vec())
}

pub fn rewind_icon() -> image::Handle {
    image::Handle::from_bytes(REWIND_ICON.to_vec())
}

pub fn fast_forward_icon() -> image::Handle {
    image::Handle::from_bytes(FAST_FORWARD_ICON.to_vec())
}

pub fn volume_icon() -> image::Handle {
    image::Handle::from_bytes(VOLUME_ICON.to_vec())
}

pub fn x_icon() -> image::Handle {
    image::Handle::from_bytes(X_ICON.to_vec())
}

pub fn file_plus_icon() -> image::Handle {
    image::Handle::from_bytes(FILE_PLUS_ICON.to_vec())
}

pub fn folder_plus_icon() -> image::Handle {
    image::Handle::from_bytes(FOLDER_PLUS_ICON.to_vec())
}

pub fn grid_icon() -> image::Handle {
    image::Handle::from_bytes(GRID_ICON.to_vec())
}

pub fn list_icon() -> image::Handle {
    image::Handle::from_bytes(LIST_ICON.to_vec())
}