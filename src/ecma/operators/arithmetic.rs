//! Arithmetic operator passes.
//!
//! - `Add` — `1 + 2` → `3`
//! - `Sub` — `5 - 3` → `2`
//! - `Mul` — `2 * 3` → `6`
//! - `Div` — `6 / 2` → `3`
//! - `Mod` — `5 % 2` → `1`
//! - `Pow` — `2 ** 3` → `8`

use oxc::ast::ast::{BinaryOperator, Expression};
use oxc_traverse::TraverseCtx;

use crate::utils::literal;

// ============================================================================
// Individual Passes
// ============================================================================

/// Evaluates numeric addition: `1 + 2` → `3`
#[derive(Default)]
pub struct Add;

impl Add {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Expression::BinaryExpression(bin) = expr else {
            return 0;
        };
        if bin.operator != BinaryOperator::Addition {
            return 0;
        }

        let Some(left) = literal::number(&bin.left) else {
            return 0;
        };
        let Some(right) = literal::number(&bin.right) else {
            return 0;
        };

        *expr = literal::make_number(left + right, &ctx.ast);
        1
    }
}

/// Evaluates numeric subtraction: `5 - 3` → `2`
#[derive(Default)]
pub struct Sub;

impl Sub {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Expression::BinaryExpression(bin) = expr else {
            return 0;
        };
        if bin.operator != BinaryOperator::Subtraction {
            return 0;
        }

        let Some(left) = literal::number(&bin.left) else {
            return 0;
        };
        let Some(right) = literal::number(&bin.right) else {
            return 0;
        };

        *expr = literal::make_number(left - right, &ctx.ast);
        1
    }
}

/// Evaluates numeric multiplication: `2 * 3` → `6`
#[derive(Default)]
pub struct Mul;

impl Mul {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Expression::BinaryExpression(bin) = expr else {
            return 0;
        };
        if bin.operator != BinaryOperator::Multiplication {
            return 0;
        };

        let Some(left) = literal::number(&bin.left) else {
            return 0;
        };
        let Some(right) = literal::number(&bin.right) else {
            return 0;
        };

        *expr = literal::make_number(left * right, &ctx.ast);
        1
    }
}

/// Evaluates numeric division: `6 / 2` → `3`
#[derive(Default)]
pub struct Div;

impl Div {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Expression::BinaryExpression(bin) = expr else {
            return 0;
        };
        if bin.operator != BinaryOperator::Division {
            return 0;
        };

        let Some(left) = literal::number(&bin.left) else {
            return 0;
        };
        let Some(right) = literal::number(&bin.right) else {
            return 0;
        };

        // Division by zero produces Infinity/-Infinity/NaN in JS
        *expr = literal::make_number(left / right, &ctx.ast);
        1
    }
}

/// Evaluates modulo: `5 % 2` → `1`
#[derive(Default)]
pub struct Mod;

impl Mod {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Expression::BinaryExpression(bin) = expr else {
            return 0;
        };
        if bin.operator != BinaryOperator::Remainder {
            return 0;
        };

        let Some(left) = literal::number(&bin.left) else {
            return 0;
        };
        let Some(right) = literal::number(&bin.right) else {
            return 0;
        };

        *expr = literal::make_number(left % right, &ctx.ast);
        1
    }
}

/// Evaluates exponentiation: `2 ** 3` → `8`
#[derive(Default)]
pub struct Pow;

impl Pow {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Expression::BinaryExpression(bin) = expr else {
            return 0;
        };
        if bin.operator != BinaryOperator::Exponential {
            return 0;
        };

        let Some(left) = literal::number(&bin.left) else {
            return 0;
        };
        let Some(right) = literal::number(&bin.right) else {
            return 0;
        };

        *expr = literal::make_number(left.powf(right), &ctx.ast);
        1
    }
}

// ============================================================================
// Arithmetic Group
// ============================================================================

/// Group of all arithmetic passes.
#[derive(Default)]
pub struct Arithmetic {
    add: Add,
    sub: Sub,
    mul: Mul,
    div: Div,
    mod_: Mod,
    pow: Pow,
}

impl Arithmetic {
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
        mods += self.add.transform(expr, ctx);
        mods += self.sub.transform(expr, ctx);
        mods += self.mul.transform(expr, ctx);
        mods += self.div.transform(expr, ctx);
        mods += self.mod_.transform(expr, ctx);
        mods += self.pow.transform(expr, ctx);
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

        struct Visitor(Arithmetic);
        impl<'a> oxc_traverse::Traverse<'a, ()> for Visitor {
            fn exit_expression(
                &mut self,
                expr: &mut Expression<'a>,
                ctx: &mut TraverseCtx<'a, ()>,
            ) {
                self.0.transform(expr, ctx);
            }
        }

        let mut visitor = Visitor(Arithmetic::new());
        traverse_mut(&mut visitor, &allocator, &mut program, scoping, ());
        codegen(&program)
    }

    #[test]
    fn test_add() {
        assert_eq!(transform("1 + 2"), "3;\n");
        assert_eq!(transform("-1 + 3"), "2;\n");
        assert_eq!(transform("1.5 + 2.5"), "4;\n");
    }

    #[test]
    fn test_sub() {
        assert_eq!(transform("5 - 3"), "2;\n");
        assert_eq!(transform("1 - 5"), "-4;\n");
    }

    #[test]
    fn test_mul() {
        assert_eq!(transform("2 * 3"), "6;\n");
        assert_eq!(transform("-2 * 3"), "-6;\n");
    }

    #[test]
    fn test_div() {
        assert_eq!(transform("6 / 2"), "3;\n");
        assert_eq!(transform("7 / 2"), "3.5;\n");
    }

    #[test]
    fn test_mod() {
        assert_eq!(transform("5 % 2"), "1;\n");
        assert_eq!(transform("10 % 3"), "1;\n");
    }

    #[test]
    fn test_pow() {
        assert_eq!(transform("2 ** 3"), "8;\n");
        assert_eq!(transform("3 ** 2"), "9;\n");
    }

    #[test]
    fn test_non_numeric_unchanged() {
        assert_eq!(transform("x + 2"), "x + 2;\n");
        assert_eq!(transform("\"a\" + \"b\""), "\"a\" + \"b\";\n");
    }
}
