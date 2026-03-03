use crate::audio::Device;
use std::process::Command;

pub fn initialise() {
    // No initialisation needed on Linux (no COM equivalent).
}

pub fn list_devices() -> Vec<Device> {
    let output = Command::new("pactl")
        .args(["-f", "json", "list", "sinks"])
        .output();

    let output = match output {
        Ok(o) if o.status.success() => o,
        _ => {
            eprintln!("Failed to run `pactl -f json list sinks`");
            return Vec::new();
        }
    };

    let json_str = match String::from_utf8(output.stdout) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    // pactl JSON output is an array of sink objects with "name" and "description".
    let sinks: Vec<serde_json::Value> = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to parse pactl JSON: {e}");
            return Vec::new();
        }
    };

    sinks
        .iter()
        .filter_map(|sink| {
            let name = sink.get("name")?.as_str()?.to_string();
            let description = sink
                .get("description")
                .and_then(|d| d.as_str())
                .unwrap_or(&name)
                .to_string();
            Some(Device {
                id: name,
                name: description,
            })
        })
        .collect()
}

pub fn get_default_device_id() -> Option<String> {
    let output = Command::new("pactl")
        .args(["get-default-sink"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let name = String::from_utf8(output.stdout).ok()?;
    let name = name.trim().to_string();
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

pub fn set_default_device(device_id: &str) -> Result<(), String> {
    let output = Command::new("pactl")
        .args(["set-default-sink", device_id])
        .output()
        .map_err(|e| format!("Failed to run pactl: {e}"))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("pactl set-default-sink failed: {stderr}"))
    }
}
