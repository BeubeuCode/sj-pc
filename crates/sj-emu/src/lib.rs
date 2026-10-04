mod error;
mod ffi;
mod frame;

pub use error::EmuError;
pub use ffi::core::{Core, CoreConfig};
pub use frame::{AvInfo, Frame, FrameOutput, Pointer};
