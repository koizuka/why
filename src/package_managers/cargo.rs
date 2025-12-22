use super::{Confidence, DetectionContext, DetectionResult, PackageManagerDetector};
use crate::platform::Platform;

/// Detector for Cargo installed packages.
pub struct CargoDetector;

impl CargoDetector {
    pub fn new() -> Self {
        Self
    }
}

impl PackageManagerDetector for CargoDetector {
    fn id(&self) -> &'static str {
        "cargo"
    }

    fn name(&self) -> &str {
        "Cargo"
    }

    fn supports_platform(&self, _platform: Platform) -> bool {
        true // Cargo is cross-platform
    }

    fn priority(&self) -> i32 {
        85 // Same level as Scoop
    }

    fn detect(&self, ctx: &DetectionContext) -> Option<DetectionResult> {
        for path in &ctx.symlink_chain {
            let path_str = path.to_string_lossy();

            // Check for Cargo bin paths:
            // Unix: ~/.cargo/bin/ or $CARGO_HOME/bin/
            // Windows: %USERPROFILE%\.cargo\bin\ or %CARGO_HOME%\bin\
            if path_str.contains("/.cargo/bin/")
                || path_str.contains(r"\.cargo\bin\")
                || path_str.ends_with("/.cargo/bin")
                || path_str.ends_with(r"\.cargo\bin")
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
    fn test_detects_cargo_unix() {
        let detector = CargoDetector::new();
        let ctx = make_context(
            "rg",
            vec!["/home/user/.cargo/bin/rg"],
            Platform::Linux,
        );
        let result = detector.detect(&ctx);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.manager_id, "cargo");
        assert_eq!(result.package_name, Some("rg".to_string()));
    }

    #[test]
    fn test_detects_cargo_macos() {
        let detector = CargoDetector::new();
        let ctx = make_context(
            "bat",
            vec!["/Users/user/.cargo/bin/bat"],
            Platform::MacOS,
        );
        let result = detector.detect(&ctx);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.manager_id, "cargo");
        assert_eq!(result.package_name, Some("bat".to_string()));
    }

    #[test]
    fn test_detects_cargo_windows() {
        let detector = CargoDetector::new();
        let ctx = make_context(
            "rg",
            vec![r"C:\Users\test\.cargo\bin\rg.exe"],
            Platform::Windows,
        );
        let result = detector.detect(&ctx);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.manager_id, "cargo");
        assert_eq!(result.package_name, Some("rg".to_string()));
    }

    #[test]
    fn test_ignores_non_cargo_paths() {
        let detector = CargoDetector::new();
        let ctx = make_context(
            "rg",
            vec!["/usr/bin/rg"],
            Platform::Linux,
        );
        let result = detector.detect(&ctx);
        assert!(result.is_none());
    }

    #[test]
    fn test_supports_all_platforms() {
        let detector = CargoDetector::new();
        assert!(detector.supports_platform(Platform::Windows));
        assert!(detector.supports_platform(Platform::MacOS));
        assert!(detector.supports_platform(Platform::Linux));
    }
}
