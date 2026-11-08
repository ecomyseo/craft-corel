// No console window in release builds on Windows.
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

//! The Craft Corel program: the whole app is the `tracedraw` library; this
//! is the desktop and browser entry (the Android app loads the library).

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    tracedraw::run()
}

#[cfg(target_arch = "wasm32")]
fn main() {
    tracedraw::run();
}
