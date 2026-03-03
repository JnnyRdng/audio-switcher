use crate::settings::{Settings, Theme};
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
                // Handle global hotkeys before dispatching.
                if let Some(device_id) = state.hotkey_manager.handle_hotkey(&msg) {
                    super::switch_to_device(state, &device_id);
                    continue;
                }
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

/// Apply the dark/light theme to native context menus using undocumented uxtheme APIs.
/// SetPreferredAppMode (ordinal 135) controls whether Win32 menus render in dark mode.
/// FlushMenuThemes (ordinal 136) forces a repaint of cached menu visuals.
pub fn apply_theme(settings: &Settings) {
    use windows::core::{w, PCSTR};
    use windows::Win32::System::LibraryLoader::{
        GetProcAddress, LoadLibraryExW, LOAD_LIBRARY_SEARCH_SYSTEM32,
    };

    let mode: u32 = match settings.theme {
        Theme::System => 1, // AllowDark — follows the OS preference
        Theme::Dark => 2,   // ForceDark
        Theme::Light => 3,  // ForceLight
    };

    unsafe {
        let Ok(module) = LoadLibraryExW(w!("uxtheme.dll"), None, LOAD_LIBRARY_SEARCH_SYSTEM32)
        else {
            return;
        };

        // Ordinal 135: SetPreferredAppMode
        if let Some(func) = GetProcAddress(module, PCSTR::from_raw(135usize as *const u8)) {
            let set_mode: unsafe extern "system" fn(u32) -> u32 = std::mem::transmute(func);
            set_mode(mode);
        }

        // Ordinal 136: FlushMenuThemes
        if let Some(func) = GetProcAddress(module, PCSTR::from_raw(136usize as *const u8)) {
            let flush: unsafe extern "system" fn() = std::mem::transmute(func);
            flush();
        }
    }
}
