#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::ffi::c_void;
use std::mem::{size_of, zeroed};
use std::ptr::{null, null_mut};
use std::sync::{Mutex, OnceLock};

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
const WM_TIMER: Uint = 0x0113;
const WM_KEYDOWN: Uint = 0x0100;
const VK_ESCAPE: Wparam = 0x1B;
const SW_SHOW: i32 = 5;
const CS_HREDRAW: Uint = 0x0002;
const CS_VREDRAW: Uint = 0x0001;
const COLOR_WINDOW: isize = 5;
const DT_LEFT: Uint = 0x0000;
const DT_TOP: Uint = 0x0000;
const DT_SINGLELINE: Uint = 0x0020;

#[repr(C)]
struct JOYCAPSW {
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
struct JOYINFOEX {
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
    fn joyGetDevCapsW(uJoyID: Uint, pjc: *mut JOYCAPSW, cbjc: Uint) -> Uint;
    fn joyGetPosEx(uJoyID: Uint, pji: *mut JOYINFOEX) -> Uint;
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
    fn DrawTextW(hdc: Hdc, lpchText: *const u16, cchText: i32, lprc: *mut Rect, format: Uint) -> i32;
    fn EndPaint(hwnd: Hwnd, lpPaint: *const PaintStruct) -> Bool;
    fn FillRect(hDC: Hdc, lprc: *const Rect, hbr: Hbrush) -> i32;
    fn GetClientRect(hWnd: Hwnd, lpRect: *mut Rect) -> Bool;
    fn GetMessageW(lpMsg: *mut Msg, hWnd: Hwnd, wMsgFilterMin: Uint, wMsgFilterMax: Uint) -> Bool;
    fn InvalidateRect(hWnd: Hwnd, lpRect: *const Rect, bErase: Bool) -> Bool;
    fn LoadCursorW(hInstance: Hinstance, lpCursorName: *const u16) -> Hcursor;
    fn PostQuitMessage(nExitCode: i32);
    fn RegisterClassW(lpWndClass: *const WndClassW) -> u16;
    fn SetTimer(hWnd: Hwnd, nIDEvent: usize, uElapse: Uint, lpTimerFunc: *const c_void) -> usize;
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

#[derive(Clone)]
struct Axis {
    label: &'static str,
    min: u32,
    max: u32,
    raw: u32,
}

impl Axis {
    fn percent(&self) -> f32 {
        if self.max <= self.min {
            return 0.0;
        }

        let clamped = self.raw.clamp(self.min, self.max);
        (clamped - self.min) as f32 / (self.max - self.min) as f32
    }
}

struct AppState {
    device_id: Option<u32>,
    device_name: String,
    axes: Vec<Axis>,
    status: String,
}

static STATE: OnceLock<Mutex<AppState>> = OnceLock::new();

fn main() {
    let state = STATE.get_or_init(|| Mutex::new(AppState {
        device_id: None,
        device_name: "No device".to_string(),
        axes: Vec::new(),
        status: "Searching for pedals...".to_string(),
    }));

    if let Ok(mut state) = state.lock() {
        refresh_device(&mut state);
        poll_device(&mut state);
    }

    unsafe { run_window() };
}

unsafe fn run_window() {
    let class_name = wide("ApexFootworkWindow");
    let title = wide("MOZA pedal input prototype");
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
            720,
            420,
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

unsafe extern "system" fn window_proc(hwnd: Hwnd, msg: Uint, wparam: Wparam, lparam: Lparam) -> Lresult {
    match msg {
        WM_CREATE => {
            unsafe { SetTimer(hwnd, 1, 16, null()) };
            0
        }
        WM_TIMER => {
            if let Some(lock) = STATE.get() {
                if let Ok(mut state) = lock.lock() {
                    if state.device_id.is_none() {
                        refresh_device(&mut state);
                    }
                    poll_device(&mut state);
                }
            }
            unsafe { InvalidateRect(hwnd, null(), 1) };
            0
        }
        WM_PAINT => {
            draw(hwnd);
            0
        }
        WM_KEYDOWN if wparam == VK_ESCAPE => {
            unsafe { DestroyWindow(hwnd) };
            0
        }
        WM_DESTROY => {
            unsafe { PostQuitMessage(0) };
            0
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

fn refresh_device(state: &mut AppState) {
    let count = unsafe { joyGetNumDevs() };
    let mut fallback: Option<(u32, JOYCAPSW)> = None;
    let mut moza: Option<(u32, JOYCAPSW)> = None;

    for id in 0..count {
        let mut caps: JOYCAPSW = unsafe { zeroed() };
        let result = unsafe { joyGetDevCapsW(id, &mut caps, size_of::<JOYCAPSW>() as u32) };
        if result != JOYERR_NOERROR {
            continue;
        }

        let device_name = name_from_wide(&caps.sz_pname);
        if device_name.to_uppercase().contains("MOZA") {
            moza = Some((id, caps));
            break;
        }

        if fallback.is_none() {
            fallback = Some((id, caps));
        }
    }

    if let Some((id, caps)) = moza.or(fallback) {
        state.device_id = Some(id);
        state.device_name = name_from_wide(&caps.sz_pname);
        state.status = if state.device_name.to_uppercase().contains("MOZA") {
            "MOZA device detected".to_string()
        } else {
            "MOZA not found; showing first joystick device".to_string()
        };
        state.axes = vec![
            Axis { label: "X", min: caps.w_xmin, max: caps.w_xmax, raw: 0 },
            Axis { label: "Y", min: caps.w_ymin, max: caps.w_ymax, raw: 0 },
            Axis { label: "Z", min: caps.w_zmin, max: caps.w_zmax, raw: 0 },
            Axis { label: "R", min: caps.w_rmin, max: caps.w_rmax, raw: 0 },
            Axis { label: "U", min: caps.w_umin, max: caps.w_umax, raw: 0 },
            Axis { label: "V", min: caps.w_vmin, max: caps.w_vmax, raw: 0 },
        ];
    } else {
        state.device_id = None;
        state.device_name = "No joystick/HID pedal device found".to_string();
        state.axes.clear();
        state.status = "Plug in pedals, then wait a moment".to_string();
    }
}

fn poll_device(state: &mut AppState) {
    let Some(device_id) = state.device_id else {
        return;
    };

    let mut info = JOYINFOEX {
        dw_size: size_of::<JOYINFOEX>() as u32,
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
        state.device_id = None;
        state.status = "Lost device; searching again...".to_string();
        return;
    }

    let values = [
        info.dw_xpos,
        info.dw_ypos,
        info.dw_zpos,
        info.dw_rpos,
        info.dw_upos,
        info.dw_vpos,
    ];

    for (axis, raw) in state.axes.iter_mut().zip(values) {
        axis.raw = raw;
    }
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

    if let Some(lock) = STATE.get() {
        if let Ok(state) = lock.lock() {
            draw_text(hdc, 24, 22, 660, 28, &format!("Device: {}", state.device_name));
            draw_text(hdc, 24, 52, 660, 24, &state.status);

            if state.axes.is_empty() {
                draw_text(hdc, 24, 112, 660, 24, "No axes to display yet.");
            } else {
                for (i, axis) in state.axes.iter().enumerate() {
                    draw_axis(hdc, 24, 102 + (i as i32 * 46), 620, axis);
                }
            }

            draw_text(hdc, 24, rect.bottom - 38, 660, 24, "Esc closes the prototype");
        }
    }

    unsafe { EndPaint(hwnd, &ps) };
}

fn draw_axis(hdc: Hdc, x: i32, y: i32, width: i32, axis: &Axis) {
    let percent = axis.percent();
    let label = format!("{}  raw={}  {:.0}%", axis.label, axis.raw, percent * 100.0);
    draw_text(hdc, x, y, width, 20, &label);

    let bar = Rect {
        left: x,
        top: y + 22,
        right: x + width,
        bottom: y + 38,
    };
    let filled = Rect {
        left: x,
        top: y + 22,
        right: x + ((width as f32 * percent).round() as i32),
        bottom: y + 38,
    };

    let empty_brush = unsafe { CreateSolidBrush(rgb(222, 226, 231)) };
    let fill_brush = unsafe { CreateSolidBrush(rgb(19, 132, 109)) };
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
