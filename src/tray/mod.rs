use crate::audio;
use crate::audio::Device;
use crate::hotkey::HotkeyManager;
use crate::settings::Settings;
use crate::settings_window;
use std::sync::{Arc, Mutex};
use tray_icon::menu::accelerator::Accelerator;
use tray_icon::menu::{CheckMenuItem, Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "windows")]
pub use self::windows::run_event_loop;
#[cfg(target_os = "linux")]
pub use self::linux::run_event_loop;

const EXIT_ID: &str = "exit";
const SETTINGS_ID: &str = "settings";

pub struct TrayState {
    pub tray_icon: TrayIcon,
    pub devices: Vec<Device>,
    pub settings: Arc<Mutex<Settings>>,
    pub hotkey_manager: HotkeyManager,
}

pub fn create(settings: Arc<Mutex<Settings>>) -> TrayState {
    apply_theme(&settings);

    let icon = load_icon();
    let devices = audio::list_devices();
    let s = settings.lock().unwrap();
    let default_id = audio::get_default_device_id();
    let menu = build_menu(&devices, default_id.as_deref(), &s);

    let mut hotkey_manager = HotkeyManager::new();
    hotkey_manager.register_all(&s);
    drop(s);

    let tray_icon = TrayIconBuilder::new()
        .with_tooltip("Audio Switcher")
        .with_icon(icon)
        .with_menu(Box::new(menu))
        .build()
        .expect("Failed to create tray icon");

    TrayState {
        tray_icon,
        devices,
        settings,
        hotkey_manager,
    }
}

/// Rebuild the menu with a fresh device list and update the tray icon.
pub fn refresh_menu(state: &mut TrayState) {
    apply_theme(&state.settings);
    state.devices = audio::list_devices();
    let s = state.settings.lock().unwrap();
    let default_id = audio::get_default_device_id();
    let menu = build_menu(&state.devices, default_id.as_deref(), &s);
    state.hotkey_manager.register_all(&s);
    drop(s);
    state.tray_icon.set_menu(Some(Box::new(menu)));
}

/// Handle a menu event. Returns true if the app should exit.
pub fn handle_menu_event(state: &mut TrayState, event: &MenuEvent) -> bool {
    let id = event.id().0.as_str();

    if id == EXIT_ID {
        return true;
    }

    if id == SETTINGS_ID {
        settings_window::open(state.settings.clone());
        return false;
    }

    // Otherwise it's a device ID.
    switch_to_device(state, id);
    false
}

/// Switch to the device with the given ID, show toast, and refresh menu.
pub fn switch_to_device(state: &mut TrayState, device_id: &str) {
    let device_name = state
        .devices
        .iter()
        .find(|d| d.id == device_id)
        .map(|d| d.name.clone());

    if let Some(ref name) = device_name {
        println!("Switching to: {name} [{device_id}]");
    }

    if let Err(e) = audio::set_default_device(device_id) {
        eprintln!("Failed to switch device: {e}");
        return;
    }

    let s = state.settings.lock().unwrap();

    // Play switch sound.
    if s.play_sound {
        play_switch_sound();
    }

    // Show toast notification.
    if let Some(name) = device_name {
        crate::toast::show(&name, &s);
    }

    drop(s);

    // Rebuild the menu so the checkmark moves to the new default.
    refresh_menu(state);
}

fn build_menu(devices: &[Device], default_id: Option<&str>, settings: &Settings) -> Menu {
    let menu = Menu::new();

    for device in devices {
        let checked = default_id == Some(device.id.as_str());
        let label = if let Some(shortcut) = settings.shortcuts.get(&device.id) {
            format!("{}\t{}", device.name, shortcut)
        } else {
            device.name.clone()
        };
        let _ = menu.append(&CheckMenuItem::with_id(
            MenuId::new(&device.id),
            &label,
            true,
            checked,
            None::<Accelerator>,
        ));
    }

    let _ = menu.append(&PredefinedMenuItem::separator());
    let _ = menu.append(&MenuItem::with_id(
        MenuId::new(SETTINGS_ID),
        "Settings",
        true,
        None::<Accelerator>,
    ));
    let _ = menu.append(&MenuItem::with_id(
        MenuId::new(EXIT_ID),
        "Exit",
        true,
        None::<Accelerator>,
    ));

    menu
}

/// Apply the context menu theme immediately. Safe to call from any thread.
pub fn apply_current_theme(settings: &Settings) {
    #[cfg(target_os = "windows")]
    windows::apply_theme(settings);
    #[cfg(target_os = "linux")]
    let _ = settings;
}

fn apply_theme(settings: &Arc<Mutex<Settings>>) {
    let s = settings.lock().unwrap();
    apply_current_theme(&s);
}

fn play_switch_sound() {
    #[cfg(target_os = "windows")]
    windows::play_switch_sound();
    #[cfg(target_os = "linux")]
    linux::play_switch_sound();
}

fn load_icon() -> Icon {
    let bytes= include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icon.ico"));
    let image = image::load_from_memory(bytes).unwrap().into_rgba8();
    let (width, height) = image.dimensions();
    Icon::from_rgba(image.into_raw(), width, height).unwrap()
}
