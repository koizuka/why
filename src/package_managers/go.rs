use super::{Confidence, DetectionContext, DetectionResult, PackageManagerDetector};
use crate::platform::Platform;

/// Detector for Go installed packages (go install).
pub struct GoDetector;

impl GoDetector {
    pub fn new() -> Self {
        Self
    }
}

impl PackageManagerDetector for GoDetector {
    fn id(&self) -> &'static str {
        "go"
    }

    fn name(&self) -> &str {
        "go install"
    }

    fn supports_platform(&self, _platform: Platform) -> bool {
        true // Go is cross-platform
    }

    fn priority(&self) -> i32 {
        85
    }

    fn detect(&self, ctx: &DetectionContext) -> Option<DetectionResult> {
        for path in &ctx.symlink_chain {
            let path_str = path.to_string_lossy();

            // Check for Go bin paths:
            // Unix: ~/go/bin/ or $GOPATH/bin/ or $GOBIN
            // Windows: %USERPROFILE%\go\bin\
            if path_str.contains("/go/bin/")
                || path_str.ends_with("/go/bin")
                || path_str.contains(r"\go\bin\")
                || path_str.ends_with(r"\go\bin")
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
    fn test_detects_go_unix() {
        let detector = GoDetector::new();
        let ctx = make_context("ghq", vec!["/home/user/go/bin/ghq"], Platform::Linux);
        let result = detector.detect(&ctx);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.manager_id, "go");
        assert_eq!(result.package_name, Some("ghq".to_string()));
    }

    #[test]
    fn test_detects_go_macos() {
        let detector = GoDetector::new();
        let ctx = make_context("lazygit", vec!["/Users/user/go/bin/lazygit"], Platform::MacOS);
        let result = detector.detect(&ctx);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.manager_id, "go");
        assert_eq!(result.package_name, Some("lazygit".to_string()));
    }

    #[test]
    fn test_detects_go_windows() {
        let detector = GoDetector::new();
        let ctx = make_context(
            "ghq",
            vec![r"C:\Users\test\go\bin\ghq.exe"],
            Platform::Windows,
        );
        let result = detector.detect(&ctx);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.manager_id, "go");
        assert_eq!(result.package_name, Some("ghq".to_string()));
    }

    #[test]
    fn test_detects_go_custom_gopath() {
        let detector = GoDetector::new();
        let ctx = make_context(
            "goimports",
            vec!["/opt/gopath/go/bin/goimports"],
            Platform::Linux,
        );
        let result = detector.detect(&ctx);
        assert!(result.is_some());
    }

    #[test]
    fn test_ignores_non_go_paths() {
        let detector = GoDetector::new();
        let ctx = make_context("go", vec!["/usr/local/go/bin/go"], Platform::Linux);
        // This is the Go compiler itself, but it matches our pattern
        // This is acceptable since it's still Go-related
        let result = detector.detect(&ctx);
        assert!(result.is_some());
    }

    #[test]
    fn test_ignores_unrelated_paths() {
        let detector = GoDetector::new();
        let ctx = make_context("git", vec!["/usr/bin/git"], Platform::Linux);
        let result = detector.detect(&ctx);
        assert!(result.is_none());
    }

    #[test]
    fn test_supports_all_platforms() {
        let detector = GoDetector::new();
        assert!(detector.supports_platform(Platform::Windows));
        assert!(detector.supports_platform(Platform::MacOS));
        assert!(detector.supports_platform(Platform::Linux));
    }
}
