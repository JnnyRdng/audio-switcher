use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{fmt, fs};
use std::path::PathBuf;
use strum_macros::EnumIter;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, EnumIter)]
pub enum Theme {
    Light,
    Dark,
    System,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, EnumIter)]
pub enum ToastPosition {
    TopLeft,
    TopCenter,
    TopRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Shortcut {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub win_key: bool,
    pub key: String,
}

impl fmt::Display for Shortcut {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();
        if self.ctrl {
            parts.push("Ctrl".to_string());
        }
        if self.alt {
            parts.push("Alt".to_string());
        }
        if self.shift {
            parts.push("Shift".to_string());
        }
        if self.win_key {
            parts.push("Win".to_string());
        }
        parts.push(display_key_name(&self.key));
        write!(f, "{}", parts.join("+"))
    }
}

impl fmt::Display for Theme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            Theme::System => "System",
            Theme::Dark => "Dark",
            Theme::Light => "Light",
        };
        write!(f, "{}", str)
    }
}

/// Convert egui Key::name() strings to human-readable labels.
fn display_key_name(key: &str) -> String {
    match key {
        "OpenBracket" => "[".to_string(),
        "CloseBracket" => "]".to_string(),
        "OpenCurlyBracket" => "{".to_string(),
        "CloseCurlyBracket" => "}".to_string(),
        "Backtick" => "`".to_string(),
        "Backslash" => "\\".to_string(),
        "Slash" => "/".to_string(),
        "Semicolon" => ";".to_string(),
        "Quote" => "'".to_string(),
        "Comma" => ",".to_string(),
        "Period" => ".".to_string(),
        "Minus" => "-".to_string(),
        "Plus" => "+".to_string(),
        "Equals" => "=".to_string(),
        "Colon" => ":".to_string(),
        "Pipe" => "|".to_string(),
        "Questionmark" => "?".to_string(),
        "Exclamationmark" => "!".to_string(),
        "PageUp" => "PgUp".to_string(),
        "PageDown" => "PgDn".to_string(),
        other => other.to_string(),
    }
}

impl fmt::Display for ToastPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            ToastPosition::TopLeft => "Top left",
            ToastPosition::TopCenter => "Top center",
            ToastPosition::TopRight => "Top right",
            ToastPosition::BottomLeft => "Bottom left",
            ToastPosition::BottomCenter => "Bottom center",
            ToastPosition::BottomRight => "Bottom right",
        };
        write!(f, "{}", text)
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct Settings {
    pub play_sound: bool,
    pub show_toast: bool,
    pub toast_fade: bool,
    pub toast_duration_ms: u32,
    pub toast_opacity: f32,
    pub theme: Theme,
    pub toast_position: ToastPosition,
    #[serde(default)]
    pub shortcuts: HashMap<String, Shortcut>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            play_sound: true,
            show_toast: true,
            toast_fade: true,
            toast_duration_ms: 1000,
            toast_opacity: 0.75,
            theme: Theme::System,
            toast_position: ToastPosition::BottomCenter,
            shortcuts: HashMap::new(),
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        let path = config_path();
        match fs::read_to_string(&path) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) {
        let path = config_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(&path, json);
        }
    }

    /// Resolve the theme to a concrete dark/light value.
    pub fn is_dark(&self) -> bool {
        match self.theme {
            Theme::Dark => true,
            Theme::Light => false,
            Theme::System => system_is_dark(),
        }
    }
}

fn config_path() -> PathBuf {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("audio-switcher");
    dir.join("settings.json")
}

/// Detect whether the OS is set to dark mode.
#[cfg(target_os = "windows")]
fn system_is_dark() -> bool {
    use windows::Win32::System::Registry::{HKEY_CURRENT_USER, RRF_RT_REG_DWORD, RegGetValueW};
    use windows::core::w;

    let mut data: u32 = 1; // default: light
    let mut size = std::mem::size_of::<u32>() as u32;

    let result = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            w!(r"SOFTWARE\Microsoft\Windows\CurrentVersion\Themes\Personalize"),
            w!("AppsUseLightTheme"),
            RRF_RT_REG_DWORD,
            None,
            Some(&mut data as *mut u32 as *mut _),
            Some(&mut size),
        )
    };

    if result.is_ok() {
        data == 0 // 0 means dark mode
    } else {
        false
    }
}

#[cfg(target_os = "linux")]
fn system_is_dark() -> bool {
    // Future: detect GTK/dbus dark preference.
    false
}
