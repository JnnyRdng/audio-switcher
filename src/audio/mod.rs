#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "linux")]
mod linux;

pub struct Device {
    pub id: String,
    pub name: String,
    pub label: String,
}

#[cfg(target_os = "windows")]
pub use self::windows::{initialise, list_devices};

#[cfg(target_os = "linux")]
pub use self::linux::{initialise, list_devices};
