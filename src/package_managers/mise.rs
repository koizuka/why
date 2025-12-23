use super::{Confidence, DetectionContext, DetectionResult, PackageManagerDetector};
use crate::platform::Platform;

/// Detector for mise (formerly rtx) installed packages.
/// mise is a polyglot runtime manager (like asdf).
pub struct MiseDetector;

impl MiseDetector {
    pub fn new() -> Self {
        Self
    }
}

impl PackageManagerDetector for MiseDetector {
    fn id(&self) -> &'static str {
        "mise"
    }

    fn name(&self) -> &str {
        "mise"
    }

    fn supports_platform(&self, _platform: Platform) -> bool {
        true // mise is cross-platform
    }

    fn priority(&self) -> i32 {
        90 // Higher priority since it uses shims
    }

    fn detect(&self, ctx: &DetectionContext) -> Option<DetectionResult> {
        for path in &ctx.symlink_chain {
            let path_str = path.to_string_lossy();

            // Check for mise paths:
            // Unix: ~/.local/share/mise/installs/*/ or ~/.local/share/mise/shims/
            // XDG: $XDG_DATA_HOME/mise/installs/*/ or $XDG_DATA_HOME/mise/shims/
            // Windows: %LOCALAPPDATA%\mise\installs\*\
            if path_str.contains("/mise/installs/")
                || path_str.contains(r"\mise\installs\")
                || path_str.contains("/mise/shims/")
                || path_str.contains(r"\mise\shims\")
                || path_str.ends_with("/mise/shims")
                || path_str.ends_with(r"\mise\shims")
            {
                let tool_name = extract_mise_tool_name(&path_str);
                return Some(DetectionResult {
                    manager_id: self.id().to_string(),
                    manager_name: self.name().to_string(),
                    package_name: tool_name,
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

fn extract_mise_tool_name(path: &str) -> Option<String> {
    // Pattern: .../mise/installs/{tool}/{version}/bin/...
    let patterns = ["/mise/installs/", r"\mise\installs\"];

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

    // For shims, we can't determine the tool name
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
    fn test_detects_mise_installs_unix() {
        let detector = MiseDetector::new();
        let ctx = make_context(
            "node",
            vec!["/home/user/.local/share/mise/installs/node/20.10.0/bin/node"],
            Platform::Linux,
        );
        let result = detector.detect(&ctx);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.manager_id, "mise");
        assert_eq!(result.package_name, Some("node".to_string()));
    }

    #[test]
    fn test_detects_mise_installs_macos() {
        let detector = MiseDetector::new();
        let ctx = make_context(
            "python",
            vec!["/Users/user/.local/share/mise/installs/python/3.12.0/bin/python"],
            Platform::MacOS,
        );
        let result = detector.detect(&ctx);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.manager_id, "mise");
        assert_eq!(result.package_name, Some("python".to_string()));
    }

    #[test]
    fn test_detects_mise_shims_unix() {
        let detector = MiseDetector::new();
        let ctx = make_context(
            "node",
            vec!["/home/user/.local/share/mise/shims/node"],
            Platform::Linux,
        );
        let result = detector.detect(&ctx);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.manager_id, "mise");
        // Shims don't have tool name in path
        assert_eq!(result.package_name, None);
    }

    #[test]
    fn test_detects_mise_xdg_path() {
        let detector = MiseDetector::new();
        let ctx = make_context(
            "ruby",
            vec!["/home/user/.data/mise/installs/ruby/3.2.0/bin/ruby"],
            Platform::Linux,
        );
        let result = detector.detect(&ctx);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.manager_id, "mise");
        assert_eq!(result.package_name, Some("ruby".to_string()));
    }

    #[test]
    fn test_detects_mise_windows() {
        let detector = MiseDetector::new();
        let ctx = make_context(
            "node",
            vec![r"C:\Users\test\AppData\Local\mise\installs\node\20.10.0\bin\node.exe"],
            Platform::Windows,
        );
        let result = detector.detect(&ctx);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.manager_id, "mise");
        assert_eq!(result.package_name, Some("node".to_string()));
    }

    #[test]
    fn test_ignores_non_mise_paths() {
        let detector = MiseDetector::new();
        let ctx = make_context("node", vec!["/usr/bin/node"], Platform::Linux);
        let result = detector.detect(&ctx);
        assert!(result.is_none());
    }

    #[test]
    fn test_supports_all_platforms() {
        let detector = MiseDetector::new();
        assert!(detector.supports_platform(Platform::Windows));
        assert!(detector.supports_platform(Platform::MacOS));
        assert!(detector.supports_platform(Platform::Linux));
    }
}
