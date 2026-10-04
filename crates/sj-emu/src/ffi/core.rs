use std::ffi::{c_char, c_uint, c_void, CString};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use libloading::Library;
use sj_game::game_api::GameApi;

use super::host::{self, with_host, GlProcAddressLookup};
use super::types::{
    AudioSampleBatchFn, AudioSampleFn, EnvironmentFn, GameInfo, InputPollFn, InputStateFn,
    SystemAvInfo, SystemInfo, VideoRefreshFn, MEMORY_SYSTEM_RAM, RETRO_API_VERSION,
};
use crate::error::EmuError;
use crate::frame::{AvInfo, FrameOutput, Pointer};

static CORE_LOADED: AtomicBool = AtomicBool::new(false);

pub struct CoreConfig {
    pub core_path: PathBuf,
    pub system_dir: PathBuf,
    pub save_dir: PathBuf,
    pub variables: Vec<(String, String)>,
}

struct Api {
    set_environment: unsafe extern "C" fn(EnvironmentFn),
    set_video_refresh: unsafe extern "C" fn(VideoRefreshFn),
    set_audio_sample: unsafe extern "C" fn(AudioSampleFn),
    set_audio_sample_batch: unsafe extern "C" fn(AudioSampleBatchFn),
    set_input_poll: unsafe extern "C" fn(InputPollFn),
    set_input_state: unsafe extern "C" fn(InputStateFn),
    init: unsafe extern "C" fn(),
    deinit: unsafe extern "C" fn(),
    libretro_version: unsafe extern "C" fn() -> c_uint,
    get_system_info: unsafe extern "C" fn(*mut SystemInfo),
    get_system_av_info: unsafe extern "C" fn(*mut SystemAvInfo),
    load_game: unsafe extern "C" fn(*const GameInfo) -> bool,
    unload_game: unsafe extern "C" fn(),
    run: unsafe extern "C" fn(),
    serialize_size: unsafe extern "C" fn() -> usize,
    serialize: unsafe extern "C" fn(*mut c_void, usize) -> bool,
    unserialize: unsafe extern "C" fn(*const c_void, usize) -> bool,
    get_memory_data: unsafe extern "C" fn(c_uint) -> *mut c_void,
    get_memory_size: unsafe extern "C" fn(c_uint) -> usize,
    cheat_set: unsafe extern "C" fn(c_uint, bool, *const c_char),
}

pub struct Core {
    api: Api,
    game_loaded: bool,
    rom_bytes: Vec<u8>,
    rom_path: CString,
    _library: Library,
}

macro_rules! symbol {
    ($library:expr, $name:literal) => {
        // SAFETY: the symbol has the signature the libretro API defines for it, and the fn
        // pointer is only used while `Core` keeps the library loaded.
        *unsafe { $library.get(concat!($name, "\0").as_bytes()) }
            .map_err(|_| EmuError::MissingSymbol($name))?
    };
}

impl Core {
    pub fn load(
        config: &CoreConfig,
        gl_proc_address: GlProcAddressLookup,
    ) -> Result<Self, EmuError> {
        if CORE_LOADED.swap(true, Ordering::SeqCst) {
            return Err(EmuError::AlreadyLoaded);
        }
        let core = Self::open_library(&config.core_path)
            .inspect_err(|_| CORE_LOADED.store(false, Ordering::SeqCst))?;
        prepare_host(config, gl_proc_address);
        core.install_callbacks_and_init()?;
        Ok(core)
    }

    fn open_library(core_path: &Path) -> Result<Self, EmuError> {
        // SAFETY: loading a libretro core runs its static initialisers, which is what we want.
        let library = unsafe { Library::new(core_path) }.map_err(|source| EmuError::LoadCore {
            path: core_path.to_path_buf(),
            source,
        })?;
        let api = Api {
            set_environment: symbol!(library, "retro_set_environment"),
            set_video_refresh: symbol!(library, "retro_set_video_refresh"),
            set_audio_sample: symbol!(library, "retro_set_audio_sample"),
            set_audio_sample_batch: symbol!(library, "retro_set_audio_sample_batch"),
            set_input_poll: symbol!(library, "retro_set_input_poll"),
            set_input_state: symbol!(library, "retro_set_input_state"),
            init: symbol!(library, "retro_init"),
            deinit: symbol!(library, "retro_deinit"),
            libretro_version: symbol!(library, "retro_api_version"),
            get_system_info: symbol!(library, "retro_get_system_info"),
            get_system_av_info: symbol!(library, "retro_get_system_av_info"),
            load_game: symbol!(library, "retro_load_game"),
            unload_game: symbol!(library, "retro_unload_game"),
            run: symbol!(library, "retro_run"),
            serialize_size: symbol!(library, "retro_serialize_size"),
            serialize: symbol!(library, "retro_serialize"),
            unserialize: symbol!(library, "retro_unserialize"),
            get_memory_data: symbol!(library, "retro_get_memory_data"),
            get_memory_size: symbol!(library, "retro_get_memory_size"),
            cheat_set: symbol!(library, "retro_cheat_set"),
        };
        Ok(Self {
            api,
            game_loaded: false,
            rom_bytes: Vec::new(),
            rom_path: CString::default(),
            _library: library,
        })
    }

    fn install_callbacks_and_init(&self) -> Result<(), EmuError> {
        // SAFETY: plain libretro calls in the order the API requires.
        let api_version = unsafe { (self.api.libretro_version)() };
        if api_version != RETRO_API_VERSION {
            return Err(EmuError::UnsupportedApiVersion(api_version));
        }
        // SAFETY: as above; the callbacks are `extern "C"` functions with matching signatures.
        unsafe {
            (self.api.set_environment)(host::environment);
            (self.api.set_video_refresh)(host::video_refresh);
            (self.api.set_audio_sample)(host::audio_sample);
            (self.api.set_audio_sample_batch)(host::audio_sample_batch);
            (self.api.set_input_poll)(host::input_poll);
            (self.api.set_input_state)(host::input_state);
            (self.api.init)();
        }
        Ok(())
    }

    pub fn library_name(&self) -> String {
        let mut info = SystemInfo {
            library_name: std::ptr::null(),
            library_version: std::ptr::null(),
            valid_extensions: std::ptr::null(),
            need_fullpath: false,
            block_extract: false,
        };
        // SAFETY: the core fills `info` with pointers to static strings.
        unsafe { (self.api.get_system_info)(&raw mut info) };
        if info.library_name.is_null() {
            return String::new();
        }
        // SAFETY: non-null library_name is a static NUL-terminated string.
        unsafe { std::ffi::CStr::from_ptr(info.library_name) }
            .to_string_lossy()
            .into_owned()
    }

    pub fn load_game(&mut self, rom_path: &Path) -> Result<AvInfo, EmuError> {
        self.rom_bytes = std::fs::read(rom_path).map_err(|source| EmuError::ReadRom {
            path: rom_path.to_path_buf(),
            source,
        })?;
        self.rom_path = path_to_cstring(rom_path);
        let game = GameInfo {
            path: self.rom_path.as_ptr(),
            data: self.rom_bytes.as_ptr().cast(),
            size: self.rom_bytes.len(),
            meta: std::ptr::null(),
        };
        // SAFETY: `game` points into buffers owned by `self`, which outlive the loaded game.
        if !unsafe { (self.api.load_game)(&raw const game) } {
            return Err(EmuError::GameRejected);
        }
        self.game_loaded = true;
        Ok(self.av_info())
    }

    pub fn av_info(&self) -> AvInfo {
        let mut av_info = SystemAvInfo::default();
        // SAFETY: the core fills a caller-owned struct.
        unsafe { (self.api.get_system_av_info)(&raw mut av_info) };
        with_host(|host| host.last_av_info = av_info);
        to_av_info(av_info)
    }

    pub fn wants_opengl(&self) -> bool {
        with_host(|host| host.hw_render.is_some())
    }

    pub fn set_framebuffer(&self, framebuffer_id: u32) {
        with_host(|host| host.framebuffer_id = framebuffer_id as usize);
    }

    pub fn reset_gl_context(&self) {
        let reset = with_host(|host| host.hw_render.and_then(|callback| callback.context_reset));
        if let Some(reset) = reset {
            reset();
        }
    }

    pub fn set_variable(&self, key: &str, value: &str) {
        with_host(|host| {
            host.variables.insert(key.to_string(), to_cstring(value));
            host.variables_changed = true;
        });
    }

    // Replaces cheat `index` with an Action Replay code ("XXXXXXXX YYYYYYYY" pairs) that the core
    // runs every VBlank. Its writes go through the emulated bus, so the JIT drops code they change.
    pub fn set_cheat(&self, index: u32, code: &str) {
        let code = to_cstring(code);
        // SAFETY: `code` is NUL-terminated and outlives the call; the core copies it.
        unsafe { (self.api.cheat_set)(index, true, code.as_ptr()) };
    }

    pub fn set_input(&self, joypad_mask: u16, pointer: Pointer) {
        with_host(|host| {
            host.joypad_mask = joypad_mask;
            host.pointer = pointer;
        });
    }

    pub fn run_frame(&mut self) -> FrameOutput {
        // SAFETY: a game is loaded; retro_run re-enters only our host callbacks.
        unsafe { (self.api.run)() };
        with_host(|host| FrameOutput {
            frame: host.frame.take(),
            stereo_samples: std::mem::take(&mut host.stereo_samples),
            new_av_info: host.new_av_info.take().map(to_av_info),
        })
    }

    pub fn save_state(&self) -> Result<Vec<u8>, EmuError> {
        // SAFETY: serialize writes at most serialize_size bytes into our buffer.
        let size = unsafe { (self.api.serialize_size)() };
        let mut state = vec![0u8; size];
        if size == 0 || !unsafe { (self.api.serialize)(state.as_mut_ptr().cast(), size) } {
            return Err(EmuError::SaveStateFailed);
        }
        Ok(state)
    }

    pub fn load_state(&mut self, state: &[u8]) -> Result<(), EmuError> {
        // SAFETY: unserialize only reads `state.len()` bytes.
        if !unsafe { (self.api.unserialize)(state.as_ptr().cast(), state.len()) } {
            return Err(EmuError::LoadStateFailed);
        }
        Ok(())
    }
}

impl GameApi for Core {
    fn main_ram(&self) -> &[u8] {
        let (data, size) = self.system_ram();
        if data.is_null() {
            return &[];
        }
        // SAFETY: the core owns `size` bytes of main RAM at `data` for as long as the game runs.
        unsafe { std::slice::from_raw_parts(data, size) }
    }

    fn main_ram_mut(&mut self) -> &mut [u8] {
        let (data, size) = self.system_ram();
        if data.is_null() {
            return &mut [];
        }
        // SAFETY: as above, and `&mut self` guarantees no other Rust borrow of that memory.
        unsafe { std::slice::from_raw_parts_mut(data, size) }
    }
}

impl Core {
    fn system_ram(&self) -> (*mut u8, usize) {
        if !self.game_loaded {
            return (std::ptr::null_mut(), 0);
        }
        // SAFETY: plain libretro queries with a game loaded.
        unsafe {
            (
                (self.api.get_memory_data)(MEMORY_SYSTEM_RAM).cast(),
                (self.api.get_memory_size)(MEMORY_SYSTEM_RAM),
            )
        }
    }
}

impl Drop for Core {
    fn drop(&mut self) {
        // SAFETY: tear down in the order libretro requires, before the library unloads.
        unsafe {
            if self.game_loaded {
                (self.api.unload_game)();
            }
            (self.api.deinit)();
        }
        with_host(|host| *host = host::HostState::default());
        CORE_LOADED.store(false, Ordering::SeqCst);
    }
}

fn prepare_host(config: &CoreConfig, gl_proc_address: GlProcAddressLookup) {
    with_host(|host| {
        *host = host::HostState::default();
        host.system_dir = path_to_cstring(&config.system_dir);
        host.save_dir = path_to_cstring(&config.save_dir);
        host.gl_proc_address = Some(gl_proc_address);
        host.variables = config
            .variables
            .iter()
            .map(|(key, value)| (key.clone(), to_cstring(value)))
            .collect();
    });
}

fn to_av_info(av_info: SystemAvInfo) -> AvInfo {
    AvInfo {
        base_width_px: av_info.geometry.base_width,
        base_height_px: av_info.geometry.base_height,
        max_width_px: av_info.geometry.max_width,
        max_height_px: av_info.geometry.max_height,
        fps: av_info.timing.fps,
        sample_rate_hz: av_info.timing.sample_rate,
    }
}

fn path_to_cstring(path: &Path) -> CString {
    to_cstring(&path.to_string_lossy())
}

fn to_cstring(text: &str) -> CString {
    CString::new(text.replace('\0', "")).unwrap_or_default()
}
