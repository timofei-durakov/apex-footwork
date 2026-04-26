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
}

impl InputRole {
    pub fn label(self) -> &'static str {
        match self {
            InputRole::Throttle => "Throttle",
            InputRole::Brake => "Brake",
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
}

impl BindingView {
    fn value(&self, axes: &[AxisSnapshot]) -> f32 {
        let Some(axis) = axes.get(self.axis_index) else {
            return 0.0;
        };

        let idle = self.idle_raw as f32;
        let active = self.active_raw as f32;
        let raw = axis.raw as f32;
        let span = active - idle;
        if span.abs() < 1.0 {
            return 0.0;
        }

        ((raw - idle) / span).clamp(0.0, 1.0)
    }
}

#[derive(Clone, Debug, Default)]
pub struct PedalBindings {
    pub throttle: Option<BindingView>,
    pub brake: Option<BindingView>,
}

impl PedalBindings {
    #[allow(dead_code)]
    pub fn is_complete(&self) -> bool {
        self.throttle.is_some() && self.brake.is_some()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WizardCommand {
    SelectDevice(usize),
    Confirm,
    Restart,
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
        device: Option<DeviceSnapshot>,
        bindings: PedalBindings,
    },
    Ready {
        device: Option<DeviceSnapshot>,
        bindings: PedalBindings,
        values: Vec<(InputRole, f32)>,
        history: Vec<(f32, f32)>,
    },
}

impl WizardStepView {
    pub fn title(&self) -> &'static str {
        match self {
            WizardStepView::SelectDevice { .. } => "Step 1/3: Choose device",
            WizardStepView::Capture {
                role: InputRole::Throttle,
                ..
            } => "Step 2/3: Detect throttle",
            WizardStepView::Capture {
                role: InputRole::Brake,
                ..
            } => "Step 3/3: Detect brake",
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

enum CaptureUpdate {
    Waiting,
    Candidate(CaptureCandidate),
    Finished(BindingView),
}

enum WizardStep {
    SelectDevice,
    Capture {
        role: InputRole,
        armed: bool,
        baseline: Vec<u32>,
        candidate: Option<CaptureCandidate>,
    },
    Ready,
}

const CAPTURE_START_THRESHOLD: f32 = 0.18;
const CAPTURE_RELEASE_THRESHOLD: f32 = 0.06;

pub struct Wizard<P: DeviceProvider> {
    provider: P,
    devices: Vec<DeviceSnapshot>,
    selected_index: usize,
    active_device: Option<DeviceSnapshot>,
    step: WizardStep,
    bindings: PedalBindings,
    value_history: Vec<(f32, f32)>,
    status: String,
    tick: u32,
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
            status,
            tick: 0,
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

        if let WizardStep::Capture {
            role,
            armed: true,
            baseline,
            candidate,
        } = &self.step
        {
            let role = *role;
            let baseline = baseline.clone();
            let candidate = candidate.clone();

            match self.update_capture(role, &baseline, candidate.as_ref()) {
                CaptureUpdate::Waiting => {}
                CaptureUpdate::Candidate(candidate) => {
                    if let WizardStep::Capture {
                        candidate: current, ..
                    } = &mut self.step
                    {
                        *current = Some(candidate);
                    }
                    self.status = format!(
                        "{} movement detected. Release the pedal to save the full press.",
                        role.label()
                    );
                }
                CaptureUpdate::Finished(binding) => {
                    self.finish_binding(binding);
                }
            }
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
            WizardCommand::Restart => self.restart(),
            _ => {}
        }
    }

    pub fn view(&self) -> WizardView {
        let step = match &self.step {
            WizardStep::SelectDevice => WizardStepView::SelectDevice {
                devices: self.devices.clone(),
                selected_index: self.selected_index,
            },
            WizardStep::Capture { role, armed, .. } => WizardStepView::Capture {
                role: *role,
                armed: *armed,
                device: self.active_device.clone(),
                bindings: self.bindings.clone(),
            },
            WizardStep::Ready => WizardStepView::Ready {
                device: self.active_device.clone(),
                bindings: self.bindings.clone(),
                values: self.live_values(),
                history: self.value_history.clone(),
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

    fn confirm(&mut self) {
        match self.step {
            WizardStep::SelectDevice => self.choose_selected_device(),
            WizardStep::Capture { armed: false, .. } => self.arm_capture(),
            WizardStep::Capture {
                armed: true, role, ..
            } => {
                self.status = format!("Still waiting for {} movement.", role.label());
            }
            WizardStep::Ready => {}
        }
    }

    fn restart(&mut self) {
        self.active_device = None;
        self.bindings = PedalBindings::default();
        self.value_history.clear();
        self.step = WizardStep::SelectDevice;
        self.refresh_devices();
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
        self.value_history.clear();
        self.step = WizardStep::Capture {
            role: InputRole::Throttle,
            armed: false,
            baseline: Vec::new(),
            candidate: None,
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
            self.step = WizardStep::Capture {
                role,
                armed: true,
                baseline,
                candidate: None,
            };
            self.status = format!(
                "Press {} fully, then release it. The peak will become 100%.",
                role.label()
            );
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
        Some((
            BindingView {
                role,
                axis_index,
                axis_label: axis.label,
                idle_raw: baseline[axis_index],
                active_raw: axis.raw,
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

    fn finish_binding(&mut self, binding: BindingView) {
        match binding.role {
            InputRole::Throttle => {
                self.bindings.throttle = Some(binding);
                self.step = WizardStep::Capture {
                    role: InputRole::Brake,
                    armed: false,
                    baseline: Vec::new(),
                    candidate: None,
                };
                self.status =
                    "Throttle detected. Release all pedals, then click Capture brake.".to_string();
            }
            InputRole::Brake => {
                self.bindings.brake = Some(binding);
                self.step = WizardStep::Ready;
                self.value_history.clear();
                self.status =
                    "Pedal inputs are configured. Press R to run the wizard again.".to_string();
            }
        }
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
        values
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

        self.value_history.push((throttle, brake));
        if self.value_history.len() > 180 {
            self.value_history.remove(0);
        }
    }
}
