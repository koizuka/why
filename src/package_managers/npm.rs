use super::node_global::{detect_node_global, SKIP_BIN};
use super::{DetectionContext, DetectionResult, PackageManagerDetector};
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
        detect_node_global(ctx, self.id(), self.name(), is_npm_global_path, SKIP_BIN)
    }
}

fn is_npm_global_path(path: &str) -> bool {
    // Unix: /usr/local/lib/node_modules/, ~/.npm-global/lib/node_modules/, etc.
    // Windows: %APPDATA%\npm\node_modules\, etc.
    path.contains("/node_modules/")
        || path.contains("/.npm-global/")
        || path.contains(r"\node_modules\")
        || path.contains(r"\.npm-global\")
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
    fn test_npm_windows_scoped_via_symlink() {
        let detector = NpmGlobalDetector::new();
        let ctx = make_context(
            "ng",
            vec![
                r"C:\Users\u\AppData\Roaming\npm\ng.cmd",
                r"C:\Users\u\AppData\Roaming\npm\node_modules\@angular\cli\bin\ng",
            ],
        );
        let result = detector.detect(&ctx);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.manager_id, "npm_global");
        assert_eq!(result.package_name, Some("@angular/cli".to_string()));
    }

    #[test]
    fn test_non_npm_path() {
        let detector = NpmGlobalDetector::new();
        let ctx = make_context("git", vec!["/usr/bin/git"]);
        let result = detector.detect(&ctx);
        assert!(result.is_none());
    }
}
