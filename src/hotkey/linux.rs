use crate::settings::Settings;

pub struct HotkeyManager;

impl HotkeyManager {
    pub fn new() -> Self {
        Self
    }

    pub fn register_all(&mut self, _settings: &Settings) {
        // Future: X11/Wayland global hotkey support.
    }

    pub fn unregister_all(&mut self) {}
}
