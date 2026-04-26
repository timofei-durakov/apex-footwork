#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod wizard;

use std::ffi::c_void;
use std::mem::{size_of, zeroed};
use std::ptr::{null, null_mut};
use std::sync::{Mutex, OnceLock};
use wizard::{
    AxisSnapshot, BindingView, DeviceProvider, DeviceSnapshot, InputRole, PedalBindings, Wizard,
    WizardCommand, WizardStepView, WizardView,
};

type Bool = i32;
type Dword = u32;
type Hbrush = isize;
type Hcursor = isize;
type Hdc = isize;
type Hinstance = isize;
type Hwnd = isize;
type Lparam = isize;
type Lresult = isize;
type Uint = u32;
type Wparam = usize;

const JOYERR_NOERROR: Uint = 0;
const JOY_RETURNX: Dword = 0x0000_0001;
const JOY_RETURNY: Dword = 0x0000_0002;
const JOY_RETURNZ: Dword = 0x0000_0004;
const JOY_RETURNR: Dword = 0x0000_0008;
const JOY_RETURNU: Dword = 0x0000_0010;
const JOY_RETURNV: Dword = 0x0000_0020;
const JOY_RETURNPOV: Dword = 0x0000_0040;
const JOY_RETURNBUTTONS: Dword = 0x0000_0080;
const JOY_RETURNALL: Dword = JOY_RETURNX
    | JOY_RETURNY
    | JOY_RETURNZ
    | JOY_RETURNR
    | JOY_RETURNU
    | JOY_RETURNV
    | JOY_RETURNPOV
    | JOY_RETURNBUTTONS;

const WM_CREATE: Uint = 0x0001;
const WM_DESTROY: Uint = 0x0002;
const WM_PAINT: Uint = 0x000F;
const WM_COMMAND: Uint = 0x0111;
const WM_TIMER: Uint = 0x0113;
const WM_KEYDOWN: Uint = 0x0100;
const VK_ESCAPE: Wparam = 0x1B;
const VK_RETURN: Wparam = 0x0D;
const KEY_R: Wparam = 0x52;
const SW_HIDE: i32 = 0;
const SW_SHOW: i32 = 5;
const CS_HREDRAW: Uint = 0x0002;
const CS_VREDRAW: Uint = 0x0001;
const COLOR_WINDOW: isize = 5;
const DT_LEFT: Uint = 0x0000;
const DT_TOP: Uint = 0x0000;
const DT_SINGLELINE: Uint = 0x0020;
const WS_CHILD: Dword = 0x4000_0000;
const WS_TABSTOP: Dword = 0x0001_0000;
const WS_VSCROLL: Dword = 0x0020_0000;
const CBS_DROPDOWNLIST: Dword = 0x0003;
const BS_DEFPUSHBUTTON: Dword = 0x0001;
const BS_PUSHBUTTON: Dword = 0x0000;
const CBN_SELCHANGE: u16 = 1;
const BN_CLICKED: u16 = 0;
const CB_ADDSTRING: Uint = 0x0143;
const CB_GETCURSEL: Uint = 0x0147;
const CB_RESETCONTENT: Uint = 0x014B;
const CB_SETCURSEL: Uint = 0x014E;
const CB_ERR: Lresult = -1;
const IDC_DEVICE_COMBO: u16 = 1001;
const IDC_PRIMARY_BUTTON: u16 = 1002;
const IDC_RESTART_BUTTON: u16 = 1003;

#[repr(C)]
struct JoyCapsW {
    w_mid: u16,
    w_pid: u16,
    sz_pname: [u16; 32],
    w_xmin: Uint,
    w_xmax: Uint,
    w_ymin: Uint,
    w_ymax: Uint,
    w_zmin: Uint,
    w_zmax: Uint,
    w_num_buttons: Uint,
    w_period_min: Uint,
    w_period_max: Uint,
    w_rmin: Uint,
    w_rmax: Uint,
    w_umin: Uint,
    w_umax: Uint,
    w_vmin: Uint,
    w_vmax: Uint,
    w_caps: Uint,
    w_max_axes: Uint,
    w_num_axes: Uint,
    w_max_buttons: Uint,
    sz_reg_key: [u16; 32],
    sz_oem_vx_d: [u16; 260],
}

#[repr(C)]
struct JoyInfoEx {
    dw_size: Dword,
    dw_flags: Dword,
    dw_xpos: Dword,
    dw_ypos: Dword,
    dw_zpos: Dword,
    dw_rpos: Dword,
    dw_upos: Dword,
    dw_vpos: Dword,
    dw_buttons: Dword,
    dw_button_number: Dword,
    dw_pov: Dword,
    dw_reserved1: Dword,
    dw_reserved2: Dword,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Point {
    x: i32,
    y: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Rect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[repr(C)]
struct PaintStruct {
    hdc: Hdc,
    f_erase: Bool,
    rc_paint: Rect,
    f_restore: Bool,
    f_inc_update: Bool,
    rgb_reserved: [u8; 32],
}

#[repr(C)]
struct Msg {
    hwnd: Hwnd,
    message: Uint,
    w_param: Wparam,
    l_param: Lparam,
    time: Dword,
    pt: Point,
}

#[repr(C)]
struct WndClassW {
    style: Uint,
    lpfn_wnd_proc: Option<unsafe extern "system" fn(Hwnd, Uint, Wparam, Lparam) -> Lresult>,
    cb_cls_extra: i32,
    cb_wnd_extra: i32,
    h_instance: Hinstance,
    h_icon: isize,
    h_cursor: Hcursor,
    hbr_background: Hbrush,
    lpsz_menu_name: *const u16,
    lpsz_class_name: *const u16,
}

#[link(name = "winmm")]
unsafe extern "system" {
    fn joyGetNumDevs() -> Uint;
    fn joyGetDevCapsW(uJoyID: Uint, pjc: *mut JoyCapsW, cbjc: Uint) -> Uint;
    fn joyGetPosEx(uJoyID: Uint, pji: *mut JoyInfoEx) -> Uint;
}

#[link(name = "user32")]
unsafe extern "system" {
    fn BeginPaint(hwnd: Hwnd, lpPaint: *mut PaintStruct) -> Hdc;
    fn CreateWindowExW(
        dwExStyle: Dword,
        lpClassName: *const u16,
        lpWindowName: *const u16,
        dwStyle: Dword,
        X: i32,
        Y: i32,
        nWidth: i32,
        nHeight: i32,
        hWndParent: Hwnd,
        hMenu: isize,
        hInstance: Hinstance,
        lpParam: *mut c_void,
    ) -> Hwnd;
    fn DefWindowProcW(hwnd: Hwnd, msg: Uint, wparam: Wparam, lparam: Lparam) -> Lresult;
    fn DestroyWindow(hwnd: Hwnd) -> Bool;
    fn DispatchMessageW(lpMsg: *const Msg) -> Lresult;
    fn DrawTextW(
        hdc: Hdc,
        lpchText: *const u16,
        cchText: i32,
        lprc: *mut Rect,
        format: Uint,
    ) -> i32;
    fn EndPaint(hwnd: Hwnd, lpPaint: *const PaintStruct) -> Bool;
    fn EnableWindow(hWnd: Hwnd, bEnable: Bool) -> Bool;
    fn FillRect(hDC: Hdc, lprc: *const Rect, hbr: Hbrush) -> i32;
    fn GetClientRect(hWnd: Hwnd, lpRect: *mut Rect) -> Bool;
    fn GetMessageW(lpMsg: *mut Msg, hWnd: Hwnd, wMsgFilterMin: Uint, wMsgFilterMax: Uint) -> Bool;
    fn InvalidateRect(hWnd: Hwnd, lpRect: *const Rect, bErase: Bool) -> Bool;
    fn LoadCursorW(hInstance: Hinstance, lpCursorName: *const u16) -> Hcursor;
    fn PostQuitMessage(nExitCode: i32);
    fn RegisterClassW(lpWndClass: *const WndClassW) -> u16;
    fn SendMessageW(hWnd: Hwnd, Msg: Uint, wParam: Wparam, lParam: Lparam) -> Lresult;
    fn SetTimer(hWnd: Hwnd, nIDEvent: usize, uElapse: Uint, lpTimerFunc: *const c_void) -> usize;
    fn SetWindowTextW(hWnd: Hwnd, lpString: *const u16) -> Bool;
    fn ShowWindow(hWnd: Hwnd, nCmdShow: i32) -> Bool;
    fn TranslateMessage(lpMsg: *const Msg) -> Bool;
    fn UpdateWindow(hWnd: Hwnd) -> Bool;
}

#[link(name = "gdi32")]
unsafe extern "system" {
    fn CreateSolidBrush(color: Dword) -> Hbrush;
    fn DeleteObject(ho: isize) -> Bool;
    fn SetBkMode(hdc: Hdc, mode: i32) -> i32;
    fn SetTextColor(hdc: Hdc, color: Dword) -> Dword;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetModuleHandleW(lpModuleName: *const u16) -> Hinstance;
}

struct WinmmDeviceProvider;

impl DeviceProvider for WinmmDeviceProvider {
    fn enumerate_devices(&self) -> Vec<DeviceSnapshot> {
        let count = unsafe { joyGetNumDevs() };
        let mut devices = Vec::new();

        for id in 0..count {
            let mut caps: JoyCapsW = unsafe { zeroed() };
            let result = unsafe { joyGetDevCapsW(id, &mut caps, size_of::<JoyCapsW>() as u32) };
            if result != JOYERR_NOERROR {
                continue;
            }

            let mut device = DeviceSnapshot {
                id,
                name: name_from_wide(&caps.sz_pname),
                axes: axes_from_caps(&caps),
            };

            if let Some(values) = self.read_axes(id) {
                for (axis, raw) in device.axes.iter_mut().zip(values) {
                    axis.raw = raw;
                }
            }

            devices.push(device);
        }

        devices
    }

    fn read_axes(&self, device_id: u32) -> Option<Vec<u32>> {
        read_axis_values(device_id).map(Vec::from)
    }
}

static STATE: OnceLock<Mutex<Wizard<WinmmDeviceProvider>>> = OnceLock::new();
static CONTROLS: OnceLock<Mutex<UiControls>> = OnceLock::new();

struct UiControls {
    device_combo: Hwnd,
    primary_button: Hwnd,
    restart_button: Hwnd,
    device_signature: Vec<(u32, String)>,
    combo_selected: Option<usize>,
    combo_visible: bool,
    primary_visible: bool,
    primary_enabled: bool,
    primary_text: String,
    restart_visible: bool,
}

fn main() {
    STATE.get_or_init(|| Mutex::new(Wizard::new(WinmmDeviceProvider)));
    unsafe { run_window() };
}

unsafe fn run_window() {
    let class_name = wide("ApexFootworkWindow");
    let title = wide("Apex Footwork setup");
    let h_instance = unsafe { GetModuleHandleW(null()) };
    let cursor = unsafe { LoadCursorW(0, 32512usize as *const u16) };

    let wc = WndClassW {
        style: CS_HREDRAW | CS_VREDRAW,
        lpfn_wnd_proc: Some(window_proc),
        cb_cls_extra: 0,
        cb_wnd_extra: 0,
        h_instance,
        h_icon: 0,
        h_cursor: cursor,
        hbr_background: COLOR_WINDOW + 1,
        lpsz_menu_name: null(),
        lpsz_class_name: class_name.as_ptr(),
    };

    unsafe { RegisterClassW(&wc) };
    let hwnd = unsafe {
        CreateWindowExW(
            0,
            class_name.as_ptr(),
            title.as_ptr(),
            0x10CF_0000,
            100,
            100,
            780,
            520,
            0,
            0,
            h_instance,
            null_mut(),
        )
    };

    if hwnd == 0 {
        return;
    }

    unsafe {
        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);
    }

    let mut msg: Msg = unsafe { zeroed() };
    while unsafe { GetMessageW(&mut msg, 0, 0, 0) } > 0 {
        unsafe {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

unsafe extern "system" fn window_proc(
    hwnd: Hwnd,
    msg: Uint,
    wparam: Wparam,
    lparam: Lparam,
) -> Lresult {
    match msg {
        WM_CREATE => {
            unsafe { SetTimer(hwnd, 1, 16, null()) };
            create_controls(hwnd);
            0
        }
        WM_TIMER => {
            with_wizard(|wizard| wizard.update());
            sync_controls();
            unsafe { InvalidateRect(hwnd, null(), 1) };
            0
        }
        WM_COMMAND => {
            handle_control_command(wparam, lparam);
            sync_controls();
            unsafe { InvalidateRect(hwnd, null(), 1) };
            0
        }
        WM_KEYDOWN if wparam == VK_ESCAPE => {
            unsafe { DestroyWindow(hwnd) };
            0
        }
        WM_KEYDOWN => {
            if let Some(command) = command_from_key(wparam) {
                with_wizard(|wizard| wizard.handle_command(command));
                sync_controls();
            }
            unsafe { InvalidateRect(hwnd, null(), 1) };
            0
        }
        WM_PAINT => {
            draw(hwnd);
            0
        }
        WM_DESTROY => {
            unsafe { PostQuitMessage(0) };
            0
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

fn command_from_key(key: Wparam) -> Option<WizardCommand> {
    match key {
        VK_RETURN => Some(WizardCommand::Confirm),
        KEY_R => Some(WizardCommand::Restart),
        _ => None,
    }
}

fn with_wizard(action: impl FnOnce(&mut Wizard<WinmmDeviceProvider>)) {
    if let Some(lock) = STATE.get() {
        if let Ok(mut wizard) = lock.lock() {
            action(&mut wizard);
        }
    }
}

fn create_controls(parent: Hwnd) {
    let h_instance = unsafe { GetModuleHandleW(null()) };
    let combo_class = wide("COMBOBOX");
    let button_class = wide("BUTTON");
    let use_device = wide("Use device");
    let restart = wide("Restart");

    let device_combo = unsafe {
        CreateWindowExW(
            0,
            combo_class.as_ptr(),
            null(),
            WS_CHILD | WS_VSCROLL | CBS_DROPDOWNLIST,
            24,
            150,
            520,
            180,
            parent,
            IDC_DEVICE_COMBO as isize,
            h_instance,
            null_mut(),
        )
    };
    let primary_button = unsafe {
        CreateWindowExW(
            0,
            button_class.as_ptr(),
            use_device.as_ptr(),
            WS_CHILD | WS_TABSTOP | BS_DEFPUSHBUTTON,
            560,
            149,
            118,
            28,
            parent,
            IDC_PRIMARY_BUTTON as isize,
            h_instance,
            null_mut(),
        )
    };
    let restart_button = unsafe {
        CreateWindowExW(
            0,
            button_class.as_ptr(),
            restart.as_ptr(),
            WS_CHILD | WS_TABSTOP | BS_PUSHBUTTON,
            688,
            149,
            68,
            28,
            parent,
            IDC_RESTART_BUTTON as isize,
            h_instance,
            null_mut(),
        )
    };

    let _ = CONTROLS.set(Mutex::new(UiControls {
        device_combo,
        primary_button,
        restart_button,
        device_signature: Vec::new(),
        combo_selected: None,
        combo_visible: false,
        primary_visible: false,
        primary_enabled: false,
        primary_text: String::new(),
        restart_visible: false,
    }));

    sync_controls();
}

fn handle_control_command(wparam: Wparam, lparam: Lparam) {
    let control_id = loword(wparam);
    let notification = hiword(wparam);

    match (control_id, notification) {
        (IDC_DEVICE_COMBO, CBN_SELCHANGE) => {
            if let Some(index) = combo_current_selection(lparam as Hwnd) {
                with_wizard(|wizard| wizard.handle_command(WizardCommand::SelectDevice(index)));
            }
        }
        (IDC_PRIMARY_BUTTON, BN_CLICKED) => {
            with_wizard(|wizard| wizard.handle_command(WizardCommand::Confirm));
        }
        (IDC_RESTART_BUTTON, BN_CLICKED) => {
            with_wizard(|wizard| wizard.handle_command(WizardCommand::Restart));
        }
        _ => {}
    }
}

fn sync_controls() {
    let Some(view) = current_view() else {
        return;
    };
    let Some(lock) = CONTROLS.get() else {
        return;
    };
    let Ok(mut controls) = lock.lock() else {
        return;
    };

    match &view.step {
        WizardStepView::SelectDevice {
            devices,
            selected_index,
        } => {
            populate_device_combo(&mut controls, devices, *selected_index);
            set_visible(controls.device_combo, &mut controls.combo_visible, true);
            set_visible(controls.primary_button, &mut controls.primary_visible, true);
            set_enabled(
                controls.primary_button,
                &mut controls.primary_enabled,
                !devices.is_empty(),
            );
            set_window_text(
                controls.primary_button,
                &mut controls.primary_text,
                "Use device",
            );
            set_visible(controls.restart_button, &mut controls.restart_visible, true);
        }
        WizardStepView::Capture { role, armed, .. } => {
            set_visible(controls.device_combo, &mut controls.combo_visible, false);
            set_visible(controls.primary_button, &mut controls.primary_visible, true);
            set_enabled(
                controls.primary_button,
                &mut controls.primary_enabled,
                !armed,
            );
            let text = if *armed {
                format!("Waiting for {}", role.label())
            } else {
                format!("Capture {}", role.label())
            };
            set_window_text(controls.primary_button, &mut controls.primary_text, &text);
            set_visible(controls.restart_button, &mut controls.restart_visible, true);
        }
        WizardStepView::Ready { .. } => {
            set_visible(controls.device_combo, &mut controls.combo_visible, false);
            set_visible(
                controls.primary_button,
                &mut controls.primary_visible,
                false,
            );
            set_visible(controls.restart_button, &mut controls.restart_visible, true);
        }
    }
}

fn populate_device_combo(
    controls: &mut UiControls,
    devices: &[DeviceSnapshot],
    selected_index: usize,
) {
    let signature = devices
        .iter()
        .map(|device| (device.id, device.name.clone()))
        .collect::<Vec<_>>();

    if controls.device_signature != signature {
        unsafe {
            SendMessageW(controls.device_combo, CB_RESETCONTENT, 0, 0);
        }

        for device in devices {
            let text = wide(&format!(
                "[{}] {} ({} axes)",
                device.id,
                device.name,
                device.axes.len()
            ));
            unsafe {
                SendMessageW(
                    controls.device_combo,
                    CB_ADDSTRING,
                    0,
                    text.as_ptr() as Lparam,
                );
            }
        }

        controls.device_signature = signature;
        controls.combo_selected = None;
    }

    if !devices.is_empty() && controls.combo_selected != Some(selected_index) {
        unsafe {
            SendMessageW(controls.device_combo, CB_SETCURSEL, selected_index, 0);
        }
        controls.combo_selected = Some(selected_index);
    }
}

fn combo_current_selection(combo: Hwnd) -> Option<usize> {
    if combo == 0 {
        return None;
    }

    let index = unsafe { SendMessageW(combo, CB_GETCURSEL, 0, 0) };
    (index != CB_ERR).then_some(index as usize)
}

fn set_visible(hwnd: Hwnd, current: &mut bool, visible: bool) {
    if *current != visible {
        unsafe { ShowWindow(hwnd, if visible { SW_SHOW } else { SW_HIDE }) };
        *current = visible;
    }
}

fn set_enabled(hwnd: Hwnd, current: &mut bool, enabled: bool) {
    if *current != enabled {
        unsafe { EnableWindow(hwnd, enabled as Bool) };
        *current = enabled;
    }
}

fn set_window_text(hwnd: Hwnd, current: &mut String, text: &str) {
    if current != text {
        let value = wide(text);
        unsafe { SetWindowTextW(hwnd, value.as_ptr()) };
        current.clear();
        current.push_str(text);
    }
}

fn loword(value: Wparam) -> u16 {
    (value & 0xFFFF) as u16
}

fn hiword(value: Wparam) -> u16 {
    ((value >> 16) & 0xFFFF) as u16
}

fn axes_from_caps(caps: &JoyCapsW) -> Vec<AxisSnapshot> {
    vec![
        AxisSnapshot::new("X", caps.w_xmin, caps.w_xmax, 0),
        AxisSnapshot::new("Y", caps.w_ymin, caps.w_ymax, 0),
        AxisSnapshot::new("Z", caps.w_zmin, caps.w_zmax, 0),
        AxisSnapshot::new("R", caps.w_rmin, caps.w_rmax, 0),
        AxisSnapshot::new("U", caps.w_umin, caps.w_umax, 0),
        AxisSnapshot::new("V", caps.w_vmin, caps.w_vmax, 0),
    ]
}

fn read_axis_values(device_id: u32) -> Option<[u32; 6]> {
    let mut info = JoyInfoEx {
        dw_size: size_of::<JoyInfoEx>() as u32,
        dw_flags: JOY_RETURNALL,
        dw_xpos: 0,
        dw_ypos: 0,
        dw_zpos: 0,
        dw_rpos: 0,
        dw_upos: 0,
        dw_vpos: 0,
        dw_buttons: 0,
        dw_button_number: 0,
        dw_pov: 0,
        dw_reserved1: 0,
        dw_reserved2: 0,
    };

    let result = unsafe { joyGetPosEx(device_id, &mut info) };
    if result != JOYERR_NOERROR {
        return None;
    }

    Some([
        info.dw_xpos,
        info.dw_ypos,
        info.dw_zpos,
        info.dw_rpos,
        info.dw_upos,
        info.dw_vpos,
    ])
}

fn draw(hwnd: Hwnd) {
    let mut ps: PaintStruct = unsafe { zeroed() };
    let hdc = unsafe { BeginPaint(hwnd, &mut ps) };
    unsafe {
        SetBkMode(hdc, 1);
        SetTextColor(hdc, rgb(28, 31, 35));
    }

    let mut rect: Rect = unsafe { zeroed() };
    unsafe { GetClientRect(hwnd, &mut rect) };
    let bg = unsafe { CreateSolidBrush(rgb(246, 247, 249)) };
    unsafe {
        FillRect(hdc, &rect, bg);
        DeleteObject(bg);
    }

    if let Some(view) = current_view() {
        draw_view(hdc, &view, rect);
    }

    unsafe { EndPaint(hwnd, &ps) };
}

fn current_view() -> Option<WizardView> {
    let lock = STATE.get()?;
    let wizard = lock.lock().ok()?;
    Some(wizard.view())
}

fn draw_view(hdc: Hdc, view: &WizardView, rect: Rect) {
    draw_text(hdc, 24, 20, 720, 28, "Apex Footwork setup wizard");
    draw_text(hdc, 24, 52, 720, 24, &view.status);
    draw_text(hdc, 24, 78, 720, 22, view.step.title());

    match &view.step {
        WizardStepView::SelectDevice {
            devices,
            selected_index,
        } => {
            draw_device_picker(hdc, devices, *selected_index);
        }
        WizardStepView::Capture {
            role,
            armed,
            device,
            bindings,
        } => {
            draw_capture_step(hdc, *role, *armed, device, bindings);
        }
        WizardStepView::Ready {
            device,
            bindings,
            values,
        } => {
            draw_ready(hdc, device, bindings, values);
        }
    }

    draw_text(
        hdc,
        24,
        rect.bottom - 34,
        720,
        22,
        "Esc closes  |  Restart runs setup again",
    );
}

fn draw_device_picker(hdc: Hdc, devices: &[DeviceSnapshot], _selected_index: usize) {
    draw_text(
        hdc,
        24,
        124,
        720,
        22,
        "Choose a controller from the dropdown, then click Use device.",
    );

    if devices.is_empty() {
        draw_text(hdc, 24, 196, 720, 22, "No devices are currently available.");
    }
}

fn draw_capture_step(
    hdc: Hdc,
    role: InputRole,
    armed: bool,
    device: &Option<DeviceSnapshot>,
    bindings: &PedalBindings,
) {
    let device_name = device
        .as_ref()
        .map(|device| device.name.as_str())
        .unwrap_or("No active device");
    draw_text(hdc, 24, 124, 720, 22, &format!("Device: {}", device_name));

    let instruction = if armed {
        format!(
            "Press {} fully now. The moving axis will be detected.",
            role.label()
        )
    } else {
        format!("Release all pedals, then click Capture {}.", role.label())
    };
    draw_text(hdc, 24, 150, 520, 22, &instruction);

    if let Some(device) = device {
        for (i, axis) in device.axes.iter().enumerate() {
            draw_axis(hdc, 24, 194 + (i as i32 * 42), 560, axis);
        }
    }

    draw_binding_summary(hdc, bindings, 610, 194);
}

fn draw_ready(
    hdc: Hdc,
    device: &Option<DeviceSnapshot>,
    bindings: &PedalBindings,
    values: &[(InputRole, f32)],
) {
    let device_name = device
        .as_ref()
        .map(|device| device.name.as_str())
        .unwrap_or("No active device");
    draw_text(hdc, 24, 124, 720, 22, &format!("Using: {}", device_name));

    let throttle = values
        .iter()
        .find_map(|(role, value)| (*role == InputRole::Throttle).then_some(*value))
        .unwrap_or(0.0);
    let brake = values
        .iter()
        .find_map(|(role, value)| (*role == InputRole::Brake).then_some(*value))
        .unwrap_or(0.0);

    draw_value_bar(hdc, 24, 176, 620, "Throttle", throttle, rgb(19, 132, 109));
    draw_value_bar(hdc, 24, 238, 620, "Brake", brake, rgb(180, 47, 54));
    draw_text(
        hdc,
        24,
        316,
        720,
        22,
        "Live normalized pedal values are ready for the overlay layer.",
    );
    draw_binding_summary(hdc, bindings, 24, 354);
}

fn draw_binding_summary(hdc: Hdc, bindings: &PedalBindings, x: i32, y: i32) {
    draw_text(hdc, x, y, 150, 22, "Bindings");
    draw_binding_line(
        hdc,
        x,
        y + 30,
        InputRole::Throttle,
        bindings.throttle.as_ref(),
    );
    draw_binding_line(hdc, x, y + 58, InputRole::Brake, bindings.brake.as_ref());
}

fn draw_binding_line(hdc: Hdc, x: i32, y: i32, role: InputRole, binding: Option<&BindingView>) {
    let text = if let Some(binding) = binding {
        format!(
            "{}: {} idle={} active={}",
            role.label(),
            binding.axis_label,
            binding.idle_raw,
            binding.active_raw
        )
    } else {
        format!("{}: not set", role.label())
    };
    draw_text(hdc, x, y, 520, 22, &text);
}

fn draw_axis(hdc: Hdc, x: i32, y: i32, width: i32, axis: &AxisSnapshot) {
    let percent = axis.percent();
    let label = format!("{}  raw={}  {:.0}%", axis.label, axis.raw, percent * 100.0);
    draw_text(hdc, x, y, width, 18, &label);
    draw_bar(hdc, x, y + 20, width, 14, percent, rgb(19, 132, 109));
}

fn draw_value_bar(hdc: Hdc, x: i32, y: i32, width: i32, label: &str, value: f32, color: Dword) {
    let text = format!("{}  {:.0}%", label, value * 100.0);
    draw_text(hdc, x, y, width, 22, &text);
    draw_bar(hdc, x, y + 28, width, 22, value, color);
}

fn draw_bar(hdc: Hdc, x: i32, y: i32, width: i32, height: i32, percent: f32, fill_color: Dword) {
    let percent = percent.clamp(0.0, 1.0);
    let bar = Rect {
        left: x,
        top: y,
        right: x + width,
        bottom: y + height,
    };
    let filled = Rect {
        left: x,
        top: y,
        right: x + ((width as f32 * percent).round() as i32),
        bottom: y + height,
    };

    let empty_brush = unsafe { CreateSolidBrush(rgb(222, 226, 231)) };
    let fill_brush = unsafe { CreateSolidBrush(fill_color) };
    unsafe {
        FillRect(hdc, &bar, empty_brush);
        FillRect(hdc, &filled, fill_brush);
        DeleteObject(empty_brush);
        DeleteObject(fill_brush);
    }
}

fn draw_text(hdc: Hdc, x: i32, y: i32, width: i32, height: i32, text: &str) {
    let mut rect = Rect {
        left: x,
        top: y,
        right: x + width,
        bottom: y + height,
    };
    let wide_text = wide(text);
    unsafe {
        DrawTextW(
            hdc,
            wide_text.as_ptr(),
            (wide_text.len() - 1) as i32,
            &mut rect,
            DT_LEFT | DT_TOP | DT_SINGLELINE,
        );
    }
}

fn name_from_wide(chars: &[u16]) -> String {
    let len = chars.iter().position(|&c| c == 0).unwrap_or(chars.len());
    String::from_utf16_lossy(&chars[..len])
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

fn rgb(r: u8, g: u8, b: u8) -> Dword {
    (r as u32) | ((g as u32) << 8) | ((b as u32) << 16)
}
