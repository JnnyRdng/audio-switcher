use tray_icon::menu::MenuId;
use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_DeviceDesc;
use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;
use windows::Win32::Media::Audio::{
    DEVICE_STATE_ACTIVE, IMMDeviceEnumerator, IMMNotificationClient, MMDeviceEnumerator, eRender,
};
use windows::Win32::System::Com::STGM;
use windows::Win32::System::Com::{
    CLSCTX_ALL, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx, CoUninitialize,
};

pub struct Device {
    pub id: String,
    pub name: String,
    pub label: String,
    pub menu_id: Option<MenuId>,
}

pub fn initialise() {
    init_com();
}

/// Initialise COM. Call this once at app startup, before any audio calls.
/// COM is aparatment-threaded here because our message loop is single-threaded.
fn init_com() {
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED)
            .ok()
            .expect("Failed to initialize COM");
    }
}

pub fn list_devices() -> Vec<Device> {
    unsafe {
        enumerate_devices().unwrap_or_else(|e| {
            eprintln!("Failed to enumerate Devices: {}", e);
            Vec::new()
        })
    }
}

unsafe fn enumerate_devices() -> windows::core::Result<Vec<Device>> {
    unsafe {
        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
        let collection = enumerator.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)?;

        let count = collection.GetCount()?;
        let mut devices = Vec::with_capacity(count as usize);

        for i in 0..count {
            let device = collection.Item(i)?;
            let id_raw = device.GetId()?;
            let id = id_raw.to_string()?;

            let store = device.OpenPropertyStore(STGM(0))?;

            let name_prop = store.GetValue(&PKEY_Device_FriendlyName)?;
            let name = name_prop.to_string();

            let desc_prop = store.GetValue(&PKEY_Device_DeviceDesc)?;
            let desc = desc_prop.to_string();

            let name = if name.is_empty() { id.clone() } else { name };
            devices.push(Device {
                id,
                name,
                label: desc,
                menu_id: None,
            })
        }

        Ok(devices)
    }
}
