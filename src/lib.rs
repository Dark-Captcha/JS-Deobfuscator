//! Universal JavaScript Deobfuscator
//!
//! A high-accuracy JavaScript deobfuscator built on OXC with a 3-layer architecture.
//!
//! # Architecture
//!
//! The deobfuscator uses a **3-layer architecture**:
//!
//! - **Layer 1: ECMA** — ECMAScript standard transforms (works in ANY JS runtime)
//! - **Layer 2: Runtime** — Browser/Node runtime APIs (atob, btoa, escape, etc.)
//! - **Layer 3: Extensions** — Obfuscator-specific patterns (string arrays, control flow)
//!
//! # Quick Start
//!
//! ## Simple API (Recommended)
//!
//! ```ignore
//! use js_deobfuscator::{JSDeobfuscator, Extension};
//!
//! // Standard deobfuscation (ECMA + Runtime)
//! let output = JSDeobfuscator::new()
//!     .deobfuscate("var a = 1 + 2;")?;
//!
//! // With string rotator extension
//! let output = JSDeobfuscator::new()
//!     .ecma(true)
//!     .runtime(true)
//!     .extensions([Extension::StringRotator])
//!     .deobfuscate(&source)?;
//! ```
//!
//! ## One-liner Functions
//!
//! ```ignore
//! use js_deobfuscator::{deobfuscate, deobfuscate_full};
//!
//! let output = deobfuscate("var a = 1 + 2;")?;  // ECMA + Runtime
//! let output = deobfuscate_full(&source)?;       // All extensions
//! ```
//!
//! ## With Result Details
//!
//! ```ignore
//! use js_deobfuscator::{JSDeobfuscator, Extension};
//!
//! let result = JSDeobfuscator::new()
//!     .extensions([Extension::StringRotator])
//!     .deobfuscate_with_result(&source)?;
//!
//! println!("Iterations: {}", result.iterations);
//! println!("Modifications: {}", result.modifications);
//! println!("{}", result.code);
//! ```
//!
//! # Modules
//!
//! - [`core`] — Engine, configuration, high-level API
//! - [`ecma`] — Layer 1: ECMAScript standard passes
//! - [`runtime`] — Layer 2: Runtime API passes
//! - [`extensions`] — Layer 3: Obfuscator pattern handlers
//! - [`transforms`] — Multi-pass transformation modules
//! - [`utils`] — Shared utilities

// ============================================================================
// Modules
// ============================================================================

pub mod core;
pub mod ecma;
pub mod extensions;
pub mod runtime;
pub mod transforms;
pub mod utils;

// ============================================================================
// Re-exports
// ============================================================================

pub use core::{
    deobfuscate, deobfuscate_full, DeobError, DeobfuscateResult, EcmaConfig, Engine, EngineConfig,
    EngineResult, Extension, ExtensionsConfig, JSDeobfuscator, LayerConfig, LogLevel, PassError,
    RuntimeConfig,
};
