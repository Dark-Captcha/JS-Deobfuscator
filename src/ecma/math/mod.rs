//! Math object method passes.
//!
//! All Math methods consolidated into logical files:
//!
//! - [`rounding`] — floor, ceil, round, trunc
//! - [`arithmetic`] — abs, sign, pow, sqrt
//! - [`minmax`] — min, max
//! - [`trig`] — sin, cos, tan, log, exp, hypot, imul, clz32, fround

use oxc::ast::ast::Expression;
use oxc_traverse::TraverseCtx;

// ============================================================================
// Modules
// ============================================================================

pub mod arithmetic;
pub mod minmax;
pub mod rounding;
pub mod trig;

// ============================================================================
// Re-exports
// ============================================================================

pub use arithmetic::Arithmetic;
pub use minmax::MinMax;
pub use rounding::Rounding;
pub use trig::Trig;

// ============================================================================
// MathGroup
// ============================================================================

/// Group of all Math passes.
#[derive(Default)]
pub struct MathGroup {
    rounding: Rounding,
    arithmetic: Arithmetic,
    minmax: MinMax,
    trig: Trig,
}

impl MathGroup {
    pub fn new() -> Self {
        Self::default()
    }

    /// Transform expression through all math passes.
    /// Returns number of modifications.
    #[inline]
    pub fn exit_expression<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let mut mods = 0;
        mods += self.rounding.transform(expr, ctx);
        mods += self.arithmetic.transform(expr, ctx);
        mods += self.minmax.transform(expr, ctx);
        mods += self.trig.transform(expr, ctx);
        mods
    }
}
