//! Array.prototype method passes.
//!
//! All Array methods consolidated into logical files:
//!
//! - [`access`] — at, length, bracket access
//! - [`search`] — indexOf, lastIndexOf, includes
//! - [`transform`] — join, toString, reverse, flat
//! - [`extract`] — slice, concat

use oxc::ast::ast::Expression;
use oxc_traverse::TraverseCtx;

// ============================================================================
// Modules
// ============================================================================

pub mod access;
pub mod extract;
pub mod search;
pub mod transform;

// ============================================================================
// Re-exports
// ============================================================================

pub use access::Access;
pub use extract::Extract;
pub use search::Search;
pub use transform::Transform;

// ============================================================================
// ArrayGroup
// ============================================================================

/// Group of all Array.prototype passes.
#[derive(Default)]
pub struct ArrayGroup {
    access: Access,
    search: Search,
    transform: Transform,
    extract: Extract,
}

impl ArrayGroup {
    pub fn new() -> Self {
        Self::default()
    }

    /// Transform expression through all array passes.
    /// Returns number of modifications.
    #[inline]
    pub fn exit_expression<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let mut mods = 0;
        mods += self.access.transform(expr, ctx);
        mods += self.search.transform(expr, ctx);
        mods += self.transform.transform(expr, ctx);
        mods += self.extract.transform(expr, ctx);
        mods
    }
}
