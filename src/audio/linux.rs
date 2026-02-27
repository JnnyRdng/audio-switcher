use crate::audio::Device;

pub fn initialise() {
    // No-op on Linux for now.
    // Future: initialise PulseAudio/PipeWire connection.
}

pub fn list_devices() -> Vec<Device> {
    // Stub: return empty list.
    // Future: enumerate via PulseAudio or PipeWire.
    eprintln!("Audio device enumeration not yet implemented on Linux.");
    Vec::new()
}
