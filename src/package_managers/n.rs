use super::{Confidence, DetectionContext, DetectionResult, PackageManagerDetector};
use crate::platform::Platform;
use std::path::Path;

/// Detector for n (Node version manager) installed Node.js.
/// n is a Node.js version manager that installs to /usr/local or $N_PREFIX.
pub struct NDetector;

impl NDetector {
    pub fn new() -> Self {
        Self
    }
}

impl PackageManagerDetector for NDetector {
    fn id(&self) -> &'static str {
        "n"
    }

    fn name(&self) -> &str {
        "n (Node version manager)"
    }

    fn supports_platform(&self, platform: Platform) -> bool {
        // n only works on POSIX systems (macOS, Linux)
        matches!(platform, Platform::MacOS | Platform::Linux)
    }

    fn priority(&self) -> i32 {
        95 // Higher than mise (90) to detect n-managed node first
    }

    fn detect(&self, ctx: &DetectionContext) -> Option<DetectionResult> {
        // Only detect node, npm, npx, corepack commands
        // Extract base command name from path if full path is provided
        let command_base = std::path::Path::new(&ctx.command_name)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(&ctx.command_name);

        let target_commands = ["node", "npm", "npx", "corepack"];
        if !target_commands.contains(&command_base) {
            return None;
        }

        // Try to get PREFIX from command_path first (works for symlinks like npm/npx),
        // then fall back to resolved_path (works for direct binaries like node)
        // Example: /usr/local/bin/npm -> /usr/local
        let prefix = get_n_prefix_from_bin_path(&ctx.command_path)
            .or_else(|| get_n_prefix_from_bin_path(&ctx.resolved_path))?;

        // Check if {PREFIX}/n/versions/node/ directory exists
        let versions_dir = prefix.join("n/versions/node");
        if !versions_dir.exists() || !versions_dir.is_dir() {
            return None;
        }

        // Get version from directory name
        let version = get_installed_version(&versions_dir);

        Some(DetectionResult {
            manager_id: self.id().to_string(),
            manager_name: self.name().to_string(),
            package_name: Some(command_base.to_string()),
            version,
            confidence: Confidence::Medium,
            command_path: ctx.command_path.clone(),
            resolved_path: ctx.resolved_path.clone(),
        })
    }
}

fn get_n_prefix_from_bin_path(path: &Path) -> Option<std::path::PathBuf> {
    // /usr/local/bin/node -> parent(/usr/local/bin) -> parent(/usr/local)
    // Only match if parent directory is "bin"
    let parent = path.parent()?;
    if parent.file_name()?.to_str()? != "bin" {
        return None;
    }
    parent.parent().map(|p| p.to_path_buf())
}

fn get_installed_version(versions_dir: &Path) -> Option<String> {
    std::fs::read_dir(versions_dir)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .next() // Return first version (typically only one active)
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
    fn test_supports_macos_and_linux() {
        let detector = NDetector::new();
        assert!(detector.supports_platform(Platform::MacOS));
        assert!(detector.supports_platform(Platform::Linux));
        assert!(!detector.supports_platform(Platform::Windows));
    }

    #[test]
    fn test_only_detects_node_commands() {
        let detector = NDetector::new();

        // Should not detect non-node commands
        let ctx = make_context("git", vec!["/usr/local/bin/git"], Platform::MacOS);
        assert!(detector.detect(&ctx).is_none());

        let ctx = make_context("python", vec!["/usr/local/bin/python"], Platform::MacOS);
        assert!(detector.detect(&ctx).is_none());
    }

    #[test]
    fn test_get_n_prefix_from_bin_path() {
        let path = Path::new("/usr/local/bin/node");
        let prefix = get_n_prefix_from_bin_path(path);
        assert_eq!(prefix, Some(PathBuf::from("/usr/local")));

        let path = Path::new("/home/user/.n/bin/node");
        let prefix = get_n_prefix_from_bin_path(path);
        assert_eq!(prefix, Some(PathBuf::from("/home/user/.n")));

        // Should return None if parent is not "bin"
        let path = Path::new("/usr/local/lib/node_modules/npm/package.json");
        let prefix = get_n_prefix_from_bin_path(path);
        assert_eq!(prefix, None);

        // This returns Some, but detect() will filter it out via n/versions/node check
        let path = Path::new("/usr/local/lib/node_modules/npm/bin/npm-cli.js");
        let prefix = get_n_prefix_from_bin_path(path);
        assert_eq!(
            prefix,
            Some(PathBuf::from("/usr/local/lib/node_modules/npm"))
        );
    }

    #[test]
    fn test_priority() {
        let detector = NDetector::new();
        assert_eq!(detector.priority(), 95);
    }

    #[test]
    fn test_id_and_name() {
        let detector = NDetector::new();
        assert_eq!(detector.id(), "n");
        assert_eq!(detector.name(), "n (Node version manager)");
    }
}
