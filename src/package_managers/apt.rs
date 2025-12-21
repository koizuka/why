use super::{Confidence, DetectionContext, DetectionResult, PackageManagerDetector};
use crate::platform::Platform;
use std::process::Command;

/// Detector for apt/dpkg packages (Debian/Ubuntu).
pub struct AptDetector;

impl AptDetector {
    pub fn new() -> Self {
        Self
    }
}

impl PackageManagerDetector for AptDetector {
    fn id(&self) -> &'static str {
        "apt"
    }

    fn name(&self) -> &str {
        "apt"
    }

    fn supports_platform(&self, platform: Platform) -> bool {
        matches!(platform, Platform::Linux)
    }

    fn priority(&self) -> i32 {
        50
    }

    fn detect(&self, ctx: &DetectionContext) -> Option<DetectionResult> {
        let path_str = ctx.resolved_path.to_string_lossy();

        // Only check system paths
        if !path_str.starts_with("/usr/bin/")
            && !path_str.starts_with("/usr/sbin/")
            && !path_str.starts_with("/bin/")
            && !path_str.starts_with("/sbin/")
        {
            return None;
        }

        // Query dpkg to find which package owns this file
        if let Some((package, version)) = query_dpkg(&ctx.resolved_path.to_string_lossy()) {
            return Some(DetectionResult {
                manager_id: self.id().to_string(),
                manager_name: self.name().to_string(),
                package_name: Some(package),
                version: Some(version),
                confidence: Confidence::High,
                command_path: ctx.command_path.clone(),
                resolved_path: ctx.resolved_path.clone(),
            });
        }

        None
    }
}

fn query_dpkg(path: &str) -> Option<(String, String)> {
    // dpkg -S /path/to/file returns: package-name: /path/to/file
    let output = Command::new("dpkg").args(["-S", path]).output().ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let package_name = stdout.split(':').next()?.trim().to_string();

    // Get version with dpkg-query
    let version_output = Command::new("dpkg-query")
        .args(["-W", "-f=${Version}", &package_name])
        .output()
        .ok()?;

    let version = String::from_utf8_lossy(&version_output.stdout)
        .trim()
        .to_string();

    Some((package_name, version))
}
