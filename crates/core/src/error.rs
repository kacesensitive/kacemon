use thiserror::Error;

/// Core errors for the system monitor
#[derive(Error, Debug)]
pub enum CoreError {
    #[error("System information error: {0}")]
    SystemInfo(String),

    #[error("Process information error: {0}")]
    ProcessInfo(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Platform-specific error: {0}")]
    Platform(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Feature not supported on this platform: {0}")]
    UnsupportedPlatform(String),

    #[cfg(feature = "linux_procfs")]
    #[error("Procfs error: {0}")]
    Procfs(#[from] procfs::Error),

    #[cfg(unix)]
    #[error("Unix system error: {0}")]
    Unix(#[from] nix::Error),
}

pub type Result<T> = std::result::Result<T, CoreError>;

impl CoreError {
    pub fn system_info<S: Into<String>>(msg: S) -> Self {
        Self::SystemInfo(msg.into())
    }

    pub fn process_info<S: Into<String>>(msg: S) -> Self {
        Self::ProcessInfo(msg.into())
    }

    pub fn config<S: Into<String>>(msg: S) -> Self {
        Self::Config(msg.into())
    }

    pub fn platform<S: Into<String>>(msg: S) -> Self {
        Self::Platform(msg.into())
    }

    pub fn permission_denied<S: Into<String>>(msg: S) -> Self {
        Self::PermissionDenied(msg.into())
    }

    pub fn unsupported_platform<S: Into<String>>(msg: S) -> Self {
        Self::UnsupportedPlatform(msg.into())
    }
}
