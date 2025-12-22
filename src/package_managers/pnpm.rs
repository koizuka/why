use super::{Confidence, DetectionContext, DetectionResult, PackageManagerDetector};
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
        let mut matched = false;

        for path in &ctx.symlink_chain {
            let path_str = path.to_string_lossy();

            // Check for pnpm global paths:
            // Unix: ~/.local/share/pnpm/ or $PNPM_HOME
            // Windows: %LOCALAPPDATA%\pnpm\ or %APPDATA%\pnpm\
            // Also: pnpm/global/5/node_modules/.bin/
            if path_str.contains("/.local/share/pnpm/")
                || path_str.ends_with("/.local/share/pnpm")
                || path_str.contains("/pnpm/global/")
                || path_str.contains(r"\pnpm\")
                || path_str.contains(r"\AppData\Local\pnpm")
                || path_str.contains(r"\AppData\Roaming\pnpm")
            {
                matched = true;
            }
        }

        if matched {
            // Try to extract package name from any path in the chain
            let package_name = ctx
                .symlink_chain
                .iter()
                .filter_map(|p| extract_pnpm_package_name(&p.to_string_lossy()))
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

fn extract_pnpm_package_name(path: &str) -> Option<String> {
    // Pattern: .../pnpm/global/{version}/node_modules/{package}/... or similar
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
                } else if *first != ".bin" && *first != ".pnpm" {
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
