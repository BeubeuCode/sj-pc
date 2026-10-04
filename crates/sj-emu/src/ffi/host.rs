use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::{c_char, c_uint, c_void, CStr, CString};

use super::types::{
    GameGeometry, HwRenderCallback, ProcAddress, SystemAvInfo, Variable, DEVICE_ID_JOYPAD_MASK,
    DEVICE_ID_POINTER_PRESSED, DEVICE_ID_POINTER_X, DEVICE_ID_POINTER_Y, DEVICE_JOYPAD,
    DEVICE_POINTER, ENV_GET_CAN_DUPE, ENV_GET_CORE_OPTIONS_VERSION, ENV_GET_INPUT_BITMASKS,
    ENV_GET_PREFERRED_HW_RENDER, ENV_GET_SAVE_DIRECTORY, ENV_GET_SYSTEM_DIRECTORY,
    ENV_GET_VARIABLE, ENV_GET_VARIABLE_UPDATE, ENV_SET_GEOMETRY, ENV_SET_HW_RENDER,
    ENV_SET_PIXEL_FORMAT, ENV_SET_SYSTEM_AV_INFO, ENV_SET_VARIABLES, HW_CONTEXT_OPENGL,
    HW_CONTEXT_OPENGL_CORE, HW_FRAME_BUFFER_VALID, PIXEL_FORMAT_XRGB8888,
};
use crate::frame::{Frame, Pointer};

pub type GlProcAddressLookup = Box<dyn Fn(&str) -> *const c_void>;

// libretro callbacks carry no user pointer, so the one loaded core talks to this thread-local.
// Callbacks only borrow it briefly and never call back into the core while borrowed.
#[derive(Default)]
pub struct HostState {
    pub variables: HashMap<String, CString>,
    pub variables_changed: bool,
    pub system_dir: CString,
    pub save_dir: CString,
    pub hw_render: Option<HwRenderCallback>,
    pub framebuffer_id: usize,
    pub gl_proc_address: Option<GlProcAddressLookup>,
    pub frame: Option<Frame>,
    pub stereo_samples: Vec<i16>,
    pub joypad_mask: u16,
    pub pointer: Pointer,
    pub new_av_info: Option<SystemAvInfo>,
    pub last_av_info: SystemAvInfo,
}

thread_local! {
    pub static HOST: RefCell<HostState> = RefCell::new(HostState::default());
}

pub fn with_host<T>(action: impl FnOnce(&mut HostState) -> T) -> T {
    HOST.with(|host| action(&mut host.borrow_mut()))
}

pub extern "C" fn environment(command: c_uint, data: *mut c_void) -> bool {
    match command {
        ENV_GET_CAN_DUPE => write_out(data, true),
        ENV_SET_PIXEL_FORMAT => read_in::<i32>(data) == Some(PIXEL_FORMAT_XRGB8888),
        ENV_GET_SYSTEM_DIRECTORY => with_host(|host| write_out(data, host.system_dir.as_ptr())),
        ENV_GET_SAVE_DIRECTORY => with_host(|host| write_out(data, host.save_dir.as_ptr())),
        ENV_SET_HW_RENDER => accept_hw_render(data.cast()),
        ENV_GET_VARIABLE => lookup_variable(data.cast()),
        ENV_GET_VARIABLE_UPDATE => {
            with_host(|host| write_out(data, std::mem::take(&mut host.variables_changed)))
        }
        ENV_SET_SYSTEM_AV_INFO => store_av_info(read_in::<SystemAvInfo>(data)),
        ENV_SET_GEOMETRY => store_geometry(read_in::<GameGeometry>(data)),
        ENV_GET_CORE_OPTIONS_VERSION => write_out::<c_uint>(data, 0),
        ENV_GET_PREFERRED_HW_RENDER => write_out(data, HW_CONTEXT_OPENGL_CORE),
        ENV_SET_VARIABLES | ENV_GET_INPUT_BITMASKS => true,
        _ => false,
    }
}

fn write_out<T>(data: *mut c_void, value: T) -> bool {
    if data.is_null() {
        return false;
    }
    // SAFETY: libretro defines the pointee type for every command we route here.
    unsafe { data.cast::<T>().write(value) };
    true
}

fn read_in<T: Copy>(data: *mut c_void) -> Option<T> {
    if data.is_null() {
        return None;
    }
    // SAFETY: libretro defines the pointee type for every command we route here.
    Some(unsafe { data.cast::<T>().read() })
}

fn accept_hw_render(request: *mut HwRenderCallback) -> bool {
    let Some(mut callback) = read_in::<HwRenderCallback>(request.cast()) else {
        return false;
    };
    if callback.context_type != HW_CONTEXT_OPENGL_CORE && callback.context_type != HW_CONTEXT_OPENGL
    {
        return false;
    }
    callback.get_current_framebuffer = Some(current_framebuffer);
    callback.get_proc_address = Some(gl_proc_address);
    with_host(|host| host.hw_render = Some(callback));
    write_out(request.cast(), callback)
}

fn lookup_variable(variable: *mut Variable) -> bool {
    if variable.is_null() {
        return false;
    }
    // SAFETY: the core passes a valid retro_variable whose key is a NUL-terminated string.
    let key = unsafe { CStr::from_ptr((*variable).key) }.to_string_lossy();
    let value = with_host(|host| host.variables.get(key.as_ref()).map(|value| value.as_ptr()));
    // SAFETY: same retro_variable; the value points into HostState, which outlives the core.
    unsafe { (*variable).value = value.unwrap_or(std::ptr::null()) };
    value.is_some()
}

fn store_av_info(av_info: Option<SystemAvInfo>) -> bool {
    let Some(av_info) = av_info else {
        return false;
    };
    with_host(|host| {
        host.last_av_info = av_info;
        host.new_av_info = Some(av_info);
    });
    true
}

fn store_geometry(geometry: Option<GameGeometry>) -> bool {
    let Some(geometry) = geometry else {
        return false;
    };
    let timing = with_host(|host| host.last_av_info.timing);
    store_av_info(Some(SystemAvInfo { geometry, timing }))
}

extern "C" fn current_framebuffer() -> usize {
    with_host(|host| host.framebuffer_id)
}

extern "C" fn gl_proc_address(symbol: *const c_char) -> Option<ProcAddress> {
    // SAFETY: the core passes a NUL-terminated GL symbol name.
    let name = unsafe { CStr::from_ptr(symbol) }.to_string_lossy();
    let address = with_host(|host| {
        host.gl_proc_address
            .as_ref()
            .map_or(std::ptr::null(), |lookup| lookup(&name))
    });
    if address.is_null() {
        return None;
    }
    // SAFETY: a non-null GL proc address is a C function pointer; libretro types it as void(*)(void).
    Some(unsafe { std::mem::transmute::<*const c_void, ProcAddress>(address) })
}

pub extern "C" fn video_refresh(
    data: *const c_void,
    width: c_uint,
    height: c_uint,
    pitch_bytes: usize,
) {
    if data.is_null() {
        return;
    }
    let frame = if data == HW_FRAME_BUFFER_VALID {
        Frame::Hardware {
            width_px: width,
            height_px: height,
        }
    } else {
        copy_software_frame(data, width, height, pitch_bytes)
    };
    with_host(|host| host.frame = Some(frame));
}

fn copy_software_frame(
    data: *const c_void,
    width: c_uint,
    height: c_uint,
    pitch_bytes: usize,
) -> Frame {
    let pixels_per_row = pitch_bytes / 4;
    let mut xrgb_pixels = Vec::with_capacity((width * height) as usize);
    for row in 0..height as usize {
        // SAFETY: the core guarantees `height` rows of `pitch_bytes` XRGB8888 pixels at `data`.
        let row_pixels = unsafe {
            std::slice::from_raw_parts(data.cast::<u32>().add(row * pixels_per_row), width as usize)
        };
        xrgb_pixels.extend_from_slice(row_pixels);
    }
    Frame::Software {
        width_px: width,
        height_px: height,
        xrgb_pixels,
    }
}

pub extern "C" fn audio_sample(left: i16, right: i16) {
    with_host(|host| host.stereo_samples.extend_from_slice(&[left, right]));
}

pub extern "C" fn audio_sample_batch(data: *const i16, frames: usize) -> usize {
    if data.is_null() {
        return 0;
    }
    // SAFETY: the core passes `frames` interleaved stereo frames at `data`.
    let samples = unsafe { std::slice::from_raw_parts(data, frames * 2) };
    with_host(|host| host.stereo_samples.extend_from_slice(samples));
    frames
}

pub extern "C" fn input_poll() {}

pub extern "C" fn input_state(port: c_uint, device: c_uint, _index: c_uint, id: c_uint) -> i16 {
    if port != 0 {
        return 0;
    }
    with_host(|host| match (device, id) {
        (DEVICE_JOYPAD, DEVICE_ID_JOYPAD_MASK) => host.joypad_mask as i16,
        (DEVICE_JOYPAD, button) if button < 16 => i16::from(host.joypad_mask >> button & 1 == 1),
        (DEVICE_POINTER, DEVICE_ID_POINTER_X) => host.pointer.x,
        (DEVICE_POINTER, DEVICE_ID_POINTER_Y) => host.pointer.y,
        (DEVICE_POINTER, DEVICE_ID_POINTER_PRESSED) => i16::from(host.pointer.pressed),
        _ => 0,
    })
}
