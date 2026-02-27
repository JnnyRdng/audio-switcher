use tray_icon::menu::MenuEvent;
use tray_icon::TrayIconEvent;

pub fn run_event_loop(state: &mut super::TrayState) {
    use windows::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, GetMessageW, TranslateMessage, MSG,
    };

    let mut msg = MSG::default();

    loop {
        if let Ok(event) = MenuEvent::receiver().try_recv() {
            if super::handle_menu_event(state, &event) {
                println!("Exiting...");
                break;
            }
        }

        // On right-click, refresh the menu so it's up to date for next open.
        if let Ok(event) = TrayIconEvent::receiver().try_recv() {
            if is_right_click(&event) {
                super::refresh_menu(state);
            }
        }

        unsafe {
            if GetMessageW(&mut msg, None, 0, 0).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            } else {
                break;
            }
        }
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
