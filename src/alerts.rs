use crate::wizard::LiveSample;

pub const LIVE_SAMPLE_INTERVAL_MS: u32 = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlertId {
    PedalOverlap,
    Coasting,
    ThrottleWithLock,
    BrakeReleaseSnap,
    SteeringSaw,
    SteeringSaturated,
}

impl AlertId {
    fn sort_key(self) -> u8 {
        match self {
            AlertId::PedalOverlap => 0,
            AlertId::ThrottleWithLock => 1,
            AlertId::BrakeReleaseSnap => 2,
            AlertId::SteeringSaturated => 3,
            AlertId::Coasting => 4,
            AlertId::SteeringSaw => 5,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlertSeverity {
    Notice,
    Warning,
}

impl AlertSeverity {
    pub fn priority(self) -> u8 {
        match self {
            AlertSeverity::Notice => 0,
            AlertSeverity::Warning => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlertSensitivity {
    Quiet,
    Balanced,
    Sensitive,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AlertSettings {
    pub enabled: bool,
    pub sensitivity: AlertSensitivity,
}

impl Default for AlertSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            sensitivity: AlertSensitivity::Balanced,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AlertView {
    pub id: AlertId,
    pub severity: AlertSeverity,
    pub label: &'static str,
    pub message: &'static str,
    pub age_ms: u32,
    pub opacity: f32,
}

#[derive(Clone, Copy)]
pub struct AlertSpec {
    id: AlertId,
    severity: AlertSeverity,
    label: &'static str,
    message: &'static str,
    hold_ms: u32,
    cooldown_ms: u32,
    ttl_ms: u32,
}

#[derive(Clone, Copy)]
pub struct AlertThresholds {
    pedal_overlap_value: f32,
    pedal_overlap_hold_ms: u32,
    coasting_pedal_max: f32,
    coasting_steering_min: f32,
    coasting_hold_ms: u32,
    throttle_lock_steering_min: f32,
    throttle_lock_rise: f32,
    throttle_lock_min: f32,
    brake_release_steering_min: f32,
    brake_release_drop: f32,
    brake_release_start_min: f32,
    fast_window_ms: u32,
    steering_saw_delta: f32,
    steering_saw_range: f32,
    steering_saw_changes: usize,
    steering_saw_window_ms: u32,
    steering_saturated_min: f32,
    steering_saturated_pedal_min: f32,
    steering_saturated_hold_ms: u32,
    state_cooldown_ms: u32,
    event_cooldown_ms: u32,
    ttl_ms: u32,
}

impl AlertThresholds {
    fn for_sensitivity(sensitivity: AlertSensitivity) -> Self {
        match sensitivity {
            AlertSensitivity::Quiet => Self {
                pedal_overlap_value: 0.16,
                pedal_overlap_hold_ms: 240,
                coasting_pedal_max: 0.03,
                coasting_steering_min: 0.16,
                coasting_hold_ms: 500,
                throttle_lock_steering_min: 0.55,
                throttle_lock_rise: 0.30,
                throttle_lock_min: 0.34,
                brake_release_steering_min: 0.28,
                brake_release_drop: 0.36,
                brake_release_start_min: 0.45,
                fast_window_ms: 160,
                steering_saw_delta: 0.10,
                steering_saw_range: 0.24,
                steering_saw_changes: 4,
                steering_saw_window_ms: 640,
                steering_saturated_min: 0.96,
                steering_saturated_pedal_min: 0.12,
                steering_saturated_hold_ms: 500,
                state_cooldown_ms: 360,
                event_cooldown_ms: 760,
                ttl_ms: 320,
            },
            AlertSensitivity::Balanced => Self {
                pedal_overlap_value: 0.12,
                pedal_overlap_hold_ms: 160,
                coasting_pedal_max: 0.04,
                coasting_steering_min: 0.12,
                coasting_hold_ms: 350,
                throttle_lock_steering_min: 0.45,
                throttle_lock_rise: 0.22,
                throttle_lock_min: 0.28,
                brake_release_steering_min: 0.20,
                brake_release_drop: 0.28,
                brake_release_start_min: 0.35,
                fast_window_ms: 160,
                steering_saw_delta: 0.08,
                steering_saw_range: 0.16,
                steering_saw_changes: 3,
                steering_saw_window_ms: 640,
                steering_saturated_min: 0.92,
                steering_saturated_pedal_min: 0.08,
                steering_saturated_hold_ms: 350,
                state_cooldown_ms: 280,
                event_cooldown_ms: 640,
                ttl_ms: 320,
            },
            AlertSensitivity::Sensitive => Self {
                pedal_overlap_value: 0.08,
                pedal_overlap_hold_ms: 96,
                coasting_pedal_max: 0.06,
                coasting_steering_min: 0.10,
                coasting_hold_ms: 240,
                throttle_lock_steering_min: 0.35,
                throttle_lock_rise: 0.16,
                throttle_lock_min: 0.22,
                brake_release_steering_min: 0.15,
                brake_release_drop: 0.20,
                brake_release_start_min: 0.28,
                fast_window_ms: 160,
                steering_saw_delta: 0.06,
                steering_saw_range: 0.12,
                steering_saw_changes: 2,
                steering_saw_window_ms: 640,
                steering_saturated_min: 0.88,
                steering_saturated_pedal_min: 0.06,
                steering_saturated_hold_ms: 240,
                state_cooldown_ms: 220,
                event_cooldown_ms: 520,
                ttl_ms: 320,
            },
        }
    }
}

pub trait AlertDetector: Send {
    fn spec(&self, thresholds: &AlertThresholds) -> AlertSpec;
    fn detect(&self, history: &[LiveSample], thresholds: &AlertThresholds) -> bool;
}

#[derive(Clone, Default)]
struct AlertState {
    hold_frames: u32,
    visible_frames: u32,
    cooldown_frames: u32,
    age_frames: u32,
    raw_active: bool,
}

impl AlertState {
    fn update(&mut self, raw: bool, spec: AlertSpec) -> Option<AlertView> {
        if self.cooldown_frames > 0 && self.visible_frames == 0 {
            self.cooldown_frames -= 1;
        }

        let hold_frames = frames_for_ms(spec.hold_ms);
        let ttl_frames = frames_for_ms(spec.ttl_ms);

        if raw {
            self.hold_frames = self.hold_frames.saturating_add(1);
            if self.visible_frames > 0 {
                self.visible_frames = ttl_frames;
                self.age_frames = self.age_frames.saturating_add(1);
                self.raw_active = true;
            } else if self.hold_frames >= hold_frames && self.cooldown_frames == 0 {
                self.visible_frames = ttl_frames;
                self.age_frames = 0;
                self.raw_active = true;
            } else {
                self.raw_active = false;
            }
        } else {
            self.hold_frames = 0;
            self.raw_active = false;
            if self.visible_frames > 0 {
                self.visible_frames -= 1;
                self.age_frames = self.age_frames.saturating_add(1);
                if self.visible_frames == 0 {
                    self.cooldown_frames = frames_for_ms(spec.cooldown_ms);
                }
            }
        }

        if self.visible_frames == 0 {
            return None;
        }

        let opacity = if self.raw_active {
            1.0
        } else {
            self.visible_frames as f32 / ttl_frames as f32
        }
        .clamp(0.0, 1.0);

        Some(AlertView {
            id: spec.id,
            severity: spec.severity,
            label: spec.label,
            message: spec.message,
            age_ms: self.age_frames.saturating_mul(LIVE_SAMPLE_INTERVAL_MS),
            opacity,
        })
    }
}

pub struct AlertEngine {
    settings: AlertSettings,
    detectors: Vec<Box<dyn AlertDetector>>,
    states: Vec<AlertState>,
    active: Vec<AlertView>,
}

impl Default for AlertEngine {
    fn default() -> Self {
        Self::new(AlertSettings::default())
    }
}

impl AlertEngine {
    pub fn new(settings: AlertSettings) -> Self {
        let detectors = default_detectors();
        let states = vec![AlertState::default(); detectors.len()];
        Self {
            settings,
            detectors,
            states,
            active: Vec::new(),
        }
    }

    pub fn set_settings(&mut self, settings: AlertSettings) {
        if self.settings != settings {
            self.settings = settings;
            self.clear();
        }
    }

    pub fn clear(&mut self) {
        for state in &mut self.states {
            *state = AlertState::default();
        }
        self.active.clear();
    }

    pub fn update(&mut self, history: &[LiveSample]) {
        self.active.clear();

        if !self.settings.enabled {
            self.clear();
            return;
        }

        let thresholds = AlertThresholds::for_sensitivity(self.settings.sensitivity);
        for (detector, state) in self.detectors.iter().zip(self.states.iter_mut()) {
            let spec = detector.spec(&thresholds);
            let raw = detector.detect(history, &thresholds);
            if let Some(view) = state.update(raw, spec) {
                self.active.push(view);
            }
        }

        self.active.sort_by(|left, right| {
            right
                .severity
                .priority()
                .cmp(&left.severity.priority())
                .then_with(|| left.id.sort_key().cmp(&right.id.sort_key()))
        });
    }

    pub fn alerts(&self) -> &[AlertView] {
        &self.active
    }
}

struct PedalOverlapDetector;
struct CoastingDetector;
struct ThrottleWithLockDetector;
struct BrakeReleaseSnapDetector;
struct SteeringSawDetector;
struct SteeringSaturatedDetector;

fn default_detectors() -> Vec<Box<dyn AlertDetector>> {
    vec![
        Box::new(PedalOverlapDetector),
        Box::new(CoastingDetector),
        Box::new(ThrottleWithLockDetector),
        Box::new(BrakeReleaseSnapDetector),
        Box::new(SteeringSawDetector),
        Box::new(SteeringSaturatedDetector),
    ]
}

impl AlertDetector for PedalOverlapDetector {
    fn spec(&self, thresholds: &AlertThresholds) -> AlertSpec {
        AlertSpec {
            id: AlertId::PedalOverlap,
            severity: AlertSeverity::Warning,
            label: "Throttle + brake",
            message: "Pedal overlap",
            hold_ms: thresholds.pedal_overlap_hold_ms,
            cooldown_ms: thresholds.state_cooldown_ms,
            ttl_ms: thresholds.ttl_ms,
        }
    }

    fn detect(&self, history: &[LiveSample], thresholds: &AlertThresholds) -> bool {
        last(history)
            .map(|sample| {
                sample.throttle > thresholds.pedal_overlap_value
                    && sample.brake > thresholds.pedal_overlap_value
            })
            .unwrap_or(false)
    }
}

impl AlertDetector for CoastingDetector {
    fn spec(&self, thresholds: &AlertThresholds) -> AlertSpec {
        AlertSpec {
            id: AlertId::Coasting,
            severity: AlertSeverity::Notice,
            label: "Coasting",
            message: "No pedal load",
            hold_ms: thresholds.coasting_hold_ms,
            cooldown_ms: thresholds.state_cooldown_ms,
            ttl_ms: thresholds.ttl_ms,
        }
    }

    fn detect(&self, history: &[LiveSample], thresholds: &AlertThresholds) -> bool {
        last(history)
            .map(|sample| {
                sample.throttle < thresholds.coasting_pedal_max
                    && sample.brake < thresholds.coasting_pedal_max
                    && sample.steering.abs() > thresholds.coasting_steering_min
            })
            .unwrap_or(false)
    }
}

impl AlertDetector for ThrottleWithLockDetector {
    fn spec(&self, thresholds: &AlertThresholds) -> AlertSpec {
        AlertSpec {
            id: AlertId::ThrottleWithLock,
            severity: AlertSeverity::Warning,
            label: "Unwind first",
            message: "Throttle with lock",
            hold_ms: 0,
            cooldown_ms: thresholds.event_cooldown_ms,
            ttl_ms: thresholds.ttl_ms,
        }
    }

    fn detect(&self, history: &[LiveSample], thresholds: &AlertThresholds) -> bool {
        let Some(current) = last(history) else {
            return false;
        };
        let Some(previous) = sample_before(history, thresholds.fast_window_ms) else {
            return false;
        };

        current.steering.abs() > thresholds.throttle_lock_steering_min
            && current.throttle >= thresholds.throttle_lock_min
            && current.throttle - previous.throttle >= thresholds.throttle_lock_rise
    }
}

impl AlertDetector for BrakeReleaseSnapDetector {
    fn spec(&self, thresholds: &AlertThresholds) -> AlertSpec {
        AlertSpec {
            id: AlertId::BrakeReleaseSnap,
            severity: AlertSeverity::Warning,
            label: "Ease release",
            message: "Brake release snap",
            hold_ms: 0,
            cooldown_ms: thresholds.event_cooldown_ms,
            ttl_ms: thresholds.ttl_ms,
        }
    }

    fn detect(&self, history: &[LiveSample], thresholds: &AlertThresholds) -> bool {
        let Some(current) = last(history) else {
            return false;
        };
        let Some(previous) = sample_before(history, thresholds.fast_window_ms) else {
            return false;
        };

        current.steering.abs() > thresholds.brake_release_steering_min
            && previous.brake >= thresholds.brake_release_start_min
            && previous.brake - current.brake >= thresholds.brake_release_drop
    }
}

impl AlertDetector for SteeringSawDetector {
    fn spec(&self, thresholds: &AlertThresholds) -> AlertSpec {
        AlertSpec {
            id: AlertId::SteeringSaw,
            severity: AlertSeverity::Notice,
            label: "Sawing wheel",
            message: "Steering oscillation",
            hold_ms: 0,
            cooldown_ms: thresholds.event_cooldown_ms,
            ttl_ms: thresholds.ttl_ms,
        }
    }

    fn detect(&self, history: &[LiveSample], thresholds: &AlertThresholds) -> bool {
        let window = recent_window(history, thresholds.steering_saw_window_ms);
        let minimum = frames_for_ms(thresholds.steering_saw_window_ms) as usize / 2;
        if window.len() < minimum.max(3) {
            return false;
        }

        let mut min_steering = window[0].steering;
        let mut max_steering = window[0].steering;
        let mut last_direction = 0;
        let mut direction_changes = 0;

        for pair in window.windows(2) {
            let previous = pair[0].steering;
            let current = pair[1].steering;
            min_steering = min_steering.min(current);
            max_steering = max_steering.max(current);

            let delta = current - previous;
            if delta.abs() < thresholds.steering_saw_delta {
                continue;
            }

            let direction = if delta > 0.0 { 1 } else { -1 };
            if last_direction != 0 && direction != last_direction {
                direction_changes += 1;
            }
            last_direction = direction;
        }

        direction_changes >= thresholds.steering_saw_changes
            && max_steering - min_steering >= thresholds.steering_saw_range
    }
}

impl AlertDetector for SteeringSaturatedDetector {
    fn spec(&self, thresholds: &AlertThresholds) -> AlertSpec {
        AlertSpec {
            id: AlertId::SteeringSaturated,
            severity: AlertSeverity::Warning,
            label: "Too much lock",
            message: "Steering saturated",
            hold_ms: thresholds.steering_saturated_hold_ms,
            cooldown_ms: thresholds.state_cooldown_ms,
            ttl_ms: thresholds.ttl_ms,
        }
    }

    fn detect(&self, history: &[LiveSample], thresholds: &AlertThresholds) -> bool {
        last(history)
            .map(|sample| {
                sample.steering.abs() > thresholds.steering_saturated_min
                    && (sample.throttle > thresholds.steering_saturated_pedal_min
                        || sample.brake > thresholds.steering_saturated_pedal_min)
            })
            .unwrap_or(false)
    }
}

fn last(history: &[LiveSample]) -> Option<&LiveSample> {
    history.last()
}

fn sample_before(history: &[LiveSample], ms: u32) -> Option<&LiveSample> {
    let offset = frames_for_ms(ms) as usize;
    history
        .len()
        .checked_sub(offset + 1)
        .and_then(|index| history.get(index))
}

fn recent_window(history: &[LiveSample], ms: u32) -> &[LiveSample] {
    let frames = frames_for_ms(ms) as usize;
    let start = history.len().saturating_sub(frames);
    &history[start..]
}

fn frames_for_ms(ms: u32) -> u32 {
    ((ms + LIVE_SAMPLE_INTERVAL_MS - 1) / LIVE_SAMPLE_INTERVAL_MS).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(throttle: f32, brake: f32, steering: f32) -> LiveSample {
        LiveSample {
            throttle,
            brake,
            steering,
        }
    }

    fn feed(engine: &mut AlertEngine, history: &mut Vec<LiveSample>, sample: LiveSample) {
        history.push(sample);
        engine.update(history);
    }

    fn alert_ids(engine: &AlertEngine) -> Vec<AlertId> {
        engine.alerts().iter().map(|alert| alert.id).collect()
    }

    #[test]
    fn pedal_overlap_requires_hold_time() {
        let mut engine = AlertEngine::default();
        let mut history = Vec::new();

        for _ in 0..19 {
            feed(&mut engine, &mut history, sample(0.2, 0.2, 0.0));
            assert!(!alert_ids(&engine).contains(&AlertId::PedalOverlap));
        }

        feed(&mut engine, &mut history, sample(0.2, 0.2, 0.0));
        assert!(alert_ids(&engine).contains(&AlertId::PedalOverlap));
    }

    #[test]
    fn coasting_requires_steering_and_no_pedals() {
        let mut engine = AlertEngine::default();
        let mut history = Vec::new();

        for _ in 0..43 {
            feed(&mut engine, &mut history, sample(0.0, 0.0, 0.15));
            assert!(!alert_ids(&engine).contains(&AlertId::Coasting));
        }

        feed(&mut engine, &mut history, sample(0.0, 0.0, 0.15));
        assert!(alert_ids(&engine).contains(&AlertId::Coasting));
    }

    #[test]
    fn throttle_with_lock_triggers_on_fast_throttle_rise() {
        let mut engine = AlertEngine::default();
        let mut history = Vec::new();

        for _ in 0..21 {
            feed(&mut engine, &mut history, sample(0.0, 0.0, 0.5));
        }

        feed(&mut engine, &mut history, sample(0.3, 0.0, 0.5));
        assert!(alert_ids(&engine).contains(&AlertId::ThrottleWithLock));
    }

    #[test]
    fn brake_release_snap_triggers_on_fast_release_under_lock() {
        let mut engine = AlertEngine::default();
        let mut history = Vec::new();

        for _ in 0..21 {
            feed(&mut engine, &mut history, sample(0.0, 0.7, 0.3));
        }

        feed(&mut engine, &mut history, sample(0.0, 0.35, 0.3));
        assert!(alert_ids(&engine).contains(&AlertId::BrakeReleaseSnap));
    }

    #[test]
    fn steering_saw_triggers_on_repeated_direction_changes() {
        let mut engine = AlertEngine::default();
        let mut history = Vec::new();
        let sequence = [0.0, 0.12, -0.12, 0.12, -0.12, 0.12];

        for index in 0..48 {
            feed(
                &mut engine,
                &mut history,
                sample(0.0, 0.0, sequence[index % sequence.len()]),
            );
        }

        assert!(alert_ids(&engine).contains(&AlertId::SteeringSaw));
    }

    #[test]
    fn steering_saturation_requires_hold_time_and_pedal_input() {
        let mut engine = AlertEngine::default();
        let mut history = Vec::new();

        for _ in 0..43 {
            feed(&mut engine, &mut history, sample(0.2, 0.0, 0.95));
            assert!(!alert_ids(&engine).contains(&AlertId::SteeringSaturated));
        }

        feed(&mut engine, &mut history, sample(0.2, 0.0, 0.95));
        assert!(alert_ids(&engine).contains(&AlertId::SteeringSaturated));
    }

    #[test]
    fn cooldown_prevents_immediate_retrigger_after_clear() {
        let mut engine = AlertEngine::default();
        let mut history = Vec::new();

        for _ in 0..20 {
            feed(&mut engine, &mut history, sample(0.2, 0.2, 0.0));
        }
        assert!(alert_ids(&engine).contains(&AlertId::PedalOverlap));

        for _ in 0..40 {
            feed(&mut engine, &mut history, sample(0.0, 0.0, 0.0));
        }
        assert!(!alert_ids(&engine).contains(&AlertId::PedalOverlap));

        for _ in 0..20 {
            feed(&mut engine, &mut history, sample(0.2, 0.2, 0.0));
        }
        assert!(!alert_ids(&engine).contains(&AlertId::PedalOverlap));
    }

    #[test]
    fn disabled_settings_clear_alerts() {
        let mut engine = AlertEngine::default();
        let mut history = Vec::new();

        for _ in 0..20 {
            feed(&mut engine, &mut history, sample(0.2, 0.2, 0.0));
        }
        assert!(!engine.alerts().is_empty());

        engine.set_settings(AlertSettings {
            enabled: false,
            sensitivity: AlertSensitivity::Balanced,
        });
        engine.update(&history);

        assert!(engine.alerts().is_empty());
    }
}
