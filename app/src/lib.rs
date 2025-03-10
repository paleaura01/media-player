// app/src/lib.rs
// This file is the entry point for the dynamic library.
// We include the UI module without re-exporting it publicly to avoid duplicate symbols.
#[path = "ui/mod.rs"]
mod ui;
