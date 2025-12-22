use super::{Confidence, DetectionContext, DetectionResult, PackageManagerDetector};
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
        let mut matched = false;

        for path in &ctx.symlink_chain {
            let path_str = path.to_string_lossy();

            // Check for Yarn global paths:
            // Unix: ~/.yarn/bin/ or ~/.config/yarn/global/node_modules/.bin/
            // Windows: %LOCALAPPDATA%\Yarn\bin\ or %LOCALAPPDATA%\Yarn\Data\global\node_modules\.bin\
            if path_str.contains("/.yarn/bin/")
                || path_str.ends_with("/.yarn/bin")
                || path_str.contains("/yarn/global/node_modules/")
                || path_str.contains(r"\Yarn\bin\")
                || path_str.ends_with(r"\Yarn\bin")
                || path_str.contains(r"\Yarn\Data\global\node_modules\")
            {
                matched = true;
            }
        }

        if matched {
            // Try to extract package name from any path in the chain
            let package_name = ctx
                .symlink_chain
                .iter()
                .filter_map(|p| extract_yarn_package_name(&p.to_string_lossy()))
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

fn extract_yarn_package_name(path: &str) -> Option<String> {
    // Pattern: .../yarn/global/node_modules/{package}/... or similar
    let patterns = ["/node_modules/", r"\node_modules\"];

    for pattern in patterns {
        if let Some(idx) = path.find(pattern) {
            let after = &path[idx + pattern.len()..];
            let parts: Vec<&str> = if pattern.contains('\\') {
                after.split('\\').collect()
            } else {
                after.split('/').collect()
            };

            if let Some(first) = parts.first() {
                if first.is_empty() {
                    continue;
                }
                if first.starts_with('@') && parts.len() >= 2 && !parts[1].is_empty() {
                    // Scoped package
                    return Some(format!("{}/{}", first, parts[1]));
                } else if *first != ".bin" {
                    return Some(first.to_string());
                }
            }
        }
    }

    // Fallback to command name if in .yarn/bin or similar
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
