use super::{Confidence, DetectionContext, DetectionResult, PackageManagerDetector};
use crate::platform::Platform;

/// Detector for Scoop packages (Windows).
pub struct ScoopDetector;

impl ScoopDetector {
    pub fn new() -> Self {
        Self
    }
}

impl PackageManagerDetector for ScoopDetector {
    fn id(&self) -> &'static str {
        "scoop"
    }

    fn name(&self) -> &str {
        "Scoop"
    }

    fn supports_platform(&self, platform: Platform) -> bool {
        matches!(platform, Platform::Windows)
    }

    fn priority(&self) -> i32 {
        85
    }

    fn detect(&self, ctx: &DetectionContext) -> Option<DetectionResult> {
        for path in &ctx.symlink_chain {
            let path_str = path.to_string_lossy();

            // Check for Scoop paths
            // User install: C:\Users\<user>\scoop\apps\
            // User shims: C:\Users\<user>\scoop\shims\
            // Global install: C:\ProgramData\scoop\apps\
            // Global shims: C:\ProgramData\scoop\shims\
            if path_str.contains(r"\scoop\apps\") || path_str.contains(r"\scoop\shims\") {
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
    fn test_detects_scoop_user_apps() {
        let detector = ScoopDetector::new();
        let ctx = make_context(
            "git",
            vec![r"C:\Users\test\scoop\apps\git\current\bin\git.exe"],
            Platform::Windows,
        );
        let result = detector.detect(&ctx);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.manager_id, "scoop");
        assert_eq!(result.package_name, Some("git".to_string()));
    }

    #[test]
    fn test_detects_scoop_shims() {
        let detector = ScoopDetector::new();
        let ctx = make_context(
            "git",
            vec![r"C:\Users\test\scoop\shims\git.exe"],
            Platform::Windows,
        );
        let result = detector.detect(&ctx);
        assert!(result.is_some());
        assert_eq!(result.unwrap().manager_id, "scoop");
    }

    #[test]
    fn test_detects_scoop_global() {
        let detector = ScoopDetector::new();
        let ctx = make_context(
            "nodejs",
            vec![r"C:\ProgramData\scoop\apps\nodejs\current\node.exe"],
            Platform::Windows,
        );
        let result = detector.detect(&ctx);
        assert!(result.is_some());
        assert_eq!(result.unwrap().manager_id, "scoop");
    }

    #[test]
    fn test_ignores_non_scoop_paths() {
        let detector = ScoopDetector::new();
        let ctx = make_context(
            "git",
            vec![r"C:\Program Files\Git\bin\git.exe"],
            Platform::Windows,
        );
        let result = detector.detect(&ctx);
        assert!(result.is_none());
    }

    #[test]
    fn test_supports_windows_only() {
        let detector = ScoopDetector::new();
        assert!(detector.supports_platform(Platform::Windows));
        assert!(!detector.supports_platform(Platform::MacOS));
        assert!(!detector.supports_platform(Platform::Linux));
    }
}
