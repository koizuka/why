use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WhyError {
    #[error("Command '{0}' not found in PATH")]
    CommandNotFound(String),

    #[error("Failed to resolve path: {path}")]
    PathResolutionError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to read symlink: {path}")]
    SymlinkError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Package manager '{0}' is not available on this system")]
    PackageManagerUnavailable(String),

    #[error("Verification command failed: {command}")]
    VerificationFailed {
        command: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Could not determine package manager for '{0}'")]
    UnknownSource(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, WhyError>;
