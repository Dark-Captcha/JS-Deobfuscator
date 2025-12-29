//! Control flow simplification passes.
//!
//! - `IfConstant` — `if(true){a}else{b}` → `{a}`
//! - `WhileFalse` — `while(false){}` → remove
//! - `DoWhileFalse` — `do{}while(false)` → unwrap body
//! - `ForFalse` — `for(;false;){}` → remove

use oxc::allocator::CloneIn;
use oxc::ast::ast::Statement;
use oxc_traverse::TraverseCtx;

use crate::utils::literal;

// ============================================================================
// Individual Passes
// ============================================================================

/// Simplifies if statements with constant conditions
#[derive(Default)]
pub struct IfConstant;

impl IfConstant {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        stmt: &mut Statement<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Statement::IfStatement(if_stmt) = stmt else {
            return 0;
        };

        let Some(is_truthy) = literal::is_truthy(&if_stmt.test) else {
            return 0;
        };

        if is_truthy {
            // Replace with consequent
            *stmt = if_stmt.consequent.clone_in(ctx.ast.allocator);
        } else if let Some(alt) = &if_stmt.alternate {
            // Replace with alternate
            *stmt = alt.clone_in(ctx.ast.allocator);
        } else {
            // No alternate, remove entirely (empty statement)
            *stmt = ctx.ast.statement_empty(if_stmt.span);
        }
        1
    }
}

/// Removes while(false) loops
#[derive(Default)]
pub struct WhileFalse;

impl WhileFalse {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        stmt: &mut Statement<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Statement::WhileStatement(while_stmt) = stmt else {
            return 0;
        };

        let Some(is_truthy) = literal::is_truthy(&while_stmt.test) else {
            return 0;
        };

        if !is_truthy {
            // Condition is always false, remove the loop
            *stmt = ctx.ast.statement_empty(while_stmt.span);
            return 1;
        }
        0
    }
}

/// Simplifies do-while(false) loops by unwrapping the body
#[derive(Default)]
pub struct DoWhileFalse;

impl DoWhileFalse {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        stmt: &mut Statement<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Statement::DoWhileStatement(do_while) = stmt else {
            return 0;
        };

        let Some(is_truthy) = literal::is_truthy(&do_while.test) else {
            return 0;
        };

        if !is_truthy {
            // Condition is always false, execute body once then exit
            // Replace with just the body
            *stmt = do_while.body.clone_in(ctx.ast.allocator);
            return 1;
        }
        0
    }
}

/// Removes for(;false;) loops
#[derive(Default)]
pub struct ForFalse;

impl ForFalse {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        stmt: &mut Statement<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Statement::ForStatement(for_stmt) = stmt else {
            return 0;
        };

        // Check if test is present and always false
        let Some(test) = &for_stmt.test else { return 0 };
        let Some(is_truthy) = literal::is_truthy(test) else {
            return 0;
        };

        if !is_truthy {
            // Condition is always false, loop never executes
            // But we need to keep the init if it has side effects
            if let Some(init) = &for_stmt.init {
                match init {
                    oxc::ast::ast::ForStatementInit::VariableDeclaration(decl) => {
                        // Keep variable declaration
                        *stmt = Statement::VariableDeclaration(decl.clone_in(ctx.ast.allocator));
                        return 1;
                    }
                    _ => {
                        // Check if it's an expression with side effects
                        if let Some(expr) = init.as_expression()
                            && !crate::utils::side_effects::is_side_effect_free(expr)
                        {
                            *stmt = ctx.ast.statement_expression(
                                for_stmt.span,
                                expr.clone_in(ctx.ast.allocator),
                            );
                            return 1;
                        }
                    }
                }
            }
            // No init or init is side-effect free, remove entirely
            *stmt = ctx.ast.statement_empty(for_stmt.span);
            return 1;
        }
        0
    }
}

// ============================================================================
// ControlGroup
// ============================================================================

/// Group of all control flow simplification passes.
#[derive(Default)]
pub struct ControlGroup {
    if_constant: IfConstant,
    while_false: WhileFalse,
    do_while_false: DoWhileFalse,
    for_false: ForFalse,
}

impl ControlGroup {
    pub fn new() -> Self {
        Self::default()
    }

    /// Transform a statement.
    #[inline]
    pub fn exit_statement<'a>(
        &mut self,
        stmt: &mut Statement<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let mut mods = 0;
        mods += self.if_constant.transform(stmt, ctx);
        mods += self.while_false.transform(stmt, ctx);
        mods += self.do_while_false.transform(stmt, ctx);
        mods += self.for_false.transform(stmt, ctx);
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
    use oxc::semantic::SemanticBuilder;
    use oxc_traverse::traverse_mut;

    fn transform(source: &str) -> String {
        let allocator = Allocator::default();
        let mut program = parse_program(&allocator, source);
        let scoping = SemanticBuilder::new()
            .build(&program)
            .semantic
            .into_scoping();

        struct Visitor(ControlGroup);
        impl<'a> oxc_traverse::Traverse<'a, ()> for Visitor {
            fn exit_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a, ()>) {
                self.0.exit_statement(stmt, ctx);
            }
        }

        let mut visitor = Visitor(ControlGroup::new());
        traverse_mut(&mut visitor, &allocator, &mut program, scoping, ());
        codegen(&program)
    }

    #[test]
    fn test_if_true() {
        let result = transform("if (true) { x = 1; } else { x = 2; }");
        assert!(result.contains("x = 1") && !result.contains("x = 2"));
    }

    #[test]
    fn test_if_false() {
        let result = transform("if (false) { x = 1; } else { x = 2; }");
        assert!(result.contains("x = 2") && !result.contains("x = 1"));
    }

    #[test]
    fn test_while_false() {
        let result = transform("while (false) { x = 1; }");
        assert!(!result.contains("while") && !result.contains("x = 1"));
    }

    #[test]
    fn test_do_while_false() {
        let result = transform("do { x = 1; } while (false);");
        assert!(result.contains("x = 1") && !result.contains("while"));
    }

    #[test]
    fn test_for_false() {
        let result = transform("for (; false; ) { x = 1; }");
        assert!(!result.contains("for") && !result.contains("x = 1"));
    }
}
