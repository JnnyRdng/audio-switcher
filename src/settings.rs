use serde::{Deserialize, Serialize};
use std::{fmt, fs};
use std::path::PathBuf;
use strum_macros::EnumIter;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Theme {
    System,
    Dark,
    Light,
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
            toast_position: ToastPosition::BottomCenter
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
