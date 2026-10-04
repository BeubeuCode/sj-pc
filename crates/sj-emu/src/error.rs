use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum EmuError {
    #[error("cannot load core {path}: {source}")]
    LoadCore {
        path: PathBuf,
        source: libloading::Error,
    },
    #[error("core is missing symbol {0}")]
    MissingSymbol(&'static str),
    #[error("core speaks libretro API {0}, we speak 1")]
    UnsupportedApiVersion(u32),
    #[error("only one core can be loaded per process")]
    AlreadyLoaded,
    #[error("cannot read ROM {path}: {source}")]
    ReadRom {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("core refused to load the ROM")]
    GameRejected,
    #[error("core failed to save state")]
    SaveStateFailed,
    #[error("core rejected the save state")]
    LoadStateFailed,
}
