// Tray icon + manual popup menu using Win32 directly.
//
// We use the tray-icon crate ONLY for the icon itself (no menu).
// On right-click, we enumerate devices fresh, build a Win32 popup menu,
// and show it with TrackPopupMenu. This gives us full control over timing.

use crate::audio;
use crate::audio::Device;
use super::create_placeholder_icon;
use tray_icon::{TrayIconBuilder, TrayIcon, TrayIconEvent};
use windows::Win32::Foundation::{HWND, WPARAM, LPARAM, LRESULT};
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::PCWSTR;

const EXIT_CMD_ID: u16 = 9999;
const DEVICE_CMD_BASE: u16 = 1000; // device 0 = 1000, device 1 = 1001, etc.

pub struct TrayState {
    pub tray_icon: TrayIcon,
    pub hwnd: HWND, // our hidden window for menu messages
    pub devices: Vec<Device>,
}

pub fn create() -> TrayState {
    let icon = create_placeholder_icon();
    let tray_icon = TrayIconBuilder::new()
        .with_tooltip("Audio Switcher")
        .with_icon(icon)
        // NO menu — we handle it ourselves
        .build()
        .expect("Failed to create tray icon");

    let hwnd = create_hidden_window();

    TrayState {
        tray_icon,
        hwnd,
        devices: Vec::new(),
    }
}

pub fn run_event_loop(state: &mut TrayState) {
    use windows::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, GetMessageW, TranslateMessage, MSG, WM_COMMAND,
    };

    let mut msg = MSG::default();

    loop {
        // Check for tray icon events (right-click)
        if let Ok(event) = TrayIconEvent::receiver().try_recv() {
            if is_right_click(&event) {
                show_context_menu(state);
            }
        }

        unsafe {
            if GetMessageW(&mut msg, None, 0, 0).as_bool() {
                // WM_COMMAND is sent by TrackPopupMenu when user clicks an item
                if msg.message == WM_COMMAND {
                    if handle_menu_command(state, msg.wParam) {
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

/// Show a fresh popup menu at the cursor position.
/// Called when the user right-clicks the tray icon.
fn show_context_menu(state: &mut TrayState) {
    state.devices = audio::list_devices();

    unsafe {
        let hmenu = CreatePopupMenu().expect("Failed to create popup menu");

        for (i, device) in state.devices.iter().enumerate() {
            let label: Vec<u16> = device.name.encode_utf16().chain(std::iter::once(0)).collect();
            AppendMenuW(
                hmenu,
                MF_STRING,
                (DEVICE_CMD_BASE + i as u16) as usize,
                PCWSTR(label.as_ptr()),
            ).ok();
        }

        // Separator
        AppendMenuW(hmenu, MF_SEPARATOR, 0, PCWSTR::null()).ok();

        // Exit
        let exit_label: Vec<u16> = "Exit".encode_utf16().chain(std::iter::once(0)).collect();
        AppendMenuW(
            hmenu,
            MF_STRING,
            EXIT_CMD_ID as usize,
            PCWSTR(exit_label.as_ptr()),
        ).ok();

        // Get cursor position for menu placement
        let mut pt = Default::default();
        GetCursorPos(&mut pt).ok();

        let mut work_area: windows::Win32::Foundation::RECT = Default::default();
        SystemParametersInfoW(
            SPI_GETWORKAREA,
            0,
            Some(&mut work_area as *mut _ as *mut _),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        ).expect("Could not get work area");

        // Required: set our window as foreground so the menu dismisses properly
        let _ = SetForegroundWindow(state.hwnd);

        // Show the menu. This BLOCKS until the user picks an item or dismisses.
        let _ = TrackPopupMenu(
            hmenu,
            TPM_BOTTOMALIGN | TPM_LEFTALIGN,
            pt.x,
            work_area.bottom - 4,
            0,
            state.hwnd,
            None,
        );

        // Clean up
        DestroyMenu(hmenu).ok();
    }
}

/// Handle WM_COMMAND from the popup menu. Returns true if we should exit.
fn handle_menu_command(state: &TrayState, wparam: WPARAM) -> bool {
    let cmd = (wparam.0 & 0xFFFF) as u16;

    if cmd == EXIT_CMD_ID {
        return true;
    }

    if cmd >= DEVICE_CMD_BASE {
        let idx = (cmd - DEVICE_CMD_BASE) as usize;
        if let Some(device) = state.devices.get(idx) {
            println!("Switched to: {} [{}]", device.name, device.id);
            // TODO: actually switch the device
        }
    }

    false
}

/// Create a hidden message-only window to receive WM_COMMAND from TrackPopupMenu.
fn create_hidden_window() -> HWND {
    unsafe {
        let class_name: Vec<u16> = "AudioSwitcherHidden\0".encode_utf16().collect();

        let wc = WNDCLASSW {
            lpfnWndProc: Some(hidden_wnd_proc),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };
        RegisterClassW(&wc);

        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR::null(),
            WINDOW_STYLE::default(),
            0, 0, 0, 0,
            HWND_MESSAGE, // message-only window, never visible
            None,
            None,
            None,
        ).expect("Failed to create hidden window")
    }
}

unsafe extern "system" fn hidden_wnd_proc(
    hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM,
) -> LRESULT {
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}
