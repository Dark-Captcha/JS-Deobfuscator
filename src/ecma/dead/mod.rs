//! Dead code removal passes.
//!
//! - `EmptyStatement` — `;` → remove
//! - `DebuggerRemove` — `debugger;` → remove

use oxc::allocator::Vec as ArenaVec;
use oxc::ast::ast::Statement;
use oxc_traverse::TraverseCtx;

// ============================================================================
// Individual Passes
// ============================================================================

/// Removes empty statements
#[derive(Default)]
pub struct EmptyStatement;

impl EmptyStatement {
    /// Remove empty statements from a statement list
    #[inline]
    pub fn transform_stmts<'a>(
        &mut self,
        stmts: &mut ArenaVec<'a, Statement<'a>>,
        _ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let before = stmts.len();
        stmts.retain(|stmt| !matches!(stmt, Statement::EmptyStatement(_)));
        before - stmts.len()
    }
}

/// Removes debugger statements
#[derive(Default)]
pub struct DebuggerRemove;

impl DebuggerRemove {
    /// Remove debugger statements from a statement list
    #[inline]
    pub fn transform_stmts<'a>(
        &mut self,
        stmts: &mut ArenaVec<'a, Statement<'a>>,
        _ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let before = stmts.len();
        stmts.retain(|stmt| !matches!(stmt, Statement::DebuggerStatement(_)));
        before - stmts.len()
    }
}

// ============================================================================
// DeadGroup
// ============================================================================

/// Group of all dead code removal passes.
#[derive(Default)]
pub struct DeadGroup {
    empty_statement: EmptyStatement,
    debugger_remove: DebuggerRemove,
}

impl DeadGroup {
    pub fn new() -> Self {
        Self::default()
    }

    /// Transform statement list to remove dead code.
    #[inline]
    pub fn exit_statements<'a>(
        &mut self,
        stmts: &mut ArenaVec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let mut mods = 0;
        mods += self.empty_statement.transform_stmts(stmts, ctx);
        mods += self.debugger_remove.transform_stmts(stmts, ctx);
        mods
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::testing::{codegen, parse_program};
    use oxc::allocator::Allocator;
    use oxc::allocator::Vec as ArenaVec;
    use oxc::semantic::SemanticBuilder;
    use oxc_traverse::traverse_mut;

    fn transform(source: &str) -> String {
        let allocator = Allocator::default();
        let mut program = parse_program(&allocator, source);
        let scoping = SemanticBuilder::new()
            .build(&program)
            .semantic
            .into_scoping();

        struct Visitor(DeadGroup);
        impl<'a> oxc_traverse::Traverse<'a, ()> for Visitor {
            fn exit_statements(
                &mut self,
                stmts: &mut ArenaVec<'a, Statement<'a>>,
                ctx: &mut TraverseCtx<'a, ()>,
            ) {
                self.0.exit_statements(stmts, ctx);
            }
        }

        let mut visitor = Visitor(DeadGroup::new());
        traverse_mut(&mut visitor, &allocator, &mut program, scoping, ());
        codegen(&program)
    }

    #[test]
    fn test_remove_empty_statement() {
        let result = transform("x = 1;;; y = 2;");
        // Should have fewer semicolons
        assert!(result.contains("x = 1") && result.contains("y = 2"));
    }

    #[test]
    fn test_remove_debugger() {
        let result = transform("x = 1; debugger; y = 2;");
        assert!(!result.contains("debugger"));
    }

    #[test]
    fn test_keep_non_empty() {
        let result = transform("x = 1; y = 2;");
        assert!(result.contains("x = 1") && result.contains("y = 2"));
    }
}
