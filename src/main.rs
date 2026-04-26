#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod profile;
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
type Hfont = isize;
type Hgdiobj = isize;
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
const WM_ERASEBKGND: Uint = 0x0014;
const WM_COMMAND: Uint = 0x0111;
const WM_TIMER: Uint = 0x0113;
const WM_KEYDOWN: Uint = 0x0100;
const WM_LBUTTONDOWN: Uint = 0x0201;
const WM_SETFONT: Uint = 0x0030;
const VK_ESCAPE: Wparam = 0x1B;
const VK_RETURN: Wparam = 0x0D;
const KEY_R: Wparam = 0x52;
const SW_HIDE: i32 = 0;
const SW_SHOW: i32 = 5;
const CS_HREDRAW: Uint = 0x0002;
const CS_VREDRAW: Uint = 0x0001;
const DT_LEFT: Uint = 0x0000;
const DT_CENTER: Uint = 0x0001;
const DT_VCENTER: Uint = 0x0004;
const DT_TOP: Uint = 0x0000;
const DT_SINGLELINE: Uint = 0x0020;
const WS_CHILD: Dword = 0x4000_0000;
const WS_TABSTOP: Dword = 0x0001_0000;
const WS_VSCROLL: Dword = 0x0020_0000;
const CBS_DROPDOWNLIST: Dword = 0x0003;
const BS_DEFPUSHBUTTON: Dword = 0x0001;
const WS_CLIPCHILDREN: Dword = 0x0200_0000;
const WS_CLIPSIBLINGS: Dword = 0x0400_0000;
const CBN_SELCHANGE: u16 = 1;
const BN_CLICKED: u16 = 0;
const CB_ADDSTRING: Uint = 0x0143;
const CB_GETCURSEL: Uint = 0x0147;
const CB_RESETCONTENT: Uint = 0x014B;
const CB_SETCURSEL: Uint = 0x014E;
const CB_ERR: Lresult = -1;
const IDC_DEVICE_COMBO: u16 = 1001;
const IDC_PRIMARY_BUTTON: u16 = 1002;
const SRCCOPY: Dword = 0x00CC_0020;
const TRANSPARENT: i32 = 1;
const FW_NORMAL: i32 = 400;
const FW_SEMIBOLD: i32 = 600;
const DEFAULT_CHARSET: Dword = 1;
const OUT_DEFAULT_PRECIS: Dword = 0;
const CLIP_DEFAULT_PRECIS: Dword = 0;
const CLEARTYPE_QUALITY: Dword = 5;
const DEFAULT_PITCH: Dword = 0;
const PS_SOLID: i32 = 0;

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
    fn BitBlt(
        hdc: Hdc,
        x: i32,
        y: i32,
        cx: i32,
        cy: i32,
        hdcSrc: Hdc,
        x1: i32,
        y1: i32,
        rop: Dword,
    ) -> Bool;
    fn CreateCompatibleBitmap(hdc: Hdc, cx: i32, cy: i32) -> Hgdiobj;
    fn CreateCompatibleDC(hdc: Hdc) -> Hdc;
    fn CreateFontW(
        cHeight: i32,
        cWidth: i32,
        cEscapement: i32,
        cOrientation: i32,
        cWeight: i32,
        bItalic: Dword,
        bUnderline: Dword,
        bStrikeOut: Dword,
        iCharSet: Dword,
        iOutPrecision: Dword,
        iClipPrecision: Dword,
        iQuality: Dword,
        iPitchAndFamily: Dword,
        pszFaceName: *const u16,
    ) -> Hfont;
    fn CreatePen(iStyle: i32, cWidth: i32, color: Dword) -> Hgdiobj;
    fn CreateSolidBrush(color: Dword) -> Hbrush;
    fn DeleteDC(hdc: Hdc) -> Bool;
    fn DeleteObject(ho: isize) -> Bool;
    fn LineTo(hdc: Hdc, x: i32, y: i32) -> Bool;
    fn MoveToEx(hdc: Hdc, x: i32, y: i32, lppt: *mut Point) -> Bool;
    fn SelectObject(hdc: Hdc, h: Hgdiobj) -> Hgdiobj;
    fn SetBkMode(hdc: Hdc, mode: i32) -> i32;
    fn SetTextColor(hdc: Hdc, color: Dword) -> Dword;
}

#[link(name = "uxtheme")]
unsafe extern "system" {
    fn SetWindowTheme(hwnd: Hwnd, pszSubAppName: *const u16, pszSubIdList: *const u16) -> i32;
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
static FONTS: OnceLock<UiFonts> = OnceLock::new();
static SAVED_PROFILE_SIGNATURE: OnceLock<Mutex<Option<String>>> = OnceLock::new();

struct UiFonts {
    title: Hfont,
    heading: Hfont,
    body: Hfont,
    meta: Hfont,
}

struct UiControls {
    device_combo: Hwnd,
    primary_button: Hwnd,
    device_signature: Vec<(u32, String)>,
    combo_selected: Option<usize>,
    combo_visible: bool,
    primary_visible: bool,
    primary_enabled: bool,
    primary_text: String,
}

fn main() {
    let mut wizard = Wizard::new(WinmmDeviceProvider);
    if let Some(saved_profile) = profile::load_profile() {
        if wizard.restore_profile(&saved_profile) {
            remember_saved_profile(&saved_profile);
        }
    }

    STATE.get_or_init(|| Mutex::new(wizard));
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
        hbr_background: 0,
        lpsz_menu_name: null(),
        lpsz_class_name: class_name.as_ptr(),
    };

    unsafe { RegisterClassW(&wc) };
    let hwnd = unsafe {
        CreateWindowExW(
            0,
            class_name.as_ptr(),
            title.as_ptr(),
            0x10CF_0000 | WS_CLIPCHILDREN,
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
        WM_ERASEBKGND => 1,
        WM_TIMER => {
            with_wizard(|wizard| wizard.update());
            sync_controls();
            unsafe { InvalidateRect(hwnd, null(), 0) };
            0
        }
        WM_COMMAND => {
            handle_control_command(wparam, lparam);
            sync_controls();
            unsafe { InvalidateRect(hwnd, null(), 0) };
            0
        }
        WM_LBUTTONDOWN => {
            if handle_mouse_down(hwnd, lparam) {
                sync_controls();
                unsafe { InvalidateRect(hwnd, null(), 0) };
                return 0;
            }
            unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
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
            unsafe { InvalidateRect(hwnd, null(), 0) };
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
        KEY_R => Some(WizardCommand::Configure),
        _ => None,
    }
}

fn handle_mouse_down(hwnd: Hwnd, lparam: Lparam) -> bool {
    let mut rect: Rect = unsafe { zeroed() };
    unsafe { GetClientRect(hwnd, &mut rect) };

    let x = signed_loword(lparam as usize);
    let y = signed_hiword(lparam as usize);
    if point_in_rect(x, y, configure_button_rect(rect)) {
        with_wizard(|wizard| wizard.handle_command(WizardCommand::Configure));
        return true;
    }

    false
}

fn with_wizard(action: impl FnOnce(&mut Wizard<WinmmDeviceProvider>)) {
    if let Some(lock) = STATE.get() {
        if let Ok(mut wizard) = lock.lock() {
            action(&mut wizard);
            maybe_save_profile(&wizard);
        }
    }
}

fn remember_saved_profile(profile: &profile::StoredProfile) {
    let signature = profile::profile_signature(profile);
    if let Ok(mut saved) = SAVED_PROFILE_SIGNATURE
        .get_or_init(|| Mutex::new(None))
        .lock()
    {
        *saved = Some(signature);
    }
}

fn maybe_save_profile(wizard: &Wizard<WinmmDeviceProvider>) {
    let Some(profile) = wizard.profile() else {
        return;
    };

    let signature = profile::profile_signature(&profile);
    let Ok(mut saved_signature) = SAVED_PROFILE_SIGNATURE
        .get_or_init(|| Mutex::new(None))
        .lock()
    else {
        return;
    };

    if saved_signature.as_ref() == Some(&signature) {
        return;
    }

    if profile::save_profile(&profile).is_ok() {
        *saved_signature = Some(signature);
    }
}

fn create_controls(parent: Hwnd) {
    init_fonts();
    let h_instance = unsafe { GetModuleHandleW(null()) };
    let combo_class = wide("COMBOBOX");
    let button_class = wide("BUTTON");
    let use_device = wide("Use device");

    let device_combo = unsafe {
        CreateWindowExW(
            0,
            combo_class.as_ptr(),
            null(),
            WS_CHILD | WS_CLIPSIBLINGS | WS_VSCROLL | CBS_DROPDOWNLIST,
            32,
            154,
            520,
            220,
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
            WS_CHILD | WS_CLIPSIBLINGS | WS_TABSTOP | BS_DEFPUSHBUTTON,
            568,
            153,
            124,
            30,
            parent,
            IDC_PRIMARY_BUTTON as isize,
            h_instance,
            null_mut(),
        )
    };
    let _ = CONTROLS.set(Mutex::new(UiControls {
        device_combo,
        primary_button,
        device_signature: Vec::new(),
        combo_selected: None,
        combo_visible: false,
        primary_visible: false,
        primary_enabled: false,
        primary_text: String::new(),
    }));

    apply_control_style(device_combo);
    apply_control_style(primary_button);
    sync_controls();
}

fn init_fonts() {
    let _ = FONTS.set(UiFonts {
        title: create_ui_font(-26, FW_SEMIBOLD),
        heading: create_ui_font(-18, FW_SEMIBOLD),
        body: create_ui_font(-15, FW_NORMAL),
        meta: create_ui_font(-13, FW_NORMAL),
    });
}

fn create_ui_font(height: i32, weight: i32) -> Hfont {
    let face = wide("Segoe UI");
    unsafe {
        CreateFontW(
            height,
            0,
            0,
            0,
            weight,
            0,
            0,
            0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            CLEARTYPE_QUALITY,
            DEFAULT_PITCH,
            face.as_ptr(),
        )
    }
}

fn apply_control_style(hwnd: Hwnd) {
    if hwnd == 0 {
        return;
    }

    if let Some(fonts) = FONTS.get() {
        unsafe {
            SendMessageW(hwnd, WM_SETFONT, fonts.body as Wparam, 1);
        }
    }

    let explorer = wide("Explorer");
    unsafe {
        SetWindowTheme(hwnd, explorer.as_ptr(), null());
    }
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
        }
        WizardStepView::Ready { .. } => {
            set_visible(controls.device_combo, &mut controls.combo_visible, false);
            set_visible(
                controls.primary_button,
                &mut controls.primary_visible,
                false,
            );
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

fn signed_loword(value: usize) -> i32 {
    (value as u16) as i16 as i32
}

fn signed_hiword(value: usize) -> i32 {
    ((value >> 16) as u16) as i16 as i32
}

fn point_in_rect(x: i32, y: i32, rect: Rect) -> bool {
    x >= rect.left && x < rect.right && y >= rect.top && y < rect.bottom
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
    let mut rect: Rect = unsafe { zeroed() };
    unsafe { GetClientRect(hwnd, &mut rect) };

    let width = rect.right - rect.left;
    let height = rect.bottom - rect.top;
    if width <= 0 || height <= 0 {
        unsafe { EndPaint(hwnd, &ps) };
        return;
    }

    let mem_dc = unsafe { CreateCompatibleDC(hdc) };
    let bitmap = unsafe { CreateCompatibleBitmap(hdc, width, height) };
    if mem_dc == 0 || bitmap == 0 {
        draw_scene(hdc, rect);
        unsafe {
            if bitmap != 0 {
                DeleteObject(bitmap);
            }
            if mem_dc != 0 {
                DeleteDC(mem_dc);
            }
            EndPaint(hwnd, &ps);
        }
        return;
    }

    let old_bitmap = unsafe { SelectObject(mem_dc, bitmap) };
    draw_scene(mem_dc, rect);
    unsafe {
        BitBlt(hdc, 0, 0, width, height, mem_dc, 0, 0, SRCCOPY);
        SelectObject(mem_dc, old_bitmap);
        DeleteObject(bitmap);
        DeleteDC(mem_dc);
        EndPaint(hwnd, &ps);
    }
}

fn draw_scene(hdc: Hdc, rect: Rect) {
    unsafe {
        SetBkMode(hdc, TRANSPARENT);
        SetTextColor(hdc, color_text());
    }

    fill_rect(hdc, rect, color_app_bg());
    draw_shell(hdc, rect);

    if let Some(view) = current_view() {
        draw_view(hdc, &view, rect);
    }
}

fn current_view() -> Option<WizardView> {
    let lock = STATE.get()?;
    let wizard = lock.lock().ok()?;
    Some(wizard.view())
}

#[derive(Clone, Copy)]
enum TextKind {
    Title,
    Heading,
    Body,
    Meta,
}

fn draw_shell(hdc: Hdc, rect: Rect) {
    fill_rect(hdc, rect, color_app_bg());
    fill_rect(
        hdc,
        Rect {
            left: 0,
            top: 0,
            right: rect.right,
            bottom: 4,
        },
        color_accent(),
    );

    draw_panel(hdc, 16, 16, (rect.right - 32).max(0), 88, color_panel());
    draw_panel(
        hdc,
        16,
        118,
        (rect.right - 32).max(0),
        (rect.bottom - 166).max(0),
        color_panel(),
    );
}

fn draw_panel(hdc: Hdc, x: i32, y: i32, width: i32, height: i32, color: Dword) {
    if width <= 0 || height <= 0 {
        return;
    }

    fill_rect(hdc, rect_xywh(x, y, width, height), color);
    fill_rect(hdc, rect_xywh(x, y, width, 1), color_border());
    fill_rect(hdc, rect_xywh(x, y + height - 1, width, 1), color_border());
    fill_rect(hdc, rect_xywh(x, y, 1, height), color_border());
    fill_rect(hdc, rect_xywh(x + width - 1, y, 1, height), color_border());
}

fn draw_hud_button(hdc: Hdc, rect: Rect, label: &str, active: bool) {
    let fill = if active {
        rgb(0, 168, 151)
    } else {
        rgb(32, 37, 47)
    };
    let border = if active {
        color_accent()
    } else {
        rgb(74, 84, 102)
    };

    fill_rect(hdc, rect, fill);
    fill_rect(
        hdc,
        rect_xywh(rect.left, rect.top, rect.right - rect.left, 1),
        border,
    );
    fill_rect(
        hdc,
        rect_xywh(rect.left, rect.bottom - 1, rect.right - rect.left, 1),
        border,
    );
    fill_rect(
        hdc,
        rect_xywh(rect.left, rect.top, 1, rect.bottom - rect.top),
        border,
    );
    fill_rect(
        hdc,
        rect_xywh(rect.right - 1, rect.top, 1, rect.bottom - rect.top),
        border,
    );

    draw_centered_text_kind(
        hdc,
        rect,
        label,
        TextKind::Body,
        if active { rgb(6, 18, 20) } else { color_text() },
    );
}

fn configure_button_rect(client: Rect) -> Rect {
    let width = 104;
    let right = (client.right - 32).max(width + 32);
    rect_xywh(right - width, 34, width, 34)
}

fn draw_footer(hdc: Hdc, rect: Rect) {
    let y = (rect.bottom - 34).max(0);
    fill_rect(
        hdc,
        Rect {
            left: 0,
            top: y - 8,
            right: rect.right,
            bottom: rect.bottom,
        },
        color_app_bg(),
    );
    draw_text_kind(
        hdc,
        32,
        y,
        (rect.right - 64).max(0),
        22,
        "Esc closes  |  Configure opens setup",
        TextKind::Meta,
        color_muted(),
    );
}

fn fill_rect(hdc: Hdc, rect: Rect, color: Dword) {
    let brush = unsafe { CreateSolidBrush(color) };
    unsafe {
        FillRect(hdc, &rect, brush);
        DeleteObject(brush);
    }
}

fn rect_xywh(x: i32, y: i32, width: i32, height: i32) -> Rect {
    Rect {
        left: x,
        top: y,
        right: x + width,
        bottom: y + height,
    }
}

fn draw_view(hdc: Hdc, view: &WizardView, rect: Rect) {
    let configure_rect = configure_button_rect(rect);
    let header_width = (configure_rect.left - 48).max(220);
    draw_text_kind(
        hdc,
        32,
        26,
        header_width,
        32,
        "Apex Footwork",
        TextKind::Title,
        color_text(),
    );
    draw_text_kind(
        hdc,
        32,
        60,
        header_width,
        22,
        &view.status,
        TextKind::Body,
        color_muted(),
    );
    draw_text_kind(
        hdc,
        32,
        84,
        220,
        20,
        view.step.title(),
        TextKind::Meta,
        color_accent(),
    );
    draw_hud_button(hdc, configure_rect, "Configure", false);

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
            history,
        } => {
            draw_ready(hdc, rect, device, bindings, values, history);
        }
    }

    draw_footer(hdc, rect);
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
            "Press {} fully, then release it. The highest point becomes 100%.",
            role.label()
        )
    } else {
        format!("Release all pedals, then click Capture {}.", role.label())
    };
    draw_text(hdc, 24, 150, 520, 22, &instruction);

    if let Some(device) = device {
        draw_panel(hdc, 24, 184, 574, 262, color_panel_raised());
        for (i, axis) in device.axes.iter().enumerate() {
            draw_axis(hdc, 38, 200 + (i as i32 * 38), 532, axis);
        }
    }

    draw_panel(hdc, 612, 184, 142, 104, color_panel_raised());
    draw_binding_summary(hdc, bindings, 626, 198);
}

fn draw_ready(
    hdc: Hdc,
    rect: Rect,
    device: &Option<DeviceSnapshot>,
    _bindings: &PedalBindings,
    values: &[(InputRole, f32)],
    history: &[(f32, f32)],
) {
    let device_name = device
        .as_ref()
        .map(|device| device.name.as_str())
        .unwrap_or("No active device");
    let content_left = 32;
    let content_right = (rect.right - 32).max(content_left + 1);
    let content_width = content_right - content_left;
    let gap = 24;
    let column_width = ((content_width - gap) / 2).max(220);
    let right_x = content_left + column_width + gap;
    let card_y = 166;
    let chart_y = 286;
    let footer_top = rect.bottom - 48;
    let chart_height = (footer_top - chart_y - 20).max(96);

    draw_text_kind(
        hdc,
        content_left,
        132,
        content_width,
        22,
        &format!("Input source: {}", device_name),
        TextKind::Body,
        color_muted(),
    );

    let throttle = values
        .iter()
        .find_map(|(role, value)| (*role == InputRole::Throttle).then_some(*value))
        .unwrap_or(0.0);
    let brake = values
        .iter()
        .find_map(|(role, value)| (*role == InputRole::Brake).then_some(*value))
        .unwrap_or(0.0);

    draw_value_bar(
        hdc,
        content_left,
        card_y,
        column_width,
        "Throttle",
        throttle,
        color_accent(),
    );
    draw_value_bar(
        hdc,
        right_x,
        card_y,
        column_width,
        "Brake",
        brake,
        rgb(255, 82, 96),
    );
    draw_overlay_chart(
        hdc,
        content_left,
        chart_y,
        column_width,
        chart_height,
        "Throttle trace",
        history,
        InputRole::Throttle,
    );
    draw_overlay_chart(
        hdc,
        right_x,
        chart_y,
        column_width,
        chart_height,
        "Brake trace",
        history,
        InputRole::Brake,
    );
}

fn draw_binding_summary(hdc: Hdc, bindings: &PedalBindings, x: i32, y: i32) {
    draw_text_kind(
        hdc,
        x,
        y,
        150,
        22,
        "Bindings",
        TextKind::Heading,
        color_text(),
    );
    draw_binding_line(
        hdc,
        x,
        y + 30,
        InputRole::Throttle,
        bindings.throttle.as_ref(),
    );
    draw_binding_line(hdc, x, y + 58, InputRole::Brake, bindings.brake.as_ref());
}

fn draw_overlay_chart(
    hdc: Hdc,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    label: &str,
    history: &[(f32, f32)],
    role: InputRole,
) {
    draw_panel(hdc, x, y, width, height, color_panel_raised());
    draw_text_kind(
        hdc,
        x + 14,
        y + 10,
        width - 28,
        20,
        label,
        TextKind::Heading,
        color_text(),
    );

    let plot = rect_xywh(x + 14, y + 38, width - 28, height - 54);
    for i in 0..=3 {
        let grid_y = plot.top + ((plot.bottom - plot.top) * i / 3);
        fill_rect(
            hdc,
            Rect {
                left: plot.left,
                top: grid_y,
                right: plot.right,
                bottom: grid_y + 1,
            },
            rgb(40, 46, 57),
        );
    }

    if history.len() < 2 {
        draw_text_kind(
            hdc,
            plot.left,
            plot.top + 16,
            plot.right - plot.left,
            20,
            "Waiting for input",
            TextKind::Meta,
            color_muted(),
        );
        return;
    }

    let color = match role {
        InputRole::Throttle => color_accent(),
        InputRole::Brake => rgb(255, 82, 96),
    };
    draw_history_line(hdc, plot, history, role, color);
}

fn draw_history_line(hdc: Hdc, plot: Rect, history: &[(f32, f32)], role: InputRole, color: Dword) {
    let plot_width = (plot.right - plot.left).max(1) as usize;
    let start = history.len().saturating_sub(plot_width);
    let visible = &history[start..];
    if visible.len() < 2 {
        return;
    }

    let pen = unsafe { CreatePen(PS_SOLID, 2, color) };
    let old_pen = unsafe { SelectObject(hdc, pen) };
    let span_x = (plot.right - plot.left - 1).max(1) as f32;
    let span_y = (plot.bottom - plot.top - 1).max(1) as f32;

    for (index, sample) in visible.iter().enumerate() {
        let value = match role {
            InputRole::Throttle => sample.0,
            InputRole::Brake => sample.1,
        }
        .clamp(0.0, 1.0);

        let x = plot.left + ((index as f32 / (visible.len() - 1) as f32) * span_x).round() as i32;
        let y = plot.bottom - 1 - (value * span_y).round() as i32;

        unsafe {
            if index == 0 {
                MoveToEx(hdc, x, y, null_mut());
            } else {
                LineTo(hdc, x, y);
            }
        }
    }

    unsafe {
        SelectObject(hdc, old_pen);
        DeleteObject(pen);
    }
}

fn draw_binding_line(hdc: Hdc, x: i32, y: i32, role: InputRole, binding: Option<&BindingView>) {
    let width = if x > 600 { 112 } else { 520 };
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
    draw_text_kind(hdc, x, y, width, 22, &text, TextKind::Meta, color_muted());
}

fn draw_axis(hdc: Hdc, x: i32, y: i32, width: i32, axis: &AxisSnapshot) {
    let percent = axis.percent();
    let label = format!("{}  raw={}  {:.0}%", axis.label, axis.raw, percent * 100.0);
    draw_text_kind(hdc, x, y, width, 18, &label, TextKind::Meta, color_muted());
    draw_bar(hdc, x, y + 20, width, 12, percent, color_accent());
}

fn draw_value_bar(hdc: Hdc, x: i32, y: i32, width: i32, label: &str, value: f32, color: Dword) {
    draw_panel(hdc, x, y, width, 94, color_panel_raised());
    let text = format!("{}  {:.0}%", label, value * 100.0);
    draw_text_kind(
        hdc,
        x + 14,
        y + 12,
        width - 28,
        24,
        &text,
        TextKind::Heading,
        color_text(),
    );
    draw_bar(hdc, x + 14, y + 52, width - 28, 18, value, color);
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

    let empty_brush = unsafe { CreateSolidBrush(color_track()) };
    let fill_brush = unsafe { CreateSolidBrush(fill_color) };
    unsafe {
        FillRect(hdc, &bar, empty_brush);
        FillRect(hdc, &filled, fill_brush);
        DeleteObject(empty_brush);
        DeleteObject(fill_brush);
    }
}

fn draw_text(hdc: Hdc, x: i32, y: i32, width: i32, height: i32, text: &str) {
    draw_text_kind(hdc, x, y, width, height, text, TextKind::Body, color_text());
}

fn draw_text_kind(
    hdc: Hdc,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    text: &str,
    kind: TextKind,
    color: Dword,
) {
    let mut rect = Rect {
        left: x,
        top: y,
        right: x + width,
        bottom: y + height,
    };
    let wide_text = wide(text);
    let old_font = select_text_font(hdc, kind);
    unsafe {
        SetTextColor(hdc, color);
        DrawTextW(
            hdc,
            wide_text.as_ptr(),
            (wide_text.len() - 1) as i32,
            &mut rect,
            DT_LEFT | DT_TOP | DT_SINGLELINE,
        );
        if old_font != 0 {
            SelectObject(hdc, old_font);
        }
    }
}

fn draw_centered_text_kind(hdc: Hdc, rect: Rect, text: &str, kind: TextKind, color: Dword) {
    let mut rect = rect;
    let wide_text = wide(text);
    let old_font = select_text_font(hdc, kind);
    unsafe {
        SetTextColor(hdc, color);
        DrawTextW(
            hdc,
            wide_text.as_ptr(),
            (wide_text.len() - 1) as i32,
            &mut rect,
            DT_CENTER | DT_VCENTER | DT_SINGLELINE,
        );
        if old_font != 0 {
            SelectObject(hdc, old_font);
        }
    }
}

fn select_text_font(hdc: Hdc, kind: TextKind) -> Hgdiobj {
    let Some(fonts) = FONTS.get() else {
        return 0;
    };
    let font = match kind {
        TextKind::Title => fonts.title,
        TextKind::Heading => fonts.heading,
        TextKind::Body => fonts.body,
        TextKind::Meta => fonts.meta,
    };
    unsafe { SelectObject(hdc, font as Hgdiobj) }
}

fn color_app_bg() -> Dword {
    rgb(14, 16, 20)
}

fn color_panel() -> Dword {
    rgb(24, 27, 34)
}

fn color_panel_raised() -> Dword {
    rgb(31, 35, 44)
}

fn color_border() -> Dword {
    rgb(48, 55, 68)
}

fn color_text() -> Dword {
    rgb(236, 240, 247)
}

fn color_muted() -> Dword {
    rgb(150, 160, 174)
}

fn color_accent() -> Dword {
    rgb(0, 210, 183)
}

fn color_track() -> Dword {
    rgb(48, 55, 66)
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
