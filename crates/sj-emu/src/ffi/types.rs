use std::ffi::{c_char, c_uint, c_void};

pub const RETRO_API_VERSION: c_uint = 1;

pub const ENV_GET_CAN_DUPE: c_uint = 3;
pub const ENV_GET_SYSTEM_DIRECTORY: c_uint = 9;
pub const ENV_SET_PIXEL_FORMAT: c_uint = 10;
pub const ENV_SET_HW_RENDER: c_uint = 14;
pub const ENV_GET_VARIABLE: c_uint = 15;
pub const ENV_SET_VARIABLES: c_uint = 16;
pub const ENV_GET_VARIABLE_UPDATE: c_uint = 17;
pub const ENV_GET_SAVE_DIRECTORY: c_uint = 31;
pub const ENV_SET_SYSTEM_AV_INFO: c_uint = 32;
pub const ENV_SET_GEOMETRY: c_uint = 37;
pub const ENV_EXPERIMENTAL_FLAG: c_uint = 0x1_0000;
pub const ENV_GET_INPUT_BITMASKS: c_uint = 0x33 | ENV_EXPERIMENTAL_FLAG;
pub const ENV_GET_CORE_OPTIONS_VERSION: c_uint = 52;
pub const ENV_GET_PREFERRED_HW_RENDER: c_uint = 56;

pub const PIXEL_FORMAT_XRGB8888: i32 = 1;
pub const HW_CONTEXT_OPENGL: c_uint = 1;
pub const HW_CONTEXT_OPENGL_CORE: c_uint = 3;
pub const HW_FRAME_BUFFER_VALID: *const c_void = usize::MAX as *const c_void;

pub const DEVICE_JOYPAD: c_uint = 1;
pub const DEVICE_POINTER: c_uint = 6;
pub const DEVICE_ID_JOYPAD_MASK: c_uint = 256;
pub const DEVICE_ID_POINTER_X: c_uint = 0;
pub const DEVICE_ID_POINTER_Y: c_uint = 1;
pub const DEVICE_ID_POINTER_PRESSED: c_uint = 2;

pub const MEMORY_SYSTEM_RAM: c_uint = 2;

#[repr(C)]
pub struct SystemInfo {
    pub library_name: *const c_char,
    pub library_version: *const c_char,
    pub valid_extensions: *const c_char,
    pub need_fullpath: bool,
    pub block_extract: bool,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct GameGeometry {
    pub base_width: c_uint,
    pub base_height: c_uint,
    pub max_width: c_uint,
    pub max_height: c_uint,
    pub aspect_ratio: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct SystemTiming {
    pub fps: f64,
    pub sample_rate: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct SystemAvInfo {
    pub geometry: GameGeometry,
    pub timing: SystemTiming,
}

#[repr(C)]
pub struct GameInfo {
    pub path: *const c_char,
    pub data: *const c_void,
    pub size: usize,
    pub meta: *const c_char,
}

#[repr(C)]
pub struct Variable {
    pub key: *const c_char,
    pub value: *const c_char,
}

pub type ProcAddress = extern "C" fn();
pub type HwContextReset = extern "C" fn();
pub type HwGetCurrentFramebuffer = extern "C" fn() -> usize;
pub type HwGetProcAddress = extern "C" fn(symbol: *const c_char) -> Option<ProcAddress>;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct HwRenderCallback {
    pub context_type: c_uint,
    pub context_reset: Option<HwContextReset>,
    pub get_current_framebuffer: Option<HwGetCurrentFramebuffer>,
    pub get_proc_address: Option<HwGetProcAddress>,
    pub depth: bool,
    pub stencil: bool,
    pub bottom_left_origin: bool,
    pub version_major: c_uint,
    pub version_minor: c_uint,
    pub cache_context: bool,
    pub context_destroy: Option<HwContextReset>,
    pub debug_context: bool,
}

pub type EnvironmentFn = extern "C" fn(command: c_uint, data: *mut c_void) -> bool;
pub type VideoRefreshFn =
    extern "C" fn(data: *const c_void, width: c_uint, height: c_uint, pitch: usize);
pub type AudioSampleFn = extern "C" fn(left: i16, right: i16);
pub type AudioSampleBatchFn = extern "C" fn(data: *const i16, frames: usize) -> usize;
pub type InputPollFn = extern "C" fn();
pub type InputStateFn =
    extern "C" fn(port: c_uint, device: c_uint, index: c_uint, id: c_uint) -> i16;
