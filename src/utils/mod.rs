//! Shared Utilities
//!
//! This module provides shared utilities used across all layers.
//!
//! # Literal Operations
//!
//! Use the [`literal`] module for all literal value operations:
//!
//! ```ignore
//! use crate::utils::literal;
//!
//! // Extract values
//! let n = literal::number(&expr)?;
//! let s = literal::string(&expr)?;
//! let b = literal::boolean(&expr)?;
//!
//! // Check truthiness
//! if literal::is_truthy(&expr) == Some(true) { ... }
//!
//! // Create expressions
//! *expr = literal::make_number(42.0, &ctx.ast);
//! *expr = literal::make_string("hello", &ctx.ast);
//! ```
//!
//! # Scoping
//!
//! Use [`scoping`] for symbol resolution and reference tracking.
//!
//! # Side Effects
//!
//! Use [`side_effects`] for side effect analysis.

// ============================================================================
// Modules
// ============================================================================

pub mod codegen;
pub mod literal;
pub mod sandbox;
pub mod scoping;
pub mod side_effects;
pub mod testing;

// ============================================================================
// Re-exports
// ============================================================================

pub use codegen::{call_to_code, expr_to_code, stmt_to_code};
pub use literal::LiteralValue;
pub use sandbox::JsEvaluator;
pub use scoping::{get_binding_symbol, get_reference_symbol, has_reads, has_writes};
pub use side_effects::is_side_effect_free;
