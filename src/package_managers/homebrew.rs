use super::{Confidence, DetectionContext, DetectionResult, PackageManagerDetector};
use crate::platform::Platform;
use once_cell::sync::Lazy;
use regex::Regex;

/// Detector for Homebrew package manager (macOS and Linux).
pub struct HomebrewDetector;

impl HomebrewDetector {
    pub fn new() -> Self {
        Self
    }
}

// Regex to extract package name and version from Cellar path
// /opt/homebrew/Cellar/{package}/{version}/... (ARM Mac)
// /usr/local/Cellar/{package}/{version}/... (Intel Mac)
// /home/linuxbrew/.linuxbrew/Cellar/{package}/{version}/... (Linux)
static CELLAR_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?:/opt/homebrew|/usr/local|/home/linuxbrew/\.linuxbrew)/Cellar/([^/]+)/([^/]+)/")
        .unwrap()
});

impl PackageManagerDetector for HomebrewDetector {
    fn id(&self) -> &'static str {
        "homebrew"
    }

    fn name(&self) -> &str {
        "Homebrew"
    }

    fn supports_platform(&self, platform: Platform) -> bool {
        matches!(platform, Platform::MacOS | Platform::Linux)
    }

    fn priority(&self) -> i32 {
        100 // High priority
    }

    fn detect(&self, ctx: &DetectionContext) -> Option<DetectionResult> {
        // Check all paths in the symlink chain for Cellar pattern
        for path in &ctx.symlink_chain {
            let path_str = path.to_string_lossy();

            if let Some(captures) = CELLAR_REGEX.captures(&path_str) {
                let package_name = captures.get(1).map(|m| m.as_str().to_string());
                let version = captures.get(2).map(|m| m.as_str().to_string());

                return Some(DetectionResult {
                    manager_id: self.id().to_string(),
                    manager_name: self.name().to_string(),
                    package_name,
                    version,
                    confidence: Confidence::High,
                    command_path: ctx.command_path.clone(),
                    resolved_path: ctx.resolved_path.clone(),
                });
            }
        }

        // Also check for Homebrew bin paths without Cellar (e.g., keg-only formulas)
        let resolved_str = ctx.resolved_path.to_string_lossy();
        if resolved_str.contains("/opt/homebrew/")
            || resolved_str.contains("/usr/local/Homebrew/")
            || resolved_str.contains("/home/linuxbrew/.linuxbrew/")
        {
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

    // Regex tests
    #[test]
    fn test_cellar_regex() {
        let path = "/opt/homebrew/Cellar/git/2.51.2/bin/git";
        let caps = CELLAR_REGEX.captures(path).unwrap();
        assert_eq!(caps.get(1).unwrap().as_str(), "git");
        assert_eq!(caps.get(2).unwrap().as_str(), "2.51.2");
    }

    #[test]
    fn test_intel_mac_cellar() {
        let path = "/usr/local/Cellar/node/22.0.0/bin/node";
        let caps = CELLAR_REGEX.captures(path).unwrap();
        assert_eq!(caps.get(1).unwrap().as_str(), "node");
        assert_eq!(caps.get(2).unwrap().as_str(), "22.0.0");
    }

    #[test]
    fn test_linuxbrew_cellar() {
        let path = "/home/linuxbrew/.linuxbrew/Cellar/gcc/14.1.0/bin/gcc";
        let caps = CELLAR_REGEX.captures(path).unwrap();
        assert_eq!(caps.get(1).unwrap().as_str(), "gcc");
        assert_eq!(caps.get(2).unwrap().as_str(), "14.1.0");
    }

    #[test]
    fn test_cellar_regex_no_match() {
        let path = "/usr/bin/git";
        assert!(CELLAR_REGEX.captures(path).is_none());
    }

    // Detection tests
    #[test]
    fn test_homebrew_arm_mac_detection() {
        let detector = HomebrewDetector::new();
        let ctx = make_context(
            "git",
            vec![
                "/opt/homebrew/bin/git",
                "/opt/homebrew/Cellar/git/2.51.2/bin/git",
            ],
            Platform::MacOS,
        );
        let result = detector.detect(&ctx);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.manager_id, "homebrew");
        assert_eq!(result.package_name, Some("git".to_string()));
        assert_eq!(result.version, Some("2.51.2".to_string()));
        assert_eq!(result.confidence, Confidence::High);
    }

    #[test]
    fn test_homebrew_intel_mac_detection() {
        let detector = HomebrewDetector::new();
        let ctx = make_context(
            "node",
            vec!["/usr/local/Cellar/node/22.0.0/bin/node"],
            Platform::MacOS,
        );
        let result = detector.detect(&ctx);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.package_name, Some("node".to_string()));
        assert_eq!(result.version, Some("22.0.0".to_string()));
    }

    #[test]
    fn test_homebrew_linuxbrew_detection() {
        let detector = HomebrewDetector::new();
        let ctx = make_context(
            "gcc",
            vec!["/home/linuxbrew/.linuxbrew/Cellar/gcc/14.1.0/bin/gcc"],
            Platform::Linux,
        );
        let result = detector.detect(&ctx);
        assert!(result.is_some());
        assert_eq!(result.unwrap().package_name, Some("gcc".to_string()));
    }

    #[test]
    fn test_homebrew_keg_only_detection() {
        let detector = HomebrewDetector::new();
        // Keg-only formulas don't have Cellar in path
        let ctx = make_context(
            "openssl",
            vec!["/opt/homebrew/opt/openssl/bin/openssl"],
            Platform::MacOS,
        );
        let result = detector.detect(&ctx);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.confidence, Confidence::Medium);
        assert!(result.package_name.is_none()); // Can't extract from this path
    }

    #[test]
    fn test_homebrew_not_supported_on_windows() {
        let detector = HomebrewDetector::new();
        assert!(!detector.supports_platform(Platform::Windows));
        assert!(detector.supports_platform(Platform::MacOS));
        assert!(detector.supports_platform(Platform::Linux));
    }

    #[test]
    fn test_non_homebrew_path() {
        let detector = HomebrewDetector::new();
        let ctx = make_context("ls", vec!["/bin/ls"], Platform::MacOS);
        let result = detector.detect(&ctx);
        assert!(result.is_none());
    }
}
