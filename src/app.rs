// In app.rs

use crate::audio;
use crate::tray;
use tray_icon::TrayIconEvent;

pub fn build() {
    audio::initialise();
    let mut state = tray::create();
    run_event_loop(&mut state);
}

fn run_event_loop(state: &mut tray::TrayState) {
    use windows::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, GetMessageW, TranslateMessage, MSG, WM_COMMAND,
    };

    let mut msg = MSG::default();

    loop {
        // Check for tray icon events (right-click)
        if let Ok(event) = TrayIconEvent::receiver().try_recv() {
            // In 0.21, TrayIconEvent is an enum. Match on Click with Right button.
            // In 0.19, it's a struct — adjust the match to your version.
            // The key thing: on right-click, show the menu.
            if is_right_click(&event) {
                tray::show_context_menu(state);
            }
        }

        unsafe {
            if GetMessageW(&mut msg, None, 0, 0).as_bool() {
                // WM_COMMAND is sent by TrackPopupMenu when user clicks an item
                if msg.message == WM_COMMAND {
                    if tray::handle_menu_command(state, msg.wParam) {
                        println!("Exiting...");
                        break;
                    }
                }

                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            } else {
                break;
            }
        }
    }
}

// Adjust this to match your tray-icon version.
// For 0.21 (enum-based):
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

// For 0.19 (struct-based), you'd use something like:
// fn is_right_click(event: &TrayIconEvent) -> bool {
//     event.click_type == tray_icon::ClickType::Right
// }