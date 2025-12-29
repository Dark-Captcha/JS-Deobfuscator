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
//! ```ignore
//! use js_deobfuscator::{Engine, EngineConfig};
//! use oxc::allocator::Allocator;
//! use oxc::codegen::Codegen;
//! use oxc::parser::Parser;
//! use oxc::span::SourceType;
//!
//! let source = r#"var a = 1 + 2; var b = "hello" + " world";"#;
//!
//! let allocator = Allocator::default();
//! let mut program = Parser::new(&allocator, source, SourceType::mjs()).parse().program;
//!
//! let engine = Engine::with_config(EngineConfig::default().with_max_iterations(50));
//! let result = engine.run(&allocator, &mut program).unwrap();
//!
//! let output = Codegen::new().build(&program).code;
//! // var a = 3;
//! // var b = "hello world";
//! ```
//!
//! # Modules
//!
//! - [`core`] — Engine, configuration, traits
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
