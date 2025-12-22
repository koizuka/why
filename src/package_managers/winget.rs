use super::{Confidence, DetectionContext, DetectionResult, PackageManagerDetector};
use crate::platform::Platform;

/// Detector for Winget (Windows Package Manager) packages.
pub struct WingetDetector;

impl WingetDetector {
    pub fn new() -> Self {
        Self
    }
}

impl PackageManagerDetector for WingetDetector {
    fn id(&self) -> &'static str {
        "winget"
    }

    fn name(&self) -> &str {
        "Winget"
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

            // Check for WinGet paths
            // Portable packages: %LOCALAPPDATA%\Microsoft\WinGet\Packages\
            // Machine-wide: C:\Program Files\WinGet\Packages\
            // x86: C:\Program Files (x86)\WinGet\Packages\
            if path_str.contains(r"\Microsoft\WinGet\Packages\")
                || path_str.contains(r"\WinGet\Packages\")
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
    fn test_detects_winget_portable_user() {
        let detector = WingetDetector::new();
        let ctx = make_context(
            "code",
            vec![
                r"C:\Users\test\AppData\Local\Microsoft\WinGet\Packages\Microsoft.VisualStudioCode_8wekyb3d8bbwe\code.exe",
            ],
            Platform::Windows,
        );
        let result = detector.detect(&ctx);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.manager_id, "winget");
        assert_eq!(result.package_name, Some("code".to_string()));
    }

    #[test]
    fn test_detects_winget_program_files() {
        let detector = WingetDetector::new();
        let ctx = make_context(
            "app",
            vec![r"C:\Program Files\WinGet\Packages\SomeApp\app.exe"],
            Platform::Windows,
        );
        let result = detector.detect(&ctx);
        assert!(result.is_some());
        assert_eq!(result.unwrap().manager_id, "winget");
    }

    #[test]
    fn test_ignores_non_winget_paths() {
        let detector = WingetDetector::new();
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
        let detector = WingetDetector::new();
        assert!(detector.supports_platform(Platform::Windows));
        assert!(!detector.supports_platform(Platform::MacOS));
        assert!(!detector.supports_platform(Platform::Linux));
    }
}
