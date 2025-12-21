pub mod cli;
pub mod detector;
pub mod error;
pub mod package_managers;
pub mod platform;

pub use cli::{Cli, OutputFormat};
pub use detector::detect_command;
pub use error::{Result, WhyError};
pub use package_managers::{Confidence, DetectionResult};
pub use platform::Platform;
