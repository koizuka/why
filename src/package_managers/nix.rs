use super::{Confidence, DetectionContext, DetectionResult, PackageManagerDetector};
use crate::platform::Platform;

/// Detector for Nix packages.
pub struct NixDetector;

impl NixDetector {
    pub fn new() -> Self {
        Self
    }
}

impl PackageManagerDetector for NixDetector {
    fn id(&self) -> &'static str {
        "nix"
    }

    fn name(&self) -> &str {
        "Nix"
    }

    fn supports_platform(&self, platform: Platform) -> bool {
        // Nix is available on Linux and macOS
        platform == Platform::Linux || platform == Platform::MacOS
    }

    fn priority(&self) -> i32 {
        85
    }

    fn detect(&self, ctx: &DetectionContext) -> Option<DetectionResult> {
        let mut matched = false;

        for path in &ctx.symlink_chain {
            let path_str = path.to_string_lossy();

            // Check for Nix paths:
            // /nix/store/{hash}-{package}-{version}/bin/{command}
            // ~/.nix-profile/bin/{command}
            // /nix/var/nix/profiles/default/bin/{command}
            // /run/current-system/sw/bin/{command} (NixOS)
            // /etc/profiles/per-user/{user}/bin/{command}
            if path_str.contains("/nix/store/")
                || path_str.contains("/.nix-profile/bin/")
                || path_str.ends_with("/.nix-profile/bin")
                || path_str.contains("/nix/var/nix/profiles/")
                || path_str.contains("/current-system/sw/bin/")
                || path_str.contains("/profiles/per-user/")
            {
                matched = true;
            }
        }

        if matched {
            // Try to extract package name and version from any path in the chain
            let package_name = ctx
                .symlink_chain
                .iter()
                .filter_map(|p| extract_nix_package_name(&p.to_string_lossy()))
                .next()
                .or_else(|| Some(ctx.command_name.clone()));

            let version = ctx
                .symlink_chain
                .iter()
                .filter_map(|p| extract_nix_version(&p.to_string_lossy()))
                .next();

            return Some(DetectionResult {
                manager_id: self.id().to_string(),
                manager_name: self.name().to_string(),
                package_name,
                version,
                confidence: Confidence::Medium,
                command_path: ctx.command_path.clone(),
                resolved_path: ctx.resolved_path.clone(),
            });
        }

        None
    }
}

fn extract_nix_package_name(path: &str) -> Option<String> {
    // Pattern: /nix/store/{hash}-{package}-{version}/...
    // Example: /nix/store/abc123-hello-2.10/bin/hello
    if let Some(store_idx) = path.find("/nix/store/") {
        let after_store = &path[store_idx + 11..]; // skip "/nix/store/"
        if let Some(slash_idx) = after_store.find('/') {
            let store_path = &after_store[..slash_idx];
            // Format: {hash}-{name}-{version} or {hash}-{name}
            // Find first dash after the hash (hash is 32 chars)
            if store_path.len() > 33 && store_path.chars().nth(32) == Some('-') {
                let name_version = &store_path[33..];
                // Try to extract just the name (before last dash if it looks like a version)
                if let Some(last_dash) = name_version.rfind('-') {
                    let potential_version = &name_version[last_dash + 1..];
                    // If it starts with a digit, it's likely a version
                    if potential_version
                        .chars()
                        .next()
                        .is_some_and(|c| c.is_ascii_digit())
                    {
                        return Some(name_version[..last_dash].to_string());
                    }
                }
                return Some(name_version.to_string());
            }
        }
    }
    None
}

fn extract_nix_version(path: &str) -> Option<String> {
    // Pattern: /nix/store/{hash}-{package}-{version}/...
    if let Some(store_idx) = path.find("/nix/store/") {
        let after_store = &path[store_idx + 11..];
        if let Some(slash_idx) = after_store.find('/') {
            let store_path = &after_store[..slash_idx];
            if store_path.len() > 33 && store_path.chars().nth(32) == Some('-') {
                let name_version = &store_path[33..];
                if let Some(last_dash) = name_version.rfind('-') {
                    let potential_version = &name_version[last_dash + 1..];
                    if potential_version
                        .chars()
                        .next()
                        .is_some_and(|c| c.is_ascii_digit())
                    {
                        return Some(potential_version.to_string());
                    }
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
    fn test_detects_nix_store() {
        let detector = NixDetector::new();
        let ctx = make_context(
            "hello",
            vec![
                "/home/user/.nix-profile/bin/hello",
                "/nix/store/abcdefghijklmnopqrstuvwxyz123456-hello-2.10/bin/hello",
            ],
            Platform::Linux,
        );
        let result = detector.detect(&ctx);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.manager_id, "nix");
        assert_eq!(result.package_name, Some("hello".to_string()));
        assert_eq!(result.version, Some("2.10".to_string()));
    }

    #[test]
    fn test_detects_nix_profile() {
        let detector = NixDetector::new();
        let ctx = make_context(
            "ripgrep",
            vec!["/home/user/.nix-profile/bin/rg"],
            Platform::Linux,
        );
        let result = detector.detect(&ctx);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.manager_id, "nix");
    }

    #[test]
    fn test_detects_nix_macos() {
        let detector = NixDetector::new();
        let ctx = make_context(
            "nix",
            vec!["/Users/user/.nix-profile/bin/nix"],
            Platform::MacOS,
        );
        let result = detector.detect(&ctx);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.manager_id, "nix");
    }

    #[test]
    fn test_detects_nixos_system() {
        let detector = NixDetector::new();
        let ctx = make_context(
            "bash",
            vec!["/run/current-system/sw/bin/bash"],
            Platform::Linux,
        );
        let result = detector.detect(&ctx);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.manager_id, "nix");
    }

    #[test]
    fn test_detects_nix_default_profile() {
        let detector = NixDetector::new();
        let ctx = make_context(
            "nix",
            vec!["/nix/var/nix/profiles/default/bin/nix"],
            Platform::MacOS,
        );
        let result = detector.detect(&ctx);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.manager_id, "nix");
    }

    #[test]
    fn test_ignores_non_nix_paths() {
        let detector = NixDetector::new();
        let ctx = make_context("git", vec!["/usr/bin/git"], Platform::Linux);
        let result = detector.detect(&ctx);
        assert!(result.is_none());
    }

    #[test]
    fn test_supports_linux_and_macos() {
        let detector = NixDetector::new();
        assert!(detector.supports_platform(Platform::Linux));
        assert!(detector.supports_platform(Platform::MacOS));
        assert!(!detector.supports_platform(Platform::Windows));
    }

    #[test]
    fn test_extract_nix_package_name() {
        assert_eq!(
            extract_nix_package_name(
                "/nix/store/abcdefghijklmnopqrstuvwxyz123456-hello-2.10/bin/hello"
            ),
            Some("hello".to_string())
        );
        assert_eq!(
            extract_nix_package_name(
                "/nix/store/abcdefghijklmnopqrstuvwxyz123456-ripgrep-14.1.0/bin/rg"
            ),
            Some("ripgrep".to_string())
        );
    }

    #[test]
    fn test_extract_nix_version() {
        assert_eq!(
            extract_nix_version("/nix/store/abcdefghijklmnopqrstuvwxyz123456-hello-2.10/bin/hello"),
            Some("2.10".to_string())
        );
        assert_eq!(
            extract_nix_version(
                "/nix/store/abcdefghijklmnopqrstuvwxyz123456-ripgrep-14.1.0/bin/rg"
            ),
            Some("14.1.0".to_string())
        );
    }
}
