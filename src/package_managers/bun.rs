use super::{Confidence, DetectionContext, DetectionResult, PackageManagerDetector};
use crate::platform::Platform;

/// Detector for bun global packages.
pub struct BunGlobalDetector;

impl BunGlobalDetector {
    pub fn new() -> Self {
        Self
    }
}

impl PackageManagerDetector for BunGlobalDetector {
    fn id(&self) -> &'static str {
        "bun_global"
    }

    fn name(&self) -> &str {
        "bun (global)"
    }

    fn supports_platform(&self, _platform: Platform) -> bool {
        true // bun is cross-platform
    }

    fn priority(&self) -> i32 {
        95 // Higher than npm (90) to check bun paths first
    }

    fn detect(&self, ctx: &DetectionContext) -> Option<DetectionResult> {
        for path in &ctx.symlink_chain {
            let path_str = path.to_string_lossy();

            // Check bun global patterns
            // ~/.bun/bin/ or ~/.bun/install/global/
            if path_str.contains("/.bun/bin/") || path_str.contains("/.bun/install/global/") {
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

    fn make_context(command: &str, paths: Vec<&str>) -> DetectionContext {
        let command_path = PathBuf::from(paths.first().unwrap_or(&""));
        let resolved_path = PathBuf::from(paths.last().unwrap_or(&""));
        DetectionContext {
            command_name: command.to_string(),
            command_path: command_path.clone(),
            symlink_chain: paths.iter().map(PathBuf::from).collect(),
            resolved_path,
            platform: Platform::MacOS,
        }
    }

    #[test]
    fn test_bun_bin_path() {
        let detector = BunGlobalDetector::new();
        let ctx = make_context("vite", vec!["/Users/user/.bun/bin/vite"]);
        let result = detector.detect(&ctx);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.manager_id, "bun_global");
        assert_eq!(result.package_name, Some("vite".to_string()));
    }

    #[test]
    fn test_bun_install_global_path() {
        let detector = BunGlobalDetector::new();
        let ctx = make_context(
            "eslint",
            vec!["/Users/user/.bun/install/global/node_modules/.bin/eslint"],
        );
        let result = detector.detect(&ctx);
        assert!(result.is_some());
        assert_eq!(result.unwrap().manager_id, "bun_global");
    }

    #[test]
    fn test_non_bun_path() {
        let detector = BunGlobalDetector::new();
        let ctx = make_context("node", vec!["/usr/local/bin/node"]);
        let result = detector.detect(&ctx);
        assert!(result.is_none());
    }
}
