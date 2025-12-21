mod bun;
mod homebrew;
mod npm;
mod system;

#[cfg(target_os = "linux")]
mod apt;

#[cfg(target_os = "windows")]
mod chocolatey;

use crate::platform::Platform;
use serde::Serialize;
use std::cmp::Reverse;
use std::path::{Path, PathBuf};

/// Detection context containing all information about the command being analyzed.
#[derive(Debug)]
pub struct DetectionContext {
    pub command_name: String,
    pub command_path: PathBuf,
    pub symlink_chain: Vec<PathBuf>,
    pub resolved_path: PathBuf,
    pub platform: Platform,
}

/// Result of a successful detection.
#[derive(Debug, Clone, Serialize)]
pub struct DetectionResult {
    pub manager_id: String,
    pub manager_name: String,
    pub package_name: Option<String>,
    pub version: Option<String>,
    pub confidence: Confidence,
    #[serde(serialize_with = "serialize_path")]
    pub command_path: PathBuf,
    #[serde(serialize_with = "serialize_path")]
    pub resolved_path: PathBuf,
}

fn serialize_path<S>(path: &Path, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&path.display().to_string())
}

/// Confidence level of the detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Confidence {
    /// Verified with package manager query
    High,
    /// Path pattern match + symlink analysis
    Medium,
    /// Path pattern match only
    Low,
    /// Best guess
    Uncertain,
}

/// Trait for package manager detectors.
pub trait PackageManagerDetector: Send + Sync {
    /// Unique identifier for this package manager.
    fn id(&self) -> &'static str;

    /// Human-readable name.
    fn name(&self) -> &str;

    /// Check if this detector applies to the given platform.
    fn supports_platform(&self, platform: Platform) -> bool;

    /// Priority for detection (higher = checked first).
    fn priority(&self) -> i32;

    /// Attempt to detect if command was installed by this package manager.
    fn detect(&self, ctx: &DetectionContext) -> Option<DetectionResult>;
}

/// Registry of all package manager detectors.
pub struct PackageManagerRegistry {
    detectors: Vec<Box<dyn PackageManagerDetector>>,
}

impl PackageManagerRegistry {
    pub fn new() -> Self {
        let mut detectors: Vec<Box<dyn PackageManagerDetector>> = vec![
            Box::new(homebrew::HomebrewDetector::new()),
            Box::new(npm::NpmGlobalDetector::new()),
            Box::new(bun::BunGlobalDetector::new()),
            Box::new(system::SystemDetector::new()),
        ];

        #[cfg(target_os = "linux")]
        {
            detectors.push(Box::new(apt::AptDetector::new()));
        }

        #[cfg(target_os = "windows")]
        {
            detectors.push(Box::new(chocolatey::ChocolateyDetector::new()));
        }

        // Sort by priority (higher first)
        detectors.sort_by_key(|d| Reverse(d.priority()));

        Self { detectors }
    }

    /// Try to detect the package manager for the given context.
    pub fn detect(&self, ctx: &DetectionContext, verbose: bool) -> Option<DetectionResult> {
        for detector in &self.detectors {
            if !detector.supports_platform(ctx.platform) {
                continue;
            }

            if verbose {
                eprintln!("Trying {}...", detector.name());
            }

            if let Some(result) = detector.detect(ctx) {
                if verbose {
                    eprintln!("✓ Matched: {}", detector.name());
                }
                return Some(result);
            }
        }
        None
    }
}

impl Default for PackageManagerRegistry {
    fn default() -> Self {
        Self::new()
    }
}
