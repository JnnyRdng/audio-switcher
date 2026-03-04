#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "linux")]
mod linux;

use crate::settings::Settings;

/// Show a toast notification for a device switch.
/// Spawns a background thread, returns immediately.
pub fn show(device_name: &str, settings: &Settings) {
    if !settings.show_toast {
        return;
    }

    let name = device_name.to_string();
    let duration_ms = settings.toast_duration_ms;
    let opacity = settings.toast_opacity;
    let fade = settings.toast_fade;
    let position = settings.toast_position;
    let dark = settings.is_dark();

    std::thread::spawn(move || {
        #[cfg(target_os = "windows")]
        windows::show_toast(&name, duration_ms, opacity, fade, position, dark);
        #[cfg(target_os = "linux")]
        linux::show_toast(&name, duration_ms, opacity, fade, position, dark);
    });
}
