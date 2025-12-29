//! Multi-pass Transformation Modules
//!
//! This module contains transformations that require multiple passes or
//! state across traversals. Unlike single-pass passes in layers, transforms
//! implement a **collect → apply** pattern.
//!
//! # Modules
//!
//! - [`constant_propagator`] — Collect constants, then inline references

// ============================================================================
// Submodules (TODO: implement in Phase 12)
// ============================================================================

// pub mod constant_propagator;

// ============================================================================
// Module Trait
// ============================================================================

use oxc::allocator::Allocator;
use oxc::ast::ast::Program;
use oxc::semantic::Scoping;

/// Result of module transformation.
#[derive(Debug, Default)]
pub struct ModuleResult {
    /// Number of modifications made.
    pub modifications: usize,
}

impl ModuleResult {
    /// Create result with no changes.
    pub fn unchanged() -> Self {
        Self { modifications: 0 }
    }

    /// Create result with changes.
    pub fn changed(modifications: usize) -> Self {
        Self { modifications }
    }
}

/// A multi-pass transformation module.
///
/// Modules implement a collect → apply pattern for transformations
/// that need to gather information before making changes.
pub trait Module: Send + Sync {
    /// Module name for logging.
    fn name(&self) -> &'static str;

    /// Description.
    fn description(&self) -> &'static str {
        ""
    }

    /// Does this module change symbol structure?
    fn changes_symbols(&self) -> bool {
        false
    }

    /// Transform the program.
    fn transform<'a>(
        &mut self,
        allocator: &'a Allocator,
        program: &mut Program<'a>,
        scoping: Scoping,
    ) -> ModuleResult;

    /// Reset state for new iteration.
    fn reset(&mut self) {}
}
