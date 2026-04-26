use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredProfile {
    pub device_id: u32,
    pub device_name: String,
    pub throttle: StoredBinding,
    pub brake: StoredBinding,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredBinding {
    pub axis_index: usize,
    pub axis_label: String,
    pub idle_raw: u32,
    pub active_raw: u32,
    pub calibration: StoredCalibration,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StoredCalibration {
    DriverRange,
    CustomRange { idle_raw: u32, active_raw: u32 },
}

pub fn load_profile() -> Option<StoredProfile> {
    let content = fs::read_to_string(profile_path()).ok()?;
    parse_profile(&content)
}

pub fn save_profile(profile: &StoredProfile) -> io::Result<()> {
    let path = profile_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serialize_profile(profile))
}

pub fn profile_signature(profile: &StoredProfile) -> String {
    serialize_profile(profile)
}

fn profile_path() -> PathBuf {
    let base = env::var_os("APPDATA")
        .or_else(|| env::var_os("LOCALAPPDATA"))
        .map(PathBuf::from)
        .or_else(|| env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."));

    base.join("ApexFootwork").join("profile.txt")
}

fn serialize_profile(profile: &StoredProfile) -> String {
    let mut lines = Vec::new();
    lines.push("version=1".to_string());
    lines.push(format!("device_id={}", profile.device_id));
    lines.push(format!("device_name={}", encode_text(&profile.device_name)));
    push_binding(&mut lines, "throttle", &profile.throttle);
    push_binding(&mut lines, "brake", &profile.brake);
    lines.join("\n")
}

fn push_binding(lines: &mut Vec<String>, prefix: &str, binding: &StoredBinding) {
    lines.push(format!("{}_axis_index={}", prefix, binding.axis_index));
    lines.push(format!(
        "{}_axis_label={}",
        prefix,
        encode_text(&binding.axis_label)
    ));
    lines.push(format!("{}_idle_raw={}", prefix, binding.idle_raw));
    lines.push(format!("{}_active_raw={}", prefix, binding.active_raw));
    match &binding.calibration {
        StoredCalibration::DriverRange => {
            lines.push(format!("{}_calibration=driver_range", prefix));
        }
        StoredCalibration::CustomRange {
            idle_raw,
            active_raw,
        } => {
            lines.push(format!("{}_calibration=custom_range", prefix));
            lines.push(format!("{}_calibration_idle_raw={}", prefix, idle_raw));
            lines.push(format!("{}_calibration_active_raw={}", prefix, active_raw));
        }
    }
}

fn parse_profile(content: &str) -> Option<StoredProfile> {
    let values = content
        .lines()
        .filter_map(|line| line.split_once('='))
        .map(|(key, value)| (key.trim().to_string(), value.trim().to_string()))
        .collect::<HashMap<_, _>>();

    if values.get("version")? != "1" {
        return None;
    }

    Some(StoredProfile {
        device_id: values.get("device_id")?.parse().ok()?,
        device_name: decode_text(values.get("device_name")?)?,
        throttle: parse_binding(&values, "throttle")?,
        brake: parse_binding(&values, "brake")?,
    })
}

fn parse_binding(values: &HashMap<String, String>, prefix: &str) -> Option<StoredBinding> {
    Some(StoredBinding {
        axis_index: values
            .get(&format!("{}_axis_index", prefix))?
            .parse()
            .ok()?,
        axis_label: decode_text(values.get(&format!("{}_axis_label", prefix))?)?,
        idle_raw: values.get(&format!("{}_idle_raw", prefix))?.parse().ok()?,
        active_raw: values
            .get(&format!("{}_active_raw", prefix))?
            .parse()
            .ok()?,
        calibration: parse_calibration(values, prefix)?,
    })
}

fn parse_calibration(values: &HashMap<String, String>, prefix: &str) -> Option<StoredCalibration> {
    match values
        .get(&format!("{}_calibration", prefix))
        .map(String::as_str)
        .unwrap_or("driver_range")
    {
        "driver_range" => Some(StoredCalibration::DriverRange),
        "custom_range" => Some(StoredCalibration::CustomRange {
            idle_raw: values
                .get(&format!("{}_calibration_idle_raw", prefix))?
                .parse()
                .ok()?,
            active_raw: values
                .get(&format!("{}_calibration_active_raw", prefix))?
                .parse()
                .ok()?,
        }),
        _ => None,
    }
}

fn encode_text(value: &str) -> String {
    value
        .as_bytes()
        .iter()
        .map(|byte| format!("{:02X}", byte))
        .collect()
}

fn decode_text(value: &str) -> Option<String> {
    if value.len() % 2 != 0 {
        return None;
    }

    let mut bytes = Vec::new();
    for chunk in value.as_bytes().chunks(2) {
        let hex = std::str::from_utf8(chunk).ok()?;
        bytes.push(u8::from_str_radix(hex, 16).ok()?);
    }
    String::from_utf8(bytes).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_profile() -> StoredProfile {
        StoredProfile {
            device_id: 12,
            device_name: "Pedals=Alpha\nBeta".to_string(),
            throttle: StoredBinding {
                axis_index: 0,
                axis_label: "X axis".to_string(),
                idle_raw: 101,
                active_raw: 801,
                calibration: StoredCalibration::DriverRange,
            },
            brake: StoredBinding {
                axis_index: 2,
                axis_label: "Z axis".to_string(),
                idle_raw: 900,
                active_raw: 120,
                calibration: StoredCalibration::CustomRange {
                    idle_raw: 920,
                    active_raw: 140,
                },
            },
        }
    }

    #[test]
    fn serialized_profile_roundtrips_special_text() {
        let profile = sample_profile();
        let serialized = serialize_profile(&profile);

        assert_eq!(parse_profile(&serialized), Some(profile));
    }

    #[test]
    fn profile_signature_matches_serialized_content() {
        let profile = sample_profile();

        assert_eq!(profile_signature(&profile), serialize_profile(&profile));
    }

    #[test]
    fn parser_rejects_unknown_version() {
        let content = serialize_profile(&sample_profile()).replacen("version=1", "version=2", 1);

        assert_eq!(parse_profile(&content), None);
    }

    #[test]
    fn parser_rejects_missing_binding_field() {
        let content = serialize_profile(&sample_profile())
            .lines()
            .filter(|line| !line.starts_with("brake_active_raw="))
            .collect::<Vec<_>>()
            .join("\n");

        assert_eq!(parse_profile(&content), None);
    }

    #[test]
    fn parser_defaults_old_profiles_to_driver_range() {
        let content = serialize_profile(&sample_profile())
            .lines()
            .filter(|line| !line.contains("_calibration"))
            .collect::<Vec<_>>()
            .join("\n");
        let profile = parse_profile(&content).unwrap();

        assert_eq!(profile.throttle.calibration, StoredCalibration::DriverRange);
        assert_eq!(profile.brake.calibration, StoredCalibration::DriverRange);
    }

    #[test]
    fn parser_rejects_incomplete_custom_calibration() {
        let content = serialize_profile(&sample_profile())
            .lines()
            .filter(|line| !line.starts_with("brake_calibration_active_raw="))
            .collect::<Vec<_>>()
            .join("\n");

        assert_eq!(parse_profile(&content), None);
    }

    #[test]
    fn text_decoder_rejects_invalid_hex() {
        assert_eq!(decode_text("A"), None);
        assert_eq!(decode_text("XX"), None);
    }
}
