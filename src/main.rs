#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod audio;
mod settings;
mod settings_window;
mod toast;
mod tray;

fn main() {
    if std::env::args().any(|a| a == "--settings") {
        settings_window::run();
        return;
    }
    app::run();
}
