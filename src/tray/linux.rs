use crate::audio;
use crate::audio::Device;
use super::create_placeholder_icon;
use tray_icon::menu::{Menu, MenuItem, MenuEvent, PredefinedMenuItem};
use tray_icon::{TrayIconBuilder, TrayIcon};

pub struct TrayState {
    pub tray_icon: TrayIcon,
    pub menu: Menu,
    pub devices: Vec<Device>,
}

pub fn create() -> TrayState {
    let icon = create_placeholder_icon();
    let menu = Menu::new();

    // On Linux, the menu must be attached to the tray icon — TrayIconEvent is not emitted.
    // The desktop environment (via libappindicator) shows the menu automatically on click.
    let tray_icon = TrayIconBuilder::new()
        .with_tooltip("Audio Switcher")
        .with_icon(icon)
        .with_menu(Box::new(menu.clone()))
        .build()
        .expect("Failed to create tray icon");

    TrayState {
        tray_icon,
        menu,
        devices: Vec::new(),
    }
}

pub fn run_event_loop(state: &mut TrayState) {
    // On Linux, tray-icon requires a running GTK main loop.
    // For now this is a stub that polls MenuEvent.

    eprintln!("Linux tray event loop is not yet fully implemented.");
    eprintln!("The tray icon should be visible. Press Ctrl+C to exit.");

    // Future: call gtk::init() and gtk::main() here, with a glib
    // idle_add or timeout_add to poll MenuEvent::receiver().

    loop {
        std::thread::sleep(std::time::Duration::from_millis(100));
        if let Ok(event) = MenuEvent::receiver().try_recv() {
            println!("Menu event: {:?}", event);
        }
    }
}
