use super::{Confidence, DetectionContext, DetectionResult, PackageManagerDetector};
use crate::platform::Platform;

/// Detector for Snap packages.
pub struct SnapDetector;

impl SnapDetector {
    pub fn new() -> Self {
        Self
    }
}

impl PackageManagerDetector for SnapDetector {
    fn id(&self) -> &'static str {
        "snap"
    }

    fn name(&self) -> &str {
        "Snap"
    }

    fn supports_platform(&self, platform: Platform) -> bool {
        platform == Platform::Linux
    }

    fn priority(&self) -> i32 {
        80
    }

    fn detect(&self, ctx: &DetectionContext) -> Option<DetectionResult> {
        for path in &ctx.symlink_chain {
            let path_str = path.to_string_lossy();

            // Check for Snap paths:
            // /snap/bin/{command}
            // /var/lib/snapd/snap/bin/{command}
            // /snap/{package}/{revision}/{path}
            if path_str.contains("/snap/bin/")
                || path_str.starts_with("/snap/")
                || path_str.contains("/snapd/snap/")
            {
                let package_name =
                    extract_snap_package_name(&path_str).or_else(|| Some(ctx.command_name.clone()));

                return Some(DetectionResult {
                    manager_id: self.id().to_string(),
                    manager_name: self.name().to_string(),
                    package_name,
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

fn extract_snap_package_name(path: &str) -> Option<String> {
    // Pattern: /snap/{package}/{revision}/... or /snap/bin/{command}
    if let Some(rest) = path.strip_prefix("/snap/") {
        let parts: Vec<&str> = rest.split('/').collect();
        if let Some(first) = parts.first() {
            if *first != "bin" && !first.is_empty() {
                return Some(first.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_context(command: &str, paths: Vec<&str>, platform: Platform) -> DetectionContext {
        let command_path = PathBuf::from(paths.first().unwrap_or(&""));
        let resolved_path = PathBuf::from(paths.last().unwrap_or(&""));
        DetectionContext {
            command_name: command.to_string(),
            command_path: command_path.clone(),
            symlink_chain: paths.iter().map(PathBuf::from).collect(),
            resolved_path,
            platform,
        }
    }

    #[test]
    fn test_detects_snap_bin() {
        let detector = SnapDetector::new();
        let ctx = make_context("code", vec!["/snap/bin/code"], Platform::Linux);
        let result = detector.detect(&ctx);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.manager_id, "snap");
        assert_eq!(result.package_name, Some("code".to_string()));
    }

    #[test]
    fn test_detects_snap_package_path() {
        let detector = SnapDetector::new();
        let ctx = make_context(
            "code",
            vec!["/snap/bin/code", "/snap/code/174/usr/share/code/bin/code"],
            Platform::Linux,
        );
        let result = detector.detect(&ctx);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.manager_id, "snap");
        assert_eq!(result.package_name, Some("code".to_string()));
    }

    #[test]
    fn test_detects_snapd_path() {
        let detector = SnapDetector::new();
        let ctx = make_context("lxd", vec!["/var/lib/snapd/snap/bin/lxd"], Platform::Linux);
        let result = detector.detect(&ctx);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.manager_id, "snap");
    }

    #[test]
    fn test_ignores_non_snap_paths() {
        let detector = SnapDetector::new();
        let ctx = make_context("git", vec!["/usr/bin/git"], Platform::Linux);
        let result = detector.detect(&ctx);
        assert!(result.is_none());
    }

    #[test]
    fn test_only_supports_linux() {
        let detector = SnapDetector::new();
        assert!(detector.supports_platform(Platform::Linux));
        assert!(!detector.supports_platform(Platform::MacOS));
        assert!(!detector.supports_platform(Platform::Windows));
    }
}
