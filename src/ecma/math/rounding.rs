//! Math rounding passes.
//!
//! - `Floor` — `Math.floor(3.7)` → `3`
//! - `Ceil` — `Math.ceil(3.2)` → `4`
//! - `Round` — `Math.round(3.5)` → `4`
//! - `Trunc` — `Math.trunc(3.7)` → `3`

use oxc::ast::ast::{Argument, Expression};
use oxc_traverse::TraverseCtx;

use crate::utils::literal;

// ============================================================================
// Individual Passes
// ============================================================================

/// Evaluates Math.floor: `Math.floor(3.7)` → `3`
#[derive(Default)]
pub struct Floor;

impl Floor {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Some(n) = extract_math_call(expr, "floor") else {
            return 0;
        };
        *expr = literal::make_number(n.floor(), &ctx.ast);
        1
    }
}

/// Evaluates Math.ceil: `Math.ceil(3.2)` → `4`
#[derive(Default)]
pub struct Ceil;

impl Ceil {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Some(n) = extract_math_call(expr, "ceil") else {
            return 0;
        };
        *expr = literal::make_number(n.ceil(), &ctx.ast);
        1
    }
}

/// Evaluates Math.round: `Math.round(3.5)` → `4`
#[derive(Default)]
pub struct Round;

impl Round {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Some(n) = extract_math_call(expr, "round") else {
            return 0;
        };
        // JavaScript's Math.round rounds half away from zero
        let result = js_round(n);
        *expr = literal::make_number(result, &ctx.ast);
        1
    }
}

/// Evaluates Math.trunc: `Math.trunc(3.7)` → `3`
#[derive(Default)]
pub struct Trunc;

impl Trunc {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Some(n) = extract_math_call(expr, "trunc") else {
            return 0;
        };
        *expr = literal::make_number(n.trunc(), &ctx.ast);
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

/// JavaScript's Math.round: rounds half away from zero
fn js_round(n: f64) -> f64 {
    if n.is_nan() || n.is_infinite() {
        return n;
    }
    let floor = n.floor();
    if n - floor >= 0.5 { floor + 1.0 } else { floor }
}

// ============================================================================
// Rounding Group
// ============================================================================

/// Group of all Math rounding passes.
#[derive(Default)]
pub struct Rounding {
    floor: Floor,
    ceil: Ceil,
    round: Round,
    trunc: Trunc,
}

impl Rounding {
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
        mods += self.floor.transform(expr, ctx);
        mods += self.ceil.transform(expr, ctx);
        mods += self.round.transform(expr, ctx);
        mods += self.trunc.transform(expr, ctx);
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

        struct Visitor(Rounding);
        impl<'a> oxc_traverse::Traverse<'a, ()> for Visitor {
            fn exit_expression(
                &mut self,
                expr: &mut Expression<'a>,
                ctx: &mut TraverseCtx<'a, ()>,
            ) {
                self.0.transform(expr, ctx);
            }
        }

        let mut visitor = Visitor(Rounding::new());
        traverse_mut(&mut visitor, &allocator, &mut program, scoping, ());
        codegen(&program)
    }

    #[test]
    fn test_floor() {
        let result = transform("Math.floor(3.7)");
        assert!(result.contains('3'));
    }

    #[test]
    fn test_ceil() {
        let result = transform("Math.ceil(3.2)");
        assert!(result.contains('4'));
    }

    #[test]
    fn test_round() {
        let result = transform("Math.round(3.5)");
        assert!(result.contains('4'));
    }

    #[test]
    fn test_trunc() {
        let result = transform("Math.trunc(3.7)");
        assert!(result.contains('3'));
    }
}
