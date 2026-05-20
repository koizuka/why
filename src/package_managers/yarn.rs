use super::node_global::{detect_node_global, SKIP_BIN};
use super::{DetectionContext, DetectionResult, PackageManagerDetector};
use crate::platform::Platform;

/// Detector for Yarn global packages.
pub struct YarnGlobalDetector;

impl YarnGlobalDetector {
    pub fn new() -> Self {
        Self
    }
}

impl PackageManagerDetector for YarnGlobalDetector {
    fn id(&self) -> &'static str {
        "yarn_global"
    }

    fn name(&self) -> &str {
        "yarn (global)"
    }

    fn supports_platform(&self, _platform: Platform) -> bool {
        true // Yarn is cross-platform
    }

    fn priority(&self) -> i32 {
        91 // Higher than npm to check first
    }

    fn detect(&self, ctx: &DetectionContext) -> Option<DetectionResult> {
        detect_node_global(ctx, self.id(), self.name(), is_yarn_global_path, SKIP_BIN)
    }
}

fn is_yarn_global_path(path: &str) -> bool {
    // Unix: ~/.yarn/bin/ or ~/.config/yarn/global/node_modules/.bin/
    // Windows: %LOCALAPPDATA%\Yarn\bin\ or %LOCALAPPDATA%\Yarn\Data\global\node_modules\.bin\
    path.contains("/.yarn/bin/")
        || path.ends_with("/.yarn/bin")
        || path.contains("/yarn/global/node_modules/")
        || path.contains(r"\Yarn\bin\")
        || path.ends_with(r"\Yarn\bin")
        || path.contains(r"\Yarn\Data\global\node_modules\")
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
    fn test_detects_yarn_bin_unix() {
        let detector = YarnGlobalDetector::new();
        let ctx = make_context(
            "create-react-app",
            vec!["/home/user/.yarn/bin/create-react-app"],
            Platform::Linux,
        );
        let result = detector.detect(&ctx);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.manager_id, "yarn_global");
    }

    #[test]
    fn test_detects_yarn_global_node_modules_unix() {
        let detector = YarnGlobalDetector::new();
        let ctx = make_context(
            "tsc",
            vec![
                "/home/user/.yarn/bin/tsc",
                "/home/user/.config/yarn/global/node_modules/typescript/bin/tsc",
            ],
            Platform::Linux,
        );
        let result = detector.detect(&ctx);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.manager_id, "yarn_global");
        assert_eq!(result.package_name, Some("typescript".to_string()));
    }

    #[test]
    fn test_detects_yarn_windows() {
        let detector = YarnGlobalDetector::new();
        let ctx = make_context(
            "tsc",
            vec![r"C:\Users\test\AppData\Local\Yarn\bin\tsc.cmd"],
            Platform::Windows,
        );
        let result = detector.detect(&ctx);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.manager_id, "yarn_global");
    }

    #[test]
    fn test_detects_scoped_package() {
        let detector = YarnGlobalDetector::new();
        let ctx = make_context(
            "ng",
            vec![
                "/home/user/.yarn/bin/ng",
                "/home/user/.config/yarn/global/node_modules/@angular/cli/bin/ng",
            ],
            Platform::Linux,
        );
        let result = detector.detect(&ctx);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.package_name, Some("@angular/cli".to_string()));
    }

    #[test]
    fn test_ignores_non_yarn_paths() {
        let detector = YarnGlobalDetector::new();
        let ctx = make_context("git", vec!["/usr/bin/git"], Platform::Linux);
        let result = detector.detect(&ctx);
        assert!(result.is_none());
    }

    #[test]
    fn test_supports_all_platforms() {
        let detector = YarnGlobalDetector::new();
        assert!(detector.supports_platform(Platform::Windows));
        assert!(detector.supports_platform(Platform::MacOS));
        assert!(detector.supports_platform(Platform::Linux));
    }
}
