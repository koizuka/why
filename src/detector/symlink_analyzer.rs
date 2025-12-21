use std::fs;
use std::path::{Path, PathBuf};

/// Follow the symlink chain from a path and return all paths in the chain.
/// The first element is the original path, the last is the final target.
pub fn follow_symlinks(path: PathBuf) -> Vec<PathBuf> {
    let mut chain = vec![path.clone()];
    let mut current = path;
    let mut seen = std::collections::HashSet::new();

    // Prevent infinite loops
    while seen.insert(current.clone()) {
        match fs::read_link(&current) {
            Ok(target) => {
                let resolved = if target.is_absolute() {
                    target
                } else {
                    // Resolve relative to parent directory
                    current.parent().map(|p| p.join(&target)).unwrap_or(target)
                };

                // Canonicalize to resolve any .. or . in the path
                let resolved = resolved.canonicalize().unwrap_or(resolved);
                chain.push(resolved.clone());
                current = resolved;
            }
            Err(_) => break, // Not a symlink or can't read
        }
    }

    chain
}

/// Get the final resolved path after following all symlinks.
pub fn resolve_final_path(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    #[cfg(unix)]
    fn test_non_symlink_path() {
        let path = PathBuf::from("/bin/ls");
        if path.exists() {
            let chain = follow_symlinks(path.clone());
            assert!(!chain.is_empty());
            assert_eq!(chain[0], path);
        }
    }

    #[test]
    fn test_nonexistent_path() {
        let path = PathBuf::from("/nonexistent/path/to/file");
        let chain = follow_symlinks(path.clone());
        assert_eq!(chain.len(), 1);
        assert_eq!(chain[0], path);
    }

    #[cfg(unix)]
    #[test]
    fn test_single_symlink() {
        use std::os::unix::fs::symlink;

        let dir = tempdir().unwrap();
        let original = dir.path().join("original");
        let link = dir.path().join("link");

        std::fs::write(&original, "content").unwrap();
        symlink(&original, &link).unwrap();

        let chain = follow_symlinks(link.clone());
        assert_eq!(chain.len(), 2);
        assert_eq!(chain[0], link);
        // The second element should resolve to the original file
    }

    #[cfg(unix)]
    #[test]
    fn test_symlink_chain() {
        use std::os::unix::fs::symlink;

        let dir = tempdir().unwrap();
        let original = dir.path().join("original");
        let link1 = dir.path().join("link1");
        let link2 = dir.path().join("link2");

        std::fs::write(&original, "content").unwrap();
        symlink(&original, &link1).unwrap();
        symlink(&link1, &link2).unwrap();

        let chain = follow_symlinks(link2.clone());
        assert!(chain.len() >= 2); // At least link2 and final target
        assert_eq!(chain[0], link2);
    }

    #[test]
    fn test_resolve_final_path_regular_file() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("file.txt");
        std::fs::write(&file, "content").unwrap();

        let resolved = resolve_final_path(&file);
        assert!(resolved.exists());
    }

    #[test]
    fn test_resolve_final_path_nonexistent() {
        let path = PathBuf::from("/nonexistent/path");
        let resolved = resolve_final_path(&path);
        assert_eq!(resolved, path);
    }

    #[test]
    fn test_empty_path() {
        let path = PathBuf::from("");
        let chain = follow_symlinks(path.clone());
        assert_eq!(chain.len(), 1);
    }
}
