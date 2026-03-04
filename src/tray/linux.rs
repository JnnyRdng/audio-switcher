use std::process::Command;
use tray_icon::menu::MenuEvent;

pub fn run_event_loop(state: &mut super::TrayState) {
    // On Linux, tray-icon requires a GTK main loop for the icon to appear.
    // For now, poll MenuEvent in a simple loop.
    // Future: integrate with gtk::main() via glib::timeout_add.

    loop {
        std::thread::sleep(std::time::Duration::from_millis(100));
        if let Ok(event) = MenuEvent::receiver().try_recv() {
            if super::handle_menu_event(state, &event) {
                break;
            }
        }
    }
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
