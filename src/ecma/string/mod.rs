//! String.prototype method passes.
//!
//! All String methods consolidated into logical files:
//!
//! - [`access`] — charAt, charCodeAt, at, length, bracket access
//! - [`search`] — indexOf, lastIndexOf, includes, startsWith, endsWith
//! - [`transform`] — toLowerCase, toUpperCase, trim, trimStart, trimEnd
//! - [`extract`] — slice, substring, substr, split
//! - [`modify`] — concat, repeat, replace, replaceAll, padStart, padEnd
//! - [`static_methods`] — String.fromCharCode, String.fromCodePoint

use oxc::ast::ast::Expression;
use oxc_traverse::TraverseCtx;

// ============================================================================
// Modules
// ============================================================================

pub mod access;
pub mod extract;
pub mod modify;
pub mod search;
pub mod static_methods;
pub mod transform;

// ============================================================================
// Re-exports
// ============================================================================

pub use access::Access;
pub use extract::Extract;
pub use modify::Modify;
pub use search::Search;
pub use static_methods::StaticMethods;
pub use transform::Transform;

// ============================================================================
// StringGroup
// ============================================================================

/// Group of all String.prototype passes.
#[derive(Default)]
pub struct StringGroup {
    access: Access,
    search: Search,
    transform: Transform,
    extract: Extract,
    modify: Modify,
    static_methods: StaticMethods,
}

impl StringGroup {
    pub fn new() -> Self {
        Self::default()
    }

    /// Transform expression through all string passes.
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
        mods += self.modify.transform(expr, ctx);
        mods += self.static_methods.transform(expr, ctx);
        mods
    }
}
