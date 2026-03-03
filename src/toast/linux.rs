use crate::settings::ToastPosition;

pub fn show_toast(
    _name: &str,
    _duration_ms: u32,
    _opacity: f32,
    _fade: bool,
    _position: ToastPosition,
    _dark: bool,
) {
    eprintln!("Toast notifications not yet implemented on Linux");
}
