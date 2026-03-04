use crate::audio;
use crate::settings::Settings;
use crate::tray;
use std::sync::{Arc, Mutex};

pub fn run() {
    #[cfg(target_os = "linux")]
    gtk::init().expect("Failed to initialize GTK");

    audio::initialise();
    let settings = Arc::new(Mutex::new(Settings::load()));
    let mut state = tray::create(settings);
    tray::run_event_loop(&mut state);
}
