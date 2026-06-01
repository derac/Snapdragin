#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

#[cfg(windows)]
mod windows_app;

#[cfg(windows)]
fn main() {
    windows_app::run();
}

#[cfg(not(windows))]
fn main() {
    eprintln!("snapdragin-windows only runs on Windows.");
}
