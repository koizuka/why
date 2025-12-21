use super::{Confidence, DetectionContext, DetectionResult, PackageManagerDetector};
use crate::platform::Platform;

/// Detector for system/OS standard binaries.
pub struct SystemDetector;

impl SystemDetector {
    pub fn new() -> Self {
        Self
    }
}

impl PackageManagerDetector for SystemDetector {
    fn id(&self) -> &'static str {
        "system"
    }

    fn name(&self) -> &str {
        "System (OS Standard)"
    }

    fn supports_platform(&self, _platform: Platform) -> bool {
        true
    }

    fn priority(&self) -> i32 {
        10 // Low priority - check last
    }

    fn detect(&self, ctx: &DetectionContext) -> Option<DetectionResult> {
        let path_str = ctx.resolved_path.to_string_lossy();

        let is_system = match ctx.platform {
            Platform::MacOS => {
                path_str.starts_with("/bin/")
                    || path_str.starts_with("/sbin/")
                    || path_str.starts_with("/usr/bin/")
                    || path_str.starts_with("/usr/sbin/")
                    || path_str.starts_with("/System/")
            }
            Platform::Linux => {
                path_str.starts_with("/bin/")
                    || path_str.starts_with("/sbin/")
                    || path_str.starts_with("/usr/bin/")
                    || path_str.starts_with("/usr/sbin/")
            }
            Platform::Windows => {
                path_str.contains(r"\Windows\System32\") || path_str.contains(r"\Windows\SysWOW64\")
            }
        };

        if is_system {
            return Some(DetectionResult {
                manager_id: self.id().to_string(),
                manager_name: self.name().to_string(),
                package_name: None,
                version: None,
                confidence: Confidence::Medium,
                command_path: ctx.command_path.clone(),
                resolved_path: ctx.resolved_path.clone(),
            });
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_context(command: &str, resolved_path: &str, platform: Platform) -> DetectionContext {
        let path = PathBuf::from(resolved_path);
        DetectionContext {
            command_name: command.to_string(),
            command_path: path.clone(),
            symlink_chain: vec![path.clone()],
            resolved_path: path,
            platform,
        }
    }

    #[test]
    fn test_macos_bin() {
        let detector = SystemDetector::new();
        let ctx = make_context("ls", "/bin/ls", Platform::MacOS);
        let result = detector.detect(&ctx);
        assert!(result.is_some());
        assert_eq!(result.unwrap().manager_id, "system");
    }

    #[test]
    fn test_macos_usr_bin() {
        let detector = SystemDetector::new();
        let ctx = make_context("env", "/usr/bin/env", Platform::MacOS);
        let result = detector.detect(&ctx);
        assert!(result.is_some());
    }

    #[test]
    fn test_macos_system_path() {
        let detector = SystemDetector::new();
        let ctx = make_context(
            "ruby",
            "/System/Library/Frameworks/Ruby.framework/Versions/2.6/usr/bin/ruby",
            Platform::MacOS,
        );
        let result = detector.detect(&ctx);
        assert!(result.is_some());
    }

    #[test]
    fn test_linux_bin() {
        let detector = SystemDetector::new();
        let ctx = make_context("ls", "/bin/ls", Platform::Linux);
        let result = detector.detect(&ctx);
        assert!(result.is_some());
    }

    #[test]
    fn test_windows_system32() {
        let detector = SystemDetector::new();
        let ctx = make_context("cmd", r"C:\Windows\System32\cmd.exe", Platform::Windows);
        let result = detector.detect(&ctx);
        assert!(result.is_some());
    }

    #[test]
    fn test_non_system_path() {
        let detector = SystemDetector::new();
        let ctx = make_context(
            "git",
            "/opt/homebrew/Cellar/git/2.51.2/bin/git",
            Platform::MacOS,
        );
        let result = detector.detect(&ctx);
        assert!(result.is_none());
    }

    #[test]
    fn test_user_local_bin_not_system() {
        let detector = SystemDetector::new();
        let ctx = make_context("myapp", "/usr/local/bin/myapp", Platform::MacOS);
        let result = detector.detect(&ctx);
        // /usr/local/bin is NOT system on macOS
        assert!(result.is_none());
    }
}
