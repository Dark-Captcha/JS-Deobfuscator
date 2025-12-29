//! Math arithmetic passes.
//!
//! - `Abs` — `Math.abs(-5)` → `5`
//! - `Sign` — `Math.sign(-5)` → `-1`
//! - `Pow` — `Math.pow(2, 3)` → `8`
//! - `Sqrt` — `Math.sqrt(16)` → `4`

use oxc::ast::ast::{Argument, Expression};
use oxc_traverse::TraverseCtx;

use crate::utils::literal;

// ============================================================================
// Individual Passes
// ============================================================================

/// Evaluates Math.abs: `Math.abs(-5)` → `5`
#[derive(Default)]
pub struct Abs;

impl Abs {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Some(n) = extract_math_call(expr, "abs") else {
            return 0;
        };
        *expr = literal::make_number(n.abs(), &ctx.ast);
        1
    }
}

/// Evaluates Math.sign: `Math.sign(-5)` → `-1`
#[derive(Default)]
pub struct Sign;

impl Sign {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Some(n) = extract_math_call(expr, "sign") else {
            return 0;
        };
        let result = if n.is_nan() {
            f64::NAN
        } else if n == 0.0 {
            n
        }
        // Preserve -0 and +0
        else if n > 0.0 {
            1.0
        } else {
            -1.0
        };
        *expr = literal::make_number(result, &ctx.ast);
        1
    }
}

/// Evaluates Math.pow: `Math.pow(2, 3)` → `8`
#[derive(Default)]
pub struct Pow;

impl Pow {
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
        if member.property.name.as_str() != "pow" {
            return 0;
        }

        let Expression::Identifier(obj) = &member.object else {
            return 0;
        };
        if obj.name.as_str() != "Math" {
            return 0;
        }

        let Some(base) = call.arguments.first().and_then(get_number_arg) else {
            return 0;
        };
        let Some(exp) = call.arguments.get(1).and_then(get_number_arg) else {
            return 0;
        };

        *expr = literal::make_number(base.powf(exp), &ctx.ast);
        1
    }
}

/// Evaluates Math.sqrt: `Math.sqrt(16)` → `4`
#[derive(Default)]
pub struct Sqrt;

impl Sqrt {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Some(n) = extract_math_call(expr, "sqrt") else {
            return 0;
        };
        *expr = literal::make_number(n.sqrt(), &ctx.ast);
        1
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn extract_math_call(expr: &Expression, method: &str) -> Option<f64> {
    let Expression::CallExpression(call) = expr else {
        return None;
    };
    let Expression::StaticMemberExpression(member) = &call.callee else {
        return None;
    };
    if member.property.name.as_str() != method {
        return None;
    }

    let Expression::Identifier(obj) = &member.object else {
        return None;
    };
    if obj.name.as_str() != "Math" {
        return None;
    }

    call.arguments.first().and_then(get_number_arg)
}

fn get_number_arg(arg: &Argument) -> Option<f64> {
    match arg {
        Argument::NumericLiteral(n) => Some(n.value),
        _ => arg.as_expression().and_then(literal::number),
    }
}

// ============================================================================
// Arithmetic Group
// ============================================================================

/// Group of all Math arithmetic passes.
#[derive(Default)]
pub struct Arithmetic {
    abs: Abs,
    sign: Sign,
    pow: Pow,
    sqrt: Sqrt,
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
        mods += self.abs.transform(expr, ctx);
        mods += self.sign.transform(expr, ctx);
        mods += self.pow.transform(expr, ctx);
        mods += self.sqrt.transform(expr, ctx);
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
    fn test_abs() {
        let result = transform("Math.abs(-5)");
        assert!(result.contains('5'));
    }

    #[test]
    fn test_sign_negative() {
        let result = transform("Math.sign(-5)");
        assert!(result.contains("-1"));
    }

    #[test]
    fn test_sign_positive() {
        let result = transform("Math.sign(5)");
        assert!(result.contains('1'));
    }

    #[test]
    fn test_pow() {
        let result = transform("Math.pow(2, 3)");
        assert!(result.contains('8'));
    }

    #[test]
    fn test_sqrt() {
        let result = transform("Math.sqrt(16)");
        assert!(result.contains('4'));
    }
}
