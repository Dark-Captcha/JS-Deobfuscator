//! Comparison operator passes.
//!
//! - `StrictEq` — `1 === 1` → `true`
//! - `StrictNeq` — `1 !== 2` → `true`
//! - `LooseEq` — `1 == "1"` → `true`
//! - `LooseNeq` — `1 != "2"` → `true`
//! - `Lt` — `1 < 2` → `true`
//! - `Lte` — `1 <= 1` → `true`
//! - `Gt` — `2 > 1` → `true`
//! - `Gte` — `2 >= 2` → `true`

use oxc::ast::ast::{BinaryOperator, Expression};
use oxc::span::SPAN;
use oxc_traverse::TraverseCtx;

use crate::utils::literal::{self, LiteralValue};

// ============================================================================
// Individual Passes
// ============================================================================

/// Evaluates strict equality: `1 === 1` → `true`
#[derive(Default)]
pub struct StrictEq;

impl StrictEq {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Expression::BinaryExpression(bin) = expr else {
            return 0;
        };
        if bin.operator != BinaryOperator::StrictEquality {
            return 0;
        }

        let Some(left) = LiteralValue::from_expr(&bin.left) else {
            return 0;
        };
        let Some(right) = LiteralValue::from_expr(&bin.right) else {
            return 0;
        };

        let result = left == right;
        *expr = ctx.ast.expression_boolean_literal(SPAN, result);
        1
    }
}

/// Evaluates strict inequality: `1 !== 2` → `true`
#[derive(Default)]
pub struct StrictNeq;

impl StrictNeq {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Expression::BinaryExpression(bin) = expr else {
            return 0;
        };
        if bin.operator != BinaryOperator::StrictInequality {
            return 0;
        }

        let Some(left) = LiteralValue::from_expr(&bin.left) else {
            return 0;
        };
        let Some(right) = LiteralValue::from_expr(&bin.right) else {
            return 0;
        };

        let result = left != right;
        *expr = ctx.ast.expression_boolean_literal(SPAN, result);
        1
    }
}

/// Evaluates loose equality: `1 == "1"` → `true`
///
/// Note: Only handles simple cases where types match or are easily coercible.
#[derive(Default)]
pub struct LooseEq;

impl LooseEq {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Expression::BinaryExpression(bin) = expr else {
            return 0;
        };
        if bin.operator != BinaryOperator::Equality {
            return 0;
        }

        let Some(left) = LiteralValue::from_expr(&bin.left) else {
            return 0;
        };
        let Some(right) = LiteralValue::from_expr(&bin.right) else {
            return 0;
        };

        let Some(result) = loose_equals(&left, &right) else {
            return 0;
        };
        *expr = ctx.ast.expression_boolean_literal(SPAN, result);
        1
    }
}

/// Evaluates loose inequality: `1 != "2"` → `true`
#[derive(Default)]
pub struct LooseNeq;

impl LooseNeq {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Expression::BinaryExpression(bin) = expr else {
            return 0;
        };
        if bin.operator != BinaryOperator::Inequality {
            return 0;
        }

        let Some(left) = LiteralValue::from_expr(&bin.left) else {
            return 0;
        };
        let Some(right) = LiteralValue::from_expr(&bin.right) else {
            return 0;
        };

        let Some(result) = loose_equals(&left, &right) else {
            return 0;
        };
        *expr = ctx.ast.expression_boolean_literal(SPAN, !result);
        1
    }
}

/// Evaluates less than: `1 < 2` → `true`
#[derive(Default)]
pub struct Lt;

impl Lt {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Expression::BinaryExpression(bin) = expr else {
            return 0;
        };
        if bin.operator != BinaryOperator::LessThan {
            return 0;
        }

        let Some(left) = literal::number(&bin.left) else {
            return 0;
        };
        let Some(right) = literal::number(&bin.right) else {
            return 0;
        };

        let result = left < right;
        *expr = ctx.ast.expression_boolean_literal(SPAN, result);
        1
    }
}

/// Evaluates less than or equal: `1 <= 1` → `true`
#[derive(Default)]
pub struct Lte;

impl Lte {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Expression::BinaryExpression(bin) = expr else {
            return 0;
        };
        if bin.operator != BinaryOperator::LessEqualThan {
            return 0;
        }

        let Some(left) = literal::number(&bin.left) else {
            return 0;
        };
        let Some(right) = literal::number(&bin.right) else {
            return 0;
        };

        let result = left <= right;
        *expr = ctx.ast.expression_boolean_literal(SPAN, result);
        1
    }
}

/// Evaluates greater than: `2 > 1` → `true`
#[derive(Default)]
pub struct Gt;

impl Gt {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Expression::BinaryExpression(bin) = expr else {
            return 0;
        };
        if bin.operator != BinaryOperator::GreaterThan {
            return 0;
        }

        let Some(left) = literal::number(&bin.left) else {
            return 0;
        };
        let Some(right) = literal::number(&bin.right) else {
            return 0;
        };

        let result = left > right;
        *expr = ctx.ast.expression_boolean_literal(SPAN, result);
        1
    }
}

/// Evaluates greater than or equal: `2 >= 2` → `true`
#[derive(Default)]
pub struct Gte;

impl Gte {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Expression::BinaryExpression(bin) = expr else {
            return 0;
        };
        if bin.operator != BinaryOperator::GreaterEqualThan {
            return 0;
        }

        let Some(left) = literal::number(&bin.left) else {
            return 0;
        };
        let Some(right) = literal::number(&bin.right) else {
            return 0;
        };

        let result = left >= right;
        *expr = ctx.ast.expression_boolean_literal(SPAN, result);
        1
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// JavaScript loose equality (==) for literal values.
fn loose_equals(left: &LiteralValue, right: &LiteralValue) -> Option<bool> {
    use LiteralValue::*;

    Some(match (left, right) {
        // Same type: strict comparison
        (Number(a), Number(b)) => a == b,
        (String(a), String(b)) => a == b,
        (Boolean(a), Boolean(b)) => a == b,
        (Null, Null) => true,
        (Undefined, Undefined) => true,

        // null == undefined
        (Null, Undefined) | (Undefined, Null) => true,

        // Number == String: convert string to number
        (Number(n), String(s)) | (String(s), Number(n)) => {
            let parsed = s.trim().parse::<f64>().unwrap_or(f64::NAN);
            *n == parsed
        }

        // Boolean == anything: convert boolean to number first
        (Boolean(b), other) | (other, Boolean(b)) => {
            let n = if *b { 1.0 } else { 0.0 };
            loose_equals(&Number(n), other)?
        }

        // null/undefined != other types
        (Null | Undefined, _) | (_, Null | Undefined) => false,
    })
}

// ============================================================================
// Comparison Group
// ============================================================================

/// Group of all comparison passes.
#[derive(Default)]
pub struct Comparison {
    strict_eq: StrictEq,
    strict_neq: StrictNeq,
    loose_eq: LooseEq,
    loose_neq: LooseNeq,
    lt: Lt,
    lte: Lte,
    gt: Gt,
    gte: Gte,
}

impl Comparison {
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
        mods += self.strict_eq.transform(expr, ctx);
        mods += self.strict_neq.transform(expr, ctx);
        mods += self.loose_eq.transform(expr, ctx);
        mods += self.loose_neq.transform(expr, ctx);
        mods += self.lt.transform(expr, ctx);
        mods += self.lte.transform(expr, ctx);
        mods += self.gt.transform(expr, ctx);
        mods += self.gte.transform(expr, ctx);
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

        struct Visitor(Comparison);
        impl<'a> oxc_traverse::Traverse<'a, ()> for Visitor {
            fn exit_expression(
                &mut self,
                expr: &mut Expression<'a>,
                ctx: &mut TraverseCtx<'a, ()>,
            ) {
                self.0.transform(expr, ctx);
            }
        }

        let mut visitor = Visitor(Comparison::new());
        traverse_mut(&mut visitor, &allocator, &mut program, scoping, ());
        codegen(&program)
    }

    #[test]
    fn test_strict_eq() {
        assert_eq!(transform("1 === 1"), "true;\n");
        assert_eq!(transform("1 === 2"), "false;\n");
        assert_eq!(transform("\"a\" === \"a\""), "true;\n");
    }

    #[test]
    fn test_strict_neq() {
        assert_eq!(transform("1 !== 2"), "true;\n");
        assert_eq!(transform("1 !== 1"), "false;\n");
    }

    #[test]
    fn test_loose_eq() {
        assert_eq!(transform("1 == 1"), "true;\n");
        assert_eq!(transform("1 == \"1\""), "true;\n");
        assert_eq!(transform("null == undefined"), "true;\n");
        assert_eq!(transform("true == 1"), "true;\n");
    }

    #[test]
    fn test_loose_neq() {
        assert_eq!(transform("1 != 2"), "true;\n");
        assert_eq!(transform("1 != \"1\""), "false;\n");
    }

    #[test]
    fn test_lt() {
        assert_eq!(transform("1 < 2"), "true;\n");
        assert_eq!(transform("2 < 1"), "false;\n");
    }

    #[test]
    fn test_lte() {
        assert_eq!(transform("1 <= 1"), "true;\n");
        assert_eq!(transform("2 <= 1"), "false;\n");
    }

    #[test]
    fn test_gt() {
        assert_eq!(transform("2 > 1"), "true;\n");
        assert_eq!(transform("1 > 2"), "false;\n");
    }

    #[test]
    fn test_gte() {
        assert_eq!(transform("2 >= 2"), "true;\n");
        assert_eq!(transform("1 >= 2"), "false;\n");
    }
}
