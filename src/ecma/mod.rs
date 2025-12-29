//! Layer 1: ECMAScript Standard Transforms
//!
//! This layer contains transforms for ECMAScript standard features.
//! These work in **any** JavaScript runtime (browser, Node.js, Deno, Bun, etc.).
//!
//! # Groups
//!
//! - [`operators`] — Arithmetic, bitwise, comparison, logical, unary, ternary
//! - [`string`] — String.prototype methods
//! - [`array`] — Array.prototype methods
//! - [`number`] — Number methods and global functions
//! - [`math`] — Math object methods
//! - [`object`] — Object static methods
//! - [`json`] — JSON methods
//! - [`regexp`] — RegExp methods
//! - [`uri`] — URI encoding/decoding functions
//! - [`coercion`] — Type coercion patterns (JSFuck)
//! - [`syntax`] — Syntax simplification
//! - [`control`] — Control flow simplification
//! - [`dead`] — Dead code removal

use oxc::allocator::Vec as ArenaVec;
use oxc::ast::ast::{Expression, Statement};
use oxc_traverse::{Traverse, TraverseCtx};

use crate::core::{EcmaConfig, LayerTransformer};

// ============================================================================
// Submodules
// ============================================================================

pub mod array;
pub mod control;
pub mod dead;
pub mod json;
pub mod math;
pub mod number;
pub mod object;
pub mod operators;
pub mod regexp;
pub mod string;
pub mod syntax;
pub mod uri;

// TODO: implement in later phases
// pub mod coercion;

// ============================================================================
// Re-exports
// ============================================================================

pub use array::ArrayGroup;
pub use control::ControlGroup;
pub use dead::DeadGroup;
pub use json::JsonGroup;
pub use math::MathGroup;
pub use number::NumberGroup;
pub use object::ObjectGroup;
pub use operators::Operators;
pub use regexp::RegExpGroup;
pub use string::StringGroup;
pub use syntax::SyntaxGroup;
pub use uri::UriGroup;

// ============================================================================
// EcmaTransformer
// ============================================================================

/// Layer 1 transformer for ECMAScript standard features.
///
/// Aggregates all ECMA pass groups and implements `Traverse`.
pub struct EcmaTransformer {
    /// Configuration.
    config: EcmaConfig,

    /// Total modifications made during traversal.
    modifications: usize,

    // Pass groups
    operators: Operators,
    string: StringGroup,
    array: ArrayGroup,
    number: NumberGroup,
    math: MathGroup,
    object: ObjectGroup,
    json: JsonGroup,
    regexp: RegExpGroup,
    uri: UriGroup,
    syntax: SyntaxGroup,
    control: ControlGroup,
    dead: DeadGroup,
    // TODO: add in later phases
    // coercion: Coercion,
}

impl EcmaTransformer {
    /// Create a new ECMA transformer with the given configuration.
    pub fn new(config: &EcmaConfig) -> Self {
        Self {
            config: config.clone(),
            modifications: 0,
            operators: Operators::new(),
            string: StringGroup::new(),
            array: ArrayGroup::new(),
            number: NumberGroup::new(),
            math: MathGroup::new(),
            object: ObjectGroup::new(),
            json: JsonGroup::new(),
            regexp: RegExpGroup::new(),
            uri: UriGroup::new(),
            syntax: SyntaxGroup::new(),
            control: ControlGroup::new(),
            dead: DeadGroup::new(),
        }
    }

    /// Get the number of modifications made.
    #[inline]
    pub fn modifications(&self) -> usize {
        self.modifications
    }
}

impl LayerTransformer for EcmaTransformer {
    fn name(&self) -> &'static str {
        "ecma"
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

impl<'a> Traverse<'a, ()> for EcmaTransformer {
    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a, ()>) {
        // Run all expression-level pass groups
        if self.config.operators {
            self.modifications += self.operators.exit_expression(expr, ctx);
        }
        if self.config.string {
            self.modifications += self.string.exit_expression(expr, ctx);
        }
        if self.config.array {
            self.modifications += self.array.exit_expression(expr, ctx);
        }
        if self.config.number {
            self.modifications += self.number.exit_expression(expr, ctx);
        }
        if self.config.math {
            self.modifications += self.math.exit_expression(expr, ctx);
        }
        if self.config.object {
            self.modifications += self.object.transform(expr, ctx);
        }
        if self.config.json {
            self.modifications += self.json.transform(expr, ctx);
        }
        if self.config.regexp {
            self.modifications += self.regexp.transform(expr, ctx);
        }
        if self.config.uri {
            self.modifications += self.uri.transform(expr, ctx);
        }
        if self.config.syntax {
            self.modifications += self.syntax.transform(expr, ctx);
        }
        // TODO: Add more groups in later phases
    }

    fn exit_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a, ()>) {
        if self.config.control {
            self.modifications += self.control.exit_statement(stmt, ctx);
        }
    }

    fn exit_statements(
        &mut self,
        stmts: &mut ArenaVec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) {
        if self.config.dead {
            self.modifications += self.dead.exit_statements(stmts, ctx);
        }
    }
}
