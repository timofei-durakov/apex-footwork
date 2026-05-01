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
    pub steering: Option<StoredBinding>,
    pub overlay_settings: StoredOverlaySettings,
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
    CustomRange {
        idle_raw: u32,
        active_raw: u32,
    },
    SteeringCustomRange {
        center_raw: u32,
        left_raw: u32,
        right_raw: u32,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredOverlaySettings {
    pub steering_graph: bool,
    pub steering_scale: StoredSteeringScale,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoredSteeringScale {
    Linear,
    Log,
}

impl Default for StoredOverlaySettings {
    fn default() -> Self {
        Self {
            steering_graph: true,
            steering_scale: StoredSteeringScale::Log,
        }
    }
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
    if let Some(steering) = &profile.steering {
        push_binding(&mut lines, "steering", steering);
    }
    lines.push(format!(
        "overlay_steering_graph={}",
        profile.overlay_settings.steering_graph
    ));
    lines.push(format!(
        "overlay_steering_scale={}",
        match profile.overlay_settings.steering_scale {
            StoredSteeringScale::Linear => "linear",
            StoredSteeringScale::Log => "log",
        }
    ));
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
        StoredCalibration::SteeringCustomRange {
            center_raw,
            left_raw,
            right_raw,
        } => {
            lines.push(format!("{}_calibration=steering_custom_range", prefix));
            lines.push(format!("{}_calibration_center_raw={}", prefix, center_raw));
            lines.push(format!("{}_calibration_left_raw={}", prefix, left_raw));
            lines.push(format!("{}_calibration_right_raw={}", prefix, right_raw));
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
        steering: parse_optional_binding(&values, "steering")?,
        overlay_settings: parse_overlay_settings(&values)?,
    })
}

fn parse_optional_binding(
    values: &HashMap<String, String>,
    prefix: &str,
) -> Option<Option<StoredBinding>> {
    let has_binding = values
        .keys()
        .any(|key| key.starts_with(&format!("{}_", prefix)));
    if has_binding {
        Some(Some(parse_binding(values, prefix)?))
    } else {
        Some(None)
    }
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
        "steering_custom_range" => {
            let center_raw = values
                .get(&format!("{}_calibration_center_raw", prefix))?
                .parse()
                .ok()?;
            let left_raw = values
                .get(&format!("{}_calibration_left_raw", prefix))?
                .parse()
                .ok()?;
            let right_raw = values
                .get(&format!("{}_calibration_right_raw", prefix))?
                .parse()
                .ok()?;
            valid_steering_range(center_raw, left_raw, right_raw).then_some(
                StoredCalibration::SteeringCustomRange {
                    center_raw,
                    left_raw,
                    right_raw,
                },
            )
        }
        _ => None,
    }
}

fn parse_overlay_settings(values: &HashMap<String, String>) -> Option<StoredOverlaySettings> {
    let steering_graph = match values.get("overlay_steering_graph").map(String::as_str) {
        Some("true") => true,
        Some("false") => false,
        Some(_) => return None,
        None => StoredOverlaySettings::default().steering_graph,
    };
    let steering_scale = match values.get("overlay_steering_scale").map(String::as_str) {
        Some("linear") => StoredSteeringScale::Linear,
        Some("log") => StoredSteeringScale::Log,
        Some(_) => return None,
        None => StoredOverlaySettings::default().steering_scale,
    };

    Some(StoredOverlaySettings {
        steering_graph,
        steering_scale,
    })
}

fn valid_steering_range(center_raw: u32, left_raw: u32, right_raw: u32) -> bool {
    (left_raw < center_raw && center_raw < right_raw)
        || (right_raw < center_raw && center_raw < left_raw)
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
            steering: Some(StoredBinding {
                axis_index: 3,
                axis_label: "R axis".to_string(),
                idle_raw: 500,
                active_raw: 900,
                calibration: StoredCalibration::SteeringCustomRange {
                    center_raw: 500,
                    left_raw: 100,
                    right_raw: 900,
                },
            }),
            overlay_settings: StoredOverlaySettings {
                steering_graph: false,
                steering_scale: StoredSteeringScale::Linear,
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
    fn parser_defaults_missing_steering_and_overlay_settings() {
        let content = serialize_profile(&sample_profile())
            .lines()
            .filter(|line| !line.starts_with("steering_"))
            .filter(|line| !line.starts_with("overlay_"))
            .collect::<Vec<_>>()
            .join("\n");
        let profile = parse_profile(&content).unwrap();

        assert_eq!(profile.steering, None);
        assert_eq!(profile.overlay_settings, StoredOverlaySettings::default());
    }

    #[test]
    fn parser_rejects_invalid_advanced_steering_range() {
        let content = serialize_profile(&sample_profile()).replace(
            "steering_calibration_left_raw=100",
            "steering_calibration_left_raw=600",
        );

        assert_eq!(parse_profile(&content), None);
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
