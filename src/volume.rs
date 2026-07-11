use std::process::Command;

pub struct PipeWireVolume;

impl PipeWireVolume {
    /// Read current system volume via `wpctl get-volume`.
    /// Returns a value between 0.0 and 1.0.
    pub fn read() -> f32 {
        let output = Command::new("wpctl")
            .args(["get-volume", "@DEFAULT_AUDIO_SINK@"])
            .output();

        match output {
            Ok(out) if out.status.success() => {
                let s = String::from_utf8_lossy(&out.stdout);
                // Output format: "Volume: 0.70\n"
                if let Some(val_str) = s.trim().strip_prefix("Volume: ") {
                    if let Ok(val) = val_str.trim().parse::<f32>() {
                        return val.clamp(0.0, 1.0);
                    }
                }
                0.7 // fallback
            }
            _ => 0.7,
        }
    }

    /// Set system volume via `wpctl set-volume`.
    pub fn set(vol: f32) {
        let vol = vol.clamp(0.0, 1.0);
        let _ = Command::new("wpctl")
            .args([
                "set-volume",
                "@DEFAULT_AUDIO_SINK@",
                &format!("{:.2}", vol),
            ])
            .output();
    }
}
