//! Core deobfuscation infrastructure.
//!
//! This module provides the foundational components:
//!
//! - [`Engine`] — Convergence loop executor
//! - [`EngineConfig`] — Engine configuration
//! - [`LayerConfig`] — Layer enable/disable settings
//! - [`DeobError`] — Error types
//! - [`Pass`], [`PassGroup`] — Transformation traits
//!
//! # Architecture
//!
//! The core uses a **convergence loop** pattern:
//!
//! ```text
//! for each iteration:
//!     1. Rebuild scoping (fresh symbol/reference info)
//!     2. Run ECMA layer (ECMAScript standard)
//!     3. Run Runtime layer (browser/Node APIs)
//!     4. Run Extensions layer (obfuscator patterns)
//!     5. Run multi-pass transforms
//!     6. If no modifications: break (converged)
//! ```

// ============================================================================
// Modules
// ============================================================================

mod config;
mod deobfuscator;
mod engine;
mod error;
mod traits;

// ============================================================================
// Re-exports
// ============================================================================

pub use config::{
    EcmaConfig, EngineConfig, ExtensionsConfig, LayerConfig, LogLevel, RuntimeConfig,
};
pub use deobfuscator::{
    DeobfuscateResult, Extension, JSDeobfuscator, deobfuscate, deobfuscate_full,
};
pub use engine::{Engine, EngineResult};
pub use error::{DeobError, PassError};
pub use traits::{LayerTransformer, Pass, PassGroup};
