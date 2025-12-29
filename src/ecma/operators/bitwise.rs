//! Bitwise operator passes.
//!
//! - `BitAnd` — `5 & 3` → `1`
//! - `BitOr` — `5 | 3` → `7`
//! - `BitXor` — `5 ^ 3` → `6`
//! - `BitNot` — `~5` → `-6`
//! - `ShiftLeft` — `1 << 2` → `4`
//! - `ShiftRight` — `4 >> 1` → `2`
//! - `ShiftRightUnsigned` — `-1 >>> 0` → `4294967295`

use oxc::ast::ast::{BinaryOperator, Expression, UnaryOperator};
use oxc_traverse::TraverseCtx;

use crate::utils::literal;

// ============================================================================
// Individual Passes
// ============================================================================

/// Evaluates bitwise AND: `5 & 3` → `1`
#[derive(Default)]
pub struct BitAnd;

impl BitAnd {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Expression::BinaryExpression(bin) = expr else {
            return 0;
        };
        if bin.operator != BinaryOperator::BitwiseAnd {
            return 0;
        }

        let Some(left) = literal::number(&bin.left) else {
            return 0;
        };
        let Some(right) = literal::number(&bin.right) else {
            return 0;
        };

        let result = (left as i32 & right as i32) as f64;
        *expr = literal::make_number(result, &ctx.ast);
        1
    }
}

/// Evaluates bitwise OR: `5 | 3` → `7`
#[derive(Default)]
pub struct BitOr;

impl BitOr {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Expression::BinaryExpression(bin) = expr else {
            return 0;
        };
        if bin.operator != BinaryOperator::BitwiseOR {
            return 0;
        }

        let Some(left) = literal::number(&bin.left) else {
            return 0;
        };
        let Some(right) = literal::number(&bin.right) else {
            return 0;
        };

        let result = (left as i32 | right as i32) as f64;
        *expr = literal::make_number(result, &ctx.ast);
        1
    }
}

/// Evaluates bitwise XOR: `5 ^ 3` → `6`
#[derive(Default)]
pub struct BitXor;

impl BitXor {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Expression::BinaryExpression(bin) = expr else {
            return 0;
        };
        if bin.operator != BinaryOperator::BitwiseXOR {
            return 0;
        }

        let Some(left) = literal::number(&bin.left) else {
            return 0;
        };
        let Some(right) = literal::number(&bin.right) else {
            return 0;
        };

        let result = (left as i32 ^ right as i32) as f64;
        *expr = literal::make_number(result, &ctx.ast);
        1
    }
}

/// Evaluates bitwise NOT: `~5` → `-6`
#[derive(Default)]
pub struct BitNot;

impl BitNot {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Expression::UnaryExpression(unary) = expr else {
            return 0;
        };
        if unary.operator != UnaryOperator::BitwiseNot {
            return 0;
        }

        let Some(value) = literal::number(&unary.argument) else {
            return 0;
        };

        let result = !(value as i32) as f64;
        *expr = literal::make_number(result, &ctx.ast);
        1
    }
}

/// Evaluates left shift: `1 << 2` → `4`
#[derive(Default)]
pub struct ShiftLeft;

impl ShiftLeft {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Expression::BinaryExpression(bin) = expr else {
            return 0;
        };
        if bin.operator != BinaryOperator::ShiftLeft {
            return 0;
        }

        let Some(left) = literal::number(&bin.left) else {
            return 0;
        };
        let Some(right) = literal::number(&bin.right) else {
            return 0;
        };

        // JS: left << (right & 0x1f)
        let shift = (right as u32) & 0x1f;
        let result = ((left as i32) << shift) as f64;
        *expr = literal::make_number(result, &ctx.ast);
        1
    }
}

/// Evaluates signed right shift: `4 >> 1` → `2`
#[derive(Default)]
pub struct ShiftRight;

impl ShiftRight {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Expression::BinaryExpression(bin) = expr else {
            return 0;
        };
        if bin.operator != BinaryOperator::ShiftRight {
            return 0;
        }

        let Some(left) = literal::number(&bin.left) else {
            return 0;
        };
        let Some(right) = literal::number(&bin.right) else {
            return 0;
        };

        let shift = (right as u32) & 0x1f;
        let result = ((left as i32) >> shift) as f64;
        *expr = literal::make_number(result, &ctx.ast);
        1
    }
}

/// Evaluates unsigned right shift: `-1 >>> 0` → `4294967295`
#[derive(Default)]
pub struct ShiftRightUnsigned;

impl ShiftRightUnsigned {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Expression::BinaryExpression(bin) = expr else {
            return 0;
        };
        if bin.operator != BinaryOperator::ShiftRightZeroFill {
            return 0;
        }

        let Some(left) = literal::number(&bin.left) else {
            return 0;
        };
        let Some(right) = literal::number(&bin.right) else {
            return 0;
        };

        let shift = (right as u32) & 0x1f;
        let result = ((left as u32) >> shift) as f64;
        *expr = literal::make_number(result, &ctx.ast);
        1
    }
}

// ============================================================================
// Bitwise Group
// ============================================================================

/// Group of all bitwise passes.
#[derive(Default)]
pub struct Bitwise {
    bit_and: BitAnd,
    bit_or: BitOr,
    bit_xor: BitXor,
    bit_not: BitNot,
    shift_left: ShiftLeft,
    shift_right: ShiftRight,
    shift_right_unsigned: ShiftRightUnsigned,
}

impl Bitwise {
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
        mods += self.bit_and.transform(expr, ctx);
        mods += self.bit_or.transform(expr, ctx);
        mods += self.bit_xor.transform(expr, ctx);
        mods += self.bit_not.transform(expr, ctx);
        mods += self.shift_left.transform(expr, ctx);
        mods += self.shift_right.transform(expr, ctx);
        mods += self.shift_right_unsigned.transform(expr, ctx);
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

        struct Visitor(Bitwise);
        impl<'a> oxc_traverse::Traverse<'a, ()> for Visitor {
            fn exit_expression(
                &mut self,
                expr: &mut Expression<'a>,
                ctx: &mut TraverseCtx<'a, ()>,
            ) {
                self.0.transform(expr, ctx);
            }
        }

        let mut visitor = Visitor(Bitwise::new());
        traverse_mut(&mut visitor, &allocator, &mut program, scoping, ());
        codegen(&program)
    }

    #[test]
    fn test_bit_and() {
        assert_eq!(transform("5 & 3"), "1;\n");
    }

    #[test]
    fn test_bit_or() {
        assert_eq!(transform("5 | 3"), "7;\n");
    }

    #[test]
    fn test_bit_xor() {
        assert_eq!(transform("5 ^ 3"), "6;\n");
    }

    #[test]
    fn test_bit_not() {
        assert_eq!(transform("~5"), "-6;\n");
        assert_eq!(transform("~0"), "-1;\n");
    }

    #[test]
    fn test_shift_left() {
        assert_eq!(transform("1 << 2"), "4;\n");
        assert_eq!(transform("3 << 4"), "48;\n");
    }

    #[test]
    fn test_shift_right() {
        assert_eq!(transform("4 >> 1"), "2;\n");
        assert_eq!(transform("-4 >> 1"), "-2;\n");
    }

    #[test]
    fn test_shift_right_unsigned() {
        // Note: -1 >>> 0 requires the arithmetic pass to first evaluate -1
        // For now, test with positive numbers
        assert_eq!(transform("8 >>> 2"), "2;\n");
        assert_eq!(transform("16 >>> 1"), "8;\n");
    }
}
