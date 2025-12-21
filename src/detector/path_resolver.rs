use crate::error::{Result, WhyError};
use std::path::PathBuf;

/// Resolve a command name to its absolute path using the system PATH.
pub fn resolve_command(name: &str) -> Result<PathBuf> {
    which::which(name).map_err(|_| WhyError::CommandNotFound(name.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_existing_command() {
        // Test with a command that definitely exists on all platforms
        let result = resolve_command("ls");
        // On Windows, this might not exist, so we just check it doesn't panic
        #[cfg(unix)]
        assert!(result.is_ok());
    }

    #[test]
    fn test_resolve_nonexistent_command() {
        let result = resolve_command("definitely_not_a_real_command_xyz_123");
        assert!(result.is_err());
    }
}
