use crate::audio::Device;
use std::ffi::c_void;
use windows::core::{GUID, HRESULT, Interface, PCWSTR};
use windows::Win32::Devices::FunctionDiscovery::{PKEY_Device_DeviceDesc, PKEY_Device_FriendlyName};
use windows::Win32::Media::Audio::{
    eConsole, eRender, DEVICE_STATE_ACTIVE, IMMDeviceEnumerator, MMDeviceEnumerator,
};
use windows::Win32::System::Com::STGM;
use windows::Win32::System::Com::{
    CLSCTX_ALL, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
};

pub fn initialise() {
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED)
            .ok()
            .expect("Failed to initialize COM");
    }
}

pub fn list_devices() -> Vec<Device> {
    unsafe {
        enumerate_devices().unwrap_or_else(|e| {
            eprintln!("Failed to enumerate devices: {}", e);
            Vec::new()
        })
    }
}

pub fn get_default_device_id() -> Option<String> {
    unsafe {
        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).ok()?;
        let device = enumerator
            .GetDefaultAudioEndpoint(eRender, eConsole)
            .ok()?;
        let id_raw = device.GetId().ok()?;
        Some(id_raw.to_string().ok()?)
    }
}

pub fn set_default_device(device_id: &str) -> Result<(), String> {
    unsafe { set_default_endpoint(device_id).map_err(|e| format!("{}", e)) }
}

// ---------------------------------------------------------------------------
// IPolicyConfig — undocumented COM interface for changing the default audio
// device on Windows. We use raw vtable access since the interface isn't in the
// Windows SDK.
// ---------------------------------------------------------------------------

const CLSID_POLICY_CONFIG: GUID =
    GUID::from_u128(0x870af99c_171d_4f9e_af0d_e63df40c2bc9);

const IID_IPOLICY_CONFIG: GUID =
    GUID::from_u128(0xf8679f50_850a_41cf_9c72_430f290290c8);

/// SetDefaultEndpoint sits at vtable index 13:
/// 3 IUnknown methods + 10 IPolicyConfig methods before it.
const SET_DEFAULT_ENDPOINT_INDEX: usize = 13;

type SetDefaultEndpointFn =
    unsafe extern "system" fn(this: *mut c_void, device_id: PCWSTR, role: u32) -> HRESULT;

type ReleaseFn = unsafe extern "system" fn(this: *mut c_void) -> u32;

type QueryInterfaceFn =
    unsafe extern "system" fn(this: *mut c_void, iid: *const GUID, out: *mut *mut c_void) -> HRESULT;

unsafe fn set_default_endpoint(device_id: &str) -> windows::core::Result<()> {
    unsafe {
        // Create the PolicyConfig COM object (via IUnknown first)
        let unknown: windows::core::IUnknown =
            CoCreateInstance(&CLSID_POLICY_CONFIG, None, CLSCTX_ALL)?;

        // QueryInterface for IPolicyConfig
        let raw = unknown.as_raw();
        let vtable = *(raw as *const *const usize);
        let query_interface: QueryInterfaceFn = std::mem::transmute(*vtable.add(0));

        let mut config_ptr: *mut c_void = std::ptr::null_mut();
        query_interface(raw, &IID_IPOLICY_CONFIG, &mut config_ptr).ok()?;

        // Get SetDefaultEndpoint from the vtable
        let config_vtable = *(config_ptr as *const *const usize);
        let set_default: SetDefaultEndpointFn =
            std::mem::transmute(*config_vtable.add(SET_DEFAULT_ENDPOINT_INDEX));

        let wide: Vec<u16> = device_id.encode_utf16().chain(std::iter::once(0)).collect();
        let pcwstr = PCWSTR(wide.as_ptr());

        // Set for all three roles so the device becomes the system default everywhere.
        set_default(config_ptr, pcwstr, 0).ok()?; // eConsole
        set_default(config_ptr, pcwstr, 1).ok()?; // eMultimedia
        set_default(config_ptr, pcwstr, 2).ok()?; // eCommunications

        // Release our reference to IPolicyConfig
        let release: ReleaseFn = std::mem::transmute(*config_vtable.add(2));
        release(config_ptr);

        Ok(())
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
            })
        }

        Ok(devices)
    }
}
