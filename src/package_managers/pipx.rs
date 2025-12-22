use super::{Confidence, DetectionContext, DetectionResult, PackageManagerDetector};
use crate::platform::Platform;

/// Detector for pipx installed packages.
/// Note: We only detect pipx (not pip install --user) to avoid false positives,
/// since ~/.local/bin is used by many other tools.
pub struct PipxDetector;

impl PipxDetector {
    pub fn new() -> Self {
        Self
    }
}

impl PackageManagerDetector for PipxDetector {
    fn id(&self) -> &'static str {
        "pipx"
    }

    fn name(&self) -> &str {
        "pipx"
    }

    fn supports_platform(&self, _platform: Platform) -> bool {
        true // pipx is cross-platform
    }

    fn priority(&self) -> i32 {
        85
    }

    fn detect(&self, ctx: &DetectionContext) -> Option<DetectionResult> {
        for path in &ctx.symlink_chain {
            let path_str = path.to_string_lossy();

            // Check for pipx paths:
            // Unix: ~/.local/pipx/venvs/{package}/bin/
            // Windows: %USERPROFILE%\.local\pipx\venvs\{package}\Scripts\
            if path_str.contains("/pipx/venvs/") || path_str.contains(r"\pipx\venvs\") {
                let package_name = extract_pipx_package_name(&path_str);
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

fn extract_pipx_package_name(path: &str) -> Option<String> {
    // Pattern: .../pipx/venvs/{package}/bin/... or .../pipx/venvs/{package}/Scripts/...
    let patterns = ["/pipx/venvs/", r"\pipx\venvs\"];

    for pattern in patterns {
        if let Some(idx) = path.find(pattern) {
            let after = &path[idx + pattern.len()..];
            let parts: Vec<&str> = if pattern.contains('\\') {
                after.split('\\').collect()
            } else {
                after.split('/').collect()
            };

            if let Some(first) = parts.first() {
                if !first.is_empty() {
                    return Some(first.to_string());
                }
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
    fn test_detects_pipx_unix() {
        let detector = PipxDetector::new();
        let ctx = make_context(
            "httpie",
            vec!["/home/user/.local/pipx/venvs/httpie/bin/http"],
            Platform::Linux,
        );
        let result = detector.detect(&ctx);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.manager_id, "pipx");
        assert_eq!(result.package_name, Some("httpie".to_string()));
    }

    #[test]
    fn test_detects_pipx_macos() {
        let detector = PipxDetector::new();
        let ctx = make_context(
            "aws",
            vec!["/Users/user/.local/pipx/venvs/awscli/bin/aws"],
            Platform::MacOS,
        );
        let result = detector.detect(&ctx);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.manager_id, "pipx");
        assert_eq!(result.package_name, Some("awscli".to_string()));
    }

    #[test]
    fn test_detects_pipx_windows() {
        let detector = PipxDetector::new();
        let ctx = make_context(
            "http",
            vec![r"C:\Users\test\.local\pipx\venvs\httpie\Scripts\http.exe"],
            Platform::Windows,
        );
        let result = detector.detect(&ctx);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.manager_id, "pipx");
        assert_eq!(result.package_name, Some("httpie".to_string()));
    }

    #[test]
    fn test_ignores_local_bin() {
        // ~/.local/bin is used by many tools, so we don't detect it
        let detector = PipxDetector::new();
        let ctx = make_context(
            "some-tool",
            vec!["/home/user/.local/bin/some-tool"],
            Platform::Linux,
        );
        let result = detector.detect(&ctx);
        assert!(result.is_none());
    }

    #[test]
    fn test_ignores_non_pipx_paths() {
        let detector = PipxDetector::new();
        let ctx = make_context("python", vec!["/usr/bin/python"], Platform::Linux);
        let result = detector.detect(&ctx);
        assert!(result.is_none());
    }

    #[test]
    fn test_supports_all_platforms() {
        let detector = PipxDetector::new();
        assert!(detector.supports_platform(Platform::Windows));
        assert!(detector.supports_platform(Platform::MacOS));
        assert!(detector.supports_platform(Platform::Linux));
    }
}
