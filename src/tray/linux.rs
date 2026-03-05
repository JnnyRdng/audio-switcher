use std::process::Command;
use tray_icon::TrayIconEvent;
use tray_icon::menu::MenuEvent;

pub fn run_event_loop(state: &mut super::TrayState) {
    loop {
        // Process all pending GTK events (required for the tray icon to work on Linux).
        while gtk::events_pending() {
            gtk::main_iteration_do(false);
        }

        if let Ok(event) = MenuEvent::receiver().try_recv() {
            if super::handle_menu_event(state, &event) {
                break;
            }
        }

        // On right-click, refresh the menu so it's up to date for next open.
        if let Ok(event) = TrayIconEvent::receiver().try_recv() {
            if is_right_click(&event) {
                super::refresh_menu(state);
            }
        }

        if let Some(device_id) = state.hotkey_manager.check_event() {
            super::switch_to_device(state, &device_id);
        }

        if crate::settings_window::take_settings_changed() {
            super::refresh_menu(state);
        }

        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

fn is_right_click(event: &TrayIconEvent) -> bool {
    matches!(
        event,
        TrayIconEvent::Click {
            button: tray_icon::MouseButton::Right,
            button_state: tray_icon::MouseButtonState::Up,
            ..
        }
    )
}

pub fn play_switch_sound() {
    static WAV: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/ui-pop.wav"));
    static WAV_PATH: std::sync::OnceLock<Option<std::path::PathBuf>> = std::sync::OnceLock::new();

    let path = WAV_PATH.get_or_init(|| {
        let path = std::env::temp_dir().join("audio-switcher-pop.wav");
        std::fs::write(&path, WAV).ok().map(|_| path)
    });

    if let Some(path) = path {
        let _ = Command::new("paplay").arg(path).spawn();
    }
}
