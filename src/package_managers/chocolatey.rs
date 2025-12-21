use super::{Confidence, DetectionContext, DetectionResult, PackageManagerDetector};
use crate::platform::Platform;

/// Detector for Chocolatey packages (Windows).
pub struct ChocolateyDetector;

impl ChocolateyDetector {
    pub fn new() -> Self {
        Self
    }
}

impl PackageManagerDetector for ChocolateyDetector {
    fn id(&self) -> &'static str {
        "chocolatey"
    }

    fn name(&self) -> &str {
        "Chocolatey"
    }

    fn supports_platform(&self, platform: Platform) -> bool {
        matches!(platform, Platform::Windows)
    }

    fn priority(&self) -> i32 {
        80
    }

    fn detect(&self, ctx: &DetectionContext) -> Option<DetectionResult> {
        for path in &ctx.symlink_chain {
            let path_str = path.to_string_lossy();

            // Check for Chocolatey paths
            if path_str.contains(r"\ProgramData\chocolatey\") || path_str.contains(r"\Chocolatey\")
            {
                return Some(DetectionResult {
                    manager_id: self.id().to_string(),
                    manager_name: self.name().to_string(),
                    package_name: Some(ctx.command_name.clone()),
                    version: None,
                    confidence: Confidence::Medium,
                    command_path: ctx.command_path.clone(),
                    resolved_path: ctx.resolved_path.clone(),
                });
            }
        }

        None
    }
}
