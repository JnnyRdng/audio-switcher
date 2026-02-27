mod app;
mod audio;
mod tray;

#[cfg(target_os = "windows")]
fn main() {
    app::build();
}

#[cfg(not(target_os = "windows"))]
fn main() {
    eprintln!("This app currently only supports Windows.");
    eprintln!("Linux support is planned for a future phase.");
    std::process::exit(1);
}
