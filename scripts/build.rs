fn main() {
    if cfg!(feature = "audio") {
        println!("cargo:rustc-env=RUST_LOG=info");
    }
}
