use crate::settings::ToastPosition;
use std::process::Command;

pub fn show_toast(
    name: &str,
    duration_ms: u32,
    _opacity: f32,
    _fade: bool,
    _position: ToastPosition,
    _dark: bool,
) {
    let body = format!("Switched to {name}");
    let timeout = duration_ms.to_string();

    let _ = Command::new("notify-send")
        .args(["Audio Switcher", &body, "-t", &timeout])
        .spawn();
}
