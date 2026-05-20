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
        // ~/.bun/bin/ or ~/.bun/install/global/
        let matched = ctx.symlink_chain.iter().any(|p| {
            let s = p.to_string_lossy();
            s.contains("/.bun/bin/") || s.contains("/.bun/install/global/")
        });

        if matched {
            let package_name = ctx
                .symlink_chain
                .iter()
                .filter_map(|p| extract_bun_package_name(&p.to_string_lossy()))
                .next()
                .or_else(|| Some(ctx.command_name.clone()));

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

        None
    }
}

fn extract_bun_package_name(path: &str) -> Option<String> {
    // Pattern: .../node_modules/{package}/... or .../node_modules/@{scope}/{package}/...
    let idx = path.find("/node_modules/")?;
    let after = &path[idx + "/node_modules/".len()..];
    let parts: Vec<&str> = after.split('/').collect();
    let first = parts.first()?;
    if first.is_empty() || *first == ".bin" {
        return None;
    }
    if first.starts_with('@') && parts.len() >= 2 && !parts[1].is_empty() {
        Some(format!("{}/{}", first, parts[1]))
    } else {
        Some(first.to_string())
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
    fn test_extract_bun_package_name() {
        assert_eq!(
            extract_bun_package_name(
                "/Users/u/.bun/install/global/node_modules/@openai/codex/bin/codex.js"
            ),
            Some("@openai/codex".to_string())
        );
        assert_eq!(
            extract_bun_package_name("/Users/u/.bun/install/global/node_modules/vite/bin/vite.js"),
            Some("vite".to_string())
        );
        assert_eq!(extract_bun_package_name("/Users/u/.bun/bin/codex"), None);
    }

    #[test]
    fn test_bun_scoped_package_via_symlink() {
        let detector = BunGlobalDetector::new();
        let ctx = make_context(
            "codex",
            vec![
                "/Users/u/.bun/bin/codex",
                "/Users/u/.bun/install/global/node_modules/@openai/codex/bin/codex.js",
            ],
        );
        let result = detector.detect(&ctx);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.manager_id, "bun_global");
        assert_eq!(result.package_name, Some("@openai/codex".to_string()));
    }

    #[test]
    fn test_non_bun_path() {
        let detector = BunGlobalDetector::new();
        let ctx = make_context("node", vec!["/usr/local/bin/node"]);
        let result = detector.detect(&ctx);
        assert!(result.is_none());
    }
}
