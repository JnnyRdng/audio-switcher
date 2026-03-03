pub fn centered_position(win_w: f32, win_h: f32) -> [f32; 2] {
    if let Some((sw, sh)) = screen_size() {
        return [(sw - win_w) / 2.0, (sh - win_h) / 2.0];
    }
    // Fallback: assume 1920x1080.
    [(1920.0 - win_w) / 2.0, (1080.0 - win_h) / 2.0]
}

fn screen_size() -> Option<(f32, f32)> {
    let output = std::process::Command::new("xrandr")
        .arg("--current")
        .output()
        .ok()?;
    let text = String::from_utf8(output.stdout).ok()?;
    for line in text.lines() {
        if line.contains(" connected") {
            for token in line.split_whitespace() {
                if let Some((res, _)) = token.split_once('+') {
                    if let Some((w, h)) = res.split_once('x') {
                        if let (Ok(w), Ok(h)) = (w.parse::<f32>(), h.parse::<f32>()) {
                            return Some((w, h));
                        }
                    }
                }
            }
        }
    }
    None
}
