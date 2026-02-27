use crate::audio;
use crate::tray;

pub fn run() {
    audio::initialise();
    let mut state = tray::create();
    tray::run_event_loop(&mut state);
}
