use std::process::Command;
use std::sync::Once;
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
    static INIT: Once = Once::new();
    static mut WAV_PATH: Option<std::path::PathBuf> = None;

    // Write the WAV to a temp file once.
    INIT.call_once(|| {
        let path = std::env::temp_dir().join("audio-switcher-pop.wav");
        if std::fs::write(&path, WAV).is_ok() {
            unsafe {
                WAV_PATH = Some(path);
            }
        }
    });

    let path = unsafe { WAV_PATH.as_ref() };
    if let Some(path) = path {
        let _ = Command::new("paplay").arg(path).spawn();
    }
}
