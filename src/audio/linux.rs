use crate::audio::Device;

pub fn initialise() {
    // No-op on Linux for now.
    // Future: initialise PulseAudio/PipeWire connection.
}

pub fn list_devices() -> Vec<Device> {
    // Future: enumerate via PulseAudio or PipeWire.
    eprintln!("Audio device enumeration not yet implemented on Linux.");
    Vec::new()
}

pub fn get_default_device_id() -> Option<String> {
    // Future: query PulseAudio/PipeWire for the default sink.
    None
}

pub fn set_default_device(_device_id: &str) -> Result<(), String> {
    Err("Audio device switching not yet implemented on Linux.".into())
}
