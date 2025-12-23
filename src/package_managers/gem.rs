use super::{Confidence, DetectionContext, DetectionResult, PackageManagerDetector};
use crate::platform::Platform;

/// Detector for RubyGems installed packages.
pub struct GemDetector;

impl GemDetector {
    pub fn new() -> Self {
        Self
    }
}

impl PackageManagerDetector for GemDetector {
    fn id(&self) -> &'static str {
        "gem"
    }

    fn name(&self) -> &str {
        "RubyGems"
    }

    fn supports_platform(&self, _platform: Platform) -> bool {
        true // RubyGems is cross-platform
    }

    fn priority(&self) -> i32 {
        85
    }

    fn detect(&self, ctx: &DetectionContext) -> Option<DetectionResult> {
        for path in &ctx.symlink_chain {
            let path_str = path.to_string_lossy();

            // Check for gem paths:
            // Unix: ~/.gem/ruby/*/bin/ or /var/lib/gems/*/bin/
            // macOS: /usr/local/lib/ruby/gems/*/bin/
            // Windows: %USERPROFILE%/.gem/ruby/*/bin/
            if path_str.contains("/.gem/ruby/")
                || path_str.contains(r"\.gem\ruby\")
                || path_str.contains("/ruby/gems/")
                || path_str.contains(r"\ruby\gems\")
                || path_str.contains("/var/lib/gems/")
            {
                // Verify it's in a bin directory
                if path_str.contains("/bin/")
                    || path_str.contains(r"\bin\")
                    || path_str.ends_with("/bin")
                    || path_str.ends_with(r"\bin")
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
    fn test_detects_gem_user_install_unix() {
        let detector = GemDetector::new();
        let ctx = make_context(
            "sass",
            vec!["/home/user/.gem/ruby/3.2.0/bin/sass"],
            Platform::Linux,
        );
        let result = detector.detect(&ctx);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.manager_id, "gem");
        assert_eq!(result.package_name, Some("sass".to_string()));
    }

    #[test]
    fn test_detects_gem_user_install_macos() {
        let detector = GemDetector::new();
        let ctx = make_context(
            "bundler",
            vec!["/Users/user/.gem/ruby/3.1.0/bin/bundler"],
            Platform::MacOS,
        );
        let result = detector.detect(&ctx);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.manager_id, "gem");
        assert_eq!(result.package_name, Some("bundler".to_string()));
    }

    #[test]
    fn test_detects_gem_system_install_macos() {
        let detector = GemDetector::new();
        let ctx = make_context(
            "rails",
            vec!["/usr/local/lib/ruby/gems/3.2.0/bin/rails"],
            Platform::MacOS,
        );
        let result = detector.detect(&ctx);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.manager_id, "gem");
        assert_eq!(result.package_name, Some("rails".to_string()));
    }

    #[test]
    fn test_detects_gem_system_install_linux() {
        let detector = GemDetector::new();
        let ctx = make_context(
            "jekyll",
            vec!["/var/lib/gems/3.0.0/bin/jekyll"],
            Platform::Linux,
        );
        let result = detector.detect(&ctx);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.manager_id, "gem");
        assert_eq!(result.package_name, Some("jekyll".to_string()));
    }

    #[test]
    fn test_detects_gem_windows() {
        let detector = GemDetector::new();
        let ctx = make_context(
            "sass",
            vec![r"C:\Users\test\.gem\ruby\3.2.0\bin\sass.bat"],
            Platform::Windows,
        );
        let result = detector.detect(&ctx);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.manager_id, "gem");
        assert_eq!(result.package_name, Some("sass".to_string()));
    }

    #[test]
    fn test_ignores_non_gem_paths() {
        let detector = GemDetector::new();
        let ctx = make_context("ruby", vec!["/usr/bin/ruby"], Platform::Linux);
        let result = detector.detect(&ctx);
        assert!(result.is_none());
    }

    #[test]
    fn test_supports_all_platforms() {
        let detector = GemDetector::new();
        assert!(detector.supports_platform(Platform::Windows));
        assert!(detector.supports_platform(Platform::MacOS));
        assert!(detector.supports_platform(Platform::Linux));
    }
}
