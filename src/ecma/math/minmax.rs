//! Math min/max passes.
//!
//! - `Min` — `Math.min(1, 2, 3)` → `1`
//! - `Max` — `Math.max(1, 2, 3)` → `3`

use oxc::ast::ast::{Argument, Expression};
use oxc_traverse::TraverseCtx;

use crate::utils::literal;

// ============================================================================
// Individual Passes
// ============================================================================

/// Evaluates Math.min: `Math.min(1, 2, 3)` → `1`
#[derive(Default)]
pub struct Min;

impl Min {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Expression::CallExpression(call) = expr else {
            return 0;
        };
        let Expression::StaticMemberExpression(member) = &call.callee else {
            return 0;
        };
        if member.property.name.as_str() != "min" {
            return 0;
        }

        let Expression::Identifier(obj) = &member.object else {
            return 0;
        };
        if obj.name.as_str() != "Math" {
            return 0;
        }

        if call.arguments.is_empty() {
            *expr = literal::make_number(f64::INFINITY, &ctx.ast);
            return 1;
        }

        let mut result = f64::INFINITY;
        for arg in &call.arguments {
            let Some(n) = get_number_arg(arg) else {
                return 0;
            };
            if n.is_nan() {
                *expr = literal::make_number(f64::NAN, &ctx.ast);
                return 1;
            }
            if n < result {
                result = n;
            }
        }

        *expr = literal::make_number(result, &ctx.ast);
        1
    }
}

/// Evaluates Math.max: `Math.max(1, 2, 3)` → `3`
#[derive(Default)]
pub struct Max;

impl Max {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Expression::CallExpression(call) = expr else {
            return 0;
        };
        let Expression::StaticMemberExpression(member) = &call.callee else {
            return 0;
        };
        if member.property.name.as_str() != "max" {
            return 0;
        }

        let Expression::Identifier(obj) = &member.object else {
            return 0;
        };
        if obj.name.as_str() != "Math" {
            return 0;
        }

        if call.arguments.is_empty() {
            *expr = literal::make_number(f64::NEG_INFINITY, &ctx.ast);
            return 1;
        }

        let mut result = f64::NEG_INFINITY;
        for arg in &call.arguments {
            let Some(n) = get_number_arg(arg) else {
                return 0;
            };
            if n.is_nan() {
                *expr = literal::make_number(f64::NAN, &ctx.ast);
                return 1;
            }
            if n > result {
                result = n;
            }
        }

        *expr = literal::make_number(result, &ctx.ast);
        1
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn get_number_arg(arg: &Argument) -> Option<f64> {
    match arg {
        Argument::NumericLiteral(n) => Some(n.value),
        _ => arg.as_expression().and_then(literal::number),
    }
}

// ============================================================================
// MinMax Group
// ============================================================================

/// Group of all Math min/max passes.
#[derive(Default)]
pub struct MinMax {
    min: Min,
    max: Max,
}

impl MinMax {
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
        mods += self.min.transform(expr, ctx);
        mods += self.max.transform(expr, ctx);
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

        struct Visitor(MinMax);
        impl<'a> oxc_traverse::Traverse<'a, ()> for Visitor {
            fn exit_expression(
                &mut self,
                expr: &mut Expression<'a>,
                ctx: &mut TraverseCtx<'a, ()>,
            ) {
                self.0.transform(expr, ctx);
            }
        }

        let mut visitor = Visitor(MinMax::new());
        traverse_mut(&mut visitor, &allocator, &mut program, scoping, ());
        codegen(&program)
    }

    #[test]
    fn test_min() {
        let result = transform("Math.min(1, 2, 3)");
        assert!(result.contains('1'));
    }

    #[test]
    fn test_max() {
        let result = transform("Math.max(1, 2, 3)");
        assert!(result.contains('3'));
    }

    #[test]
    fn test_min_negative() {
        let result = transform("Math.min(-5, 0, 5)");
        assert!(result.contains("-5"));
    }

    #[test]
    fn test_max_negative() {
        let result = transform("Math.max(-5, 0, 5)");
        assert!(result.contains('5'));
    }
}
