pub mod path_resolver;
pub mod symlink_analyzer;

use crate::error::Result;
use crate::package_managers::{DetectionContext, DetectionResult, PackageManagerRegistry};
use crate::platform::Platform;

/// Main detection orchestrator
pub struct Detector {
    registry: PackageManagerRegistry,
    verbose: bool,
}

impl Detector {
    pub fn new(verbose: bool) -> Self {
        Self {
            registry: PackageManagerRegistry::new(),
            verbose,
        }
    }

    /// Detect which package manager installed the given command.
    pub fn detect(&self, command: &str) -> Result<DetectionResult> {
        // Step 1: Resolve command to path
        if self.verbose {
            eprintln!("Resolving path for '{}'...", command);
        }
        let command_path = path_resolver::resolve_command(command)?;
        if self.verbose {
            eprintln!("Found at {}", command_path.display());
        }

        // Step 2: Follow symlinks
        let symlink_chain = symlink_analyzer::follow_symlinks(command_path.clone());
        let resolved_path = symlink_chain
            .last()
            .cloned()
            .unwrap_or(command_path.clone());

        if self.verbose && symlink_chain.len() > 1 {
            eprintln!("Following symlink to {}", resolved_path.display());
        }

        // Step 3: Create detection context
        let context = DetectionContext {
            command_name: command.to_string(),
            command_path: command_path.clone(),
            symlink_chain,
            resolved_path: resolved_path.clone(),
            platform: Platform::current(),
        };

        // Step 4: Try each package manager detector
        if let Some(result) = self.registry.detect(&context, self.verbose) {
            return Ok(result);
        }

        // Step 5: Return unknown if no detector matched
        Ok(DetectionResult {
            manager_id: "unknown".to_string(),
            manager_name: "Unknown".to_string(),
            package_name: None,
            version: None,
            confidence: crate::package_managers::Confidence::Uncertain,
            command_path,
            resolved_path,
        })
    }
}

/// Convenience function
pub fn detect_command(command: &str, verbose: bool) -> Result<DetectionResult> {
    Detector::new(verbose).detect(command)
}
