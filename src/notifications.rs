use crate::alerts::{AlertId, AlertSeverity, AlertView};
use std::ffi::c_void;
use std::mem::{size_of, zeroed};
use std::ptr::{null, null_mut};
use std::sync::{Mutex, OnceLock};

type Bool = i32;
type Dword = u32;
type Hbitmap = isize;
type Hdc = isize;
type Hwnd = isize;
type Hmonitor = isize;
type Hresult = i32;
type Long = i32;
type Uint = u32;
type Ulong = u32;

pub const POP_IN_MS: u32 = 90;
pub const SETTLE_MS: u32 = 90;
pub const MIN_VISIBLE_MS: u32 = 650;
pub const FADE_OUT_MS: u32 = 260;
pub const MAX_STICKER_OPACITY: f32 = 0.45;
const MIN_STICKER_SIZE: i32 = 187;
const MAX_STICKER_SIZE: i32 = 288;
const TARGET_MONITOR_HEIGHT_RATIO: f32 = 0.204;
const ULW_ALPHA: Dword = 0x0000_0002;
const AC_SRC_OVER: u8 = 0x00;
const AC_SRC_ALPHA: u8 = 0x01;
const BI_RGB: Dword = 0;
const DIB_RGB_COLORS: Uint = 0;
const MONITOR_DEFAULTTOPRIMARY: Dword = 0x0000_0001;
const MONITOR_DEFAULTTONEAREST: Dword = 0x0000_0002;
const CLSCTX_INPROC_SERVER: Dword = 0x1;
const COINIT_APARTMENTTHREADED: Dword = 0x2;
const WIC_DECODE_METADATA_CACHE_ON_LOAD: Dword = 0x1;
const WIC_BITMAP_DITHER_TYPE_NONE: Dword = 0x0;
const WIC_BITMAP_PALETTE_TYPE_CUSTOM: Dword = 0x0;
const RPC_E_CHANGED_MODE: Hresult = 0x8001_0106u32 as i32;

static PRESENTER: OnceLock<Mutex<AlertNotificationPresenter>> = OnceLock::new();
static STICKER_CACHE: OnceLock<Vec<StickerBitmap>> = OnceLock::new();
static COM_READY: OnceLock<()> = OnceLock::new();

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MonitorBounds {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NotificationFrame {
    pub alert_id: AlertId,
    pub opacity: f32,
    pub scale: f32,
    pub rotation_degrees: f32,
    pub offset_x: i32,
    pub offset_y: i32,
}

#[derive(Clone, Copy)]
pub struct AlertStickerAsset {
    pub id: AlertId,
    pub png: &'static [u8],
}

#[derive(Clone)]
struct StickerBitmap {
    id: AlertId,
    width: u32,
    height: u32,
    bgra_premultiplied: Vec<u8>,
}

#[derive(Clone, Copy)]
struct PresentedAlert {
    id: AlertId,
    severity: AlertSeverity,
    age_ms: u32,
    release_ms: Option<u32>,
}

#[derive(Default)]
struct AlertNotificationPresenter {
    current: Option<PresentedAlert>,
}

#[derive(Clone, Copy)]
struct AlertCandidate {
    id: AlertId,
    severity: AlertSeverity,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Point {
    x: i32,
    y: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Size {
    cx: i32,
    cy: i32,
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
struct MonitorInfo {
    cb_size: Dword,
    rc_monitor: Rect,
    rc_work: Rect,
    dw_flags: Dword,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct BlendFunction {
    blend_op: u8,
    blend_flags: u8,
    source_constant_alpha: u8,
    alpha_format: u8,
}

#[repr(C)]
struct BitmapInfoHeader {
    bi_size: Dword,
    bi_width: Long,
    bi_height: Long,
    bi_planes: u16,
    bi_bit_count: u16,
    bi_compression: Dword,
    bi_size_image: Dword,
    bi_x_pels_per_meter: Long,
    bi_y_pels_per_meter: Long,
    bi_clr_used: Dword,
    bi_clr_important: Dword,
}

#[repr(C)]
struct BitmapInfo {
    bmi_header: BitmapInfoHeader,
    bmi_colors: [Dword; 1],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Guid {
    data1: u32,
    data2: u16,
    data3: u16,
    data4: [u8; 8],
}

#[repr(C)]
struct IWICImagingFactory {
    lp_vtbl: *const IWICImagingFactoryVtbl,
}

#[repr(C)]
struct IWICImagingFactoryVtbl {
    query_interface: unsafe extern "system" fn(
        *mut IWICImagingFactory,
        *const Guid,
        *mut *mut c_void,
    ) -> Hresult,
    add_ref: unsafe extern "system" fn(*mut IWICImagingFactory) -> Ulong,
    release: unsafe extern "system" fn(*mut IWICImagingFactory) -> Ulong,
    create_decoder_from_filename: usize,
    create_decoder_from_stream: unsafe extern "system" fn(
        *mut IWICImagingFactory,
        *mut IWICStream,
        *const Guid,
        Dword,
        *mut *mut IWICBitmapDecoder,
    ) -> Hresult,
    create_decoder_from_file_handle: usize,
    create_component_info: usize,
    create_decoder: usize,
    create_encoder: usize,
    create_palette: usize,
    create_format_converter: unsafe extern "system" fn(
        *mut IWICImagingFactory,
        *mut *mut IWICFormatConverter,
    ) -> Hresult,
    create_bitmap_scaler: usize,
    create_bitmap_clipper: usize,
    create_bitmap_flip_rotator: usize,
    create_stream:
        unsafe extern "system" fn(*mut IWICImagingFactory, *mut *mut IWICStream) -> Hresult,
    create_color_context: usize,
    create_color_transformer: usize,
    create_bitmap: usize,
    create_bitmap_from_source: usize,
    create_bitmap_from_source_rect: usize,
    create_bitmap_from_memory: usize,
    create_bitmap_from_hbitmap: usize,
    create_bitmap_from_hicon: usize,
    create_component_enumerator: usize,
    create_fast_metadata_encoder_from_decoder: usize,
    create_fast_metadata_encoder_from_frame_decode: usize,
    create_query_writer: usize,
    create_query_writer_from_reader: usize,
}

#[repr(C)]
struct IWICStream {
    lp_vtbl: *const IWICStreamVtbl,
}

#[repr(C)]
struct IWICStreamVtbl {
    query_interface:
        unsafe extern "system" fn(*mut IWICStream, *const Guid, *mut *mut c_void) -> Hresult,
    add_ref: unsafe extern "system" fn(*mut IWICStream) -> Ulong,
    release: unsafe extern "system" fn(*mut IWICStream) -> Ulong,
    read: usize,
    write: usize,
    seek: usize,
    set_size: usize,
    copy_to: usize,
    commit: usize,
    revert: usize,
    lock_region: usize,
    unlock_region: usize,
    stat: usize,
    clone: usize,
    initialize_from_istream: usize,
    initialize_from_filename: usize,
    initialize_from_memory: unsafe extern "system" fn(*mut IWICStream, *mut u8, Dword) -> Hresult,
    initialize_from_istream_region: usize,
}

#[repr(C)]
struct IWICBitmapDecoder {
    lp_vtbl: *const IWICBitmapDecoderVtbl,
}

#[repr(C)]
struct IWICBitmapDecoderVtbl {
    query_interface:
        unsafe extern "system" fn(*mut IWICBitmapDecoder, *const Guid, *mut *mut c_void) -> Hresult,
    add_ref: unsafe extern "system" fn(*mut IWICBitmapDecoder) -> Ulong,
    release: unsafe extern "system" fn(*mut IWICBitmapDecoder) -> Ulong,
    query_capability: usize,
    initialize: usize,
    get_container_format: usize,
    get_decoder_info: usize,
    copy_palette: usize,
    get_metadata_query_reader: usize,
    get_preview: usize,
    get_color_contexts: usize,
    get_thumbnail: usize,
    get_frame_count: usize,
    get_frame: unsafe extern "system" fn(
        *mut IWICBitmapDecoder,
        Uint,
        *mut *mut IWICBitmapFrameDecode,
    ) -> Hresult,
}

#[repr(C)]
struct IWICBitmapFrameDecode {
    lp_vtbl: *const IWICBitmapFrameDecodeVtbl,
}

#[repr(C)]
struct IWICBitmapFrameDecodeVtbl {
    query_interface: unsafe extern "system" fn(
        *mut IWICBitmapFrameDecode,
        *const Guid,
        *mut *mut c_void,
    ) -> Hresult,
    add_ref: unsafe extern "system" fn(*mut IWICBitmapFrameDecode) -> Ulong,
    release: unsafe extern "system" fn(*mut IWICBitmapFrameDecode) -> Ulong,
    get_size: usize,
    get_pixel_format: usize,
    get_resolution: usize,
    copy_palette: usize,
    copy_pixels: usize,
}

#[repr(C)]
struct IWICFormatConverter {
    lp_vtbl: *const IWICFormatConverterVtbl,
}

#[repr(C)]
struct IWICFormatConverterVtbl {
    query_interface: unsafe extern "system" fn(
        *mut IWICFormatConverter,
        *const Guid,
        *mut *mut c_void,
    ) -> Hresult,
    add_ref: unsafe extern "system" fn(*mut IWICFormatConverter) -> Ulong,
    release: unsafe extern "system" fn(*mut IWICFormatConverter) -> Ulong,
    get_size: unsafe extern "system" fn(*mut IWICFormatConverter, *mut Uint, *mut Uint) -> Hresult,
    get_pixel_format: usize,
    get_resolution: usize,
    copy_palette: usize,
    copy_pixels: unsafe extern "system" fn(
        *mut IWICFormatConverter,
        *const Rect,
        Uint,
        Uint,
        *mut u8,
    ) -> Hresult,
    initialize: unsafe extern "system" fn(
        *mut IWICFormatConverter,
        *mut IWICBitmapFrameDecode,
        *const Guid,
        Dword,
        *mut c_void,
        f64,
        Dword,
    ) -> Hresult,
    can_convert: usize,
}

#[link(name = "user32")]
unsafe extern "system" {
    fn GetMonitorInfoW(hMonitor: Hmonitor, lpmi: *mut MonitorInfo) -> Bool;
    fn MonitorFromPoint(pt: Point, dwFlags: Dword) -> Hmonitor;
    fn MonitorFromWindow(hwnd: Hwnd, dwFlags: Dword) -> Hmonitor;
    fn UpdateLayeredWindow(
        hwnd: Hwnd,
        hdc_dst: Hdc,
        ppt_dst: *const Point,
        psize: *const Size,
        hdc_src: Hdc,
        ppt_src: *const Point,
        cr_key: Dword,
        pblend: *const BlendFunction,
        dw_flags: Dword,
    ) -> Bool;
}

#[link(name = "gdi32")]
unsafe extern "system" {
    fn CreateCompatibleDC(hdc: Hdc) -> Hdc;
    fn CreateDIBSection(
        hdc: Hdc,
        pbmi: *const BitmapInfo,
        usage: Uint,
        ppv_bits: *mut *mut c_void,
        hsection: isize,
        offset: Dword,
    ) -> Hbitmap;
    fn DeleteDC(hdc: Hdc) -> Bool;
    fn DeleteObject(ho: isize) -> Bool;
    fn SelectObject(hdc: Hdc, h: isize) -> isize;
}

#[link(name = "ole32")]
unsafe extern "system" {
    fn CoCreateInstance(
        rclsid: *const Guid,
        p_unk_outer: *mut c_void,
        dw_cls_context: Dword,
        riid: *const Guid,
        ppv: *mut *mut c_void,
    ) -> Hresult;
    fn CoInitializeEx(pv_reserved: *mut c_void, co_init: Dword) -> Hresult;
}

const CLSID_WIC_IMAGING_FACTORY: Guid = Guid {
    data1: 0xcacaf262,
    data2: 0x9370,
    data3: 0x4615,
    data4: [0xa1, 0x3b, 0x9f, 0x55, 0x39, 0xda, 0x4c, 0x0a],
};

const IID_IWIC_IMAGING_FACTORY: Guid = Guid {
    data1: 0xec5ec8a9,
    data2: 0xc395,
    data3: 0x4314,
    data4: [0x9c, 0x77, 0x54, 0xd7, 0xa9, 0x35, 0xff, 0x70],
};

const GUID_WIC_PIXEL_FORMAT_32BPP_PBGRA: Guid = Guid {
    data1: 0x6fddc324,
    data2: 0x4e03,
    data3: 0x4bfe,
    data4: [0xb1, 0x85, 0x3d, 0x77, 0x76, 0x8d, 0xc9, 0x0f],
};

const STICKER_ASSETS: [AlertStickerAsset; 6] = [
    AlertStickerAsset {
        id: AlertId::PedalOverlap,
        png: include_bytes!("../assets/alert_stickers/pedal_overlap.png"),
    },
    AlertStickerAsset {
        id: AlertId::Coasting,
        png: include_bytes!("../assets/alert_stickers/coasting.png"),
    },
    AlertStickerAsset {
        id: AlertId::ThrottleWithLock,
        png: include_bytes!("../assets/alert_stickers/throttle_with_lock.png"),
    },
    AlertStickerAsset {
        id: AlertId::BrakeReleaseSnap,
        png: include_bytes!("../assets/alert_stickers/brake_release_snap.png"),
    },
    AlertStickerAsset {
        id: AlertId::SteeringSaw,
        png: include_bytes!("../assets/alert_stickers/steering_saw.png"),
    },
    AlertStickerAsset {
        id: AlertId::SteeringSaturated,
        png: include_bytes!("../assets/alert_stickers/steering_saturated.png"),
    },
];

pub fn sticker_assets() -> &'static [AlertStickerAsset; 6] {
    &STICKER_ASSETS
}

pub fn reset_presentation() {
    if let Ok(mut presenter) = PRESENTER
        .get_or_init(|| Mutex::new(AlertNotificationPresenter::default()))
        .lock()
    {
        presenter.reset();
    }
}

pub fn next_frame(alerts: &[AlertView], delta_ms: u32) -> Option<NotificationFrame> {
    PRESENTER
        .get_or_init(|| Mutex::new(AlertNotificationPresenter::default()))
        .lock()
        .ok()
        .and_then(|mut presenter| presenter.advance(primary_alert(alerts), delta_ms))
}

pub fn render_frame(hwnd: Hwnd, overlay_hwnd: Option<Hwnd>, frame: NotificationFrame) -> bool {
    let Some(sticker) = sticker_bitmap(frame.alert_id) else {
        return false;
    };
    let monitor = notification_monitor_bounds(overlay_hwnd);
    let base_size = sticker_size_for_monitor_height(monitor.bottom - monitor.top);
    let rect = notification_rect(monitor, base_size, frame);
    let width = (rect.right - rect.left).max(1);
    let height = (rect.bottom - rect.top).max(1);
    let pixels = scale_sticker_pixels(
        sticker,
        width as usize,
        height as usize,
        frame.opacity,
        frame.rotation_degrees,
    );
    update_layered_window(hwnd, rect, &pixels)
}

fn primary_alert(alerts: &[AlertView]) -> Option<AlertCandidate> {
    alerts.first().map(|alert| AlertCandidate {
        id: alert.id,
        severity: alert.severity,
    })
}

impl AlertNotificationPresenter {
    pub fn reset(&mut self) {
        self.current = None;
    }

    fn advance(
        &mut self,
        primary: Option<AlertCandidate>,
        delta_ms: u32,
    ) -> Option<NotificationFrame> {
        self.apply_primary(primary);

        let Some(current) = &mut self.current else {
            return None;
        };

        current.age_ms = current.age_ms.saturating_add(delta_ms);
        if primary.map(|alert| alert.id) != Some(current.id)
            && current.age_ms >= minimum_active_ms()
            && current.release_ms.is_none()
        {
            current.release_ms = Some(0);
        }
        if let Some(release_ms) = &mut current.release_ms {
            *release_ms = release_ms.saturating_add(delta_ms);
            if *release_ms >= FADE_OUT_MS {
                self.current = None;
                return None;
            }
        }

        self.current.map(notification_frame_for)
    }

    fn apply_primary(&mut self, primary: Option<AlertCandidate>) {
        let Some(candidate) = primary else {
            return;
        };
        let Some(current) = &mut self.current else {
            self.current = Some(PresentedAlert::new(candidate));
            return;
        };

        if current.id == candidate.id {
            current.severity = candidate.severity;
            current.release_ms = None;
            return;
        }

        let candidate_priority = candidate.severity.priority();
        let current_priority = current.severity.priority();
        if candidate_priority > current_priority
            || current.release_ms.is_some()
            || current.age_ms >= minimum_active_ms()
        {
            self.current = Some(PresentedAlert::new(candidate));
        }
    }
}

impl PresentedAlert {
    fn new(candidate: AlertCandidate) -> Self {
        Self {
            id: candidate.id,
            severity: candidate.severity,
            age_ms: 0,
            release_ms: None,
        }
    }
}

fn notification_frame_for(alert: PresentedAlert) -> NotificationFrame {
    let age = alert.age_ms;
    let pop = (age as f32 / POP_IN_MS as f32).clamp(0.0, 1.0);
    let base_opacity = MAX_STICKER_OPACITY * ease_out_cubic(pop);
    let fade_opacity = alert
        .release_ms
        .map(|release| 1.0 - (release as f32 / FADE_OUT_MS as f32).clamp(0.0, 1.0))
        .unwrap_or(1.0);
    let settle = if age <= POP_IN_MS {
        0.90 + 0.16 * ease_out_cubic(pop)
    } else if age <= POP_IN_MS + SETTLE_MS {
        let t = ((age - POP_IN_MS) as f32 / SETTLE_MS as f32).clamp(0.0, 1.0);
        1.06 - 0.06 * ease_out_cubic(t)
    } else {
        1.0
    };
    let motion = motion_for_alert(alert.id, age);

    NotificationFrame {
        alert_id: alert.id,
        opacity: (base_opacity * fade_opacity).clamp(0.0, MAX_STICKER_OPACITY),
        scale: (settle * motion.scale).clamp(0.82, 1.12),
        rotation_degrees: motion.rotation_degrees,
        offset_x: motion.offset_x,
        offset_y: motion.offset_y,
    }
}

#[derive(Clone, Copy)]
struct StickerMotion {
    offset_x: i32,
    offset_y: i32,
    scale: f32,
    rotation_degrees: f32,
}

fn motion_for_alert(id: AlertId, age_ms: u32) -> StickerMotion {
    let early = (age_ms as f32 / 240.0).clamp(0.0, 1.0);
    match id {
        AlertId::PedalOverlap => StickerMotion {
            offset_x: 0,
            offset_y: 0,
            scale: 1.0 + 0.035 * (1.0 - early),
            rotation_degrees: 0.0,
        },
        AlertId::Coasting => {
            let y = -((age_ms as f32 / 180.0).sin() * 4.0).round() as i32;
            StickerMotion {
                offset_x: 0,
                offset_y: y,
                scale: 0.98,
                rotation_degrees: 0.0,
            }
        }
        AlertId::ThrottleWithLock => StickerMotion {
            offset_x: ((1.0 - early) * 9.0).round() as i32,
            offset_y: 0,
            scale: 1.0,
            rotation_degrees: -8.0 * (1.0 - early),
        },
        AlertId::BrakeReleaseSnap => StickerMotion {
            offset_x: 0,
            offset_y: -((1.0 - early) * 14.0).round() as i32,
            scale: 1.0,
            rotation_degrees: -3.0 * (1.0 - early),
        },
        AlertId::SteeringSaw => {
            let jitter = if age_ms < 420 {
                match (age_ms / 32) % 4 {
                    0 => -4,
                    1 => 3,
                    2 => -2,
                    _ => 4,
                }
            } else {
                0
            };
            StickerMotion {
                offset_x: jitter,
                offset_y: 0,
                scale: 1.0,
                rotation_degrees: jitter as f32 * 0.7,
            }
        }
        AlertId::SteeringSaturated => {
            let bump = if age_ms < 320 {
                match (age_ms / 52) % 4 {
                    0 => 5,
                    1 => -3,
                    2 => 3,
                    _ => 0,
                }
            } else {
                0
            };
            StickerMotion {
                offset_x: bump,
                offset_y: 0,
                scale: 1.0,
                rotation_degrees: 0.0,
            }
        }
    }
}

pub fn sticker_size_for_monitor_height(height: i32) -> i32 {
    ((height.max(1) as f32 * TARGET_MONITOR_HEIGHT_RATIO).round() as i32)
        .clamp(MIN_STICKER_SIZE, MAX_STICKER_SIZE)
}

pub fn notification_rect(
    monitor: MonitorBounds,
    base_size: i32,
    frame: NotificationFrame,
) -> MonitorBounds {
    let size = ((base_size.max(1) as f32) * frame.scale).round().max(1.0) as i32;
    let center_x = monitor.left + (monitor.right - monitor.left) / 2 + frame.offset_x;
    let center_y = monitor.top + (monitor.bottom - monitor.top) / 2 + frame.offset_y;
    rect_bounds(center_x - size / 2, center_y - size / 2, size, size)
}

fn rect_bounds(left: i32, top: i32, width: i32, height: i32) -> MonitorBounds {
    MonitorBounds {
        left,
        top,
        right: left + width,
        bottom: top + height,
    }
}

fn minimum_active_ms() -> u32 {
    POP_IN_MS + SETTLE_MS + MIN_VISIBLE_MS
}

fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t.clamp(0.0, 1.0)).powi(3)
}

fn sticker_bitmap(id: AlertId) -> Option<&'static StickerBitmap> {
    let cache = STICKER_CACHE.get_or_init(|| {
        sticker_assets()
            .iter()
            .filter_map(|asset| decode_sticker(asset))
            .collect()
    });
    cache.iter().find(|sticker| sticker.id == id)
}

fn decode_sticker(asset: &AlertStickerAsset) -> Option<StickerBitmap> {
    unsafe { decode_png_with_wic(asset.id, asset.png) }
}

fn notification_monitor_bounds(overlay_hwnd: Option<Hwnd>) -> MonitorBounds {
    unsafe {
        let monitor = if let Some(hwnd) = overlay_hwnd {
            MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST)
        } else {
            MonitorFromPoint(Point { x: 0, y: 0 }, MONITOR_DEFAULTTOPRIMARY)
        };
        let mut info = MonitorInfo {
            cb_size: size_of::<MonitorInfo>() as Dword,
            rc_monitor: zeroed(),
            rc_work: zeroed(),
            dw_flags: 0,
        };
        if monitor != 0 && GetMonitorInfoW(monitor, &mut info) != 0 {
            return MonitorBounds {
                left: info.rc_monitor.left,
                top: info.rc_monitor.top,
                right: info.rc_monitor.right,
                bottom: info.rc_monitor.bottom,
            };
        }
    }
    rect_bounds(0, 0, 1920, 1080)
}

fn scale_sticker_pixels(
    sticker: &StickerBitmap,
    width: usize,
    height: usize,
    opacity: f32,
    rotation_degrees: f32,
) -> Vec<u8> {
    let opacity = opacity.clamp(0.0, MAX_STICKER_OPACITY);
    let mut pixels = vec![0; width.saturating_mul(height).saturating_mul(4)];
    if width == 0 || height == 0 || sticker.width == 0 || sticker.height == 0 || opacity <= 0.0 {
        return pixels;
    }

    let src_width = sticker.width as usize;
    let src_height = sticker.height as usize;
    let angle = -rotation_degrees.to_radians();
    let (sin, cos) = angle.sin_cos();
    let half_w = width as f32 / 2.0;
    let half_h = height as f32 / 2.0;

    for y in 0..height {
        for x in 0..width {
            let dx = x as f32 + 0.5 - half_w;
            let dy = y as f32 + 0.5 - half_h;
            let sample_x = cos * dx - sin * dy + half_w;
            let sample_y = sin * dx + cos * dy + half_h;
            if sample_x < 0.0
                || sample_y < 0.0
                || sample_x >= width as f32
                || sample_y >= height as f32
            {
                continue;
            }
            let src_x = ((sample_x / width as f32) * src_width as f32)
                .floor()
                .clamp(0.0, (src_width - 1) as f32) as usize;
            let src_y = ((sample_y / height as f32) * src_height as f32)
                .floor()
                .clamp(0.0, (src_height - 1) as f32) as usize;
            let src = (src_y * src_width + src_x) * 4;
            let dst = (y * width + x) * 4;
            pixels[dst] = (sticker.bgra_premultiplied[src] as f32 * opacity).round() as u8;
            pixels[dst + 1] = (sticker.bgra_premultiplied[src + 1] as f32 * opacity).round() as u8;
            pixels[dst + 2] = (sticker.bgra_premultiplied[src + 2] as f32 * opacity).round() as u8;
            pixels[dst + 3] = (sticker.bgra_premultiplied[src + 3] as f32 * opacity).round() as u8;
        }
    }
    pixels
}

fn update_layered_window(hwnd: Hwnd, rect: MonitorBounds, pixels: &[u8]) -> bool {
    let width = rect.right - rect.left;
    let height = rect.bottom - rect.top;
    if hwnd == 0 || width <= 0 || height <= 0 {
        return false;
    }

    let mut bits: *mut c_void = null_mut();
    let bitmap_info = BitmapInfo {
        bmi_header: BitmapInfoHeader {
            bi_size: size_of::<BitmapInfoHeader>() as Dword,
            bi_width: width,
            bi_height: -height,
            bi_planes: 1,
            bi_bit_count: 32,
            bi_compression: BI_RGB,
            bi_size_image: (width * height * 4) as Dword,
            bi_x_pels_per_meter: 0,
            bi_y_pels_per_meter: 0,
            bi_clr_used: 0,
            bi_clr_important: 0,
        },
        bmi_colors: [0],
    };

    unsafe {
        let mem_dc = CreateCompatibleDC(0);
        if mem_dc == 0 {
            return false;
        }
        let bitmap = CreateDIBSection(mem_dc, &bitmap_info, DIB_RGB_COLORS, &mut bits, 0, 0);
        if bitmap == 0 || bits.is_null() {
            DeleteDC(mem_dc);
            return false;
        }

        std::ptr::copy_nonoverlapping(pixels.as_ptr(), bits as *mut u8, pixels.len());
        let old_bitmap = SelectObject(mem_dc, bitmap);
        let dst = Point {
            x: rect.left,
            y: rect.top,
        };
        let size = Size {
            cx: width,
            cy: height,
        };
        let src = Point { x: 0, y: 0 };
        let blend = BlendFunction {
            blend_op: AC_SRC_OVER,
            blend_flags: 0,
            source_constant_alpha: 255,
            alpha_format: AC_SRC_ALPHA,
        };
        let ok = UpdateLayeredWindow(hwnd, 0, &dst, &size, mem_dc, &src, 0, &blend, ULW_ALPHA) != 0;
        SelectObject(mem_dc, old_bitmap);
        DeleteObject(bitmap);
        DeleteDC(mem_dc);
        ok
    }
}

unsafe fn decode_png_with_wic(id: AlertId, png: &[u8]) -> Option<StickerBitmap> {
    ensure_com_initialized();

    let mut factory_ptr: *mut c_void = null_mut();
    let hr = unsafe {
        CoCreateInstance(
            &CLSID_WIC_IMAGING_FACTORY,
            null_mut(),
            CLSCTX_INPROC_SERVER,
            &IID_IWIC_IMAGING_FACTORY,
            &mut factory_ptr,
        )
    };
    if failed(hr) || factory_ptr.is_null() {
        return None;
    }
    let factory = factory_ptr as *mut IWICImagingFactory;

    let mut stream: *mut IWICStream = null_mut();
    if failed(unsafe { ((*(*factory).lp_vtbl).create_stream)(factory, &mut stream) })
        || stream.is_null()
    {
        unsafe { ((*(*factory).lp_vtbl).release)(factory) };
        return None;
    }

    let mut png_bytes = png.to_vec();
    if failed(unsafe {
        ((*(*stream).lp_vtbl).initialize_from_memory)(
            stream,
            png_bytes.as_mut_ptr(),
            png_bytes.len() as Dword,
        )
    }) {
        unsafe {
            ((*(*stream).lp_vtbl).release)(stream);
            ((*(*factory).lp_vtbl).release)(factory);
        }
        return None;
    }

    let mut decoder: *mut IWICBitmapDecoder = null_mut();
    if failed(unsafe {
        ((*(*factory).lp_vtbl).create_decoder_from_stream)(
            factory,
            stream,
            null(),
            WIC_DECODE_METADATA_CACHE_ON_LOAD,
            &mut decoder,
        )
    }) || decoder.is_null()
    {
        unsafe {
            ((*(*stream).lp_vtbl).release)(stream);
            ((*(*factory).lp_vtbl).release)(factory);
        }
        return None;
    }

    let mut frame: *mut IWICBitmapFrameDecode = null_mut();
    if failed(unsafe { ((*(*decoder).lp_vtbl).get_frame)(decoder, 0, &mut frame) })
        || frame.is_null()
    {
        unsafe {
            ((*(*decoder).lp_vtbl).release)(decoder);
            ((*(*stream).lp_vtbl).release)(stream);
            ((*(*factory).lp_vtbl).release)(factory);
        }
        return None;
    }

    let mut converter: *mut IWICFormatConverter = null_mut();
    if failed(unsafe { ((*(*factory).lp_vtbl).create_format_converter)(factory, &mut converter) })
        || converter.is_null()
    {
        unsafe {
            ((*(*frame).lp_vtbl).release)(frame);
            ((*(*decoder).lp_vtbl).release)(decoder);
            ((*(*stream).lp_vtbl).release)(stream);
            ((*(*factory).lp_vtbl).release)(factory);
        }
        return None;
    }

    if failed(unsafe {
        ((*(*converter).lp_vtbl).initialize)(
            converter,
            frame,
            &GUID_WIC_PIXEL_FORMAT_32BPP_PBGRA,
            WIC_BITMAP_DITHER_TYPE_NONE,
            null_mut(),
            0.0,
            WIC_BITMAP_PALETTE_TYPE_CUSTOM,
        )
    }) {
        unsafe {
            ((*(*converter).lp_vtbl).release)(converter);
            ((*(*frame).lp_vtbl).release)(frame);
            ((*(*decoder).lp_vtbl).release)(decoder);
            ((*(*stream).lp_vtbl).release)(stream);
            ((*(*factory).lp_vtbl).release)(factory);
        }
        return None;
    }

    let mut width: Uint = 0;
    let mut height: Uint = 0;
    if failed(unsafe { ((*(*converter).lp_vtbl).get_size)(converter, &mut width, &mut height) })
        || width == 0
        || height == 0
    {
        unsafe {
            ((*(*converter).lp_vtbl).release)(converter);
            ((*(*frame).lp_vtbl).release)(frame);
            ((*(*decoder).lp_vtbl).release)(decoder);
            ((*(*stream).lp_vtbl).release)(stream);
            ((*(*factory).lp_vtbl).release)(factory);
        }
        return None;
    }

    let stride = width.saturating_mul(4);
    let buffer_size = stride.saturating_mul(height);
    let mut bgra = vec![0; buffer_size as usize];
    let copied = unsafe {
        ((*(*converter).lp_vtbl).copy_pixels)(
            converter,
            null(),
            stride,
            buffer_size,
            bgra.as_mut_ptr(),
        )
    };

    unsafe {
        ((*(*converter).lp_vtbl).release)(converter);
        ((*(*frame).lp_vtbl).release)(frame);
        ((*(*decoder).lp_vtbl).release)(decoder);
        ((*(*stream).lp_vtbl).release)(stream);
        ((*(*factory).lp_vtbl).release)(factory);
    }

    if failed(copied) {
        return None;
    }

    Some(StickerBitmap {
        id,
        width,
        height,
        bgra_premultiplied: bgra,
    })
}

fn ensure_com_initialized() {
    COM_READY.get_or_init(|| unsafe {
        let hr = CoInitializeEx(null_mut(), COINIT_APARTMENTTHREADED);
        if failed(hr) && hr != RPC_E_CHANGED_MODE {
            return;
        }
    });
}

fn failed(hr: Hresult) -> bool {
    hr < 0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn warning(id: AlertId) -> AlertView {
        AlertView {
            id,
            severity: AlertSeverity::Warning,
            label: "",
            message: "",
            age_ms: 0,
            opacity: 1.0,
        }
    }

    fn notice(id: AlertId) -> AlertView {
        AlertView {
            severity: AlertSeverity::Notice,
            ..warning(id)
        }
    }

    #[test]
    fn sticker_assets_cover_all_alert_ids() {
        let ids = sticker_assets()
            .iter()
            .map(|asset| asset.id)
            .collect::<Vec<_>>();
        assert_eq!(
            ids,
            vec![
                AlertId::PedalOverlap,
                AlertId::Coasting,
                AlertId::ThrottleWithLock,
                AlertId::BrakeReleaseSnap,
                AlertId::SteeringSaw,
                AlertId::SteeringSaturated,
            ]
        );
        for asset in sticker_assets() {
            assert!(asset.png.starts_with(&[0x89, b'P', b'N', b'G']));
        }
    }

    #[test]
    fn sticker_assets_decode_to_cached_bitmaps() {
        for asset in sticker_assets() {
            let sticker = sticker_bitmap(asset.id).expect("sticker should decode");
            assert_eq!(sticker.width, 512);
            assert_eq!(sticker.height, 512);
            assert_eq!(sticker.bgra_premultiplied.len(), 512 * 512 * 4);
            assert!(
                sticker
                    .bgra_premultiplied
                    .chunks_exact(4)
                    .any(|pixel| pixel[3] > 0)
            );
        }
    }

    #[test]
    fn sticker_rect_centers_on_monitor_and_scales_from_height() {
        let monitor = MonitorBounds {
            left: 100,
            top: 50,
            right: 2020,
            bottom: 1130,
        };
        let size = sticker_size_for_monitor_height(monitor.bottom - monitor.top);
        assert_eq!(size, 220);

        let rect = notification_rect(
            monitor,
            size,
            NotificationFrame {
                alert_id: AlertId::Coasting,
                opacity: MAX_STICKER_OPACITY,
                scale: 1.0,
                rotation_degrees: 0.0,
                offset_x: 0,
                offset_y: 0,
            },
        );
        assert_eq!(rect.right - rect.left, 220);
        assert_eq!(rect.bottom - rect.top, 220);
        assert_eq!(rect.left + 110, 1060);
        assert_eq!(rect.top + 110, 590);
    }

    #[test]
    fn sticker_size_is_clamped_for_small_and_large_monitors() {
        assert_eq!(sticker_size_for_monitor_height(720), MIN_STICKER_SIZE);
        assert_eq!(sticker_size_for_monitor_height(1440), MAX_STICKER_SIZE);
    }

    #[test]
    fn warning_replaces_visible_notice_without_queueing() {
        let mut presenter = AlertNotificationPresenter::default();
        let notice_frame = presenter
            .advance(primary_alert(&[notice(AlertId::Coasting)]), 8)
            .unwrap();
        assert_eq!(notice_frame.alert_id, AlertId::Coasting);

        let warning_frame = presenter
            .advance(primary_alert(&[warning(AlertId::PedalOverlap)]), 8)
            .unwrap();
        assert_eq!(warning_frame.alert_id, AlertId::PedalOverlap);
    }

    #[test]
    fn throttle_lock_sticker_gets_unwind_rotation() {
        let mut presenter = AlertNotificationPresenter::default();
        let frame = presenter
            .advance(primary_alert(&[warning(AlertId::ThrottleWithLock)]), 8)
            .unwrap();

        assert!(frame.rotation_degrees < 0.0);
    }

    #[test]
    fn notification_opacity_never_exceeds_limit() {
        let mut presenter = AlertNotificationPresenter::default();
        for _ in 0..200 {
            let frame = presenter
                .advance(primary_alert(&[warning(AlertId::PedalOverlap)]), 8)
                .unwrap();
            assert!(frame.opacity <= MAX_STICKER_OPACITY);
        }
    }

    #[test]
    fn notification_honors_minimum_visible_duration_after_alert_clears() {
        let mut presenter = AlertNotificationPresenter::default();
        presenter.advance(primary_alert(&[warning(AlertId::PedalOverlap)]), 8);

        let mut frame_count = 0;
        while frame_count * 8 < minimum_active_ms() {
            assert!(presenter.advance(None, 8).is_some());
            frame_count += 1;
        }
        assert!(presenter.advance(None, FADE_OUT_MS).is_none());
    }
}
