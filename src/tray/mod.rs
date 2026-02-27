use tray_icon::Icon;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "windows")]
pub use self::windows::{TrayState, create, run_event_loop};

#[cfg(target_os = "linux")]
pub use self::linux::{TrayState, create, run_event_loop};

/// Create a placeholder 32x32 tray icon. Platform-agnostic.
pub fn create_placeholder_icon() -> Icon {
    let size = 32u32;
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let dx = (x as f32 - 15.5).abs();
            let dy = (y as f32 - 15.5).abs();
            let dist = dx.max(dy);
            if dist < 14.0 {
                let t = dist / 14.0;
                let r = (30.0 + t * 20.0) as u8;
                let g = (180.0 - t * 40.0) as u8;
                let b = (170.0 - t * 30.0) as u8;
                rgba.extend_from_slice(&[r, g, b, 255]);
            } else if dist < 15.0 {
                rgba.extend_from_slice(&[20, 140, 130, 180]);
            } else {
                rgba.extend_from_slice(&[0, 0, 0, 0]);
            }
        }
    }
    Icon::from_rgba(rgba, size, size).expect("Failed to create icon")
}
