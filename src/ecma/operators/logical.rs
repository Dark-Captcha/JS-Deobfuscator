//! Logical operator passes.
//!
//! - `LogicalAnd` — `true && x` → `x`, `false && x` → `false`
//! - `LogicalOr` — `false || x` → `x`, `true || x` → `true`
//! - `NullishCoalesce` — `null ?? x` → `x`, `1 ?? x` → `1`
//! - `BooleanAnd` — `true && false` → `false`
//! - `BooleanOr` — `true || false` → `true`

use oxc::allocator::TakeIn;
use oxc::ast::ast::{Expression, LogicalOperator};
use oxc::span::SPAN;
use oxc_traverse::TraverseCtx;

use crate::utils::literal::{self, LiteralValue};

// ============================================================================
// Individual Passes
// ============================================================================

/// Short-circuits logical AND: `false && x` → `false`, `true && x` → `x`
#[derive(Default)]
pub struct LogicalAnd;

impl LogicalAnd {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Expression::LogicalExpression(logic) = expr else {
            return 0;
        };
        if logic.operator != LogicalOperator::And {
            return 0;
        }

        let Some(left_truthy) = literal::is_truthy(&logic.left) else {
            return 0;
        };

        // false && x → false (returns left)
        // true && x → x (returns right)
        let replacement = if left_truthy {
            logic.right.take_in(ctx.ast.allocator)
        } else {
            logic.left.take_in(ctx.ast.allocator)
        };

        *expr = replacement;
        1
    }
}

/// Short-circuits logical OR: `true || x` → `true`, `false || x` → `x`
#[derive(Default)]
pub struct LogicalOr;

impl LogicalOr {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Expression::LogicalExpression(logic) = expr else {
            return 0;
        };
        if logic.operator != LogicalOperator::Or {
            return 0;
        }

        let Some(left_truthy) = literal::is_truthy(&logic.left) else {
            return 0;
        };

        // true || x → true (returns left)
        // false || x → x (returns right)
        let replacement = if left_truthy {
            logic.left.take_in(ctx.ast.allocator)
        } else {
            logic.right.take_in(ctx.ast.allocator)
        };

        *expr = replacement;
        1
    }
}

/// Simplifies nullish coalescing: `null ?? x` → `x`, `1 ?? x` → `1`
#[derive(Default)]
pub struct NullishCoalesce;

impl NullishCoalesce {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Expression::LogicalExpression(logic) = expr else {
            return 0;
        };
        if logic.operator != LogicalOperator::Coalesce {
            return 0;
        }

        let Some(left_val) = LiteralValue::from_expr(&logic.left) else {
            return 0;
        };

        // null ?? x → x
        // undefined ?? x → x
        // value ?? x → value
        let is_nullish = matches!(left_val, LiteralValue::Null | LiteralValue::Undefined);

        let replacement = if is_nullish {
            logic.right.take_in(ctx.ast.allocator)
        } else {
            logic.left.take_in(ctx.ast.allocator)
        };

        *expr = replacement;
        1
    }
}

/// Evaluates boolean AND: `true && false` → `false`
#[derive(Default)]
pub struct BooleanAnd;

impl BooleanAnd {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Expression::LogicalExpression(logic) = expr else {
            return 0;
        };
        if logic.operator != LogicalOperator::And {
            return 0;
        }

        let Some(left) = literal::boolean(&logic.left) else {
            return 0;
        };
        let Some(right) = literal::boolean(&logic.right) else {
            return 0;
        };

        *expr = ctx.ast.expression_boolean_literal(SPAN, left && right);
        1
    }
}

/// Evaluates boolean OR: `true || false` → `true`
#[derive(Default)]
pub struct BooleanOr;

impl BooleanOr {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Expression::LogicalExpression(logic) = expr else {
            return 0;
        };
        if logic.operator != LogicalOperator::Or {
            return 0;
        }

        let Some(left) = literal::boolean(&logic.left) else {
            return 0;
        };
        let Some(right) = literal::boolean(&logic.right) else {
            return 0;
        };

        *expr = ctx.ast.expression_boolean_literal(SPAN, left || right);
        1
    }
}

// ============================================================================
// Logical Group
// ============================================================================

/// Group of all logical passes.
#[derive(Default)]
pub struct Logical {
    logical_and: LogicalAnd,
    logical_or: LogicalOr,
    nullish_coalesce: NullishCoalesce,
    boolean_and: BooleanAnd,
    boolean_or: BooleanOr,
}

impl Logical {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let mut mods = 0;
        // Run short-circuit passes first (they handle more cases)
        mods += self.logical_and.transform(expr, ctx);
        mods += self.logical_or.transform(expr, ctx);
        mods += self.nullish_coalesce.transform(expr, ctx);
        // Boolean passes for pure boolean operands
        mods += self.boolean_and.transform(expr, ctx);
        mods += self.boolean_or.transform(expr, ctx);
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

        struct Visitor(Logical);
        impl<'a> oxc_traverse::Traverse<'a, ()> for Visitor {
            fn exit_expression(
                &mut self,
                expr: &mut Expression<'a>,
                ctx: &mut TraverseCtx<'a, ()>,
            ) {
                self.0.transform(expr, ctx);
            }
        }

        let mut visitor = Visitor(Logical::new());
        traverse_mut(&mut visitor, &allocator, &mut program, scoping, ());
        codegen(&program)
    }

    #[test]
    fn test_logical_and_short_circuit() {
        assert_eq!(transform("false && x"), "false;\n");
        assert_eq!(transform("true && x"), "x;\n");
        assert_eq!(transform("0 && x"), "0;\n");
        assert_eq!(transform("1 && x"), "x;\n");
    }

    #[test]
    fn test_logical_or_short_circuit() {
        assert_eq!(transform("true || x"), "true;\n");
        assert_eq!(transform("false || x"), "x;\n");
        assert_eq!(transform("1 || x"), "1;\n");
        assert_eq!(transform("0 || x"), "x;\n");
    }

    #[test]
    fn test_nullish_coalesce() {
        assert_eq!(transform("null ?? x"), "x;\n");
        assert_eq!(transform("undefined ?? x"), "x;\n");
        assert_eq!(transform("1 ?? x"), "1;\n");
        assert_eq!(transform("0 ?? x"), "0;\n"); // 0 is not nullish
    }

    #[test]
    fn test_boolean_and() {
        assert_eq!(transform("true && false"), "false;\n");
        assert_eq!(transform("true && true"), "true;\n");
    }

    #[test]
    fn test_boolean_or() {
        assert_eq!(transform("true || false"), "true;\n");
        assert_eq!(transform("false || false"), "false;\n");
    }
}
