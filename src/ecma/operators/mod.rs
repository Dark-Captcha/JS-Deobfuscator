//! Operator evaluation passes.
//!
//! All JavaScript operators consolidated into logical files:
//!
//! - [`arithmetic`] — `+`, `-`, `*`, `/`, `%`, `**`
//! - [`bitwise`] — `&`, `|`, `^`, `~`, `<<`, `>>`, `>>>`
//! - [`comparison`] — `==`, `===`, `!=`, `!==`, `<`, `<=`, `>`, `>=`, `in`
//! - [`logical`] — `&&`, `||`, `??`
//! - [`unary`] — `!`, `typeof`, `void`, `+x`, `-x`
//! - [`ternary`] — `?:`

use oxc::ast::ast::Expression;
use oxc_traverse::TraverseCtx;

// ============================================================================
// Modules
// ============================================================================

pub mod arithmetic;
pub mod bitwise;
pub mod comparison;
pub mod logical;
pub mod ternary;
pub mod unary;

// ============================================================================
// Re-exports
// ============================================================================

pub use arithmetic::Arithmetic;
pub use bitwise::Bitwise;
pub use comparison::Comparison;
pub use logical::Logical;
pub use ternary::Ternary;
pub use unary::Unary;

// ============================================================================
// Operators Group
// ============================================================================

/// Group of all operator evaluation passes.
#[derive(Default)]
pub struct Operators {
    arithmetic: Arithmetic,
    bitwise: Bitwise,
    comparison: Comparison,
    logical: Logical,
    unary: Unary,
    ternary: Ternary,
}

impl Operators {
    pub fn new() -> Self {
        Self::default()
    }

    /// Transform expression through all operator passes.
    /// Returns number of modifications.
    #[inline]
    pub fn exit_expression<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let mut mods = 0;
        mods += self.arithmetic.transform(expr, ctx);
        mods += self.bitwise.transform(expr, ctx);
        mods += self.comparison.transform(expr, ctx);
        mods += self.logical.transform(expr, ctx);
        mods += self.unary.transform(expr, ctx);
        mods += self.ternary.transform(expr, ctx);
        mods
    }
}
