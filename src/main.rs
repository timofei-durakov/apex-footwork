#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod alerts;
mod notifications;
mod profile;
mod wizard;

use alerts::{AlertSensitivity, AlertSettings, AlertView, LIVE_SAMPLE_INTERVAL_MS};
use std::ffi::c_void;
use std::mem::{size_of, zeroed};
use std::ptr::{null, null_mut};
use std::sync::{Mutex, OnceLock};
use wizard::{
    AxisSnapshot, BindingCalibration, BindingView, DeviceProvider, DeviceSnapshot, InputRole,
    LiveSample, PedalBindings, Wizard, WizardCommand, WizardStepView, WizardView,
};

type Bool = i32;
type Dword = u32;
type Hbrush = isize;
type Hcursor = isize;
type Hdc = isize;
type Hfont = isize;
type Hgdiobj = isize;
type Hicon = isize;
type Hinstance = isize;
type Hkey = isize;
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

const APP_ICON_RESOURCE_ID: usize = 1;
const UI_TIMER_INTERVAL_MS: Uint = LIVE_SAMPLE_INTERVAL_MS as Uint;
const WM_CREATE: Uint = 0x0001;
const WM_DESTROY: Uint = 0x0002;
const WM_PAINT: Uint = 0x000F;
const WM_ERASEBKGND: Uint = 0x0014;
const WM_NCLBUTTONDOWN: Uint = 0x00A1;
const WM_COMMAND: Uint = 0x0111;
const WM_TIMER: Uint = 0x0113;
const WM_KEYDOWN: Uint = 0x0100;
const WM_LBUTTONDOWN: Uint = 0x0201;
const WM_MOUSEMOVE: Uint = 0x0200;
const WM_LBUTTONUP: Uint = 0x0202;
const WM_SETFONT: Uint = 0x0030;
const VK_RETURN: Wparam = 0x0D;
const VK_SHIFT: i32 = 0x10;
const VK_CONTROL: i32 = 0x11;
const KEY_O: Wparam = 0x4F;
const KEY_R: Wparam = 0x52;
const SW_HIDE: i32 = 0;
const SW_SHOWNOACTIVATE: i32 = 4;
const SW_SHOW: i32 = 5;
const CS_HREDRAW: Uint = 0x0002;
const CS_VREDRAW: Uint = 0x0001;
const DT_LEFT: Uint = 0x0000;
const DT_CENTER: Uint = 0x0001;
const DT_VCENTER: Uint = 0x0004;
const DT_TOP: Uint = 0x0000;
const DT_SINGLELINE: Uint = 0x0020;
const WS_CHILD: Dword = 0x4000_0000;
const WS_POPUP: Dword = 0x8000_0000;
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
const HTCAPTION: Wparam = 2;
const WS_EX_TOPMOST: Dword = 0x0000_0008;
const WS_EX_TRANSPARENT: Dword = 0x0000_0020;
const WS_EX_TOOLWINDOW: Dword = 0x0000_0080;
const WS_EX_LAYERED: Dword = 0x0008_0000;
const WS_EX_NOACTIVATE: Dword = 0x0800_0000;
const LWA_ALPHA: Dword = 0x0000_0002;
const ERROR_SUCCESS: i32 = 0;
const HKEY_CURRENT_USER: Hkey = 0x8000_0001u32 as Hkey;
const HKEY_LOCAL_MACHINE: Hkey = 0x8000_0002u32 as Hkey;
const KEY_READ: Dword = 0x0002_0019;
const REG_SZ: Dword = 1;
const REG_EXPAND_SZ: Dword = 2;

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
    fn GetKeyState(nVirtKey: i32) -> i16;
    fn InvalidateRect(hWnd: Hwnd, lpRect: *const Rect, bErase: Bool) -> Bool;
    fn LoadCursorW(hInstance: Hinstance, lpCursorName: *const u16) -> Hcursor;
    fn LoadIconW(hInstance: Hinstance, lpIconName: *const u16) -> Hicon;
    fn PostQuitMessage(nExitCode: i32);
    fn RegisterClassW(lpWndClass: *const WndClassW) -> u16;
    fn SendMessageW(hWnd: Hwnd, Msg: Uint, wParam: Wparam, lParam: Lparam) -> Lresult;
    fn SetCapture(hWnd: Hwnd) -> Hwnd;
    fn SetLayeredWindowAttributes(hwnd: Hwnd, crKey: Dword, bAlpha: u8, dwFlags: Dword) -> Bool;
    fn SetTimer(hWnd: Hwnd, nIDEvent: usize, uElapse: Uint, lpTimerFunc: *const c_void) -> usize;
    fn ShowWindow(hWnd: Hwnd, nCmdShow: i32) -> Bool;
    fn TranslateMessage(lpMsg: *const Msg) -> Bool;
    fn UpdateWindow(hWnd: Hwnd) -> Bool;
    fn ReleaseCapture() -> Bool;
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

#[link(name = "advapi32")]
unsafe extern "system" {
    fn RegCloseKey(hKey: Hkey) -> i32;
    fn RegOpenKeyExW(
        hKey: Hkey,
        lpSubKey: *const u16,
        ulOptions: Dword,
        samDesired: Dword,
        phkResult: *mut Hkey,
    ) -> i32;
    fn RegQueryValueExW(
        hKey: Hkey,
        lpValueName: *const u16,
        lpReserved: *mut Dword,
        lpType: *mut Dword,
        lpData: *mut u8,
        lpcbData: *mut Dword,
    ) -> i32;
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
                name: joystick_display_name(id, &caps),
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
static OVERLAY_SETTINGS: OnceLock<Mutex<OverlaySettings>> = OnceLock::new();
static RESIZE_DRAG: OnceLock<Mutex<Option<ResizeDrag>>> = OnceLock::new();
static OVERLAY_HWND: OnceLock<Mutex<Hwnd>> = OnceLock::new();
static ALERT_NOTIFICATION_HWND: OnceLock<Mutex<Hwnd>> = OnceLock::new();
static DEVICE_DROPDOWN_OPEN: OnceLock<Mutex<bool>> = OnceLock::new();

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
}

#[derive(Clone, Copy)]
struct OverlaySettings {
    chart_width: i32,
    chart_height: i32,
    chart_opacity: f32,
    show_steering_graph: bool,
    steering_scale: SteeringGraphScale,
    alert_settings: AlertSettings,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SteeringGraphScale {
    Linear,
    Log,
}

#[derive(Clone, Copy)]
struct ResizeDrag {
    start_x: i32,
    start_y: i32,
    start_width: i32,
    start_height: i32,
}

#[derive(Clone, Copy)]
struct OverlayLayout {
    content_left: i32,
    content_width: i32,
    right_x: i32,
    column_width: i32,
    card_y: i32,
    controls_y: i32,
    chart_area: Rect,
    chart: Rect,
    steering_chart: Option<Rect>,
    opacity_minus: Rect,
    opacity_plus: Rect,
    alerts_toggle: Rect,
    steering_toggle: Rect,
    sensitivity_toggle: Rect,
}

#[derive(Clone, Copy)]
struct CaptureLayout {
    left: i32,
    button: Rect,
    advanced_toggle: Rect,
    advanced_description: Rect,
    axes_panel: Rect,
    binding_panel: Rect,
}

impl Default for OverlaySettings {
    fn default() -> Self {
        Self {
            chart_width: 0,
            chart_height: 168,
            chart_opacity: 0.9,
            show_steering_graph: true,
            steering_scale: SteeringGraphScale::Log,
            alert_settings: AlertSettings::default(),
        }
    }
}

fn main() {
    OVERLAY_SETTINGS.get_or_init(|| Mutex::new(OverlaySettings::default()));
    RESIZE_DRAG.get_or_init(|| Mutex::new(None));
    OVERLAY_HWND.get_or_init(|| Mutex::new(0));
    ALERT_NOTIFICATION_HWND.get_or_init(|| Mutex::new(0));
    DEVICE_DROPDOWN_OPEN.get_or_init(|| Mutex::new(false));

    let mut wizard = Wizard::new(WinmmDeviceProvider);
    if let Some(saved_profile) = profile::load_profile() {
        apply_saved_overlay_settings(&saved_profile.overlay_settings);
        if wizard.restore_profile(&saved_profile) {
            remember_saved_profile(&saved_profile);
        }
    }

    STATE.get_or_init(|| Mutex::new(wizard));
    unsafe { run_window() };
}

unsafe fn run_window() {
    let class_name = wide("ApexFootworkWindow");
    let overlay_class_name = wide("ApexFootworkOverlayWindow");
    let notification_class_name = wide("ApexFootworkAlertNotificationWindow");
    let title = wide("Apex Footwork");
    let h_instance = unsafe { GetModuleHandleW(null()) };
    let cursor = unsafe { LoadCursorW(0, 32512usize as *const u16) };
    let icon = unsafe { LoadIconW(h_instance, APP_ICON_RESOURCE_ID as *const u16) };

    let wc = WndClassW {
        style: CS_HREDRAW | CS_VREDRAW,
        lpfn_wnd_proc: Some(window_proc),
        cb_cls_extra: 0,
        cb_wnd_extra: 0,
        h_instance,
        h_icon: icon,
        h_cursor: cursor,
        hbr_background: 0,
        lpsz_menu_name: null(),
        lpsz_class_name: class_name.as_ptr(),
    };

    unsafe { RegisterClassW(&wc) };
    let overlay_wc = WndClassW {
        style: CS_HREDRAW | CS_VREDRAW,
        lpfn_wnd_proc: Some(overlay_window_proc),
        cb_cls_extra: 0,
        cb_wnd_extra: 0,
        h_instance,
        h_icon: icon,
        h_cursor: cursor,
        hbr_background: 0,
        lpsz_menu_name: null(),
        lpsz_class_name: overlay_class_name.as_ptr(),
    };
    unsafe { RegisterClassW(&overlay_wc) };
    let notification_wc = WndClassW {
        style: CS_HREDRAW | CS_VREDRAW,
        lpfn_wnd_proc: Some(alert_notification_window_proc),
        cb_cls_extra: 0,
        cb_wnd_extra: 0,
        h_instance,
        h_icon: icon,
        h_cursor: cursor,
        hbr_background: 0,
        lpsz_menu_name: null(),
        lpsz_class_name: notification_class_name.as_ptr(),
    };
    unsafe { RegisterClassW(&notification_wc) };

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
            unsafe { SetTimer(hwnd, 1, UI_TIMER_INTERVAL_MS, null()) };
            create_controls(hwnd);
            0
        }
        WM_ERASEBKGND => 1,
        WM_TIMER => {
            with_wizard(|wizard| wizard.update());
            sync_alert_notification_window();
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
        WM_MOUSEMOVE => {
            if handle_mouse_move(hwnd, lparam) {
                unsafe { InvalidateRect(hwnd, null(), 0) };
                return 0;
            }
            unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
        }
        WM_LBUTTONUP => {
            if finish_resize_drag() {
                unsafe {
                    ReleaseCapture();
                    InvalidateRect(hwnd, null(), 0);
                }
                return 0;
            }
            unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
        }
        WM_KEYDOWN => {
            if handle_key_down(wparam) {
                sync_controls();
                unsafe { InvalidateRect(hwnd, null(), 0) };
                return 0;
            }
            unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
        }
        WM_PAINT => {
            draw(hwnd);
            0
        }
        WM_DESTROY => {
            close_overlay_window();
            close_alert_notification_window();
            unsafe { PostQuitMessage(0) };
            0
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

unsafe extern "system" fn overlay_window_proc(
    hwnd: Hwnd,
    msg: Uint,
    wparam: Wparam,
    lparam: Lparam,
) -> Lresult {
    match msg {
        WM_CREATE => {
            unsafe { SetTimer(hwnd, 2, UI_TIMER_INTERVAL_MS, null()) };
            apply_overlay_window_opacity(hwnd);
            0
        }
        WM_ERASEBKGND => 1,
        WM_TIMER => {
            with_wizard(|wizard| wizard.update());
            apply_overlay_window_opacity(hwnd);
            unsafe { InvalidateRect(hwnd, null(), 0) };
            0
        }
        WM_LBUTTONDOWN => {
            unsafe {
                ReleaseCapture();
                SendMessageW(hwnd, WM_NCLBUTTONDOWN, HTCAPTION, 0);
            }
            0
        }
        WM_KEYDOWN => {
            if handle_shortcut_key(wparam) {
                sync_controls();
                return 0;
            }
            unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
        }
        WM_PAINT => {
            draw_overlay_window(hwnd);
            0
        }
        WM_DESTROY => {
            set_overlay_hwnd(0);
            0
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

unsafe extern "system" fn alert_notification_window_proc(
    hwnd: Hwnd,
    msg: Uint,
    wparam: Wparam,
    lparam: Lparam,
) -> Lresult {
    match msg {
        WM_ERASEBKGND => 1,
        WM_DESTROY => {
            set_alert_notification_hwnd(0);
            0
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

fn start_overlay_window() {
    if !is_ready_view() {
        return;
    }

    if let Some(hwnd) = active_overlay_hwnd() {
        unsafe {
            apply_overlay_window_opacity(hwnd);
            ShowWindow(hwnd, SW_SHOW);
            UpdateWindow(hwnd);
        }
        return;
    }

    let class_name = wide("ApexFootworkOverlayWindow");
    let title = wide("Apex Footwork overlay");
    let h_instance = unsafe { GetModuleHandleW(null()) };
    let (width, height) = overlay_window_size();
    let hwnd = unsafe {
        CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_LAYERED,
            class_name.as_ptr(),
            title.as_ptr(),
            WS_POPUP,
            220,
            220,
            width,
            height,
            0,
            0,
            h_instance,
            null_mut(),
        )
    };

    if hwnd == 0 {
        return;
    }

    set_overlay_hwnd(hwnd);
    unsafe {
        apply_overlay_window_opacity(hwnd);
        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);
    }
}

fn active_overlay_hwnd() -> Option<Hwnd> {
    let hwnd = OVERLAY_HWND
        .get_or_init(|| Mutex::new(0))
        .lock()
        .map(|overlay| *overlay)
        .unwrap_or(0);
    (hwnd != 0).then_some(hwnd)
}

fn set_overlay_hwnd(hwnd: Hwnd) {
    if let Ok(mut overlay) = OVERLAY_HWND.get_or_init(|| Mutex::new(0)).lock() {
        *overlay = hwnd;
    }
}

fn take_overlay_hwnd() -> Hwnd {
    let Ok(mut overlay) = OVERLAY_HWND.get_or_init(|| Mutex::new(0)).lock() else {
        return 0;
    };
    let hwnd = *overlay;
    *overlay = 0;
    hwnd
}

fn close_overlay_window() {
    let hwnd = take_overlay_hwnd();
    if hwnd != 0 {
        unsafe {
            DestroyWindow(hwnd);
        }
    }
    close_alert_notification_window();
}

fn start_alert_notification_window() -> Option<Hwnd> {
    if let Some(hwnd) = active_alert_notification_hwnd() {
        return Some(hwnd);
    }

    let class_name = wide("ApexFootworkAlertNotificationWindow");
    let title = wide("Apex Footwork alert notification");
    let h_instance = unsafe { GetModuleHandleW(null()) };
    let hwnd = unsafe {
        CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_NOACTIVATE,
            class_name.as_ptr(),
            title.as_ptr(),
            WS_POPUP,
            0,
            0,
            1,
            1,
            0,
            0,
            h_instance,
            null_mut(),
        )
    };

    if hwnd == 0 {
        return None;
    }
    set_alert_notification_hwnd(hwnd);
    Some(hwnd)
}

fn active_alert_notification_hwnd() -> Option<Hwnd> {
    let hwnd = ALERT_NOTIFICATION_HWND
        .get_or_init(|| Mutex::new(0))
        .lock()
        .map(|notification| *notification)
        .unwrap_or(0);
    (hwnd != 0).then_some(hwnd)
}

fn set_alert_notification_hwnd(hwnd: Hwnd) {
    if let Ok(mut notification) = ALERT_NOTIFICATION_HWND.get_or_init(|| Mutex::new(0)).lock() {
        *notification = hwnd;
    }
}

fn take_alert_notification_hwnd() -> Hwnd {
    let Ok(mut notification) = ALERT_NOTIFICATION_HWND.get_or_init(|| Mutex::new(0)).lock() else {
        return 0;
    };
    let hwnd = *notification;
    *notification = 0;
    hwnd
}

fn hide_alert_notification_window() {
    if let Some(hwnd) = active_alert_notification_hwnd() {
        unsafe { ShowWindow(hwnd, SW_HIDE) };
    }
}

fn close_alert_notification_window() {
    notifications::reset_presentation();
    let hwnd = take_alert_notification_hwnd();
    if hwnd != 0 {
        unsafe {
            DestroyWindow(hwnd);
        }
    }
}

fn sync_alert_notification_window() {
    let overlay_active = active_overlay_hwnd().is_some();
    let alerts_enabled = overlay_settings().alert_settings.enabled;
    if !should_show_alert_notification(is_ready_view(), alerts_enabled, overlay_active) {
        notifications::reset_presentation();
        hide_alert_notification_window();
        return;
    }

    let alerts = current_alerts();
    let Some(frame) = notifications::next_frame(&alerts, LIVE_SAMPLE_INTERVAL_MS) else {
        hide_alert_notification_window();
        return;
    };
    let Some(hwnd) = start_alert_notification_window() else {
        return;
    };
    if notifications::render_frame(hwnd, active_overlay_hwnd(), frame) {
        unsafe {
            ShowWindow(hwnd, SW_SHOWNOACTIVATE);
        }
    }
}

fn should_show_alert_notification(ready: bool, alerts_enabled: bool, overlay_active: bool) -> bool {
    ready && alerts_enabled && overlay_active
}

fn overlay_window_size() -> (i32, i32) {
    let settings = overlay_settings();
    let width = if settings.chart_width <= 0 {
        if settings.show_steering_graph {
            640
        } else {
            520
        }
    } else {
        settings.chart_width
    }
    .clamp(260, 900);
    let height = settings.chart_height.clamp(96, 360);
    (width, height)
}

fn apply_overlay_window_opacity(hwnd: Hwnd) {
    let alpha = (overlay_settings().chart_opacity.clamp(0.3, 1.0) * 255.0).round() as u8;
    unsafe {
        SetLayeredWindowAttributes(hwnd, 0, alpha, LWA_ALPHA);
    }
}

fn handle_key_down(key: Wparam) -> bool {
    if handle_shortcut_key(key) {
        return true;
    }

    if let Some(command) = command_from_key(key) {
        with_wizard(|wizard| wizard.handle_command(command));
        return true;
    }

    false
}

fn handle_shortcut_key(key: Wparam) -> bool {
    if !shortcut_modifiers_down() {
        return false;
    }

    match key {
        KEY_O => toggle_overlay_window(),
        KEY_R => {
            close_overlay_window();
            close_alert_notification_window();
            with_wizard(|wizard| wizard.handle_command(WizardCommand::Configure));
            true
        }
        _ => false,
    }
}

fn toggle_overlay_window() -> bool {
    if active_overlay_hwnd().is_some() {
        close_overlay_window();
        return true;
    }

    if !is_ready_view() {
        return false;
    }

    start_overlay_window();
    true
}

fn shortcut_modifiers_down() -> bool {
    unsafe { GetKeyState(VK_CONTROL) < 0 && GetKeyState(VK_SHIFT) < 0 }
}

fn command_from_key(key: Wparam) -> Option<WizardCommand> {
    match key {
        VK_RETURN => Some(WizardCommand::Confirm),
        _ => None,
    }
}

fn handle_mouse_down(hwnd: Hwnd, lparam: Lparam) -> bool {
    let mut rect: Rect = unsafe { zeroed() };
    unsafe { GetClientRect(hwnd, &mut rect) };

    let x = signed_loword(lparam as usize);
    let y = signed_hiword(lparam as usize);
    if is_ready_view() && point_in_rect(x, y, start_button_rect(rect)) {
        if active_overlay_hwnd().is_some() {
            close_overlay_window();
        } else {
            start_overlay_window();
        }
        return true;
    }

    if point_in_rect(x, y, configure_button_rect(rect)) {
        close_overlay_window();
        close_device_dropdown();
        with_wizard(|wizard| wizard.handle_command(WizardCommand::Configure));
        return true;
    }

    if handle_device_dropdown_click(rect, x, y) {
        return true;
    }

    if handle_primary_action_click(rect, x, y) {
        close_device_dropdown();
        return true;
    }

    if handle_capture_settings_click(rect, x, y) {
        close_device_dropdown();
        return true;
    }

    if start_resize_drag(hwnd, rect, x, y) {
        return true;
    }

    if handle_overlay_settings_click(rect, x, y) {
        return true;
    }

    false
}

fn handle_device_dropdown_click(client: Rect, x: i32, y: i32) -> bool {
    let Some(view) = current_view() else {
        return false;
    };

    let WizardStepView::SelectDevice {
        devices,
        selected_index: _,
    } = view.step
    else {
        close_device_dropdown();
        return false;
    };

    let header = device_dropdown_rect(client);
    if point_in_rect(x, y, header) {
        if !devices.is_empty() {
            toggle_device_dropdown();
        }
        return true;
    }

    if !is_device_dropdown_open() {
        return false;
    }

    for index in 0..devices.len() {
        if point_in_rect(x, y, device_dropdown_item_rect(header, index)) {
            with_wizard(|wizard| wizard.handle_command(WizardCommand::SelectDevice(index)));
            close_device_dropdown();
            return true;
        }
    }

    close_device_dropdown();
    false
}

fn handle_primary_action_click(client: Rect, x: i32, y: i32) -> bool {
    let Some(view) = current_view() else {
        return false;
    };

    match view.step {
        WizardStepView::SelectDevice { devices, .. } => {
            if devices.is_empty() || !point_in_rect(x, y, select_primary_button_rect(client)) {
                return false;
            }
        }
        WizardStepView::Capture { armed, .. } => {
            if armed || !point_in_rect(x, y, capture_layout(client).button) {
                return false;
            }
        }
        WizardStepView::Ready { .. } => return false,
    }

    with_wizard(|wizard| wizard.handle_command(WizardCommand::Confirm));
    true
}

fn handle_capture_settings_click(client: Rect, x: i32, y: i32) -> bool {
    let Some(view) = current_view() else {
        return false;
    };

    let WizardStepView::Capture { armed, .. } = view.step else {
        return false;
    };

    if armed || !point_in_rect(x, y, capture_layout(client).advanced_toggle) {
        return false;
    }

    with_wizard(|wizard| wizard.handle_command(WizardCommand::ToggleAdvancedCalibration));
    true
}

fn handle_mouse_move(client_hwnd: Hwnd, lparam: Lparam) -> bool {
    let is_dragging = RESIZE_DRAG
        .get()
        .and_then(|lock| lock.lock().ok().map(|drag| drag.is_some()))
        .unwrap_or(false);
    if !is_dragging {
        return false;
    }

    let x = signed_loword(lparam as usize);
    let y = signed_hiword(lparam as usize);
    let mut rect: Rect = unsafe { zeroed() };
    unsafe { GetClientRect(client_hwnd, &mut rect) };
    resize_chart_to_mouse(rect, x, y);
    true
}

fn start_resize_drag(hwnd: Hwnd, client: Rect, x: i32, y: i32) -> bool {
    if !is_ready_view() {
        return false;
    }

    let settings = overlay_settings();
    let layout = overlay_layout(client, settings);
    if !point_in_rect(x, y, chart_resize_handle_rect(layout.chart_area)) {
        return false;
    }

    if let Ok(mut drag) = RESIZE_DRAG.get_or_init(|| Mutex::new(None)).lock() {
        *drag = Some(ResizeDrag {
            start_x: x,
            start_y: y,
            start_width: layout.chart_area.right - layout.chart_area.left,
            start_height: layout.chart_area.bottom - layout.chart_area.top,
        });
    }
    unsafe { SetCapture(hwnd) };
    true
}

fn resize_chart_to_mouse(client: Rect, x: i32, y: i32) {
    let drag = {
        let Ok(drag) = RESIZE_DRAG.get_or_init(|| Mutex::new(None)).lock() else {
            return;
        };
        let Some(drag) = *drag else {
            return;
        };
        drag
    };

    let footer_top = client.bottom - 48;
    let layout = overlay_layout(client, overlay_settings());
    let max_width = (client.right - 64).max(260);
    let max_height = (footer_top - layout.chart_area.top - 20).max(48).min(360);
    let new_width = (drag.start_width + (x - drag.start_x)).clamp(260, max_width);
    let min_height = 64.min(max_height);
    let new_height = (drag.start_height + (y - drag.start_y)).clamp(min_height, max_height);

    if let Some(settings_lock) = OVERLAY_SETTINGS.get() {
        if let Ok(mut settings) = settings_lock.lock() {
            settings.chart_width = new_width;
            settings.chart_height = new_height;
        }
    }
}

fn finish_resize_drag() -> bool {
    let Some(lock) = RESIZE_DRAG.get() else {
        return false;
    };
    let Ok(mut drag) = lock.lock() else {
        return false;
    };
    let was_dragging = drag.is_some();
    *drag = None;
    was_dragging
}

fn handle_overlay_settings_click(client: Rect, x: i32, y: i32) -> bool {
    let Some(view) = current_view() else {
        return false;
    };
    if !matches!(view.step, WizardStepView::Ready { .. }) {
        return false;
    }

    let Some(lock) = OVERLAY_SETTINGS.get() else {
        return false;
    };
    let Ok(mut settings) = lock.lock() else {
        return false;
    };
    let layout = overlay_layout(client, *settings);

    if point_in_rect(x, y, layout.opacity_minus) {
        settings.chart_opacity = (settings.chart_opacity - 0.1).max(0.3);
        drop(settings);
        save_current_profile();
        return true;
    }
    if point_in_rect(x, y, layout.opacity_plus) {
        settings.chart_opacity = (settings.chart_opacity + 0.1).min(1.0);
        drop(settings);
        save_current_profile();
        return true;
    }
    if point_in_rect(x, y, layout.alerts_toggle) {
        settings.alert_settings.enabled = !settings.alert_settings.enabled;
        let alert_settings = settings.alert_settings;
        drop(settings);
        with_wizard(|wizard| wizard.set_alert_settings(alert_settings));
        sync_alert_notification_window();
        return true;
    }
    if point_in_rect(x, y, layout.steering_toggle) {
        settings.show_steering_graph = !settings.show_steering_graph;
        drop(settings);
        save_current_profile();
        return true;
    }
    if point_in_rect(x, y, layout.sensitivity_toggle) {
        settings.steering_scale = if settings.steering_scale == SteeringGraphScale::Log {
            SteeringGraphScale::Linear
        } else {
            SteeringGraphScale::Log
        };
        drop(settings);
        save_current_profile();
        return true;
    }

    false
}

fn is_ready_view() -> bool {
    current_view()
        .map(|view| matches!(view.step, WizardStepView::Ready { .. }))
        .unwrap_or(false)
}

fn current_alerts() -> Vec<AlertView> {
    current_view()
        .and_then(|view| match view.step {
            WizardStepView::Ready { alerts, .. } => Some(alerts),
            _ => None,
        })
        .unwrap_or_default()
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
    let Some(mut profile) = wizard.profile() else {
        return;
    };
    profile.overlay_settings = stored_overlay_settings();

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

fn save_current_profile() {
    let Some(lock) = STATE.get() else {
        return;
    };
    let Ok(wizard) = lock.lock() else {
        return;
    };
    maybe_save_profile(&wizard);
}

fn apply_saved_overlay_settings(stored: &profile::StoredOverlaySettings) {
    if let Ok(mut settings) = OVERLAY_SETTINGS
        .get_or_init(|| Mutex::new(OverlaySettings::default()))
        .lock()
    {
        settings.show_steering_graph = stored.steering_graph;
        settings.steering_scale = match stored.steering_scale {
            profile::StoredSteeringScale::Linear => SteeringGraphScale::Linear,
            profile::StoredSteeringScale::Log => SteeringGraphScale::Log,
        };
        settings.alert_settings = alert_settings_from_stored(stored);
    }
}

fn stored_overlay_settings() -> profile::StoredOverlaySettings {
    let settings = overlay_settings();
    profile::StoredOverlaySettings {
        steering_graph: settings.show_steering_graph,
        steering_scale: match settings.steering_scale {
            SteeringGraphScale::Linear => profile::StoredSteeringScale::Linear,
            SteeringGraphScale::Log => profile::StoredSteeringScale::Log,
        },
        alerts_enabled: settings.alert_settings.enabled,
        alert_sensitivity: match settings.alert_settings.sensitivity {
            AlertSensitivity::Quiet => profile::StoredAlertSensitivity::Quiet,
            AlertSensitivity::Balanced => profile::StoredAlertSensitivity::Balanced,
            AlertSensitivity::Sensitive => profile::StoredAlertSensitivity::Sensitive,
        },
    }
}

fn alert_settings_from_stored(stored: &profile::StoredOverlaySettings) -> AlertSettings {
    AlertSettings {
        enabled: stored.alerts_enabled,
        sensitivity: match stored.alert_sensitivity {
            profile::StoredAlertSensitivity::Quiet => AlertSensitivity::Quiet,
            profile::StoredAlertSensitivity::Balanced => AlertSensitivity::Balanced,
            profile::StoredAlertSensitivity::Sensitive => AlertSensitivity::Sensitive,
        },
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
    }));

    apply_control_style(device_combo);
    apply_control_style(primary_button);
    sync_controls();
}

fn select_primary_button_rect(client: Rect) -> Rect {
    let width = 124;
    let right = (client.right - 48).max(width + 48);
    rect_xywh(right - width, 153, width, 30)
}

fn device_dropdown_rect(client: Rect) -> Rect {
    let button = select_primary_button_rect(client);
    let left = 32;
    let right = (button.left - 16).max(left + 260);
    rect_xywh(left, 153, right - left, 34)
}

fn device_dropdown_item_rect(header: Rect, index: usize) -> Rect {
    rect_xywh(
        header.left,
        header.bottom + 4 + (index as i32 * 32),
        header.right - header.left,
        32,
    )
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
            set_visible(controls.device_combo, &mut controls.combo_visible, false);
            set_visible(
                controls.primary_button,
                &mut controls.primary_visible,
                false,
            );
            set_enabled(
                controls.primary_button,
                &mut controls.primary_enabled,
                false,
            );
        }
        WizardStepView::Capture { .. } => {
            close_device_dropdown();
            set_visible(controls.device_combo, &mut controls.combo_visible, false);
            set_visible(
                controls.primary_button,
                &mut controls.primary_visible,
                false,
            );
            set_enabled(
                controls.primary_button,
                &mut controls.primary_enabled,
                false,
            );
        }
        WizardStepView::Ready { .. } => {
            close_device_dropdown();
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
            let text = wide(&device_display_label(device));
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

fn is_device_dropdown_open() -> bool {
    DEVICE_DROPDOWN_OPEN
        .get_or_init(|| Mutex::new(false))
        .lock()
        .map(|open| *open)
        .unwrap_or(false)
}

fn toggle_device_dropdown() {
    if let Ok(mut open) = DEVICE_DROPDOWN_OPEN
        .get_or_init(|| Mutex::new(false))
        .lock()
    {
        *open = !*open;
    }
}

fn close_device_dropdown() {
    if let Ok(mut open) = DEVICE_DROPDOWN_OPEN
        .get_or_init(|| Mutex::new(false))
        .lock()
    {
        *open = false;
    }
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

fn joystick_display_name(device_id: u32, caps: &JoyCapsW) -> String {
    let fallback = name_from_wide(&caps.sz_pname);
    let mut oem_keys = Vec::new();
    push_unique_nonempty(&mut oem_keys, name_from_wide(&caps.sz_reg_key));

    let settings_value = format!("Joystick{}OEMName", device_id + 1);
    for root in [HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE] {
        if let Some(oem_key) = registry_string(
            root,
            "System\\CurrentControlSet\\Control\\MediaResources\\Joystick\\DINPUT.DLL\\CurrentJoystickSettings",
            &settings_value,
        ) {
            push_unique_nonempty(&mut oem_keys, oem_key);
        }
    }

    for oem_key in oem_keys {
        let oem_path = format!(
            "System\\CurrentControlSet\\Control\\MediaProperties\\PrivateProperties\\Joystick\\OEM\\{}",
            oem_key
        );
        for root in [HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE] {
            if let Some(name) = registry_string(root, &oem_path, "OEMName") {
                let name = name.trim().to_string();
                if !name.is_empty() {
                    return name;
                }
            }
        }
    }

    fallback
}

fn push_unique_nonempty(values: &mut Vec<String>, value: String) {
    let value = value.trim().to_string();
    if !value.is_empty() && !values.iter().any(|existing| existing == &value) {
        values.push(value);
    }
}

fn registry_string(root: Hkey, subkey: &str, value_name: &str) -> Option<String> {
    let subkey = wide(subkey);
    let value_name = wide(value_name);
    let mut key: Hkey = 0;
    let opened = unsafe { RegOpenKeyExW(root, subkey.as_ptr(), 0, KEY_READ, &mut key) };
    if opened != ERROR_SUCCESS {
        return None;
    }

    let result = query_registry_string(key, value_name.as_ptr());
    unsafe {
        RegCloseKey(key);
    }
    result
}

fn query_registry_string(key: Hkey, value_name: *const u16) -> Option<String> {
    let mut value_type: Dword = 0;
    let mut byte_len: Dword = 0;
    let size_result = unsafe {
        RegQueryValueExW(
            key,
            value_name,
            null_mut(),
            &mut value_type,
            null_mut(),
            &mut byte_len,
        )
    };
    if size_result != ERROR_SUCCESS || !matches!(value_type, REG_SZ | REG_EXPAND_SZ) || byte_len < 2
    {
        return None;
    }

    let mut buffer = vec![0u16; (byte_len as usize + 1) / 2];
    let data_result = unsafe {
        RegQueryValueExW(
            key,
            value_name,
            null_mut(),
            &mut value_type,
            buffer.as_mut_ptr() as *mut u8,
            &mut byte_len,
        )
    };
    if data_result != ERROR_SUCCESS || !matches!(value_type, REG_SZ | REG_EXPAND_SZ) {
        return None;
    }

    Some(name_from_wide(&buffer))
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

fn draw_overlay_window(hwnd: Hwnd) {
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
        draw_overlay_scene(hdc, rect);
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
    draw_overlay_scene(mem_dc, rect);
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

fn draw_overlay_scene(hdc: Hdc, rect: Rect) {
    unsafe {
        SetBkMode(hdc, TRANSPARENT);
        SetTextColor(hdc, color_text());
    }

    fill_rect(hdc, rect, color_app_bg());

    let Some(view) = current_view() else {
        draw_text_kind(
            hdc,
            18,
            18,
            (rect.right - 36).max(0),
            22,
            "Overlay not configured",
            TextKind::Meta,
            color_muted(),
        );
        return;
    };

    if let WizardStepView::Ready { history, .. } = &view.step {
        let settings = overlay_settings();
        let (pedal_chart, steering_chart) = split_history_charts(rect, settings);
        draw_combined_overlay_chart(hdc, pedal_chart, history, 1.0, false);
        if let Some(steering_chart) = steering_chart {
            draw_steering_trace(hdc, steering_chart, history, settings.steering_scale, 1.0);
        }
    } else {
        draw_text_kind(
            hdc,
            18,
            18,
            (rect.right - 36).max(0),
            22,
            "Configure pedals first",
            TextKind::Meta,
            color_muted(),
        );
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
    draw_panel_with_border(hdc, x, y, width, height, color, color_border());
}

fn draw_panel_with_border(
    hdc: Hdc,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    color: Dword,
    border: Dword,
) {
    if width <= 0 || height <= 0 {
        return;
    }

    fill_rect(hdc, rect_xywh(x, y, width, height), color);
    fill_rect(hdc, rect_xywh(x, y, width, 1), border);
    fill_rect(hdc, rect_xywh(x, y + height - 1, width, 1), border);
    fill_rect(hdc, rect_xywh(x, y, 1, height), border);
    fill_rect(hdc, rect_xywh(x + width - 1, y, 1, height), border);
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
    let text = if active { rgb(6, 18, 20) } else { color_text() };

    draw_hud_button_colored(hdc, rect, label, fill, border, text);
}

fn draw_hud_button_danger(hdc: Hdc, rect: Rect, label: &str) {
    draw_hud_button_colored(
        hdc,
        rect,
        label,
        rgb(47, 31, 39),
        rgb(255, 82, 96),
        color_text(),
    );
}

fn draw_hud_button_disabled(hdc: Hdc, rect: Rect, label: &str) {
    draw_hud_button_colored(
        hdc,
        rect,
        label,
        rgb(24, 27, 34),
        rgb(48, 55, 68),
        rgb(92, 101, 116),
    );
}

fn draw_hud_button_colored(
    hdc: Hdc,
    rect: Rect,
    label: &str,
    fill: Dword,
    border: Dword,
    text: Dword,
) {
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

    draw_centered_text_kind(hdc, rect, label, TextKind::Body, text);
}

fn draw_checkbox(hdc: Hdc, rect: Rect, label: &str, checked: bool) {
    draw_panel_with_border(
        hdc,
        rect.left,
        rect.top,
        rect.right - rect.left,
        rect.bottom - rect.top,
        color_panel_raised(),
        if checked {
            color_accent()
        } else {
            rgb(74, 84, 102)
        },
    );
    let box_rect = rect_xywh(rect.left + 8, rect.top + 6, 16, 16);
    fill_rect(
        hdc,
        box_rect,
        if checked {
            color_accent()
        } else {
            color_panel()
        },
    );
    if checked {
        fill_rect(
            hdc,
            rect_xywh(box_rect.left + 4, box_rect.top + 4, 8, 8),
            rgb(6, 18, 20),
        );
    }
    draw_text_kind(
        hdc,
        rect.left + 32,
        rect.top + 5,
        rect.right - rect.left - 38,
        18,
        label,
        TextKind::Meta,
        color_text(),
    );
}

fn configure_button_rect(client: Rect) -> Rect {
    let width = 104;
    let right = (client.right - 32).max(width + 32);
    rect_xywh(right - width, 34, width, 34)
}

fn start_button_rect(client: Rect) -> Rect {
    let configure = configure_button_rect(client);
    let width = 82;
    rect_xywh(
        (configure.left - width - 12).max(32),
        configure.top,
        width,
        configure.bottom - configure.top,
    )
}

fn capture_layout(client: Rect) -> CaptureLayout {
    let left = 24;
    let right = (client.right - 24).max(left + 620);
    let gap = 20;
    let content_width = right - left;
    let side_width = content_width.saturating_sub(gap + 500).clamp(170, 212);
    let axes_width = (content_width - side_width - gap).max(360);
    let side_left = left + axes_width + gap;
    let button = rect_xywh(side_left, 126, side_width, 34);
    let advanced_toggle = rect_xywh(left, 174, 220, 24);
    let advanced_description = rect_xywh(
        advanced_toggle.right + 18,
        advanced_toggle.top + 1,
        (side_left - advanced_toggle.right - 34).max(180),
        22,
    );
    let axes_panel = rect_xywh(left, 208, axes_width, 224);
    let binding_panel = rect_xywh(side_left, axes_panel.top, side_width, 156);

    CaptureLayout {
        left,
        button,
        advanced_toggle,
        advanced_description,
        axes_panel,
        binding_panel,
    }
}

fn chart_resize_handle_rect(chart: Rect) -> Rect {
    rect_xywh(chart.right - 28, chart.bottom - 28, 22, 22)
}

fn draw_footer(hdc: Hdc, rect: Rect, ready: bool, overlay_running: bool) {
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
    let hint = if ready && overlay_running {
        "Ctrl+Shift+O stops overlay  |  Ctrl+Shift+R configure  |  Alt+F4 exits"
    } else if ready {
        "Ctrl+Shift+O starts overlay  |  Ctrl+Shift+R configure  |  Alt+F4 exits"
    } else {
        "Enter confirms  |  Ctrl+Shift+R configure  |  Alt+F4 exits"
    };
    draw_text_kind(
        hdc,
        32,
        y,
        (rect.right - 64).max(0),
        22,
        hint,
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
    let ready = matches!(view.step, WizardStepView::Ready { .. });
    let overlay_running = active_overlay_hwnd().is_some();
    let header_right = if ready {
        start_button_rect(rect).left
    } else {
        configure_rect.left
    };
    let header_width = (header_right - 48).max(220);
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
    if ready {
        let action_rect = start_button_rect(rect);
        if overlay_running {
            draw_hud_button_danger(hdc, action_rect, "Stop");
        } else {
            draw_hud_button(hdc, action_rect, "Start", true);
        }
    }
    draw_hud_button(hdc, configure_rect, "Configure", false);

    match &view.step {
        WizardStepView::SelectDevice {
            devices,
            selected_index,
        } => {
            draw_device_picker(hdc, rect, devices, *selected_index);
        }
        WizardStepView::Capture {
            role,
            armed,
            advanced_calibration,
            device,
            bindings,
        } => {
            draw_capture_step(
                hdc,
                rect,
                *role,
                *armed,
                *advanced_calibration,
                device,
                bindings,
            );
        }
        WizardStepView::Ready {
            device,
            bindings,
            values,
            history,
            ..
        } => {
            draw_ready(hdc, rect, device, bindings, values, history);
        }
    }

    draw_footer(hdc, rect, ready, overlay_running);
}

fn draw_device_picker(hdc: Hdc, client: Rect, devices: &[DeviceSnapshot], _selected_index: usize) {
    draw_text(
        hdc,
        24,
        124,
        720,
        22,
        "Choose a controller from the dropdown, then click Use device.",
    );
    draw_device_dropdown(hdc, client, devices, _selected_index);
    if devices.is_empty() {
        draw_hud_button_disabled(hdc, select_primary_button_rect(client), "Use device");
    } else {
        draw_hud_button(hdc, select_primary_button_rect(client), "Use device", true);
    }

    if devices.is_empty() {
        draw_text(hdc, 24, 196, 720, 22, "No devices are currently available.");
    }
}

fn draw_device_dropdown(hdc: Hdc, client: Rect, devices: &[DeviceSnapshot], selected_index: usize) {
    let rect = device_dropdown_rect(client);
    let open = is_device_dropdown_open();
    let label = devices
        .get(selected_index)
        .map(device_display_label)
        .unwrap_or_else(|| "No devices available".to_string());

    draw_panel_with_border(
        hdc,
        rect.left,
        rect.top,
        rect.right - rect.left,
        rect.bottom - rect.top,
        color_panel_raised(),
        if open {
            color_accent()
        } else {
            rgb(74, 84, 102)
        },
    );
    draw_text_kind(
        hdc,
        rect.left + 12,
        rect.top + 8,
        (rect.right - rect.left - 54).max(0),
        18,
        &label,
        TextKind::Meta,
        if devices.is_empty() {
            color_muted()
        } else {
            color_text()
        },
    );
    draw_dropdown_arrow(hdc, rect, open);

    if !open || devices.is_empty() {
        return;
    }

    for (index, device) in devices.iter().enumerate() {
        let item = device_dropdown_item_rect(rect, index);
        let selected = index == selected_index;
        draw_panel_with_border(
            hdc,
            item.left,
            item.top,
            item.right - item.left,
            item.bottom - item.top,
            if selected {
                blend_colors(color_panel_raised(), color_accent(), 0.16)
            } else {
                color_panel_raised()
            },
            rgb(48, 55, 68),
        );
        draw_text_kind(
            hdc,
            item.left + 12,
            item.top + 7,
            item.right - item.left - 24,
            18,
            &device_display_label(device),
            TextKind::Meta,
            if selected {
                color_accent()
            } else {
                color_text()
            },
        );
    }
}

fn device_display_label(device: &DeviceSnapshot) -> String {
    format!("{} ({} axes)", device.name, device.axes.len())
}

fn draw_dropdown_arrow(hdc: Hdc, rect: Rect, open: bool) {
    let center_x = rect.right - 20;
    let center_y = rect.top + ((rect.bottom - rect.top) / 2);
    let color = color_muted();
    if open {
        draw_line(
            hdc,
            center_x - 5,
            center_y + 3,
            center_x,
            center_y - 3,
            color,
            2,
        );
        draw_line(
            hdc,
            center_x,
            center_y - 3,
            center_x + 5,
            center_y + 3,
            color,
            2,
        );
    } else {
        draw_line(
            hdc,
            center_x - 5,
            center_y - 3,
            center_x,
            center_y + 3,
            color,
            2,
        );
        draw_line(
            hdc,
            center_x,
            center_y + 3,
            center_x + 5,
            center_y - 3,
            color,
            2,
        );
    }
}

fn draw_capture_step(
    hdc: Hdc,
    client: Rect,
    role: InputRole,
    armed: bool,
    advanced_calibration: bool,
    device: &Option<DeviceSnapshot>,
    bindings: &PedalBindings,
) {
    let layout = capture_layout(client);
    let device_name = device
        .as_ref()
        .map(|device| device.name.as_str())
        .unwrap_or("No active device");
    draw_text(
        hdc,
        layout.left,
        124,
        (layout.button.left - layout.left - 24).max(260),
        22,
        &format!("Device: {}", device_name),
    );

    let instruction = if role == InputRole::Steering && armed && advanced_calibration {
        "Follow the status prompt: capture full left, release to center, then full right."
            .to_string()
    } else if role == InputRole::Steering && armed {
        "Turn steering right once, then release it. The driver range is used.".to_string()
    } else if role == InputRole::Steering {
        "Center steering, then click Capture Steering.".to_string()
    } else if armed {
        if advanced_calibration {
            format!(
                "Press {} fully, then release it. This custom range becomes 0-100%.",
                role.label()
            )
        } else {
            format!(
                "Press {} once, then release it. The driver range is used for 0-100%.",
                role.label()
            )
        }
    } else {
        format!("Release all pedals, then click Capture {}.", role.label())
    };
    draw_text(
        hdc,
        layout.left,
        150,
        (layout.button.left - layout.left - 24).max(260),
        22,
        &instruction,
    );
    if armed {
        draw_hud_button_disabled(hdc, layout.button, &format!("Waiting {}", role.label()));
    } else {
        draw_hud_button(
            hdc,
            layout.button,
            &format!("Capture {}", role.label()),
            true,
        );
    }
    draw_advanced_calibration_toggle(hdc, layout, role, advanced_calibration, armed);

    if let Some(device) = device {
        draw_panel(
            hdc,
            layout.axes_panel.left,
            layout.axes_panel.top,
            layout.axes_panel.right - layout.axes_panel.left,
            layout.axes_panel.bottom - layout.axes_panel.top,
            color_panel_raised(),
        );
        for (i, axis) in device.axes.iter().enumerate() {
            draw_axis(
                hdc,
                layout.axes_panel.left + 20,
                layout.axes_panel.top + 14 + (i as i32 * 32),
                (layout.axes_panel.right - layout.axes_panel.left - 40).max(120),
                axis,
            );
        }
    }

    draw_panel(
        hdc,
        layout.binding_panel.left,
        layout.binding_panel.top,
        layout.binding_panel.right - layout.binding_panel.left,
        layout.binding_panel.bottom - layout.binding_panel.top,
        color_panel_raised(),
    );
    draw_binding_summary(
        hdc,
        bindings,
        layout.binding_panel.left + 20,
        layout.binding_panel.top + 18,
        layout.binding_panel.right - layout.binding_panel.left - 40,
    );
}

fn draw_advanced_calibration_toggle(
    hdc: Hdc,
    layout: CaptureLayout,
    role: InputRole,
    active: bool,
    disabled: bool,
) {
    let row = layout.advanced_toggle;
    let box_rect = rect_xywh(row.left, row.top + 3, 18, 18);
    let border = if active {
        color_accent()
    } else {
        rgb(74, 84, 102)
    };
    let fill = if active {
        if disabled {
            blend_colors(color_panel(), color_accent(), 0.45)
        } else {
            color_accent()
        }
    } else {
        color_panel_raised()
    };

    fill_rect(hdc, box_rect, fill);
    fill_rect(
        hdc,
        rect_xywh(
            box_rect.left,
            box_rect.top,
            box_rect.right - box_rect.left,
            1,
        ),
        border,
    );
    fill_rect(
        hdc,
        rect_xywh(
            box_rect.left,
            box_rect.bottom - 1,
            box_rect.right - box_rect.left,
            1,
        ),
        border,
    );
    fill_rect(
        hdc,
        rect_xywh(
            box_rect.left,
            box_rect.top,
            1,
            box_rect.bottom - box_rect.top,
        ),
        border,
    );
    fill_rect(
        hdc,
        rect_xywh(
            box_rect.right - 1,
            box_rect.top,
            1,
            box_rect.bottom - box_rect.top,
        ),
        border,
    );
    if active {
        fill_rect(
            hdc,
            rect_xywh(box_rect.left + 5, box_rect.top + 5, 8, 8),
            rgb(6, 18, 20),
        );
    }

    draw_text_kind(
        hdc,
        row.left + 28,
        row.top + 1,
        row.right - row.left - 28,
        22,
        "Advanced calibration",
        TextKind::Meta,
        if disabled {
            color_muted()
        } else {
            color_text()
        },
    );

    let description = if role == InputRole::Steering && active {
        "Custom range: capture center, full left, and full right."
    } else if role == InputRole::Steering {
        "Recommended: use the full steering range calibrated by the driver."
    } else if active {
        "Custom range: press full pedal travel to set 100% inside Apex."
    } else {
        "Recommended: use the range calibrated by the device driver."
    };
    draw_text_kind(
        hdc,
        layout.advanced_description.left,
        layout.advanced_description.top,
        layout.advanced_description.right - layout.advanced_description.left,
        layout.advanced_description.bottom - layout.advanced_description.top,
        description,
        TextKind::Meta,
        color_muted(),
    );
}

fn draw_ready(
    hdc: Hdc,
    rect: Rect,
    device: &Option<DeviceSnapshot>,
    _bindings: &PedalBindings,
    values: &[(InputRole, f32)],
    history: &[LiveSample],
) {
    let device_name = device
        .as_ref()
        .map(|device| device.name.as_str())
        .unwrap_or("No active device");
    let settings = overlay_settings();
    let layout = overlay_layout(rect, settings);

    draw_text_kind(
        hdc,
        layout.content_left,
        132,
        layout.content_width,
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
        layout.content_left,
        layout.card_y,
        layout.column_width,
        "Throttle",
        throttle,
        color_accent(),
    );
    draw_value_bar(
        hdc,
        layout.right_x,
        layout.card_y,
        layout.column_width,
        "Brake",
        brake,
        rgb(255, 82, 96),
    );
    draw_overlay_controls(hdc, layout, settings);
    draw_combined_overlay_chart(hdc, layout.chart, history, settings.chart_opacity, false);
    if let Some(steering_chart) = layout.steering_chart {
        draw_steering_trace(
            hdc,
            steering_chart,
            history,
            settings.steering_scale,
            settings.chart_opacity,
        );
    }
    draw_resize_grip(hdc, layout.chart_area, settings.chart_opacity);
}

fn draw_binding_summary(hdc: Hdc, bindings: &PedalBindings, x: i32, y: i32, width: i32) {
    draw_text_kind(
        hdc,
        x,
        y,
        width,
        22,
        "Bindings",
        TextKind::Heading,
        color_text(),
    );
    draw_binding_line(
        hdc,
        x,
        y + 30,
        width,
        InputRole::Throttle,
        bindings.throttle.as_ref(),
    );
    draw_binding_line(
        hdc,
        x,
        y + 58,
        width,
        InputRole::Brake,
        bindings.brake.as_ref(),
    );
    draw_binding_line(
        hdc,
        x,
        y + 86,
        width,
        InputRole::Steering,
        bindings.steering.as_ref(),
    );
}

fn draw_overlay_controls(hdc: Hdc, layout: OverlayLayout, settings: OverlaySettings) {
    let y = layout.controls_y;
    let opacity_label_left = layout.opacity_minus.left - 104;
    let resize_text_left = layout.alerts_toggle.right + 12;
    let resize_width = (opacity_label_left - resize_text_left - 12).clamp(0, 220);
    if resize_width >= 96 {
        draw_text_kind(
            hdc,
            resize_text_left,
            y + 5,
            resize_width,
            20,
            "Drag graph corner to resize",
            TextKind::Meta,
            color_muted(),
        );
    }

    draw_text_kind(
        hdc,
        opacity_label_left,
        y + 5,
        92,
        20,
        &format!("Opacity {:.0}%", settings.chart_opacity * 100.0),
        TextKind::Meta,
        color_muted(),
    );
    draw_hud_button(hdc, layout.opacity_minus, "-", false);
    draw_hud_button(hdc, layout.opacity_plus, "+", false);

    draw_checkbox(
        hdc,
        layout.alerts_toggle,
        "Sticker alerts",
        settings.alert_settings.enabled,
    );
    draw_checkbox(
        hdc,
        layout.steering_toggle,
        "Steering graph",
        settings.show_steering_graph,
    );
    draw_checkbox(
        hdc,
        layout.sensitivity_toggle,
        "Sensitive steering",
        settings.steering_scale == SteeringGraphScale::Log,
    );
}

fn overlay_layout(client: Rect, settings: OverlaySettings) -> OverlayLayout {
    let content_left = 32;
    let content_right = (client.right - 32).max(content_left + 1);
    let content_width = content_right - content_left;
    let gap = if content_width < 520 { 16 } else { 24 };
    let column_width = ((content_width - gap) / 2).max(120);
    let right_x = content_left + column_width + gap;
    let source_y = 132;
    let card_y = source_y + 34;
    let controls_y = card_y + 112;
    let chart_y = controls_y + 68;
    let footer_top = client.bottom - 48;
    let available_chart_height = (footer_top - chart_y - 20).max(48);
    let chart_area_width = visible_chart_width(client, settings);
    let chart_height = settings
        .chart_height
        .min(available_chart_height)
        .max(64)
        .min(available_chart_height);
    let chart_area = rect_xywh(content_left, chart_y, chart_area_width, chart_height);
    let (chart, steering_chart) = split_history_charts(chart_area, settings);
    let opacity_plus_left = (content_left + 420)
        .min(content_right - 34)
        .max(content_left + 74);
    let opacity_plus = rect_xywh(opacity_plus_left, controls_y, 34, 28);
    let opacity_minus = rect_xywh(opacity_plus.left - 40, controls_y, 34, 28);
    let alerts_toggle_width = (content_width / 3).clamp(112, 150);
    let alerts_toggle = rect_xywh(content_left, controls_y, alerts_toggle_width, 28);
    let row2_y = controls_y + 34;
    let steering_toggle_width = (content_width / 2 - 8).clamp(112, 168);
    let steering_toggle = rect_xywh(content_left, row2_y, steering_toggle_width, 28);
    let sensitivity_toggle_width = (content_width - steering_toggle_width - 12).clamp(132, 190);
    let sensitivity_toggle = rect_xywh(
        steering_toggle.right + 12,
        row2_y,
        sensitivity_toggle_width,
        28,
    );

    OverlayLayout {
        content_left,
        content_width,
        right_x,
        column_width,
        card_y,
        controls_y,
        chart_area,
        chart,
        steering_chart,
        opacity_minus,
        opacity_plus,
        alerts_toggle,
        steering_toggle,
        sensitivity_toggle,
    }
}

fn split_history_charts(chart_area: Rect, settings: OverlaySettings) -> (Rect, Option<Rect>) {
    let width = chart_area.right - chart_area.left;
    let gap = 0;
    const MIN_PEDAL_CHART_WIDTH: i32 = 260;
    const MIN_STEERING_CHART_WIDTH: i32 = 88;
    let show_steering =
        settings.show_steering_graph && width >= MIN_PEDAL_CHART_WIDTH + MIN_STEERING_CHART_WIDTH;
    if !show_steering {
        return (chart_area, None);
    }

    let steering_width = (width / 5).clamp(MIN_STEERING_CHART_WIDTH, 132);
    let pedal_width = width - steering_width - gap;
    if pedal_width < MIN_PEDAL_CHART_WIDTH {
        return (chart_area, None);
    }

    (
        rect_xywh(
            chart_area.left,
            chart_area.top,
            pedal_width,
            chart_area.bottom - chart_area.top,
        ),
        Some(rect_xywh(
            chart_area.left + pedal_width + gap,
            chart_area.top,
            steering_width,
            chart_area.bottom - chart_area.top,
        )),
    )
}

fn visible_chart_width(client: Rect, settings: OverlaySettings) -> i32 {
    let content_width = (client.right - 64).max(260);
    let desired_width = if settings.chart_width <= 0 {
        content_width
    } else {
        settings.chart_width
    };
    desired_width.clamp(260, content_width)
}

fn draw_combined_overlay_chart(
    hdc: Hdc,
    chart: Rect,
    history: &[LiveSample],
    opacity: f32,
    show_grip: bool,
) {
    let x = chart.left;
    let y = chart.top;
    let width = chart.right - chart.left;
    let height = chart.bottom - chart.top;
    let opacity = opacity.clamp(0.3, 1.0);
    let show_legend = show_grip && width >= 420;
    draw_panel_with_border(
        hdc,
        x,
        y,
        width,
        height,
        blend_colors(color_app_bg(), color_panel_raised(), opacity),
        blend_colors(color_app_bg(), color_border(), opacity),
    );

    if show_legend {
        draw_legend(hdc, x + 14, y + 12, opacity);
    }
    if show_grip {
        draw_resize_grip(hdc, chart, opacity);
    }

    let top_padding = if show_legend { 42 } else { 18 };
    let plot = rect_xywh(
        x + 14,
        y + top_padding,
        width - 28,
        height - top_padding - 16,
    );
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
            blend_colors(color_app_bg(), rgb(44, 51, 64), opacity),
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
            blend_colors(color_app_bg(), color_muted(), opacity),
        );
        return;
    }

    draw_history_line(
        hdc,
        plot,
        history,
        InputRole::Throttle,
        blend_colors(color_app_bg(), color_accent(), opacity),
    );
    draw_history_line(
        hdc,
        plot,
        history,
        InputRole::Brake,
        blend_colors(color_app_bg(), rgb(255, 82, 96), opacity),
    );
}

fn draw_resize_grip(hdc: Hdc, chart: Rect, opacity: f32) {
    let grip = chart_resize_handle_rect(chart);
    let color = blend_colors(color_app_bg(), color_muted(), opacity);
    for offset in [6, 11, 16] {
        let x1 = grip.right - offset;
        let y1 = grip.bottom - 4;
        let x2 = grip.right - 4;
        let y2 = grip.bottom - offset;
        draw_line(hdc, x1, y1, x2, y2, color, 1);
    }
}

fn draw_line(hdc: Hdc, x1: i32, y1: i32, x2: i32, y2: i32, color: Dword, width: i32) {
    let pen = unsafe { CreatePen(PS_SOLID, width, color) };
    let old_pen = unsafe { SelectObject(hdc, pen) };
    unsafe {
        MoveToEx(hdc, x1, y1, null_mut());
        LineTo(hdc, x2, y2);
        SelectObject(hdc, old_pen);
        DeleteObject(pen);
    }
}

fn draw_legend(hdc: Hdc, x: i32, y: i32, opacity: f32) {
    draw_legend_item(
        hdc,
        x,
        y,
        "Throttle",
        blend_colors(color_app_bg(), color_accent(), opacity),
    );
    draw_legend_item(
        hdc,
        x + 104,
        y,
        "Brake",
        blend_colors(color_app_bg(), rgb(255, 82, 96), opacity),
    );
}

fn draw_legend_item(hdc: Hdc, x: i32, y: i32, label: &str, color: Dword) {
    fill_rect(hdc, rect_xywh(x, y + 8, 24, 3), color);
    draw_text_kind(hdc, x + 32, y, 72, 20, label, TextKind::Meta, color_muted());
}

fn draw_steering_trace(
    hdc: Hdc,
    chart: Rect,
    history: &[LiveSample],
    scale: SteeringGraphScale,
    opacity: f32,
) {
    let opacity = opacity.clamp(0.3, 1.0);
    let width = chart.right - chart.left;
    let height = chart.bottom - chart.top;
    draw_panel_with_border(
        hdc,
        chart.left,
        chart.top,
        width,
        height,
        blend_colors(color_app_bg(), color_panel_raised(), opacity),
        blend_colors(color_app_bg(), color_border(), opacity),
    );

    let plot = rect_xywh(chart.left + 12, chart.top + 12, width - 24, height - 24);
    let center_x = plot.left + (plot.right - plot.left) / 2;
    fill_rect(
        hdc,
        rect_xywh(center_x, plot.top, 1, plot.bottom - plot.top),
        blend_colors(color_app_bg(), rgb(74, 84, 102), opacity),
    );

    if history.len() < 2 || plot.right <= plot.left || plot.bottom <= plot.top {
        return;
    }

    let max_samples = (plot.bottom - plot.top).max(2) as usize;
    let start = history.len().saturating_sub(max_samples);
    let visible = &history[start..];
    let pen = unsafe {
        CreatePen(
            PS_SOLID,
            2,
            blend_colors(color_app_bg(), rgb(255, 198, 92), opacity),
        )
    };
    let old_pen = unsafe { SelectObject(hdc, pen) };
    let half_width = ((plot.right - plot.left - 1).max(1) as f32) / 2.0;
    let span_y = (plot.bottom - plot.top - 1).max(1) as f32;

    for (index, sample) in visible.iter().enumerate() {
        let scaled = scaled_steering_value(sample.steering, scale);
        let x = center_x + (scaled * half_width).round() as i32;
        let reverse_index = visible.len() - 1 - index;
        let y = plot.bottom
            - 1
            - ((reverse_index as f32 / (visible.len() - 1) as f32) * span_y).round() as i32;

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

fn scaled_steering_value(value: f32, scale: SteeringGraphScale) -> f32 {
    let value = value.clamp(-1.0, 1.0);
    match scale {
        SteeringGraphScale::Linear => value,
        SteeringGraphScale::Log => {
            if value == 0.0 {
                0.0
            } else {
                value.signum() * (1.0 + 9.0 * value.abs()).ln() / 10.0_f32.ln()
            }
        }
    }
}

fn draw_history_line(hdc: Hdc, plot: Rect, history: &[LiveSample], role: InputRole, color: Dword) {
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
            InputRole::Throttle => sample.throttle,
            InputRole::Brake => sample.brake,
            InputRole::Steering => 0.0,
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

fn draw_binding_line(
    hdc: Hdc,
    x: i32,
    y: i32,
    width: i32,
    role: InputRole,
    binding: Option<&BindingView>,
) {
    let text = if let Some(binding) = binding {
        if width < 170 {
            format!("{}: {}", role.label(), binding.axis_label)
        } else {
            let range = match &binding.calibration {
                BindingCalibration::DriverRange => "driver".to_string(),
                BindingCalibration::CustomRange {
                    idle_raw,
                    active_raw,
                } => format!("custom {}..{}", idle_raw, active_raw),
                BindingCalibration::SteeringCustomRange {
                    center_raw,
                    left_raw,
                    right_raw,
                } => format!("custom L{} C{} R{}", left_raw, center_raw, right_raw),
            };
            format!(
                "{}: {} idle={} sample={} range={}",
                role.label(),
                binding.axis_label,
                binding.idle_raw,
                binding.active_raw,
                range
            )
        }
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

fn overlay_settings() -> OverlaySettings {
    OVERLAY_SETTINGS
        .get_or_init(|| Mutex::new(OverlaySettings::default()))
        .lock()
        .map(|settings| *settings)
        .unwrap_or_default()
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

fn blend_colors(background: Dword, foreground: Dword, alpha: f32) -> Dword {
    let alpha = alpha.clamp(0.0, 1.0);
    let br = background & 0xFF;
    let bg = (background >> 8) & 0xFF;
    let bb = (background >> 16) & 0xFF;
    let fr = foreground & 0xFF;
    let fg = (foreground >> 8) & 0xFF;
    let fb = (foreground >> 16) & 0xFF;

    let r = br as f32 + (fr as f32 - br as f32) * alpha;
    let g = bg as f32 + (fg as f32 - bg as f32) * alpha;
    let b = bb as f32 + (fb as f32 - bb as f32) * alpha;
    rgb(r.round() as u8, g.round() as u8, b.round() as u8)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn client(width: i32, height: i32) -> Rect {
        rect_xywh(0, 0, width, height)
    }

    fn width(rect: Rect) -> i32 {
        rect.right - rect.left
    }

    fn intersects(a: Rect, b: Rect) -> bool {
        a.left < b.right && a.right > b.left && a.top < b.bottom && a.bottom > b.top
    }

    fn assert_inside(inner: Rect, outer: Rect) {
        assert!(inner.left >= outer.left);
        assert!(inner.right <= outer.right);
        assert!(inner.top >= outer.top);
        assert!(inner.bottom <= outer.bottom);
    }

    #[test]
    fn ready_overlay_layout_does_not_overlap_charts() {
        for rect in [
            client(780, 520),
            client(640, 480),
            client(360, 480),
            client(1024, 720),
        ] {
            let layout = overlay_layout(rect, OverlaySettings::default());
            let left_card = rect_xywh(layout.content_left, layout.card_y, layout.column_width, 94);
            let right_card = rect_xywh(layout.right_x, layout.card_y, layout.column_width, 94);
            let content = rect_xywh(layout.content_left, 0, layout.content_width, rect.bottom);
            assert_inside(left_card, content);
            assert_inside(right_card, content);
            assert!(!intersects(left_card, right_card));
            assert_inside(layout.chart, layout.chart_area);
            assert!(layout.chart_area.bottom <= rect.bottom - 48);
            if let Some(steering) = layout.steering_chart {
                assert_inside(steering, layout.chart_area);
                assert!(!intersects(layout.chart, steering));
                assert_eq!(layout.chart.right, steering.left);
            }
        }
    }

    #[test]
    fn steering_graph_toggle_gives_pedals_more_width_when_off() {
        let rect = client(780, 520);
        let on = overlay_layout(rect, OverlaySettings::default());
        let off = overlay_layout(
            rect,
            OverlaySettings {
                show_steering_graph: false,
                ..OverlaySettings::default()
            },
        );

        assert!(on.steering_chart.is_some());
        assert!(off.steering_chart.is_none());
        assert!(width(off.chart) > width(on.chart));
    }

    #[test]
    fn steering_graph_stays_visible_until_true_minimum_width() {
        let settings = OverlaySettings::default();
        let minimum_with_steering = rect_xywh(0, 0, 348, 120);
        let below_minimum = rect_xywh(0, 0, 347, 120);

        assert!(
            split_history_charts(minimum_with_steering, settings)
                .1
                .is_some()
        );
        assert!(split_history_charts(below_minimum, settings).1.is_none());
    }

    #[test]
    fn overlay_controls_stay_inside_content() {
        for rect in [
            client(780, 520),
            client(640, 480),
            client(360, 480),
            client(1024, 720),
        ] {
            let layout = overlay_layout(rect, OverlaySettings::default());
            let content = rect_xywh(
                layout.content_left,
                layout.controls_y,
                layout.content_width,
                62,
            );

            for control in [
                layout.opacity_minus,
                layout.opacity_plus,
                layout.alerts_toggle,
                layout.steering_toggle,
                layout.sensitivity_toggle,
            ] {
                assert_inside(control, content);
            }
            assert!(!intersects(layout.opacity_minus, layout.opacity_plus));
            assert!(!intersects(layout.alerts_toggle, layout.opacity_minus));
            assert!(!intersects(
                layout.steering_toggle,
                layout.sensitivity_toggle
            ));
        }
    }

    #[test]
    fn sticker_alerts_require_ready_enabled_and_active_overlay() {
        assert!(should_show_alert_notification(true, true, true));
        assert!(!should_show_alert_notification(false, true, true));
        assert!(!should_show_alert_notification(true, false, true));
        assert!(!should_show_alert_notification(true, true, false));
    }

    #[test]
    fn log_steering_scale_preserves_direction_and_expands_center() {
        assert_eq!(scaled_steering_value(0.0, SteeringGraphScale::Log), 0.0);
        assert!(scaled_steering_value(0.2, SteeringGraphScale::Log) > 0.2);
        assert!(scaled_steering_value(-0.2, SteeringGraphScale::Log) < -0.2);
        assert_eq!(scaled_steering_value(1.0, SteeringGraphScale::Log), 1.0);
    }
}
