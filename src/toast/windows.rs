use crate::settings::ToastPosition;
use crate::toast::{
    FADE_DURATION_MS, FADE_TICK_MS, MARGIN, TIMER_FADE_IN, TIMER_FADE_OUT, TIMER_HOLD,
    TOAST_HEIGHT, TOAST_WIDTH,
};
use std::sync::Mutex;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Dwm::{DWMWINDOWATTRIBUTE, DwmSetWindowAttribute};
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::{PCWSTR, w};

const CLASS_NAME: &str = "AudioSwitcherToast";

/// Track the current toast HWND so we can dismiss it when a new one appears.
static CURRENT_TOAST: Mutex<Option<isize>> = Mutex::new(None);

struct ToastData {
    text: Vec<u16>,
    dark: bool,
    target_alpha: u8,
    current_alpha: u8,
    fade_enabled: bool,
    fade_step: u8,
    duration_ms: u32,
}

pub fn show_toast(
    name: &str,
    duration_ms: u32,
    opacity: f32,
    fade: bool,
    position: ToastPosition,
    dark: bool,
) {
    // Dismiss any existing toast.
    if let Some(old) = CURRENT_TOAST.lock().unwrap().take() {
        let hwnd = HWND(old as *mut std::ffi::c_void);
        unsafe {
            let _ = PostMessageW(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0));
        }
    }

    unsafe {
        register_class();

        let (x, y) = calculate_position(position);
        let target_alpha = (opacity * 255.0) as u8;
        let fade_steps = (FADE_DURATION_MS / FADE_TICK_MS).max(1);
        let fade_step = (target_alpha as u32 / fade_steps).max(1) as u8;

        let text = format!("Switched to {name}");
        let text_wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();

        let data = Box::new(ToastData {
            text: text_wide,
            dark,
            target_alpha,
            current_alpha: if fade { 0 } else { target_alpha },
            fade_enabled: fade,
            fade_step,
            duration_ms,
        });
        let data_ptr = Box::into_raw(data);

        let instance: HINSTANCE = GetModuleHandleW(None).unwrap_or_default().into();
        let class: Vec<u16> = CLASS_NAME
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        let hwnd = CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            PCWSTR(class.as_ptr()),
            w!(""),
            WS_POPUP,
            x,
            y,
            TOAST_WIDTH,
            TOAST_HEIGHT,
            None,
            None,
            instance,
            None,
        )
        .unwrap_or_default();

        if hwnd.0.is_null() {
            drop(Box::from_raw(data_ptr));
            return;
        }

        SetWindowLongPtrW(hwnd, GWLP_USERDATA, data_ptr as isize);

        // Rounded corners (Windows 11+, silently fails on earlier).
        let preference: u32 = 2; // DWMWCP_ROUND
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWINDOWATTRIBUTE(33), // DWMWA_WINDOW_CORNER_PREFERENCE
            &preference as *const u32 as *const std::ffi::c_void,
            std::mem::size_of::<u32>() as u32,
        );

        // Set initial alpha and show.
        let initial_alpha = if fade { 0 } else { target_alpha };
        let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), initial_alpha, LWA_ALPHA);

        let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);

        // Store as current toast.
        *CURRENT_TOAST.lock().unwrap() = Some(hwnd.0 as isize);

        // Start timers.
        if fade {
            let _ = SetTimer(hwnd, TIMER_FADE_IN, FADE_TICK_MS, None);
        } else {
            let _ = SetTimer(hwnd, TIMER_HOLD, duration_ms, None);
        }

        // Run message loop until the toast is destroyed.
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

fn calculate_position(pos: ToastPosition) -> (i32, i32) {
    unsafe {
        // SPI_GETWORKAREA returns the usable desktop rect (excludes taskbar).
        let mut work = RECT::default();
        let _ = SystemParametersInfoW(SPI_GETWORKAREA, 0, Some(&mut work as *mut RECT as *mut _), SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0));

        let left = work.left;
        let top = work.top;
        let w = work.right - work.left;
        let h = work.bottom - work.top;

        match pos {
            ToastPosition::TopLeft => (left + MARGIN, top + MARGIN),
            ToastPosition::TopCenter => (left + (w - TOAST_WIDTH) / 2, top + MARGIN),
            ToastPosition::TopRight => (left + w - TOAST_WIDTH - MARGIN, top + MARGIN),
            ToastPosition::BottomLeft => (left + MARGIN, top + h - TOAST_HEIGHT - MARGIN),
            ToastPosition::BottomCenter => (
                left + (w - TOAST_WIDTH) / 2,
                top + h - TOAST_HEIGHT - MARGIN,
            ),
            ToastPosition::BottomRight => (
                left + w - TOAST_WIDTH - MARGIN,
                top + h - TOAST_HEIGHT - MARGIN,
            ),
        }
    }
}

fn register_class() {
    use std::sync::Once;
    static ONCE: Once = Once::new();

    ONCE.call_once(|| unsafe {
        let instance: HINSTANCE = GetModuleHandleW(None).unwrap_or_default().into();
        let class: Vec<u16> = CLASS_NAME
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        let wc = WNDCLASSW {
            lpfnWndProc: Some(wndproc),
            hInstance: instance,
            lpszClassName: PCWSTR(class.as_ptr()),
            hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
            hbrBackground: HBRUSH::default(),
            ..Default::default()
        };
        RegisterClassW(&wc);
    });
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        let data_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut ToastData;

        // Before GWLP_USERDATA is set, delegate to default.
        if data_ptr.is_null() {
            return DefWindowProcW(hwnd, msg, wparam, lparam);
        }

        let data = &mut *data_ptr;

        match msg {
            WM_PAINT => {
                paint(hwnd, data);
                LRESULT(0)
            }
            WM_TIMER => {
                handle_timer(hwnd, data, wparam.0);
                LRESULT(0)
            }
            WM_DESTROY => {
                // Remove from global tracker.
                let mut current = CURRENT_TOAST.lock().unwrap();
                if *current == Some(hwnd.0 as isize) {
                    *current = None;
                }
                drop(current);

                // Free ToastData.
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                drop(Box::from_raw(data_ptr));

                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

unsafe fn paint(hwnd: HWND, data: &ToastData) {
    unsafe {
        let mut ps = PAINTSTRUCT::default();
        let hdc = BeginPaint(hwnd, &mut ps);

        let mut rect = RECT::default();
        let _ = GetClientRect(hwnd, &mut rect);

        // Background.
        let bg = if data.dark {
            COLORREF(0x00282828) // dark grey
        } else {
            COLORREF(0x00F5F5F5) // light grey
        };
        let brush = CreateSolidBrush(bg);
        let _ = FillRect(hdc, &rect, brush);
        let _ = DeleteObject(brush);

        // Text.
        let fg = if data.dark {
            COLORREF(0x00F0F0F0)
        } else {
            COLORREF(0x001E1E1E)
        };
        SetTextColor(hdc, fg);
        SetBkMode(hdc, TRANSPARENT);

        let font_name: Vec<u16> = "Segoe UI"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let mut face_name = [0u16; 32];
        let copy_len = font_name.len().min(32);
        face_name[..copy_len].copy_from_slice(&font_name[..copy_len]);

        let font = CreateFontW(
            18,                                     // height
            0,                                      // width (auto)
            0,                                      // escapement
            0,                                      // orientation
            FW_NORMAL.0 as i32,                     // weight
            0,                                      // italic
            0,                                      // underline
            0,                                      // strikeout
            DEFAULT_CHARSET.0 as u32,               // charset
            OUT_DEFAULT_PRECIS.0 as u32,            // out precision
            CLIP_DEFAULT_PRECIS.0 as u32,           // clip precision
            CLEARTYPE_QUALITY.0 as u32,             // quality
            (FF_SWISS.0 | VARIABLE_PITCH.0) as u32, // pitch and family
            PCWSTR(face_name.as_ptr()),
        );

        let old_font = SelectObject(hdc, font);
        let _ = DrawTextW(
            hdc,
            &mut data.text.clone(),
            &mut rect,
            DT_CENTER | DT_VCENTER | DT_SINGLELINE,
        );

        SelectObject(hdc, old_font);
        let _ = DeleteObject(font);
        let _ = EndPaint(hwnd, &ps);
    }
}

unsafe fn handle_timer(hwnd: HWND, data: &mut ToastData, timer_id: usize) {
    unsafe {
        match timer_id {
            TIMER_FADE_IN => {
                data.current_alpha = data
                    .current_alpha
                    .saturating_add(data.fade_step)
                    .min(data.target_alpha);
                let _ =
                    SetLayeredWindowAttributes(hwnd, COLORREF(0), data.current_alpha, LWA_ALPHA);

                if data.current_alpha >= data.target_alpha {
                    let _ = KillTimer(hwnd, TIMER_FADE_IN);
                    let _ = SetTimer(hwnd, TIMER_HOLD, data.duration_ms, None);
                }
            }
            TIMER_HOLD => {
                let _ = KillTimer(hwnd, TIMER_HOLD);
                if data.fade_enabled {
                    let _ = SetTimer(hwnd, TIMER_FADE_OUT, FADE_TICK_MS, None);
                } else {
                    let _ = DestroyWindow(hwnd);
                }
            }
            TIMER_FADE_OUT => {
                data.current_alpha = data.current_alpha.saturating_sub(data.fade_step);
                let _ =
                    SetLayeredWindowAttributes(hwnd, COLORREF(0), data.current_alpha, LWA_ALPHA);

                if data.current_alpha == 0 {
                    let _ = KillTimer(hwnd, TIMER_FADE_OUT);
                    let _ = DestroyWindow(hwnd);
                }
            }
            _ => {}
        }
    }
}
