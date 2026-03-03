use crate::settings::{Settings, Shortcut};
use std::collections::HashMap;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT,
    MOD_SHIFT, MOD_WIN,
};
use windows::Win32::UI::WindowsAndMessaging::MSG;

pub struct HotkeyManager {
    /// Maps hotkey numeric ID → device ID string.
    registered: HashMap<i32, String>,
    next_id: i32,
}

impl HotkeyManager {
    pub fn new() -> Self {
        Self {
            registered: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn register_all(&mut self, settings: &Settings) {
        self.unregister_all();

        for (device_id, shortcut) in &settings.shortcuts {
            let Some(vk) = key_name_to_vk(&shortcut.key) else {
                eprintln!("Unknown key for hotkey: {}", shortcut.key);
                continue;
            };

            let mods = build_modifiers(shortcut);
            let id = self.next_id;
            self.next_id += 1;

            unsafe {
                if RegisterHotKey(None, id, mods, vk).is_ok() {
                    self.registered.insert(id, device_id.clone());
                } else {
                    eprintln!("Failed to register hotkey {} for device {}", shortcut, device_id);
                }
            }
        }
    }

    pub fn unregister_all(&mut self) {
        for &id in self.registered.keys() {
            unsafe {
                let _ = UnregisterHotKey(None, id);
            }
        }
        self.registered.clear();
    }

    /// If the message is WM_HOTKEY, returns the device ID to switch to.
    pub fn handle_hotkey(&self, msg: &MSG) -> Option<String> {
        if msg.message != 0x0312 {
            // WM_HOTKEY
            return None;
        }
        let id = msg.wParam.0 as i32;
        self.registered.get(&id).cloned()
    }
}

impl Drop for HotkeyManager {
    fn drop(&mut self) {
        self.unregister_all();
    }
}

fn build_modifiers(shortcut: &Shortcut) -> HOT_KEY_MODIFIERS {
    let mut mods = MOD_NOREPEAT;
    if shortcut.ctrl {
        mods |= MOD_CONTROL;
    }
    if shortcut.alt {
        mods |= MOD_ALT;
    }
    if shortcut.shift {
        mods |= MOD_SHIFT;
    }
    if shortcut.win_key {
        mods |= MOD_WIN;
    }
    mods
}

/// Map egui `Key::name()` strings to Win32 virtual key codes.
fn key_name_to_vk(key: &str) -> Option<u32> {
    // Single uppercase letter (A-Z)
    if key.len() == 1 {
        let ch = key.chars().next().unwrap();
        if ch.is_ascii_uppercase() {
            return Some(ch as u32); // 'A'=0x41 .. 'Z'=0x5A
        }
    }

    // F-keys
    if let Some(rest) = key.strip_prefix('F') {
        if let Ok(n) = rest.parse::<u32>() {
            if (1..=24).contains(&n) {
                return Some(0x6F + n); // VK_F1=0x70 .. VK_F24=0x87
            }
        }
    }

    // egui Key::name() values
    match key {
        // Digits (egui uses "0".."9" for Num0..Num9)
        "0" => Some(0x30),
        "1" => Some(0x31),
        "2" => Some(0x32),
        "3" => Some(0x33),
        "4" => Some(0x34),
        "5" => Some(0x35),
        "6" => Some(0x36),
        "7" => Some(0x37),
        "8" => Some(0x38),
        "9" => Some(0x39),
        // Commands
        "Space" => Some(0x20),
        "Enter" => Some(0x0D),
        "Tab" => Some(0x09),
        "Escape" => Some(0x1B),
        "Backspace" => Some(0x08),
        "Delete" => Some(0x2E),
        "Insert" => Some(0x2D),
        "Home" => Some(0x24),
        "End" => Some(0x23),
        "PageUp" => Some(0x21),
        "PageDown" => Some(0x22),
        "Up" => Some(0x26),
        "Down" => Some(0x28),
        "Left" => Some(0x25),
        "Right" => Some(0x27),
        // Punctuation (egui Key::name() values)
        "Minus" => Some(0xBD),         // VK_OEM_MINUS
        "Plus" | "Equals" => Some(0xBB), // VK_OEM_PLUS (the =/+ key)
        "OpenBracket" => Some(0xDB),    // VK_OEM_4
        "CloseBracket" => Some(0xDD),   // VK_OEM_6
        "Backslash" => Some(0xDC),      // VK_OEM_5
        "Semicolon" => Some(0xBA),      // VK_OEM_1
        "Quote" => Some(0xDE),          // VK_OEM_7
        "Comma" => Some(0xBC),          // VK_OEM_COMMA
        "Period" => Some(0xBE),         // VK_OEM_PERIOD
        "Slash" => Some(0xBF),          // VK_OEM_2
        "Backtick" => Some(0xC0),       // VK_OEM_3
        "Colon" => Some(0xBA),          // Same physical key as Semicolon
        "Pipe" => Some(0xDC),           // Same physical key as Backslash
        _ => None,
    }
}
