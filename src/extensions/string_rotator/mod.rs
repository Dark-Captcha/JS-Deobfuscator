//! String Rotator Decoder
//!
//! Decodes obfuscated string arrays with rotation (obfuscator.io style).
//!
//! # Quick Start
//!
//! ```ignore
//! use js_deobfuscator::extensions::string_rotator::StringRotator;
//!
//! let source = std::fs::read_to_string("obfuscated.js")?;
//! let output = StringRotator::deobfuscate(&source)?;
//! println!("{}", output);
//! ```
//!
//! # With Program (for pipeline integration)
//!
//! ```ignore
//! use js_deobfuscator::extensions::string_rotator::StringRotator;
//! use oxc::allocator::Allocator;
//! use oxc::parser::Parser;
//! use oxc::span::SourceType;
//!
//! let allocator = Allocator::default();
//! let mut program = Parser::new(&allocator, &source, SourceType::mjs()).parse().program;
//!
//! let result = StringRotator::transform(&allocator, &mut program)?;
//! println!("Decoded {} strings, removed {} functions", result.strings_decoded, result.functions_removed);
//! ```

mod call_collector;
mod component_collector;
mod executor;
mod inliner;
mod shuffler_detector;

pub use call_collector::*;
pub use component_collector::*;
pub use executor::*;
pub use inliner::*;
pub use shuffler_detector::*;

use oxc::allocator::Allocator;
use oxc::ast::ast::Program;
use oxc::codegen::Codegen;
use oxc::parser::Parser;
use oxc::semantic::SemanticBuilder;
use oxc::span::SourceType;
use oxc_traverse::traverse_mut;

// ============================================================================
// High-Level API
// ============================================================================

/// Result of string rotator transformation.
#[derive(Debug, Default)]
pub struct StringRotatorResult {
    /// Number of string rotator systems detected.
    pub systems_detected: usize,
    /// Number of strings successfully decoded.
    pub strings_decoded: usize,
    /// Number of calls inlined.
    pub calls_inlined: usize,
    /// Number of functions removed (array + accessor).
    pub functions_removed: usize,
}

/// High-level string rotator decoder.
///
/// Handles the full pipeline: detect → collect → decode → inline → cleanup.
pub struct StringRotator;

impl StringRotator {
    /// Deobfuscate source code and return cleaned JavaScript.
    ///
    /// This is the simplest API - just pass source, get result.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let output = StringRotator::deobfuscate(&source)?;
    /// ```
    pub fn deobfuscate(source: &str) -> Result<String, StringRotatorError> {
        let allocator = Allocator::default();
        let mut program = Parser::new(&allocator, source, SourceType::mjs())
            .parse()
            .program;

        Self::transform(&allocator, &mut program)?;

        Ok(Codegen::new().build(&program).code)
    }

    /// Transform a program in-place.
    ///
    /// Use this when integrating into a larger pipeline.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let result = StringRotator::transform(&allocator, &mut program)?;
    /// ```
    pub fn transform<'a>(
        allocator: &'a Allocator,
        program: &mut Program<'a>,
    ) -> Result<StringRotatorResult, StringRotatorError> {
        let mut result = StringRotatorResult::default();

        // Phase 1: Detect shuffler IIFEs (also removes them)
        let scoping = SemanticBuilder::new()
            .build(program)
            .semantic
            .into_scoping();
        let mut detector = ShufflerDetector::default();
        let scoping = traverse_mut(&mut detector, allocator, program, scoping, ());

        if detector.results.is_empty() {
            return Ok(result);
        }

        result.systems_detected = detector.results.len();

        // Collect all target symbols
        let mut all_targets = Vec::new();
        for info in detector.results.values() {
            all_targets.push(info.array_symbol_id);
            all_targets.push(info.accessor_symbol_id);
        }

        // Phase 2: Collect function code
        let mut collector = ComponentCollector::new(all_targets.clone());
        let scoping = traverse_mut(&mut collector, allocator, program, scoping, ());

        // Phase 3-5: For each system
        let mut scoping = scoping;
        for info in detector.results.values() {
            // Phase 3: Collect calls
            let mut call_collector = CallCollector::new(info.accessor_symbol_id);
            scoping = traverse_mut(&mut call_collector, allocator, program, scoping, ());

            let calls: Vec<String> = call_collector.calls.into_iter().collect();

            // Phase 4: Execute in V8 sandbox
            let array_code = collector
                .results
                .get(&info.array_symbol_id)
                .map(|f| f.code.as_str())
                .unwrap_or("");
            let accessor_code = collector
                .results
                .get(&info.accessor_symbol_id)
                .map(|f| f.code.as_str())
                .unwrap_or("");

            match execute_decoder(array_code, accessor_code, &info.code, &calls) {
                Ok(decoded) => {
                    result.strings_decoded += decoded.len();

                    // Phase 5a: Inline strings
                    let mut inliner = StringInliner::new(info.accessor_symbol_id, decoded);
                    scoping = traverse_mut(&mut inliner, allocator, program, scoping, ());
                    result.calls_inlined += inliner.replaced_count;
                }
                Err(e) => {
                    // Log but continue with other systems
                    tracing::warn!("Failed to decode system: {}", e);
                }
            }
        }

        // Phase 5b: Remove array/accessor functions
        let mut remover = ComponentRemover::new(all_targets);
        traverse_mut(&mut remover, allocator, program, scoping, ());
        result.functions_removed = remover.removed_count;

        Ok(result)
    }
}

// ============================================================================
// Error Type
// ============================================================================

/// Errors from string rotator operations.
#[derive(Debug, thiserror::Error)]
pub enum StringRotatorError {
    #[error("V8 execution error: {0}")]
    Execution(String),

    #[error("Parse error")]
    Parse,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_shuffler() {
        let code = "console.log('hello');";
        let result = StringRotator::deobfuscate(code).unwrap();
        assert!(result.contains("hello"));
    }

    // Note: Full integration tests with V8 are in examples/test_string_rotator.rs
    // because they require real obfuscated code with correct checksums.
}
