use windows::Win32::Foundation::{BOOL, RECT};
use windows::Win32::Graphics::Dwm::{DWMWINDOWATTRIBUTE, DwmSetWindowAttribute};
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowW, SPI_GETWORKAREA, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, SetForegroundWindow,
    SystemParametersInfoW,
};
use windows::core::w;

pub fn centered_position(win_w: f32, win_h: f32) -> [f32; 2] {
    unsafe {
        let mut work = RECT::default();
        let _ = SystemParametersInfoW(
            SPI_GETWORKAREA,
            0,
            Some(&mut work as *mut RECT as *mut _),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        );

        let left = work.left as f32;
        let top = work.top as f32;
        let w = (work.right - work.left) as f32;
        let h = (work.bottom - work.top) as f32;

        [(left + w - win_w) / 2.0, (top + h - win_h) / 2.0]
    }
}

pub fn bring_to_front() {
    unsafe {
        if let Ok(hwnd) = FindWindowW(None, w!("Audio Switcher Settings")) {
            let _ = SetForegroundWindow(hwnd);
        }
    }
}

pub fn set_title_bar_dark(dark: bool) {
    unsafe {
        let Ok(hwnd) = FindWindowW(None, w!("Audio Switcher Settings")) else {
            return;
        };
        let value = BOOL::from(dark);
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWINDOWATTRIBUTE(20),
            &value as *const BOOL as *const std::ffi::c_void,
            std::mem::size_of::<BOOL>() as u32,
        );
    }
}
