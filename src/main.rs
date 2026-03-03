mod app;
mod audio;
mod settings;
mod settings_window;
mod tray;

fn main() {
    if std::env::args().any(|a| a == "--settings") {
        settings_window::run();
        return;
    }
    app::run();
}
