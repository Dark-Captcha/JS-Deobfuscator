//! High-level deobfuscation API.
//!
//! # Example
//!
//! ```ignore
//! use js_deobfuscator::{JSDeobfuscator, Extension};
//!
//! let output = JSDeobfuscator::new()
//!     .ecma(true)
//!     .runtime(true)
//!     .extension(Extension::StringRotator, true)
//!     .deobfuscate(source)?;
//! ```

use oxc::allocator::Allocator;
use oxc::codegen::Codegen;
use oxc::parser::Parser;
use oxc::span::SourceType;

use super::config::EngineConfig;
use super::engine::Engine;
use super::error::DeobError;

// ============================================================================
// Extension enum
// ============================================================================

/// Available extensions for obfuscator-specific patterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Extension {
    /// String array/rotator decoder (javascript-obfuscator style).
    StringRotator,
    /// Control flow deflattener.
    ControlFlow,
    /// Proxy function inliner.
    Proxy,
}

// ============================================================================
// DeobfuscateResult
// ============================================================================

/// Result of deobfuscation.
#[derive(Debug)]
pub struct DeobfuscateResult {
    /// Deobfuscated source code.
    pub code: String,
    /// Number of iterations executed.
    pub iterations: usize,
    /// Total modifications made.
    pub modifications: usize,
    /// Whether convergence was reached.
    pub converged: bool,
}

// ============================================================================
// JSDeobfuscator
// ============================================================================

/// High-level JavaScript deobfuscator.
///
/// # Example
///
/// ```ignore
/// use js_deobfuscator::{JSDeobfuscator, Extension};
///
/// // Standard (ECMA + Runtime)
/// let output = JSDeobfuscator::new()
///     .deobfuscate(source)?;
///
/// // Full with string rotator
/// let output = JSDeobfuscator::new()
///     .ecma(true)
///     .runtime(true)
///     .extension(Extension::StringRotator, true)
///     .deobfuscate(source)?;
///
/// // ECMA only
/// let output = JSDeobfuscator::new()
///     .ecma(true)
///     .runtime(false)
///     .deobfuscate(source)?;
/// ```
#[derive(Debug, Clone)]
pub struct JSDeobfuscator {
    config: EngineConfig,
}

impl Default for JSDeobfuscator {
    fn default() -> Self {
        Self::new()
    }
}

impl JSDeobfuscator {
    /// Create deobfuscator with standard config (ECMA + Runtime enabled).
    pub fn new() -> Self {
        Self {
            config: EngineConfig::standard(),
        }
    }

    /// Set max iterations.
    #[must_use]
    pub fn max_iterations(mut self, max: usize) -> Self {
        self.config.max_iterations = max;
        self
    }

    /// Enable/disable ECMA layer (constant folding, dead code, etc.).
    #[must_use]
    pub fn ecma(mut self, enabled: bool) -> Self {
        self.config.layers.ecma = enabled;
        self
    }

    /// Enable/disable Runtime layer (atob, btoa, escape, unescape).
    #[must_use]
    pub fn runtime(mut self, enabled: bool) -> Self {
        self.config.layers.runtime = enabled;
        self
    }

    /// Enable specific extensions.
    ///
    /// ```ignore
    /// .extensions([Extension::StringRotator])
    /// .extensions([Extension::StringRotator, Extension::ControlFlow])
    /// ```
    #[must_use]
    pub fn extensions<I>(mut self, exts: I) -> Self
    where
        I: IntoIterator<Item = Extension>,
    {
        // Reset all extensions first
        self.config.layers.extensions_config.string_array = false;
        self.config.layers.extensions_config.control_flow = false;
        self.config.layers.extensions_config.proxy = false;

        for ext in exts {
            match ext {
                Extension::StringRotator => {
                    self.config.layers.extensions_config.string_array = true;
                }
                Extension::ControlFlow => {
                    self.config.layers.extensions_config.control_flow = true;
                }
                Extension::Proxy => {
                    self.config.layers.extensions_config.proxy = true;
                }
            }
        }

        // Enable extensions layer if any extension is enabled
        self.config.layers.extensions = self.config.layers.extensions_config.string_array
            || self.config.layers.extensions_config.control_flow
            || self.config.layers.extensions_config.proxy;

        self
    }

    /// Deobfuscate JavaScript source code.
    pub fn deobfuscate(&self, source: &str) -> Result<String, DeobError> {
        Ok(self.deobfuscate_with_result(source)?.code)
    }

    /// Deobfuscate JavaScript source code with detailed result.
    pub fn deobfuscate_with_result(&self, source: &str) -> Result<DeobfuscateResult, DeobError> {
        let allocator = Allocator::default();
        let ret = Parser::new(&allocator, source, SourceType::mjs()).parse();

        if !ret.errors.is_empty() {
            return Err(DeobError::Parse(
                ret.errors
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join("; "),
            ));
        }

        let mut program = ret.program;
        let engine = Engine::with_config(self.config.clone());
        let result = engine.run(&allocator, &mut program)?;

        Ok(DeobfuscateResult {
            code: Codegen::new().build(&program).code,
            iterations: result.iterations,
            modifications: result.total_modifications,
            converged: result.converged,
        })
    }
}

// ============================================================================
// Convenience functions
// ============================================================================

/// Quick deobfuscation with standard config (ECMA + Runtime).
pub fn deobfuscate(source: &str) -> Result<String, DeobError> {
    JSDeobfuscator::new().deobfuscate(source)
}

/// Full deobfuscation with all extensions enabled.
pub fn deobfuscate_full(source: &str) -> Result<String, DeobError> {
    JSDeobfuscator::new()
        .extensions([
            Extension::StringRotator,
            Extension::ControlFlow,
            Extension::Proxy,
        ])
        .deobfuscate(source)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let result = deobfuscate("var a = 1 + 2;").unwrap();
        assert!(result.contains("3"));
    }

    #[test]
    fn test_builder() {
        let result = JSDeobfuscator::new()
            .ecma(true)
            .runtime(true)
            .max_iterations(10)
            .deobfuscate("var a = 1 + 2;")
            .unwrap();
        assert!(result.contains("3"));
    }

    #[test]
    fn test_extensions() {
        let result = JSDeobfuscator::new()
            .extensions([Extension::StringRotator])
            .deobfuscate("var a = 1 + 2;")
            .unwrap();
        assert!(result.contains("3"));
    }
}
