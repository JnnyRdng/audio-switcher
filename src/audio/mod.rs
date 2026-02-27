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
pub use self::windows::{get_default_device_id, initialise, list_devices, set_default_device};

#[cfg(target_os = "linux")]
pub use self::linux::{get_default_device_id, initialise, list_devices, set_default_device};
