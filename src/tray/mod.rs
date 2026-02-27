use crate::audio;
use crate::audio::Device;
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

pub struct TrayState {
    pub tray_icon: TrayIcon,
    pub devices: Vec<Device>,
}

pub fn create() -> TrayState {
    let icon = create_placeholder_icon();
    let devices = audio::list_devices();
    let default_id = audio::get_default_device_id();
    let menu = build_menu(&devices, default_id.as_deref());

    let tray_icon = TrayIconBuilder::new()
        .with_tooltip("Audio Switcher")
        .with_icon(icon)
        .with_menu(Box::new(menu))
        .build()
        .expect("Failed to create tray icon");

    TrayState { tray_icon, devices }
}

/// Rebuild the menu with a fresh device list and update the tray icon.
pub fn refresh_menu(state: &mut TrayState) {
    state.devices = audio::list_devices();
    let default_id = audio::get_default_device_id();
    let menu = build_menu(&state.devices, default_id.as_deref());
    state.tray_icon.set_menu(Some(Box::new(menu)));
}

/// Handle a menu event. Returns true if the app should exit.
pub fn handle_menu_event(state: &mut TrayState, event: &MenuEvent) -> bool {
    if *event.id() == MenuId::new(EXIT_ID) {
        return true;
    }

    // The menu item ID is the device ID.
    let device_id = event.id().0.as_str();
    if let Some(device) = state.devices.iter().find(|d| d.id == device_id) {
        println!("Switching to: {} [{}]", device.name, device.id);
    }

    if let Err(e) = audio::set_default_device(device_id) {
        eprintln!("Failed to switch device: {}", e);
        return false;
    }

    // Rebuild the menu so the checkmark moves to the new default.
    refresh_menu(state);
    false
}

fn build_menu(devices: &[Device], default_id: Option<&str>) -> Menu {
    let menu = Menu::new();

    for device in devices {
        let checked = default_id == Some(device.id.as_str());
        let _ = menu.append(&CheckMenuItem::with_id(
            MenuId::new(&device.id),
            &device.name,
            true,
            checked,
            None::<Accelerator>,
        ));
    }

    let _ = menu.append(&PredefinedMenuItem::separator());
    let _ = menu.append(&MenuItem::with_id(
        MenuId::new(EXIT_ID),
        "Exit",
        true,
        None::<Accelerator>,
    ));

    menu
}

fn create_placeholder_icon() -> Icon {
    let size = 32u32;
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let dx = (x as f32 - 15.5).abs();
            let dy = (y as f32 - 15.5).abs();
            let dist = dx.max(dy);
            if dist < 14.0 {
                let t = dist / 14.0;
                let r = (30.0 + t * 20.0) as u8;
                let g = (180.0 - t * 40.0) as u8;
                let b = (170.0 - t * 30.0) as u8;
                rgba.extend_from_slice(&[r, g, b, 255]);
            } else if dist < 15.0 {
                rgba.extend_from_slice(&[20, 140, 130, 180]);
            } else {
                rgba.extend_from_slice(&[0, 0, 0, 0]);
            }
        }
    }
    Icon::from_rgba(rgba, size, size).expect("Failed to create icon")
}
