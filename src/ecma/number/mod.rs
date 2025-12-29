//! Number method passes.
//!
//! All Number methods consolidated into logical files:
//!
//! - [`static_methods`] — Number.isNaN, Number.isFinite, Number.isInteger, Number.isSafeInteger
//! - [`instance`] — toFixed, toString
//! - [`global`] — isNaN, isFinite, parseInt, parseFloat

use oxc::ast::ast::Expression;
use oxc_traverse::TraverseCtx;

// ============================================================================
// Modules
// ============================================================================

pub mod global;
pub mod instance;
pub mod static_methods;

// ============================================================================
// Re-exports
// ============================================================================

pub use global::Global;
pub use instance::Instance;
pub use static_methods::StaticMethods;

// ============================================================================
// NumberGroup
// ============================================================================

/// Group of all Number passes.
#[derive(Default)]
pub struct NumberGroup {
    static_methods: StaticMethods,
    instance: Instance,
    global: Global,
}

impl NumberGroup {
    pub fn new() -> Self {
        Self::default()
    }

    /// Transform expression through all number passes.
    /// Returns number of modifications.
    #[inline]
    pub fn exit_expression<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let mut mods = 0;
        mods += self.static_methods.transform(expr, ctx);
        mods += self.instance.transform(expr, ctx);
        mods += self.global.transform(expr, ctx);
        mods
    }
}
