//! Layer 2: Runtime API Transforms
//!
//! This layer contains transforms for runtime APIs that are **not** part of
//! the ECMAScript standard but are commonly available in browsers and/or Node.js.
//!
//! **Important:** These may not exist in all environments. Users can disable
//! this layer for pure ECMAScript compliance.
//!
//! # Groups
//!
//! - [`encoding`] — Base64 encoding/decoding (atob, btoa)
//! - [`deprecated`] — Deprecated APIs (escape, unescape)

use oxc::ast::ast::Expression;
use oxc_traverse::{Traverse, TraverseCtx};

use crate::core::{LayerTransformer, RuntimeConfig};

// ============================================================================
// Submodules
// ============================================================================

pub mod deprecated;
pub mod encoding;

// ============================================================================
// Re-exports
// ============================================================================

pub use deprecated::DeprecatedGroup;
pub use encoding::EncodingGroup;

// ============================================================================
// RuntimeTransformer
// ============================================================================

/// Layer 2 transformer for runtime APIs.
///
/// Handles browser/Node.js APIs not in ECMAScript standard.
pub struct RuntimeTransformer {
    /// Configuration.
    config: RuntimeConfig,

    /// Total modifications made during traversal.
    modifications: usize,

    // Pass groups
    encoding: EncodingGroup,
    deprecated: DeprecatedGroup,
}

impl RuntimeTransformer {
    /// Create a new Runtime transformer with the given configuration.
    pub fn new(config: &RuntimeConfig) -> Self {
        Self {
            config: config.clone(),
            modifications: 0,
            encoding: EncodingGroup::new(),
            deprecated: DeprecatedGroup::new(),
        }
    }

    /// Get the number of modifications made.
    #[inline]
    pub fn modifications(&self) -> usize {
        self.modifications
    }
}

impl Default for RuntimeTransformer {
    fn default() -> Self {
        Self::new(&RuntimeConfig::default())
    }
}

impl LayerTransformer for RuntimeTransformer {
    fn name(&self) -> &'static str {
        "runtime"
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

impl<'a> Traverse<'a, ()> for RuntimeTransformer {
    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a, ()>) {
        // Run all expression-level pass groups
        if self.config.encoding {
            self.modifications += self.encoding.transform(expr, ctx);
        }
        if self.config.deprecated {
            self.modifications += self.deprecated.transform(expr, ctx);
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::testing::{codegen, parse_program};
    use oxc::allocator::Allocator;
    use oxc::semantic::SemanticBuilder;
    use oxc_traverse::traverse_mut;

    fn transform(source: &str) -> String {
        let allocator = Allocator::default();
        let mut program = parse_program(&allocator, source);
        let scoping = SemanticBuilder::new()
            .build(&program)
            .semantic
            .into_scoping();

        let mut transformer = RuntimeTransformer::new(&RuntimeConfig::all());
        traverse_mut(&mut transformer, &allocator, &mut program, scoping, ());
        codegen(&program)
    }

    #[test]
    fn test_atob() {
        // Use semicolon prefix to avoid directive interpretation
        assert_eq!(transform(";atob(\"aGVsbG8=\")"), ";\n\"hello\";\n");
    }

    #[test]
    fn test_btoa() {
        assert_eq!(transform(";btoa(\"hello\")"), ";\n\"aGVsbG8=\";\n");
    }

    #[test]
    fn test_escape() {
        assert_eq!(transform(";escape(\" \")"), ";\n\"%20\";\n");
    }

    #[test]
    fn test_unescape() {
        assert_eq!(transform(";unescape(\"%20\")"), ";\n\" \";\n");
    }

    #[test]
    fn test_combined() {
        // Test that multiple passes work together
        let result = transform("atob(\"dGVzdA==\") + unescape(\"%20\")");
        assert_eq!(result, "\"test\" + \" \";\n");
    }

    #[test]
    fn test_config_disabled() {
        let allocator = Allocator::default();
        let mut program = parse_program(&allocator, "atob(\"aGVsbG8=\")");
        let scoping = SemanticBuilder::new()
            .build(&program)
            .semantic
            .into_scoping();

        // Disable encoding
        let config = RuntimeConfig {
            encoding: false,
            deprecated: true,
        };
        let mut transformer = RuntimeTransformer::new(&config);
        traverse_mut(&mut transformer, &allocator, &mut program, scoping, ());

        // Should remain unchanged
        assert_eq!(codegen(&program), "atob(\"aGVsbG8=\");\n");
    }
}
