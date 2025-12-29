//! Deobfuscation engine with convergence loop.

use std::time::{Duration, Instant};

use oxc::allocator::Allocator;
use oxc::ast::ast::Program;
use oxc::semantic::SemanticBuilder;
use oxc_traverse::traverse_mut;

use super::config::EngineConfig;
use super::error::{DeobError, PassError};
use crate::ecma::EcmaTransformer;
use crate::extensions::string_rotator::StringRotator;
use crate::runtime::RuntimeTransformer;

// ============================================================================
// EngineResult
// ============================================================================

/// Result of running the engine.
#[derive(Debug)]
pub struct EngineResult {
    /// Number of iterations executed.
    pub iterations: usize,

    /// Total modifications across all iterations.
    pub total_modifications: usize,

    /// Modifications per iteration.
    pub modifications_per_iteration: Vec<usize>,

    /// Whether convergence was reached.
    pub converged: bool,

    /// Time elapsed.
    pub elapsed: Duration,

    /// Errors encountered (non-fatal).
    pub errors: Vec<PassError>,
}

impl EngineResult {
    /// Check if successful (converged).
    pub fn is_success(&self) -> bool {
        self.converged
    }
}

// ============================================================================
// Engine
// ============================================================================

/// The deobfuscation engine.
///
/// Runs layers in a convergence loop until no modifications.
///
/// # Layers
///
/// 1. **Extensions** (string rotator, etc.) - runs first to decode strings
/// 2. **ECMA** (operators, string methods, etc.) - constant folding
/// 3. **Runtime** (atob, btoa, escape, unescape) - runtime API evaluation
///
/// # Example
///
/// ```ignore
/// use js_deobfuscator::{Engine, EngineConfig};
/// use oxc::allocator::Allocator;
/// use oxc::codegen::Codegen;
/// use oxc::parser::Parser;
/// use oxc::span::SourceType;
///
/// let allocator = Allocator::default();
/// let mut program = Parser::new(&allocator, source, SourceType::mjs()).parse().program;
///
/// let engine = Engine::with_config(EngineConfig::full());
/// let result = engine.run(&allocator, &mut program)?;
///
/// let output = Codegen::new().build(&program).code;
/// ```
#[derive(Default)]
pub struct Engine {
    config: EngineConfig,
}

impl Engine {
    /// Create engine with default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create engine with custom configuration.
    pub fn with_config(config: EngineConfig) -> Self {
        Self { config }
    }

    /// Get the configuration.
    pub fn config(&self) -> &EngineConfig {
        &self.config
    }

    /// Run deobfuscation on program.
    #[tracing::instrument(
        skip(self, allocator, program),
        fields(max_iterations = self.config.max_iterations)
    )]
    pub fn run<'a>(
        &self,
        allocator: &'a Allocator,
        program: &mut Program<'a>,
    ) -> Result<EngineResult, DeobError> {
        let start = Instant::now();
        let mut total_modifications = 0;
        let mut modifications_per_iteration = Vec::new();
        let mut converged = false;
        let errors = Vec::new();

        tracing::info!(target: "deob::engine", "starting deobfuscation");

        for iteration in 0..self.config.max_iterations {
            let iter_span = tracing::info_span!(target: "deob::engine", "iteration", n = iteration);
            let _guard = iter_span.enter();

            let mut iter_mods = 0;

            // 1. Run Extensions layer (string rotator, etc.) - FIRST to decode strings
            if self.config.layers.extensions && self.config.layers.extensions_config.string_array {
                match StringRotator::transform(allocator, program) {
                    Ok(result) => {
                        let ext_mods = result.strings_decoded + result.functions_removed;
                        tracing::debug!(
                            target: "deob::engine",
                            systems = result.systems_detected,
                            decoded = result.strings_decoded,
                            inlined = result.calls_inlined,
                            removed = result.functions_removed,
                            "Extensions layer complete"
                        );
                        iter_mods += ext_mods;
                    }
                    Err(e) => {
                        tracing::warn!(target: "deob::engine", error = %e, "Extensions layer error");
                    }
                }
            }

            // 2. Rebuild scoping (fresh symbol/reference info after extensions)
            tracing::debug!(target: "deob::engine", "building scoping");
            let semantic = SemanticBuilder::new().build(program).semantic;
            let scoping = semantic.into_scoping();

            // 3. Run ECMA layer
            if self.config.layers.ecma {
                let mut ecma = EcmaTransformer::new(&self.config.layers.ecma_config);
                traverse_mut(&mut ecma, allocator, program, scoping, ());
                let ecma_mods = ecma.modifications();
                tracing::debug!(target: "deob::engine", ecma_mods, "ECMA layer complete");
                iter_mods += ecma_mods;
            }

            // 4. Rebuild scoping for Runtime layer
            let semantic = SemanticBuilder::new().build(program).semantic;
            let scoping = semantic.into_scoping();

            // 5. Run Runtime layer
            if self.config.layers.runtime {
                let mut runtime = RuntimeTransformer::new(&self.config.layers.runtime_config);
                traverse_mut(&mut runtime, allocator, program, scoping, ());
                let runtime_mods = runtime.modifications();
                tracing::debug!(target: "deob::engine", runtime_mods, "Runtime layer complete");
                iter_mods += runtime_mods;
            }

            tracing::info!(
                target: "deob::engine",
                modifications = iter_mods,
                "iteration complete"
            );

            total_modifications += iter_mods;
            modifications_per_iteration.push(iter_mods);

            // 6. Check convergence
            if iter_mods == 0 {
                tracing::info!(target: "deob::engine", iterations = iteration + 1, "converged");
                converged = true;
                break;
            }
        }

        if !converged {
            tracing::warn!(
                target: "deob::engine",
                max = self.config.max_iterations,
                "max iterations reached without convergence"
            );
        }

        let elapsed = start.elapsed();
        tracing::info!(
            target: "deob::engine",
            total_modifications,
            iterations = modifications_per_iteration.len(),
            elapsed_ms = elapsed.as_millis(),
            "deobfuscation complete"
        );

        Ok(EngineResult {
            iterations: modifications_per_iteration.len(),
            total_modifications,
            modifications_per_iteration,
            converged,
            elapsed,
            errors,
        })
    }
}
