//! Layer 3: Obfuscator Pattern Extensions
//!
//! This layer contains transforms for **obfuscator-specific patterns**.
//! These are not JavaScript language features but patterns created by
//! obfuscation tools.
//!
//! **Important:** Extensions are opt-in. Users enable specific extensions
//! based on the obfuscator they're targeting.
//!
//! # Patterns
//!
//! - [`string_array`] — String array decoder (obfuscator.io style)
//! - [`control_flow`] — Control flow deflattener
//! - [`proxy`] — Proxy function inliner
//! - [`custom`] — User-defined patterns

use crate::core::LayerTransformer;

// ============================================================================
// Submodules
// ============================================================================

pub mod string_rotator;

// ============================================================================
// ExtensionsTransformer
// ============================================================================

/// Layer 3 transformer for obfuscator-specific patterns.
///
/// Handles patterns like string arrays, control flow flattening, etc.
pub struct ExtensionsTransformer {
    /// Total modifications made during traversal.
    modifications: usize,
}

impl ExtensionsTransformer {
    /// Create a new Extensions transformer.
    pub fn new() -> Self {
        Self { modifications: 0 }
    }

    /// Get the number of modifications made.
    #[inline]
    pub fn modifications(&self) -> usize {
        self.modifications
    }
}

impl Default for ExtensionsTransformer {
    fn default() -> Self {
        Self::new()
    }
}

impl LayerTransformer for ExtensionsTransformer {
    fn name(&self) -> &'static str {
        "extensions"
    }

    fn enabled(&self) -> bool {
        true
    }

    fn modifications(&self) -> usize {
        self.modifications
    }

    fn reset(&mut self) {
        self.modifications = 0;
    }
}

// TODO: Implement transform method in Phase 11
