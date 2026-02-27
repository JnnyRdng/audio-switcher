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
