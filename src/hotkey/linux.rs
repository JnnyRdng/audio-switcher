use crate::settings::Settings;
use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use std::collections::HashMap;

pub struct HotkeyManager {
    manager: Option<GlobalHotKeyManager>,
    /// Maps hotkey ID → (hotkey, device ID).
    registered: HashMap<u32, (HotKey, String)>,
}

impl HotkeyManager {
    pub fn new() -> Self {
        let manager = GlobalHotKeyManager::new()
            .map_err(|e| eprintln!("Failed to initialize global hotkeys: {e}"))
            .ok();
        Self {
            manager,
            registered: HashMap::new(),
        }
    }

    pub fn register_all(&mut self, settings: &Settings) {
        self.unregister_all();

        let Some(ref manager) = self.manager else {
            return;
        };

        for (device_id, shortcut) in &settings.shortcuts {
            let Some(code) = key_name_to_code(&shortcut.key) else {
                eprintln!("Unknown key for hotkey: {}", shortcut.key);
                continue;
            };

            let mut mods = Modifiers::empty();
            if shortcut.ctrl {
                mods |= Modifiers::CONTROL;
            }
            if shortcut.alt {
                mods |= Modifiers::ALT;
            }
            if shortcut.shift {
                mods |= Modifiers::SHIFT;
            }
            if shortcut.win_key {
                mods |= Modifiers::SUPER;
            }

            let hotkey = HotKey::new(if mods.is_empty() { None } else { Some(mods) }, code);
            match manager.register(hotkey) {
                Ok(_) => {
                    self.registered.insert(hotkey.id(), (hotkey, device_id.clone()));
                }
                Err(e) => {
                    eprintln!(
                        "Failed to register hotkey {} for device {}: {e}",
                        shortcut, device_id
                    );
                }
            }
        }
    }

    pub fn unregister_all(&mut self) {
        if let Some(ref manager) = self.manager {
            for (hotkey, _) in self.registered.values() {
                let _ = manager.unregister(*hotkey);
            }
        }
        self.registered.clear();
    }

    /// Check for a pending hotkey press. Returns the device ID to switch to, if any.
    pub fn check_event(&self) -> Option<String> {
        while let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            if event.state == HotKeyState::Pressed {
                if let Some((_, device_id)) = self.registered.get(&event.id) {
                    return Some(device_id.clone());
                }
            }
        }
        None
    }
}

fn key_name_to_code(key: &str) -> Option<Code> {
    use Code::*;

    // Single uppercase letter A-Z
    if key.len() == 1 {
        return match key.chars().next().unwrap() {
            'A' => Some(KeyA),
            'B' => Some(KeyB),
            'C' => Some(KeyC),
            'D' => Some(KeyD),
            'E' => Some(KeyE),
            'F' => Some(KeyF),
            'G' => Some(KeyG),
            'H' => Some(KeyH),
            'I' => Some(KeyI),
            'J' => Some(KeyJ),
            'K' => Some(KeyK),
            'L' => Some(KeyL),
            'M' => Some(KeyM),
            'N' => Some(KeyN),
            'O' => Some(KeyO),
            'P' => Some(KeyP),
            'Q' => Some(KeyQ),
            'R' => Some(KeyR),
            'S' => Some(KeyS),
            'T' => Some(KeyT),
            'U' => Some(KeyU),
            'V' => Some(KeyV),
            'W' => Some(KeyW),
            'X' => Some(KeyX),
            'Y' => Some(KeyY),
            'Z' => Some(KeyZ),
            _ => None,
        };
    }

    // F-keys: egui uses "F1", "F2", ...
    if let Some(rest) = key.strip_prefix('F') {
        if let Ok(n) = rest.parse::<u8>() {
            return match n {
                1 => Some(F1),
                2 => Some(F2),
                3 => Some(F3),
                4 => Some(F4),
                5 => Some(F5),
                6 => Some(F6),
                7 => Some(F7),
                8 => Some(F8),
                9 => Some(F9),
                10 => Some(F10),
                11 => Some(F11),
                12 => Some(F12),
                13 => Some(F13),
                14 => Some(F14),
                15 => Some(F15),
                16 => Some(F16),
                17 => Some(F17),
                18 => Some(F18),
                19 => Some(F19),
                20 => Some(F20),
                21 => Some(F21),
                22 => Some(F22),
                23 => Some(F23),
                24 => Some(F24),
                _ => None,
            };
        }
    }

    match key {
        "0" => Some(Digit0),
        "1" => Some(Digit1),
        "2" => Some(Digit2),
        "3" => Some(Digit3),
        "4" => Some(Digit4),
        "5" => Some(Digit5),
        "6" => Some(Digit6),
        "7" => Some(Digit7),
        "8" => Some(Digit8),
        "9" => Some(Digit9),
        "Space" => Some(Space),
        "Enter" => Some(Enter),
        "Tab" => Some(Tab),
        "Escape" => Some(Escape),
        "Backspace" => Some(Backspace),
        "Delete" => Some(Delete),
        "Insert" => Some(Insert),
        "Home" => Some(Home),
        "End" => Some(End),
        "PageUp" => Some(PageUp),
        "PageDown" => Some(PageDown),
        "Up" => Some(ArrowUp),
        "Down" => Some(ArrowDown),
        "Left" => Some(ArrowLeft),
        "Right" => Some(ArrowRight),
        "Minus" => Some(Minus),
        "Plus" | "Equals" => Some(Equal),
        "OpenBracket" => Some(BracketLeft),
        "CloseBracket" => Some(BracketRight),
        "Backslash" => Some(Backslash),
        "Semicolon" => Some(Semicolon),
        "Quote" => Some(Quote),
        "Comma" => Some(Comma),
        "Period" => Some(Period),
        "Slash" => Some(Slash),
        "Backtick" => Some(Backquote),
        _ => None,
    }
}
