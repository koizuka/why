use super::node_global::{detect_node_global, SKIP_BIN_PNPM};
use super::{DetectionContext, DetectionResult, PackageManagerDetector};
use crate::platform::Platform;

/// Detector for pnpm global packages.
pub struct PnpmGlobalDetector;

impl PnpmGlobalDetector {
    pub fn new() -> Self {
        Self
    }
}

impl PackageManagerDetector for PnpmGlobalDetector {
    fn id(&self) -> &'static str {
        "pnpm_global"
    }

    fn name(&self) -> &str {
        "pnpm (global)"
    }

    fn supports_platform(&self, _platform: Platform) -> bool {
        true // pnpm is cross-platform
    }

    fn priority(&self) -> i32 {
        92 // Higher than npm and yarn to check first
    }

    fn detect(&self, ctx: &DetectionContext) -> Option<DetectionResult> {
        detect_node_global(
            ctx,
            self.id(),
            self.name(),
            is_pnpm_global_path,
            SKIP_BIN_PNPM,
        )
    }
}

fn is_pnpm_global_path(path: &str) -> bool {
    // Unix: ~/.local/share/pnpm/ or $PNPM_HOME
    // Windows: %LOCALAPPDATA%\pnpm\ or %APPDATA%\pnpm\
    // Also: pnpm/global/5/node_modules/.bin/
    path.contains("/.local/share/pnpm/")
        || path.ends_with("/.local/share/pnpm")
        || path.contains("/pnpm/global/")
        || path.contains(r"\pnpm\")
        || path.contains(r"\AppData\Local\pnpm")
        || path.contains(r"\AppData\Roaming\pnpm")
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
    fn test_detects_pnpm_unix() {
        let detector = PnpmGlobalDetector::new();
        let ctx = make_context(
            "tsc",
            vec!["/home/user/.local/share/pnpm/tsc"],
            Platform::Linux,
        );
        let result = detector.detect(&ctx);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.manager_id, "pnpm_global");
    }

    #[test]
    fn test_detects_pnpm_global_node_modules() {
        let detector = PnpmGlobalDetector::new();
        let ctx = make_context(
            "tsc",
            vec![
                "/home/user/.local/share/pnpm/tsc",
                "/home/user/.local/share/pnpm/global/5/node_modules/typescript/bin/tsc",
            ],
            Platform::Linux,
        );
        let result = detector.detect(&ctx);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.manager_id, "pnpm_global");
        assert_eq!(result.package_name, Some("typescript".to_string()));
    }

    #[test]
    fn test_detects_pnpm_windows() {
        let detector = PnpmGlobalDetector::new();
        let ctx = make_context(
            "tsc",
            vec![r"C:\Users\test\AppData\Local\pnpm\tsc.cmd"],
            Platform::Windows,
        );
        let result = detector.detect(&ctx);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.manager_id, "pnpm_global");
    }

    #[test]
    fn test_detects_scoped_package() {
        let detector = PnpmGlobalDetector::new();
        let ctx = make_context(
            "ng",
            vec![
                "/home/user/.local/share/pnpm/ng",
                "/home/user/.local/share/pnpm/global/5/node_modules/@angular/cli/bin/ng",
            ],
            Platform::Linux,
        );
        let result = detector.detect(&ctx);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.package_name, Some("@angular/cli".to_string()));
    }

    #[test]
    fn test_ignores_non_pnpm_paths() {
        let detector = PnpmGlobalDetector::new();
        let ctx = make_context("git", vec!["/usr/bin/git"], Platform::Linux);
        let result = detector.detect(&ctx);
        assert!(result.is_none());
    }

    #[test]
    fn test_supports_all_platforms() {
        let detector = PnpmGlobalDetector::new();
        assert!(detector.supports_platform(Platform::Windows));
        assert!(detector.supports_platform(Platform::MacOS));
        assert!(detector.supports_platform(Platform::Linux));
    }
}
