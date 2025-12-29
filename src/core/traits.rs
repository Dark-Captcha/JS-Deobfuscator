//! Traits for transformation passes and groups.

use oxc::allocator::Vec as ArenaVec;
use oxc::ast::ast::{Expression, Statement};
use oxc_traverse::TraverseCtx;

// ============================================================================
// Pass Trait
// ============================================================================

/// A single transformation pass.
///
/// Passes are stateless and operate on individual nodes.
/// Each pass does ONE transformation (e.g., `1 + 2 → 3`).
///
/// # Implementation
///
/// ```ignore
/// #[derive(Default)]
/// pub struct Add;
///
/// impl Pass for Add {
///     fn name(&self) -> &'static str { "add" }
///
///     fn transform_expr<'a>(
///         &self,
///         expr: &mut Expression<'a>,
///         ctx: &mut TraverseCtx<'a, ()>,
///     ) -> usize {
///         // Transform 1 + 2 → 3
///         // Return 1 if modified, 0 otherwise
///     }
/// }
/// ```
pub trait Pass: Send + Sync {
    /// Unique identifier for logging.
    fn name(&self) -> &'static str;

    /// Transform an expression. Returns modification count.
    fn transform_expr<'a>(
        &self,
        _expr: &mut Expression<'a>,
        _ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        0
    }

    /// Transform a statement. Returns modification count.
    fn transform_stmt<'a>(
        &self,
        _stmt: &mut Statement<'a>,
        _ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        0
    }
}

// ============================================================================
// PassGroup Trait
// ============================================================================

/// A group of related passes.
///
/// Groups aggregate multiple passes and run them in sequence.
/// Used for organizing passes by category (e.g., all arithmetic operators).
///
/// # Implementation
///
/// ```ignore
/// #[derive(Default)]
/// pub struct Arithmetic {
///     add: Add,
///     sub: Sub,
///     mul: Mul,
/// }
///
/// impl PassGroup for Arithmetic {
///     fn name(&self) -> &'static str { "arithmetic" }
///
///     fn exit_expression<'a>(
///         &mut self,
///         expr: &mut Expression<'a>,
///         ctx: &mut TraverseCtx<'a, ()>,
///     ) -> usize {
///         let mut mods = 0;
///         mods += self.add.transform_expr(expr, ctx);
///         mods += self.sub.transform_expr(expr, ctx);
///         mods += self.mul.transform_expr(expr, ctx);
///         mods
///     }
/// }
/// ```
pub trait PassGroup: Send + Sync {
    /// Group name for logging.
    fn name(&self) -> &'static str;

    /// Transform expression through all passes in group.
    fn exit_expression<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize;

    /// Transform statement through all passes in group.
    fn exit_statement<'a>(
        &mut self,
        _stmt: &mut Statement<'a>,
        _ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        0
    }

    /// Transform statement list.
    fn exit_statements<'a>(
        &mut self,
        _stmts: &mut ArenaVec<'a, Statement<'a>>,
        _ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        0
    }
}

// ============================================================================
// LayerTransformer Trait
// ============================================================================

/// A layer transformer (ECMA, Runtime, or Extensions).
///
/// Layers aggregate multiple pass groups and implement `Traverse`.
pub trait LayerTransformer: Send + Sync {
    /// Layer name.
    fn name(&self) -> &'static str;

    /// Whether this layer is enabled.
    fn enabled(&self) -> bool;

    /// Get modification count.
    fn modifications(&self) -> usize;

    /// Reset for new iteration.
    fn reset(&mut self);
}
