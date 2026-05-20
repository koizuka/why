use super::{Confidence, DetectionContext, DetectionResult, PackageManagerDetector};
use crate::platform::Platform;

/// Detector for npm global packages.
pub struct NpmGlobalDetector;

impl NpmGlobalDetector {
    pub fn new() -> Self {
        Self
    }
}

impl PackageManagerDetector for NpmGlobalDetector {
    fn id(&self) -> &'static str {
        "npm_global"
    }

    fn name(&self) -> &str {
        "npm (global)"
    }

    fn supports_platform(&self, _platform: Platform) -> bool {
        true // npm is cross-platform
    }

    fn priority(&self) -> i32 {
        90
    }

    fn detect(&self, ctx: &DetectionContext) -> Option<DetectionResult> {
        let matched = ctx.symlink_chain.iter().any(|p| {
            let s = p.to_string_lossy();
            s.contains("/node_modules/")
                || s.contains("/.npm-global/")
                || s.contains("/lib/node_modules/")
        });

        if matched {
            let package_name = ctx
                .symlink_chain
                .iter()
                .filter_map(|p| extract_npm_package_name(&p.to_string_lossy()))
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

fn extract_npm_package_name(path: &str) -> Option<String> {
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
    fn test_extract_npm_package_name() {
        assert_eq!(
            extract_npm_package_name("/usr/local/lib/node_modules/typescript/bin/tsc"),
            Some("typescript".to_string())
        );

        assert_eq!(
            extract_npm_package_name("/home/user/.npm-global/lib/node_modules/@angular/cli/bin/ng"),
            Some("@angular/cli".to_string())
        );
    }

    #[test]
    fn test_extract_npm_package_name_edge_cases() {
        // Path without node_modules
        assert_eq!(extract_npm_package_name("/usr/local/bin/node"), None);

        // Empty after node_modules
        assert_eq!(extract_npm_package_name("/foo/node_modules/"), None);
    }

    #[test]
    fn test_npm_global_detection() {
        let detector = NpmGlobalDetector::new();
        let ctx = make_context(
            "tsc",
            vec!["/usr/local/lib/node_modules/typescript/bin/tsc"],
        );
        let result = detector.detect(&ctx);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.manager_id, "npm_global");
        assert_eq!(result.package_name, Some("typescript".to_string()));
    }

    #[test]
    fn test_npm_global_via_symlink() {
        let detector = NpmGlobalDetector::new();
        // Simulates: /usr/local/bin/tsc -> /usr/local/lib/node_modules/typescript/bin/tsc
        let ctx = make_context(
            "tsc",
            vec![
                "/usr/local/bin/tsc",
                "/usr/local/lib/node_modules/typescript/bin/tsc",
            ],
        );
        let result = detector.detect(&ctx);
        assert!(result.is_some());
        assert_eq!(result.unwrap().package_name, Some("typescript".to_string()));
    }

    #[test]
    fn test_npm_global_path() {
        let detector = NpmGlobalDetector::new();
        let ctx = make_context("eslint", vec!["/Users/user/.npm-global/bin/eslint"]);
        let result = detector.detect(&ctx);
        assert!(result.is_some());
    }

    #[test]
    fn test_npm_scoped_package_via_npm_global_symlink() {
        // Scoped package: the bin entry matches the detector but has no
        // /node_modules/ segment — the scope must be read from the resolved target.
        let detector = NpmGlobalDetector::new();
        let ctx = make_context(
            "ni",
            vec![
                "/Users/u/.npm-global/bin/ni",
                "/Users/u/.npm-global/lib/node_modules/@antfu/ni/bin/ni.mjs",
            ],
        );
        let result = detector.detect(&ctx);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.manager_id, "npm_global");
        assert_eq!(result.package_name, Some("@antfu/ni".to_string()));
    }

    #[test]
    fn test_non_npm_path() {
        let detector = NpmGlobalDetector::new();
        let ctx = make_context("git", vec!["/usr/bin/git"]);
        let result = detector.detect(&ctx);
        assert!(result.is_none());
    }
}
