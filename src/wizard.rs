use crate::alerts::{AlertEngine, AlertSensitivity, AlertSettings, AlertView};
use crate::profile::{StoredBinding, StoredCalibration, StoredProfile};

#[derive(Clone, Debug)]
pub struct AxisSnapshot {
    pub label: &'static str,
    pub min: u32,
    pub max: u32,
    pub raw: u32,
}

impl AxisSnapshot {
    pub fn new(label: &'static str, min: u32, max: u32, raw: u32) -> Self {
        Self {
            label,
            min,
            max,
            raw,
        }
    }

    pub fn percent(&self) -> f32 {
        if self.max <= self.min {
            return 0.0;
        }

        let clamped = self.raw.clamp(self.min, self.max);
        (clamped - self.min) as f32 / (self.max - self.min) as f32
    }
}

#[derive(Clone, Debug)]
pub struct DeviceSnapshot {
    pub id: u32,
    pub name: String,
    pub axes: Vec<AxisSnapshot>,
}

pub trait DeviceProvider {
    fn enumerate_devices(&self) -> Vec<DeviceSnapshot>;
    fn read_axes(&self, device_id: u32) -> Option<Vec<u32>>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputRole {
    Throttle,
    Brake,
    Steering,
}

impl InputRole {
    pub fn label(self) -> &'static str {
        match self {
            InputRole::Throttle => "Throttle",
            InputRole::Brake => "Brake",
            InputRole::Steering => "Steering",
        }
    }
}

#[derive(Clone, Debug)]
pub struct BindingView {
    pub role: InputRole,
    pub axis_index: usize,
    pub axis_label: &'static str,
    pub idle_raw: u32,
    pub active_raw: u32,
    pub calibration: BindingCalibration,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BindingCalibration {
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

impl BindingView {
    pub fn value(&self, axes: &[AxisSnapshot]) -> f32 {
        let Some(axis) = axes.get(self.axis_index) else {
            return 0.0;
        };

        match self.role {
            InputRole::Steering => self.steering_value(axis),
            InputRole::Throttle | InputRole::Brake => match self.calibration {
                BindingCalibration::DriverRange => self.driver_range_value(axis),
                BindingCalibration::CustomRange {
                    idle_raw,
                    active_raw,
                } => Self::raw_span_value(axis.raw, idle_raw, active_raw),
                BindingCalibration::SteeringCustomRange { .. } => 0.0,
            },
        }
    }

    fn driver_range_value(&self, axis: &AxisSnapshot) -> f32 {
        if axis.max <= axis.min || self.idle_raw == self.active_raw {
            return 0.0;
        }

        let normalized = Self::raw_span_value(axis.raw, axis.min, axis.max);
        if self.active_raw > self.idle_raw {
            normalized
        } else {
            1.0 - normalized
        }
    }

    fn raw_span_value(raw: u32, idle_raw: u32, active_raw: u32) -> f32 {
        let idle = idle_raw as f32;
        let active = active_raw as f32;
        let span = active - idle;
        if span.abs() < 1.0 {
            return 0.0;
        }

        ((raw as f32 - idle) / span).clamp(0.0, 1.0)
    }

    fn steering_value(&self, axis: &AxisSnapshot) -> f32 {
        match self.calibration {
            BindingCalibration::DriverRange => self.driver_range_steering_value(axis),
            BindingCalibration::SteeringCustomRange {
                center_raw,
                left_raw,
                right_raw,
            } => Self::custom_steering_value(axis.raw, center_raw, left_raw, right_raw),
            BindingCalibration::CustomRange { .. } => 0.0,
        }
    }

    fn driver_range_steering_value(&self, axis: &AxisSnapshot) -> f32 {
        if axis.max <= axis.min || self.idle_raw == self.active_raw {
            return 0.0;
        }

        let normalized = Self::raw_span_value(axis.raw, axis.min, axis.max);
        let value = normalized * 2.0 - 1.0;
        if self.active_raw > self.idle_raw {
            value
        } else {
            -value
        }
        .clamp(-1.0, 1.0)
    }

    fn custom_steering_value(raw: u32, center_raw: u32, left_raw: u32, right_raw: u32) -> f32 {
        let right_value = Self::raw_span_value(raw, center_raw, right_raw);
        let left_value = Self::raw_span_value(raw, center_raw, left_raw);

        if right_raw > center_raw && left_raw < center_raw {
            if raw >= center_raw {
                right_value
            } else {
                -left_value
            }
        } else if right_raw < center_raw && left_raw > center_raw {
            if raw <= center_raw {
                right_value
            } else {
                -left_value
            }
        } else {
            0.0
        }
        .clamp(-1.0, 1.0)
    }
}

#[derive(Clone, Debug, Default)]
pub struct PedalBindings {
    pub throttle: Option<BindingView>,
    pub brake: Option<BindingView>,
    pub steering: Option<BindingView>,
}

impl PedalBindings {
    #[allow(dead_code)]
    pub fn is_complete(&self) -> bool {
        self.throttle.is_some() && self.brake.is_some() && self.steering.is_some()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LiveSample {
    pub throttle: f32,
    pub brake: f32,
    pub steering: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WizardCommand {
    SelectDevice(usize),
    Confirm,
    Configure,
    ToggleAdvancedCalibration,
}

#[derive(Clone, Debug)]
pub enum WizardStepView {
    SelectDevice {
        devices: Vec<DeviceSnapshot>,
        selected_index: usize,
    },
    Capture {
        role: InputRole,
        armed: bool,
        advanced_calibration: bool,
        device: Option<DeviceSnapshot>,
        bindings: PedalBindings,
    },
    Ready {
        device: Option<DeviceSnapshot>,
        bindings: PedalBindings,
        values: Vec<(InputRole, f32)>,
        history: Vec<LiveSample>,
        alerts: Vec<AlertView>,
    },
}

impl WizardStepView {
    pub fn title(&self) -> &'static str {
        match self {
            WizardStepView::SelectDevice { .. } => "Step 1/4: Choose device",
            WizardStepView::Capture {
                role: InputRole::Throttle,
                ..
            } => "Step 2/4: Detect throttle",
            WizardStepView::Capture {
                role: InputRole::Brake,
                ..
            } => "Step 3/4: Detect brake",
            WizardStepView::Capture {
                role: InputRole::Steering,
                ..
            } => "Step 4/4: Detect steering",
            WizardStepView::Ready { .. } => "Configured",
        }
    }
}

#[derive(Clone, Debug)]
pub struct WizardView {
    pub status: String,
    pub step: WizardStepView,
}

#[derive(Clone, Debug)]
struct CaptureCandidate {
    binding: BindingView,
    magnitude: f32,
}

#[derive(Clone, Debug)]
struct SteeringCandidate {
    axis_index: usize,
    axis_label: &'static str,
    raw: u32,
    magnitude: f32,
}

#[derive(Clone, Debug)]
struct SteeringEdge {
    axis_index: usize,
    axis_label: &'static str,
    raw: u32,
}

enum CaptureUpdate {
    Waiting,
    Candidate(CaptureCandidate),
    Finished(BindingView),
}

enum SteeringCaptureUpdate {
    Waiting,
    Candidate(SteeringCandidate),
    Finished(SteeringCandidate),
}

enum CaptureState {
    Idle,
    Detect {
        baseline: Vec<u32>,
        candidate: Option<CaptureCandidate>,
    },
    SteeringAdvancedLeft {
        center: Vec<u32>,
        candidate: Option<SteeringCandidate>,
    },
    SteeringAdvancedRight {
        center: Vec<u32>,
        left: SteeringEdge,
        candidate: Option<SteeringCandidate>,
    },
}

impl CaptureState {
    fn is_armed(&self) -> bool {
        !matches!(self, CaptureState::Idle)
    }
}

enum WizardStep {
    SelectDevice,
    Capture {
        role: InputRole,
        state: CaptureState,
    },
    Ready,
}

const CAPTURE_START_THRESHOLD: f32 = 0.18;
const CAPTURE_RELEASE_THRESHOLD: f32 = 0.06;

fn capture_prompt(role: InputRole, advanced_calibration: bool) -> String {
    if role == InputRole::Steering {
        return if advanced_calibration {
            "Center captured. Turn steering fully left, then release it back to center.".to_string()
        } else {
            "Turn Steering right once, then release it. The driver range is used for full travel."
                .to_string()
        };
    }

    if advanced_calibration {
        format!(
            "Press {} fully, then release it. This saves an advanced custom 0-100% range.",
            role.label()
        )
    } else {
        format!(
            "Press {} once, then release it. This detects the axis and direction.",
            role.label()
        )
    }
}

fn capture_detected_prompt(role: InputRole, advanced_calibration: bool) -> String {
    if role == InputRole::Steering {
        return if advanced_calibration {
            "Steering movement detected. Release back to center to save this side.".to_string()
        } else {
            "Steering movement detected. Release back to center to save the binding.".to_string()
        };
    }

    if advanced_calibration {
        format!(
            "{} movement detected. Release the pedal to save the custom range.",
            role.label()
        )
    } else {
        format!(
            "{} movement detected. Release the pedal to save the binding.",
            role.label()
        )
    }
}

pub struct Wizard<P: DeviceProvider> {
    provider: P,
    devices: Vec<DeviceSnapshot>,
    selected_index: usize,
    active_device: Option<DeviceSnapshot>,
    step: WizardStep,
    bindings: PedalBindings,
    value_history: Vec<LiveSample>,
    alert_engine: AlertEngine,
    status: String,
    tick: u32,
    advanced_calibration: bool,
}

impl<P: DeviceProvider> Wizard<P> {
    pub fn new(provider: P) -> Self {
        let devices = provider.enumerate_devices();
        let status = if devices.is_empty() {
            "No joystick/HID devices found. Plug in a controller or pedal set.".to_string()
        } else {
            "Select the controller that owns your pedals.".to_string()
        };

        Self {
            provider,
            devices,
            selected_index: 0,
            active_device: None,
            step: WizardStep::SelectDevice,
            bindings: PedalBindings::default(),
            value_history: Vec::new(),
            alert_engine: AlertEngine::default(),
            status,
            tick: 0,
            advanced_calibration: false,
        }
    }

    pub fn update(&mut self) {
        self.tick = self.tick.wrapping_add(1);

        if matches!(self.step, WizardStep::SelectDevice) && self.tick % 30 == 0 {
            self.refresh_devices();
        }

        if self.active_device.is_some() {
            self.poll_active_device();
        }

        if matches!(self.step, WizardStep::Ready) {
            self.record_live_values();
        }

        enum PendingUpdate {
            Detect {
                role: InputRole,
                baseline: Vec<u32>,
                candidate: Option<CaptureCandidate>,
            },
            SteeringLeft {
                center: Vec<u32>,
                candidate: Option<SteeringCandidate>,
            },
            SteeringRight {
                center: Vec<u32>,
                left: SteeringEdge,
                candidate: Option<SteeringCandidate>,
            },
        }

        let pending = match &self.step {
            WizardStep::Capture {
                role,
                state:
                    CaptureState::Detect {
                        baseline,
                        candidate,
                    },
            } => Some(PendingUpdate::Detect {
                role: *role,
                baseline: baseline.clone(),
                candidate: candidate.clone(),
            }),
            WizardStep::Capture {
                state: CaptureState::SteeringAdvancedLeft { center, candidate },
                ..
            } => Some(PendingUpdate::SteeringLeft {
                center: center.clone(),
                candidate: candidate.clone(),
            }),
            WizardStep::Capture {
                state:
                    CaptureState::SteeringAdvancedRight {
                        center,
                        left,
                        candidate,
                    },
                ..
            } => Some(PendingUpdate::SteeringRight {
                center: center.clone(),
                left: left.clone(),
                candidate: candidate.clone(),
            }),
            _ => None,
        };

        match pending {
            Some(PendingUpdate::Detect {
                role,
                baseline,
                candidate,
            }) => match self.update_capture(role, &baseline, candidate.as_ref()) {
                CaptureUpdate::Waiting => {}
                CaptureUpdate::Candidate(candidate) => {
                    if let WizardStep::Capture {
                        state:
                            CaptureState::Detect {
                                candidate: current, ..
                            },
                        ..
                    } = &mut self.step
                    {
                        *current = Some(candidate);
                    }
                    self.status = capture_detected_prompt(role, self.advanced_calibration);
                }
                CaptureUpdate::Finished(binding) => {
                    self.finish_binding(binding);
                }
            },
            Some(PendingUpdate::SteeringLeft { center, candidate }) => {
                match self.update_steering_edge_capture(&center, None, candidate.as_ref()) {
                    SteeringCaptureUpdate::Waiting => {}
                    SteeringCaptureUpdate::Candidate(candidate) => {
                        if let WizardStep::Capture {
                            state:
                                CaptureState::SteeringAdvancedLeft {
                                    candidate: current, ..
                                },
                            ..
                        } = &mut self.step
                        {
                            *current = Some(candidate);
                        }
                        self.status =
                            "Left steering movement detected. Release back to center.".to_string();
                    }
                    SteeringCaptureUpdate::Finished(candidate) => {
                        let left = SteeringEdge {
                            axis_index: candidate.axis_index,
                            axis_label: candidate.axis_label,
                            raw: candidate.raw,
                        };
                        if let WizardStep::Capture { state, .. } = &mut self.step {
                            *state = CaptureState::SteeringAdvancedRight {
                                center,
                                left,
                                candidate: None,
                            };
                        }
                        self.status =
                            "Left range saved. Turn steering fully right, then release to center."
                                .to_string();
                    }
                }
            }
            Some(PendingUpdate::SteeringRight {
                center,
                left,
                candidate,
            }) => {
                match self.update_steering_edge_capture(
                    &center,
                    Some(left.axis_index),
                    candidate.as_ref(),
                ) {
                    SteeringCaptureUpdate::Waiting => {}
                    SteeringCaptureUpdate::Candidate(candidate) => {
                        if let WizardStep::Capture {
                            state:
                                CaptureState::SteeringAdvancedRight {
                                    candidate: current, ..
                                },
                            ..
                        } = &mut self.step
                        {
                            *current = Some(candidate);
                        }
                        self.status =
                            "Right steering movement detected. Release back to center.".to_string();
                    }
                    SteeringCaptureUpdate::Finished(candidate) => {
                        let right = SteeringEdge {
                            axis_index: candidate.axis_index,
                            axis_label: candidate.axis_label,
                            raw: candidate.raw,
                        };
                        self.finish_advanced_steering(center, left, right);
                    }
                }
            }
            None => {}
        }
    }

    pub fn handle_command(&mut self, command: WizardCommand) {
        match command {
            WizardCommand::SelectDevice(index) if matches!(self.step, WizardStep::SelectDevice) => {
                if !self.devices.is_empty() {
                    self.selected_index = index.min(self.devices.len() - 1);
                }
            }
            WizardCommand::Confirm => self.confirm(),
            WizardCommand::Configure => self.configure(),
            WizardCommand::ToggleAdvancedCalibration => self.toggle_advanced_calibration(),
            _ => {}
        }
    }

    pub fn view(&self) -> WizardView {
        let step = match &self.step {
            WizardStep::SelectDevice => WizardStepView::SelectDevice {
                devices: self.devices.clone(),
                selected_index: self.selected_index,
            },
            WizardStep::Capture { role, state } => WizardStepView::Capture {
                role: *role,
                armed: state.is_armed(),
                advanced_calibration: self.advanced_calibration,
                device: self.active_device.clone(),
                bindings: self.bindings.clone(),
            },
            WizardStep::Ready => WizardStepView::Ready {
                device: self.active_device.clone(),
                bindings: self.bindings.clone(),
                values: self.live_values(),
                history: self.value_history.clone(),
                alerts: self.alert_engine.alerts().to_vec(),
            },
        };

        WizardView {
            status: self.status.clone(),
            step,
        }
    }

    #[allow(dead_code)]
    pub fn bindings(&self) -> Option<PedalBindings> {
        self.bindings.is_complete().then(|| self.bindings.clone())
    }

    pub fn restore_profile(&mut self, profile: &StoredProfile) -> bool {
        self.alert_engine
            .set_settings(alert_settings_from_profile(profile));
        self.refresh_devices();

        let Some(index) = self.matching_device_index(profile) else {
            self.status = "Saved profile found, but its device is not connected.".to_string();
            return false;
        };
        let Some(device) = self.devices.get(index).cloned() else {
            return false;
        };

        let Some(throttle) =
            Self::binding_from_profile(InputRole::Throttle, &profile.throttle, &device)
        else {
            self.status = "Saved profile found, but throttle binding is invalid.".to_string();
            return false;
        };
        let Some(brake) = Self::binding_from_profile(InputRole::Brake, &profile.brake, &device)
        else {
            self.status = "Saved profile found, but brake binding is invalid.".to_string();
            return false;
        };
        let steering = if let Some(stored) = &profile.steering {
            let Some(steering) = Self::binding_from_profile(InputRole::Steering, stored, &device)
            else {
                self.status = "Saved profile found, but steering binding is invalid.".to_string();
                return false;
            };
            Some(steering)
        } else {
            None
        };

        self.selected_index = index;
        self.active_device = Some(device);
        self.poll_active_device();
        if self.active_device.is_none() {
            self.status = "Saved profile device disconnected while loading.".to_string();
            return false;
        }
        self.bindings = PedalBindings {
            throttle: Some(throttle),
            brake: Some(brake),
            steering,
        };
        self.clear_live_state();
        if self.bindings.steering.is_some() {
            self.step = WizardStep::Ready;
            self.status = "Saved profile loaded. Overlay input is ready.".to_string();
        } else {
            self.step = WizardStep::Capture {
                role: InputRole::Steering,
                state: CaptureState::Idle,
            };
            self.status =
                "Saved pedals loaded. Center steering, then click Capture steering.".to_string();
        }
        true
    }

    pub fn profile(&self) -> Option<StoredProfile> {
        let device = self.active_device.as_ref()?;
        let throttle = self.bindings.throttle.as_ref()?;
        let brake = self.bindings.brake.as_ref()?;
        let steering = self.bindings.steering.as_ref()?;

        Some(StoredProfile {
            device_id: device.id,
            device_name: device.name.clone(),
            throttle: Self::binding_to_profile(throttle),
            brake: Self::binding_to_profile(brake),
            steering: Some(Self::binding_to_profile(steering)),
            overlay_settings: Default::default(),
        })
    }

    fn confirm(&mut self) {
        match &self.step {
            WizardStep::SelectDevice => self.choose_selected_device(),
            WizardStep::Capture {
                state: CaptureState::Idle,
                ..
            } => self.arm_capture(),
            WizardStep::Capture { role, .. } => {
                self.status = format!("Still waiting for {} movement.", role.label());
            }
            WizardStep::Ready => {}
        }
    }

    fn configure(&mut self) {
        self.active_device = None;
        self.bindings = PedalBindings::default();
        self.clear_live_state();
        self.advanced_calibration = false;
        self.step = WizardStep::SelectDevice;
        self.refresh_devices();
    }

    fn toggle_advanced_calibration(&mut self) {
        let WizardStep::Capture {
            role,
            state: CaptureState::Idle,
        } = self.step
        else {
            return;
        };

        self.advanced_calibration = !self.advanced_calibration;
        self.status = if role == InputRole::Steering && self.advanced_calibration {
            "Advanced calibration enabled. Steering will capture center, left, and right."
                .to_string()
        } else if role == InputRole::Steering {
            "Advanced calibration disabled. Driver range will be used for steering.".to_string()
        } else if self.advanced_calibration {
            "Advanced calibration enabled. Capture will save a custom 0-100% range.".to_string()
        } else {
            "Advanced calibration disabled. Driver range will be used for 0-100%.".to_string()
        };
    }

    fn refresh_devices(&mut self) {
        self.devices = self.provider.enumerate_devices();
        if self.devices.is_empty() {
            self.selected_index = 0;
            self.status =
                "No joystick/HID devices found. Plug in a controller or pedal set.".to_string();
            return;
        }

        self.selected_index = self.selected_index.min(self.devices.len() - 1);
        self.status = "Select the controller that owns your pedals.".to_string();
    }

    fn poll_active_device(&mut self) {
        let Some(device) = &mut self.active_device else {
            return;
        };

        match self.provider.read_axes(device.id) {
            Some(values) => {
                for (axis, raw) in device.axes.iter_mut().zip(values) {
                    axis.raw = raw;
                }
            }
            None => {
                self.active_device = None;
                self.bindings = PedalBindings::default();
                self.step = WizardStep::SelectDevice;
                self.clear_live_state();
                self.refresh_devices();
                self.status =
                    "Lost selected device. Choose it again after reconnecting.".to_string();
            }
        }
    }

    fn choose_selected_device(&mut self) {
        let Some(device) = self.devices.get(self.selected_index).cloned() else {
            self.refresh_devices();
            return;
        };

        self.active_device = Some(device);
        self.poll_active_device();
        self.bindings = PedalBindings::default();
        self.clear_live_state();
        self.advanced_calibration = false;
        self.step = WizardStep::Capture {
            role: InputRole::Throttle,
            state: CaptureState::Idle,
        };
        self.status = "Release all pedals, then click Capture throttle.".to_string();
    }

    fn arm_capture(&mut self) {
        let Some(device) = &self.active_device else {
            self.step = WizardStep::SelectDevice;
            self.status = "No active device. Select a controller first.".to_string();
            return;
        };

        if let WizardStep::Capture { role, .. } = self.step {
            let baseline = device.axes.iter().map(|axis| axis.raw).collect();
            let state = if role == InputRole::Steering && self.advanced_calibration {
                CaptureState::SteeringAdvancedLeft {
                    center: baseline,
                    candidate: None,
                }
            } else {
                CaptureState::Detect {
                    baseline,
                    candidate: None,
                }
            };
            self.step = WizardStep::Capture { role, state };
            self.status = capture_prompt(role, self.advanced_calibration);
        }
    }

    fn update_capture(
        &self,
        role: InputRole,
        baseline: &[u32],
        candidate: Option<&CaptureCandidate>,
    ) -> CaptureUpdate {
        let Some((binding, magnitude)) = self.detect_binding(role, baseline) else {
            return if let Some(candidate) = candidate {
                if self.current_release_magnitude(candidate.binding.axis_index, baseline)
                    <= CAPTURE_RELEASE_THRESHOLD
                {
                    CaptureUpdate::Finished(candidate.binding.clone())
                } else {
                    CaptureUpdate::Candidate(candidate.clone())
                }
            } else {
                CaptureUpdate::Waiting
            };
        };

        if let Some(candidate) = candidate {
            if magnitude <= CAPTURE_RELEASE_THRESHOLD {
                return CaptureUpdate::Finished(candidate.binding.clone());
            }

            if magnitude <= candidate.magnitude {
                return CaptureUpdate::Candidate(candidate.clone());
            }
        }

        CaptureUpdate::Candidate(CaptureCandidate { binding, magnitude })
    }

    fn detect_binding(&self, role: InputRole, baseline: &[u32]) -> Option<(BindingView, f32)> {
        let device = self.active_device.as_ref()?;
        let mut best: Option<(usize, f32)> = None;

        for (index, axis) in device.axes.iter().enumerate() {
            let Some(&idle_raw) = baseline.get(index) else {
                continue;
            };
            let range = axis.max.saturating_sub(axis.min).max(1) as f32;
            let delta = axis.raw as f32 - idle_raw as f32;
            let magnitude = delta.abs() / range;

            if magnitude > best.map(|(_, value)| value).unwrap_or(0.0) {
                best = Some((index, magnitude));
            }
        }

        let (axis_index, magnitude) = best?;
        if magnitude < CAPTURE_START_THRESHOLD {
            return None;
        }

        let axis = &device.axes[axis_index];
        let idle_raw = baseline[axis_index];
        let active_raw = axis.raw;
        Some((
            BindingView {
                role,
                axis_index,
                axis_label: axis.label,
                idle_raw,
                active_raw,
                calibration: if self.advanced_calibration {
                    BindingCalibration::CustomRange {
                        idle_raw,
                        active_raw,
                    }
                } else {
                    BindingCalibration::DriverRange
                },
            },
            magnitude,
        ))
    }

    fn current_release_magnitude(&self, axis_index: usize, baseline: &[u32]) -> f32 {
        let Some(device) = self.active_device.as_ref() else {
            return 0.0;
        };
        let Some(axis) = device.axes.get(axis_index) else {
            return 0.0;
        };
        let Some(&idle_raw) = baseline.get(axis_index) else {
            return 0.0;
        };

        let range = axis.max.saturating_sub(axis.min).max(1) as f32;
        (axis.raw as f32 - idle_raw as f32).abs() / range
    }

    fn update_steering_edge_capture(
        &self,
        center: &[u32],
        required_axis: Option<usize>,
        candidate: Option<&SteeringCandidate>,
    ) -> SteeringCaptureUpdate {
        let Some((next, magnitude)) = self.detect_steering_candidate(center, required_axis) else {
            return if let Some(candidate) = candidate {
                if self.current_release_magnitude(candidate.axis_index, center)
                    <= CAPTURE_RELEASE_THRESHOLD
                {
                    SteeringCaptureUpdate::Finished(candidate.clone())
                } else {
                    SteeringCaptureUpdate::Candidate(candidate.clone())
                }
            } else {
                SteeringCaptureUpdate::Waiting
            };
        };

        if let Some(candidate) = candidate {
            if magnitude <= CAPTURE_RELEASE_THRESHOLD {
                return SteeringCaptureUpdate::Finished(candidate.clone());
            }

            if magnitude <= candidate.magnitude {
                return SteeringCaptureUpdate::Candidate(candidate.clone());
            }
        }

        SteeringCaptureUpdate::Candidate(next)
    }

    fn detect_steering_candidate(
        &self,
        center: &[u32],
        required_axis: Option<usize>,
    ) -> Option<(SteeringCandidate, f32)> {
        let device = self.active_device.as_ref()?;
        let mut best: Option<(usize, f32)> = None;

        for (index, axis) in device.axes.iter().enumerate() {
            if required_axis.is_some_and(|required| required != index) {
                continue;
            }
            let Some(&center_raw) = center.get(index) else {
                continue;
            };
            let range = axis.max.saturating_sub(axis.min).max(1) as f32;
            let magnitude = (axis.raw as f32 - center_raw as f32).abs() / range;

            if magnitude > best.map(|(_, value)| value).unwrap_or(0.0) {
                best = Some((index, magnitude));
            }
        }

        let (axis_index, magnitude) = best?;
        if magnitude < CAPTURE_START_THRESHOLD {
            return None;
        }

        let axis = &device.axes[axis_index];
        Some((
            SteeringCandidate {
                axis_index,
                axis_label: axis.label,
                raw: axis.raw,
                magnitude,
            },
            magnitude,
        ))
    }

    fn finish_binding(&mut self, binding: BindingView) {
        match binding.role {
            InputRole::Throttle => {
                self.bindings.throttle = Some(binding);
                self.step = WizardStep::Capture {
                    role: InputRole::Brake,
                    state: CaptureState::Idle,
                };
                self.status =
                    "Throttle detected. Release all pedals, then click Capture brake.".to_string();
            }
            InputRole::Brake => {
                self.bindings.brake = Some(binding);
                self.step = WizardStep::Capture {
                    role: InputRole::Steering,
                    state: CaptureState::Idle,
                };
                self.status =
                    "Brake detected. Center steering, then click Capture steering.".to_string();
            }
            InputRole::Steering => {
                self.bindings.steering = Some(binding);
                self.step = WizardStep::Ready;
                self.clear_live_state();
                self.status =
                    "Pedal and steering inputs are configured. Overlay input is ready.".to_string();
            }
        }
    }

    fn finish_advanced_steering(
        &mut self,
        center: Vec<u32>,
        left: SteeringEdge,
        right: SteeringEdge,
    ) {
        let Some(&center_raw) = center.get(left.axis_index) else {
            self.reset_steering_capture("Steering center sample was invalid. Try capture again.");
            return;
        };
        if left.axis_index != right.axis_index
            || !Self::valid_steering_range(center_raw, left.raw, right.raw)
        {
            self.reset_steering_capture(
                "Steering range was invalid. Center the wheel and capture steering again.",
            );
            return;
        }

        self.finish_binding(BindingView {
            role: InputRole::Steering,
            axis_index: left.axis_index,
            axis_label: left.axis_label,
            idle_raw: center_raw,
            active_raw: right.raw,
            calibration: BindingCalibration::SteeringCustomRange {
                center_raw,
                left_raw: left.raw,
                right_raw: right.raw,
            },
        });
    }

    fn reset_steering_capture(&mut self, status: &str) {
        self.bindings.steering = None;
        self.step = WizardStep::Capture {
            role: InputRole::Steering,
            state: CaptureState::Idle,
        };
        self.status = status.to_string();
    }

    fn valid_steering_range(center_raw: u32, left_raw: u32, right_raw: u32) -> bool {
        (left_raw < center_raw && center_raw < right_raw)
            || (right_raw < center_raw && center_raw < left_raw)
    }

    fn live_values(&self) -> Vec<(InputRole, f32)> {
        let Some(device) = &self.active_device else {
            return Vec::new();
        };

        let mut values = Vec::new();
        if let Some(binding) = &self.bindings.throttle {
            values.push((InputRole::Throttle, binding.value(&device.axes)));
        }
        if let Some(binding) = &self.bindings.brake {
            values.push((InputRole::Brake, binding.value(&device.axes)));
        }
        if let Some(binding) = &self.bindings.steering {
            values.push((InputRole::Steering, binding.value(&device.axes)));
        }
        values
    }

    fn matching_device_index(&self, profile: &StoredProfile) -> Option<usize> {
        self.devices
            .iter()
            .position(|device| device.id == profile.device_id && device.name == profile.device_name)
            .or_else(|| {
                self.devices
                    .iter()
                    .position(|device| device.name == profile.device_name)
            })
            .or_else(|| {
                self.devices
                    .iter()
                    .position(|device| device.id == profile.device_id)
            })
    }

    fn binding_from_profile(
        role: InputRole,
        stored: &StoredBinding,
        device: &DeviceSnapshot,
    ) -> Option<BindingView> {
        if stored.idle_raw == stored.active_raw {
            return None;
        }

        let axis = device.axes.get(stored.axis_index)?;
        Some(BindingView {
            role,
            axis_index: stored.axis_index,
            axis_label: axis.label,
            idle_raw: stored.idle_raw,
            active_raw: stored.active_raw,
            calibration: Self::calibration_from_profile(&stored.calibration)?,
        })
    }

    fn binding_to_profile(binding: &BindingView) -> StoredBinding {
        StoredBinding {
            axis_index: binding.axis_index,
            axis_label: binding.axis_label.to_string(),
            idle_raw: binding.idle_raw,
            active_raw: binding.active_raw,
            calibration: Self::calibration_to_profile(&binding.calibration),
        }
    }

    fn calibration_from_profile(stored: &StoredCalibration) -> Option<BindingCalibration> {
        match stored {
            StoredCalibration::DriverRange => Some(BindingCalibration::DriverRange),
            StoredCalibration::CustomRange {
                idle_raw,
                active_raw,
            } if idle_raw != active_raw => Some(BindingCalibration::CustomRange {
                idle_raw: *idle_raw,
                active_raw: *active_raw,
            }),
            StoredCalibration::CustomRange { .. } => None,
            StoredCalibration::SteeringCustomRange {
                center_raw,
                left_raw,
                right_raw,
            } if Self::valid_steering_range(*center_raw, *left_raw, *right_raw) => {
                Some(BindingCalibration::SteeringCustomRange {
                    center_raw: *center_raw,
                    left_raw: *left_raw,
                    right_raw: *right_raw,
                })
            }
            StoredCalibration::SteeringCustomRange { .. } => None,
        }
    }

    fn calibration_to_profile(calibration: &BindingCalibration) -> StoredCalibration {
        match calibration {
            BindingCalibration::DriverRange => StoredCalibration::DriverRange,
            BindingCalibration::CustomRange {
                idle_raw,
                active_raw,
            } => StoredCalibration::CustomRange {
                idle_raw: *idle_raw,
                active_raw: *active_raw,
            },
            BindingCalibration::SteeringCustomRange {
                center_raw,
                left_raw,
                right_raw,
            } => StoredCalibration::SteeringCustomRange {
                center_raw: *center_raw,
                left_raw: *left_raw,
                right_raw: *right_raw,
            },
        }
    }

    fn record_live_values(&mut self) {
        let values = self.live_values();
        let throttle = values
            .iter()
            .find_map(|(role, value)| (*role == InputRole::Throttle).then_some(*value))
            .unwrap_or(0.0);
        let brake = values
            .iter()
            .find_map(|(role, value)| (*role == InputRole::Brake).then_some(*value))
            .unwrap_or(0.0);
        let steering = values
            .iter()
            .find_map(|(role, value)| (*role == InputRole::Steering).then_some(*value))
            .unwrap_or(0.0);

        self.value_history.push(LiveSample {
            throttle,
            brake,
            steering,
        });
        if self.value_history.len() > 180 {
            self.value_history.remove(0);
        }
        self.alert_engine.update(&self.value_history);
    }

    fn clear_live_state(&mut self) {
        self.value_history.clear();
        self.alert_engine.clear();
    }
}

fn alert_settings_from_profile(profile: &StoredProfile) -> AlertSettings {
    AlertSettings {
        enabled: profile.overlay_settings.alerts_enabled,
        sensitivity: match profile.overlay_settings.alert_sensitivity {
            crate::profile::StoredAlertSensitivity::Quiet => AlertSensitivity::Quiet,
            crate::profile::StoredAlertSensitivity::Balanced => AlertSensitivity::Balanced,
            crate::profile::StoredAlertSensitivity::Sensitive => AlertSensitivity::Sensitive,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alerts::AlertId;
    use crate::profile::{StoredBinding, StoredProfile};
    use std::cell::RefCell;
    use std::rc::Rc;

    #[derive(Clone)]
    struct TestProvider {
        state: Rc<RefCell<TestProviderState>>,
    }

    struct TestProviderState {
        devices: Vec<DeviceSnapshot>,
        axes: Option<Vec<u32>>,
    }

    impl TestProvider {
        fn new(devices: Vec<DeviceSnapshot>, axes: &[u32]) -> Self {
            Self {
                state: Rc::new(RefCell::new(TestProviderState {
                    devices,
                    axes: Some(axes.to_vec()),
                })),
            }
        }

        fn set_axes(&self, axes: &[u32]) {
            self.state.borrow_mut().axes = Some(axes.to_vec());
        }

        fn disconnect(&self) {
            self.state.borrow_mut().axes = None;
        }
    }

    impl DeviceProvider for TestProvider {
        fn enumerate_devices(&self) -> Vec<DeviceSnapshot> {
            self.state.borrow().devices.clone()
        }

        fn read_axes(&self, device_id: u32) -> Option<Vec<u32>> {
            let state = self.state.borrow();
            state
                .devices
                .iter()
                .any(|device| device.id == device_id)
                .then(|| state.axes.clone())
                .flatten()
        }
    }

    fn test_device(id: u32, name: &str, raw: &[u32]) -> DeviceSnapshot {
        let labels = ["X", "Y", "Z", "R", "U", "V"];
        DeviceSnapshot {
            id,
            name: name.to_string(),
            axes: raw
                .iter()
                .enumerate()
                .map(|(index, raw)| AxisSnapshot::new(labels[index], 0, 1000, *raw))
                .collect(),
        }
    }

    fn assert_near(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 0.001,
            "expected {expected}, got {actual}"
        );
    }

    fn value_for(values: &[(InputRole, f32)], role: InputRole) -> f32 {
        values
            .iter()
            .find_map(|(candidate, value)| (*candidate == role).then_some(*value))
            .unwrap()
    }

    fn profile_for(device_id: u32, device_name: &str) -> StoredProfile {
        StoredProfile {
            device_id,
            device_name: device_name.to_string(),
            throttle: StoredBinding {
                axis_index: 0,
                axis_label: "X".to_string(),
                idle_raw: 0,
                active_raw: 1000,
                calibration: StoredCalibration::DriverRange,
            },
            brake: StoredBinding {
                axis_index: 1,
                axis_label: "Y".to_string(),
                idle_raw: 1000,
                active_raw: 0,
                calibration: StoredCalibration::DriverRange,
            },
            steering: Some(StoredBinding {
                axis_index: 0,
                axis_label: "X".to_string(),
                idle_raw: 500,
                active_raw: 1000,
                calibration: StoredCalibration::DriverRange,
            }),
            overlay_settings: Default::default(),
        }
    }

    #[test]
    fn axis_percent_clamps_and_handles_invalid_range() {
        assert_near(AxisSnapshot::new("X", 100, 900, 500).percent(), 0.5);
        assert_near(AxisSnapshot::new("X", 100, 900, 1200).percent(), 1.0);
        assert_near(AxisSnapshot::new("X", 100, 900, 20).percent(), 0.0);
        assert_near(AxisSnapshot::new("X", 100, 100, 100).percent(), 0.0);
    }

    #[test]
    fn binding_value_supports_reversed_axes() {
        let binding = BindingView {
            role: InputRole::Brake,
            axis_index: 0,
            axis_label: "Y",
            idle_raw: 900,
            active_raw: 100,
            calibration: BindingCalibration::DriverRange,
        };

        assert_near(binding.value(&[AxisSnapshot::new("Y", 0, 1000, 500)]), 0.5);
        assert_near(binding.value(&[AxisSnapshot::new("Y", 0, 1000, 1000)]), 0.0);
        assert_near(binding.value(&[AxisSnapshot::new("Y", 0, 1000, 0)]), 1.0);
    }

    #[test]
    fn binding_value_supports_advanced_custom_calibration() {
        let binding = BindingView {
            role: InputRole::Throttle,
            axis_index: 0,
            axis_label: "X",
            idle_raw: 0,
            active_raw: 1000,
            calibration: BindingCalibration::CustomRange {
                idle_raw: 100,
                active_raw: 700,
            },
        };

        assert_near(binding.value(&[AxisSnapshot::new("X", 0, 1000, 100)]), 0.0);
        assert_near(binding.value(&[AxisSnapshot::new("X", 0, 1000, 400)]), 0.5);
        assert_near(binding.value(&[AxisSnapshot::new("X", 0, 1000, 700)]), 1.0);
        assert_near(binding.value(&[AxisSnapshot::new("X", 0, 1000, 900)]), 1.0);
    }

    #[test]
    fn steering_driver_range_maps_full_axis() {
        let binding = BindingView {
            role: InputRole::Steering,
            axis_index: 0,
            axis_label: "X",
            idle_raw: 500,
            active_raw: 700,
            calibration: BindingCalibration::DriverRange,
        };

        assert_near(binding.value(&[AxisSnapshot::new("X", 0, 1000, 0)]), -1.0);
        assert_near(binding.value(&[AxisSnapshot::new("X", 0, 1000, 500)]), 0.0);
        assert_near(binding.value(&[AxisSnapshot::new("X", 0, 1000, 1000)]), 1.0);
    }

    #[test]
    fn steering_driver_range_supports_reversed_direction() {
        let binding = BindingView {
            role: InputRole::Steering,
            axis_index: 0,
            axis_label: "X",
            idle_raw: 500,
            active_raw: 300,
            calibration: BindingCalibration::DriverRange,
        };

        assert_near(binding.value(&[AxisSnapshot::new("X", 0, 1000, 0)]), 1.0);
        assert_near(binding.value(&[AxisSnapshot::new("X", 0, 1000, 500)]), 0.0);
        assert_near(
            binding.value(&[AxisSnapshot::new("X", 0, 1000, 1000)]),
            -1.0,
        );
    }

    #[test]
    fn steering_advanced_calibration_maps_center_left_and_right() {
        let binding = BindingView {
            role: InputRole::Steering,
            axis_index: 0,
            axis_label: "X",
            idle_raw: 500,
            active_raw: 900,
            calibration: BindingCalibration::SteeringCustomRange {
                center_raw: 500,
                left_raw: 100,
                right_raw: 900,
            },
        };

        assert_near(binding.value(&[AxisSnapshot::new("X", 0, 1000, 100)]), -1.0);
        assert_near(binding.value(&[AxisSnapshot::new("X", 0, 1000, 500)]), 0.0);
        assert_near(binding.value(&[AxisSnapshot::new("X", 0, 1000, 900)]), 1.0);
    }

    #[test]
    fn wizard_reports_empty_device_list() {
        let wizard = Wizard::new(TestProvider::new(Vec::new(), &[]));
        let view = wizard.view();

        assert!(view.status.contains("No joystick/HID devices found"));
        match view.step {
            WizardStepView::SelectDevice { devices, .. } => assert!(devices.is_empty()),
            _ => panic!("expected device selection"),
        }
    }

    #[test]
    fn capture_flow_uses_driver_range_not_capture_peak() {
        let provider = TestProvider::new(
            vec![test_device(7, "Test Pedals", &[0, 0, 500])],
            &[0, 0, 500],
        );
        let mut wizard = Wizard::new(provider.clone());

        wizard.handle_command(WizardCommand::Confirm);
        wizard.handle_command(WizardCommand::Confirm);

        provider.set_axes(&[620, 35, 500]);
        wizard.update();
        provider.set_axes(&[0, 0, 500]);
        wizard.update();

        wizard.handle_command(WizardCommand::Confirm);
        provider.set_axes(&[20, 760, 500]);
        wizard.update();
        provider.set_axes(&[0, 0, 500]);
        wizard.update();

        wizard.handle_command(WizardCommand::Confirm);
        provider.set_axes(&[0, 0, 700]);
        wizard.update();
        provider.set_axes(&[0, 0, 500]);
        wizard.update();

        let profile = wizard.profile().unwrap();
        assert_eq!(profile.throttle.axis_index, 0);
        assert_eq!(profile.throttle.idle_raw, 0);
        assert_eq!(profile.throttle.active_raw, 620);
        assert_eq!(profile.brake.axis_index, 1);
        assert_eq!(profile.brake.idle_raw, 0);
        assert_eq!(profile.brake.active_raw, 760);
        let steering = profile.steering.unwrap();
        assert_eq!(steering.axis_index, 2);
        assert_eq!(steering.idle_raw, 500);
        assert_eq!(steering.active_raw, 700);

        provider.set_axes(&[620, 760, 700]);
        wizard.update();

        match wizard.view().step {
            WizardStepView::Ready {
                values, history, ..
            } => {
                assert_near(value_for(&values, InputRole::Throttle), 0.62);
                assert_near(value_for(&values, InputRole::Brake), 0.76);
                assert_near(value_for(&values, InputRole::Steering), 0.4);
                assert_eq!(history.len(), 1);
                assert_near(history[0].throttle, 0.62);
                assert_near(history[0].brake, 0.76);
                assert_near(history[0].steering, 0.4);
            }
            _ => panic!("expected ready state"),
        }

        provider.set_axes(&[1000, 1000, 1000]);
        wizard.update();

        match wizard.view().step {
            WizardStepView::Ready { values, .. } => {
                assert_near(value_for(&values, InputRole::Throttle), 1.0);
                assert_near(value_for(&values, InputRole::Brake), 1.0);
                assert_near(value_for(&values, InputRole::Steering), 1.0);
            }
            _ => panic!("expected ready state"),
        }
    }

    #[test]
    fn advanced_calibration_saves_custom_range() {
        let provider = TestProvider::new(
            vec![test_device(7, "Test Pedals", &[0, 0, 500])],
            &[0, 0, 500],
        );
        let mut wizard = Wizard::new(provider.clone());

        wizard.handle_command(WizardCommand::Confirm);
        wizard.handle_command(WizardCommand::ToggleAdvancedCalibration);
        match wizard.view().step {
            WizardStepView::Capture {
                advanced_calibration,
                ..
            } => assert!(advanced_calibration),
            _ => panic!("expected capture state"),
        }

        wizard.handle_command(WizardCommand::Confirm);
        provider.set_axes(&[600, 0, 500]);
        wizard.update();
        provider.set_axes(&[0, 0, 500]);
        wizard.update();

        wizard.handle_command(WizardCommand::Confirm);
        provider.set_axes(&[0, 800, 500]);
        wizard.update();
        provider.set_axes(&[0, 0, 500]);
        wizard.update();

        wizard.handle_command(WizardCommand::Confirm);
        provider.set_axes(&[0, 0, 100]);
        wizard.update();
        provider.set_axes(&[0, 0, 500]);
        wizard.update();
        provider.set_axes(&[0, 0, 900]);
        wizard.update();
        provider.set_axes(&[0, 0, 500]);
        wizard.update();

        let profile = wizard.profile().unwrap();
        assert_eq!(
            profile.throttle.calibration,
            StoredCalibration::CustomRange {
                idle_raw: 0,
                active_raw: 600
            }
        );
        assert_eq!(
            profile.brake.calibration,
            StoredCalibration::CustomRange {
                idle_raw: 0,
                active_raw: 800
            }
        );
        assert_eq!(
            profile.steering.unwrap().calibration,
            StoredCalibration::SteeringCustomRange {
                center_raw: 500,
                left_raw: 100,
                right_raw: 900
            }
        );

        provider.set_axes(&[300, 400, 700]);
        wizard.update();

        match wizard.view().step {
            WizardStepView::Ready { values, .. } => {
                assert_near(value_for(&values, InputRole::Throttle), 0.5);
                assert_near(value_for(&values, InputRole::Brake), 0.5);
                assert_near(value_for(&values, InputRole::Steering), 0.5);
            }
            _ => panic!("expected ready state"),
        }
    }

    #[test]
    fn normal_steering_capture_detects_reversed_direction() {
        let provider = TestProvider::new(
            vec![test_device(7, "Test Pedals", &[0, 0, 500])],
            &[0, 0, 500],
        );
        let mut wizard = Wizard::new(provider.clone());

        wizard.handle_command(WizardCommand::Confirm);
        wizard.handle_command(WizardCommand::Confirm);
        provider.set_axes(&[600, 0, 500]);
        wizard.update();
        provider.set_axes(&[0, 0, 500]);
        wizard.update();

        wizard.handle_command(WizardCommand::Confirm);
        provider.set_axes(&[0, 700, 500]);
        wizard.update();
        provider.set_axes(&[0, 0, 500]);
        wizard.update();

        wizard.handle_command(WizardCommand::Confirm);
        provider.set_axes(&[0, 0, 300]);
        wizard.update();
        provider.set_axes(&[0, 0, 500]);
        wizard.update();

        provider.set_axes(&[0, 0, 0]);
        wizard.update();
        match wizard.view().step {
            WizardStepView::Ready { values, .. } => {
                assert_near(value_for(&values, InputRole::Steering), 1.0);
            }
            _ => panic!("expected ready state"),
        }
    }

    #[test]
    fn restore_profile_matches_device_by_name_when_id_changes() {
        let provider =
            TestProvider::new(vec![test_device(99, "Saved Pedals", &[0, 0])], &[500, 250]);
        let mut wizard = Wizard::new(provider);

        assert!(wizard.restore_profile(&profile_for(7, "Saved Pedals")));

        match wizard.view().step {
            WizardStepView::Ready { device, values, .. } => {
                assert_eq!(device.unwrap().id, 99);
                assert_near(value_for(&values, InputRole::Throttle), 0.5);
                assert_near(value_for(&values, InputRole::Brake), 0.75);
            }
            _ => panic!("expected ready state"),
        }
    }

    #[test]
    fn restore_profile_can_fallback_to_id_when_display_name_changes() {
        let provider = TestProvider::new(
            vec![test_device(7, "Better OEM Name", &[0, 0])],
            &[500, 250],
        );
        let mut wizard = Wizard::new(provider);

        assert!(wizard.restore_profile(&profile_for(7, "Microsoft PC-joystick driver")));

        match wizard.view().step {
            WizardStepView::Ready { device, .. } => {
                assert_eq!(device.unwrap().name, "Better OEM Name");
            }
            _ => panic!("expected ready state"),
        }
    }

    #[test]
    fn restore_old_profile_without_steering_lands_on_steering_capture() {
        let provider =
            TestProvider::new(vec![test_device(7, "Saved Pedals", &[0, 0])], &[500, 250]);
        let mut wizard = Wizard::new(provider);
        let mut profile = profile_for(7, "Saved Pedals");
        profile.steering = None;

        assert!(wizard.restore_profile(&profile));

        match wizard.view().step {
            WizardStepView::Capture { role, bindings, .. } => {
                assert_eq!(role, InputRole::Steering);
                assert!(bindings.throttle.is_some());
                assert!(bindings.brake.is_some());
                assert!(bindings.steering.is_none());
            }
            _ => panic!("expected steering capture state"),
        }
        assert!(wizard.profile().is_none());
    }

    #[test]
    fn restore_profile_rejects_invalid_binding() {
        let provider = TestProvider::new(vec![test_device(3, "Saved Pedals", &[0, 0])], &[0, 0]);
        let mut wizard = Wizard::new(provider);
        let mut profile = profile_for(3, "Saved Pedals");
        profile.throttle.active_raw = profile.throttle.idle_raw;

        assert!(!wizard.restore_profile(&profile));
        assert!(wizard.status.contains("throttle binding is invalid"));
        assert!(matches!(
            wizard.view().step,
            WizardStepView::SelectDevice { .. }
        ));
    }

    #[test]
    fn disconnected_device_returns_to_selection_and_clears_profile() {
        let provider =
            TestProvider::new(vec![test_device(3, "Saved Pedals", &[0, 0])], &[250, 500]);
        let mut wizard = Wizard::new(provider.clone());

        assert!(wizard.restore_profile(&profile_for(3, "Saved Pedals")));
        provider.disconnect();
        wizard.update();

        assert!(wizard.profile().is_none());
        assert!(wizard.status.contains("Lost selected device"));
        assert!(matches!(
            wizard.view().step,
            WizardStepView::SelectDevice { .. }
        ));
    }

    #[test]
    fn ready_history_keeps_last_180_samples() {
        let provider =
            TestProvider::new(vec![test_device(5, "History Pedals", &[0, 0])], &[0, 1000]);
        let mut wizard = Wizard::new(provider.clone());

        assert!(wizard.restore_profile(&profile_for(5, "History Pedals")));
        for index in 0..220 {
            provider.set_axes(&[(index % 1000) as u32, 1000 - (index % 1000) as u32]);
            wizard.update();
        }

        match wizard.view().step {
            WizardStepView::Ready { history, .. } => assert_eq!(history.len(), 180),
            _ => panic!("expected ready state"),
        }
    }

    #[test]
    fn ready_view_reports_alerts_from_live_history() {
        let provider = TestProvider::new(
            vec![test_device(6, "Alert Pedals", &[0, 0, 500])],
            &[0, 0, 500],
        );
        let mut wizard = Wizard::new(provider.clone());

        assert!(wizard.restore_profile(&profile_for(6, "Alert Pedals")));
        for _ in 0..20 {
            provider.set_axes(&[200, 200, 500]);
            wizard.update();
        }

        match wizard.view().step {
            WizardStepView::Ready { alerts, .. } => {
                assert!(alerts.iter().any(|alert| alert.id == AlertId::PedalOverlap))
            }
            _ => panic!("expected ready state"),
        }
    }

    #[test]
    fn alert_settings_disable_ready_alerts() {
        let provider = TestProvider::new(
            vec![test_device(6, "Alert Pedals", &[0, 0, 500])],
            &[0, 0, 500],
        );
        let mut wizard = Wizard::new(provider.clone());
        let mut profile = profile_for(6, "Alert Pedals");
        profile.overlay_settings.alerts_enabled = false;

        assert!(wizard.restore_profile(&profile));
        for _ in 0..20 {
            provider.set_axes(&[200, 200, 500]);
            wizard.update();
        }

        match wizard.view().step {
            WizardStepView::Ready { alerts, .. } => assert!(alerts.is_empty()),
            _ => panic!("expected ready state"),
        }
    }
}
