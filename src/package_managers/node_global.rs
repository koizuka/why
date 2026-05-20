use super::{Confidence, DetectionContext, DetectionResult};

/// Skip names that are never actual package names — `node_modules/.bin/` is
/// the common bin symlink directory shared by every Node package manager.
pub(super) const SKIP_BIN: &[&str] = &[".bin"];
/// pnpm additionally stores hoisted package store metadata under
/// `node_modules/.pnpm/`, which is also not a real package.
pub(super) const SKIP_BIN_PNPM: &[&str] = &[".bin", ".pnpm"];

const PATTERNS: &[(&str, char)] = &[("/node_modules/", '/'), (r"\node_modules\", '\\')];

/// Extract a package name (incl. `@scope/pkg`) from a path containing a
/// `node_modules/<pkg>` (or `node_modules/@scope/<pkg>`) segment.
/// Tries both Unix and Windows separators.
pub(super) fn extract_node_modules_package_name(path: &str, skip_names: &[&str]) -> Option<String> {
    for (pattern, sep) in PATTERNS {
        let Some(idx) = path.find(pattern) else {
            continue;
        };
        let after = &path[idx + pattern.len()..];
        let mut parts = after.split(*sep);
        let first = parts.next()?;
        if first.is_empty() || skip_names.contains(&first) {
            continue;
        }
        if first.starts_with('@') {
            let second = parts.next()?;
            if !second.is_empty() {
                return Some(format!("{first}/{second}"));
            }
            continue;
        }
        return Some(first.to_string());
    }
    None
}

/// Shared `detect()` body for Node-ecosystem global package managers
/// (npm/bun/yarn/pnpm): match the chain against `matcher`, then walk the
/// chain to extract the real package name from a `node_modules/` segment,
/// falling back to the command name when no segment is found.
pub(super) fn detect_node_global<F>(
    ctx: &DetectionContext,
    manager_id: &str,
    manager_name: &str,
    matcher: F,
    skip_names: &[&str],
) -> Option<DetectionResult>
where
    F: Fn(&str) -> bool,
{
    let matched = ctx
        .symlink_chain
        .iter()
        .any(|p| matcher(&p.to_string_lossy()));
    if !matched {
        return None;
    }

    let package_name = ctx
        .symlink_chain
        .iter()
        .find_map(|p| extract_node_modules_package_name(&p.to_string_lossy(), skip_names))
        .or_else(|| Some(ctx.command_name.clone()));

    Some(DetectionResult {
        manager_id: manager_id.to_string(),
        manager_name: manager_name.to_string(),
        package_name,
        version: None,
        confidence: Confidence::Medium,
        command_path: ctx.command_path.clone(),
        resolved_path: ctx.resolved_path.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unix_unscoped() {
        assert_eq!(
            extract_node_modules_package_name(
                "/usr/local/lib/node_modules/typescript/bin/tsc",
                SKIP_BIN,
            ),
            Some("typescript".to_string())
        );
    }

    #[test]
    fn unix_scoped() {
        assert_eq!(
            extract_node_modules_package_name(
                "/home/u/.npm-global/lib/node_modules/@angular/cli/bin/ng",
                SKIP_BIN,
            ),
            Some("@angular/cli".to_string())
        );
    }

    #[test]
    fn windows_unscoped() {
        assert_eq!(
            extract_node_modules_package_name(
                r"C:\Users\u\AppData\Roaming\npm\node_modules\typescript\bin\tsc",
                SKIP_BIN,
            ),
            Some("typescript".to_string())
        );
    }

    #[test]
    fn windows_scoped() {
        assert_eq!(
            extract_node_modules_package_name(
                r"C:\Users\u\AppData\Roaming\npm\node_modules\@angular\cli\bin\ng",
                SKIP_BIN,
            ),
            Some("@angular/cli".to_string())
        );
    }

    #[test]
    fn skips_bin() {
        // Path ends right at node_modules/.bin — should not return ".bin".
        assert_eq!(
            extract_node_modules_package_name("/foo/node_modules/.bin/eslint", SKIP_BIN),
            None
        );
    }

    #[test]
    fn pnpm_skip_returns_none_when_first_segment_is_pnpm_store() {
        // The current implementation only inspects the first /node_modules/
        // segment. When a pnpm path lands directly at `node_modules/.pnpm/...`,
        // we return None and let the symlink-chain walker try other entries.
        assert_eq!(
            extract_node_modules_package_name(
                "/home/u/.local/share/pnpm/global/5/node_modules/.pnpm/typescript@5.0.0",
                SKIP_BIN_PNPM,
            ),
            None
        );
    }

    #[test]
    fn detect_falls_back_to_command_name_when_no_node_modules_in_chain() {
        use super::super::DetectionContext;
        use crate::platform::Platform;
        use std::path::PathBuf;

        let ctx = DetectionContext {
            command_name: "vite".to_string(),
            command_path: PathBuf::from("/Users/u/.bun/bin/vite"),
            symlink_chain: vec![PathBuf::from("/Users/u/.bun/bin/vite")],
            resolved_path: PathBuf::from("/Users/u/.bun/bin/vite"),
            platform: Platform::MacOS,
        };
        let result = detect_node_global(
            &ctx,
            "bun_global",
            "bun (global)",
            |p| p.contains("/.bun/bin/"),
            SKIP_BIN,
        )
        .expect("matcher should match");
        assert_eq!(result.package_name, Some("vite".to_string()));
    }

    #[test]
    fn no_node_modules_returns_none() {
        assert_eq!(
            extract_node_modules_package_name("/usr/local/bin/node", SKIP_BIN),
            None
        );
    }

    #[test]
    fn empty_after_node_modules() {
        assert_eq!(
            extract_node_modules_package_name("/foo/node_modules/", SKIP_BIN),
            None
        );
    }

    #[test]
    fn scoped_but_no_package_segment() {
        assert_eq!(
            extract_node_modules_package_name("/foo/node_modules/@scope/", SKIP_BIN),
            None
        );
    }
}
