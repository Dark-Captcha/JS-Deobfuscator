//! Math trigonometric and other passes.
//!
//! - `Sin` — `Math.sin(0)` → `0`
//! - `Cos` — `Math.cos(0)` → `1`
//! - `Tan` — `Math.tan(0)` → `0`
//! - `Log` — `Math.log(1)` → `0`
//! - `Exp` — `Math.exp(0)` → `1`
//! - `Hypot` — `Math.hypot(3, 4)` → `5`
//! - `Imul` — `Math.imul(2, 3)` → `6`
//! - `Clz32` — `Math.clz32(1)` → `31`
//! - `Fround` — `Math.fround(1.5)` → `1.5`

use oxc::ast::ast::{Argument, Expression};
use oxc_traverse::TraverseCtx;

use crate::utils::literal;

// ============================================================================
// Individual Passes
// ============================================================================

/// Evaluates Math.sin: `Math.sin(0)` → `0`
#[derive(Default)]
pub struct Sin;

impl Sin {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Some(n) = extract_math_call(expr, "sin") else {
            return 0;
        };
        *expr = literal::make_number(n.sin(), &ctx.ast);
        1
    }
}

/// Evaluates Math.cos: `Math.cos(0)` → `1`
#[derive(Default)]
pub struct Cos;

impl Cos {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Some(n) = extract_math_call(expr, "cos") else {
            return 0;
        };
        *expr = literal::make_number(n.cos(), &ctx.ast);
        1
    }
}

/// Evaluates Math.tan: `Math.tan(0)` → `0`
#[derive(Default)]
pub struct Tan;

impl Tan {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Some(n) = extract_math_call(expr, "tan") else {
            return 0;
        };
        *expr = literal::make_number(n.tan(), &ctx.ast);
        1
    }
}

/// Evaluates Math.log: `Math.log(1)` → `0`
#[derive(Default)]
pub struct Log;

impl Log {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Some(n) = extract_math_call(expr, "log") else {
            return 0;
        };
        *expr = literal::make_number(n.ln(), &ctx.ast);
        1
    }
}

/// Evaluates Math.exp: `Math.exp(0)` → `1`
#[derive(Default)]
pub struct Exp;

impl Exp {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Some(n) = extract_math_call(expr, "exp") else {
            return 0;
        };
        *expr = literal::make_number(n.exp(), &ctx.ast);
        1
    }
}

/// Evaluates Math.hypot: `Math.hypot(3, 4)` → `5`
#[derive(Default)]
pub struct Hypot;

impl Hypot {
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
        if member.property.name.as_str() != "hypot" {
            return 0;
        }

        let Expression::Identifier(obj) = &member.object else {
            return 0;
        };
        if obj.name.as_str() != "Math" {
            return 0;
        }

        let mut sum_sq = 0.0;
        for arg in &call.arguments {
            let Some(n) = get_number_arg(arg) else {
                return 0;
            };
            sum_sq += n * n;
        }

        *expr = literal::make_number(sum_sq.sqrt(), &ctx.ast);
        1
    }
}

/// Evaluates Math.imul: `Math.imul(2, 3)` → `6`
#[derive(Default)]
pub struct Imul;

impl Imul {
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
        if member.property.name.as_str() != "imul" {
            return 0;
        }

        let Expression::Identifier(obj) = &member.object else {
            return 0;
        };
        if obj.name.as_str() != "Math" {
            return 0;
        }

        let Some(a) = call.arguments.first().and_then(get_number_arg) else {
            return 0;
        };
        let Some(b) = call.arguments.get(1).and_then(get_number_arg) else {
            return 0;
        };

        // Convert to 32-bit integers and multiply
        let result = (a as i32).wrapping_mul(b as i32);
        *expr = literal::make_number(result as f64, &ctx.ast);
        1
    }
}

/// Evaluates Math.clz32: `Math.clz32(1)` → `31`
#[derive(Default)]
pub struct Clz32;

impl Clz32 {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Some(n) = extract_math_call(expr, "clz32") else {
            return 0;
        };
        let result = (n as u32).leading_zeros();
        *expr = literal::make_number(result as f64, &ctx.ast);
        1
    }
}

/// Evaluates Math.fround: `Math.fround(1.5)` → `1.5`
#[derive(Default)]
pub struct Fround;

impl Fround {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Some(n) = extract_math_call(expr, "fround") else {
            return 0;
        };
        // Convert to f32 and back to f64
        let result = (n as f32) as f64;
        *expr = literal::make_number(result, &ctx.ast);
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
// Trig Group
// ============================================================================

/// Group of all Math trig and misc passes.
#[derive(Default)]
pub struct Trig {
    sin: Sin,
    cos: Cos,
    tan: Tan,
    log: Log,
    exp: Exp,
    hypot: Hypot,
    imul: Imul,
    clz32: Clz32,
    fround: Fround,
}

impl Trig {
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
        mods += self.sin.transform(expr, ctx);
        mods += self.cos.transform(expr, ctx);
        mods += self.tan.transform(expr, ctx);
        mods += self.log.transform(expr, ctx);
        mods += self.exp.transform(expr, ctx);
        mods += self.hypot.transform(expr, ctx);
        mods += self.imul.transform(expr, ctx);
        mods += self.clz32.transform(expr, ctx);
        mods += self.fround.transform(expr, ctx);
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

        struct Visitor(Trig);
        impl<'a> oxc_traverse::Traverse<'a, ()> for Visitor {
            fn exit_expression(
                &mut self,
                expr: &mut Expression<'a>,
                ctx: &mut TraverseCtx<'a, ()>,
            ) {
                self.0.transform(expr, ctx);
            }
        }

        let mut visitor = Visitor(Trig::new());
        traverse_mut(&mut visitor, &allocator, &mut program, scoping, ());
        codegen(&program)
    }

    #[test]
    fn test_sin() {
        let result = transform("Math.sin(0)");
        assert!(result.contains('0'));
    }

    #[test]
    fn test_cos() {
        let result = transform("Math.cos(0)");
        assert!(result.contains('1'));
    }

    #[test]
    fn test_tan() {
        let result = transform("Math.tan(0)");
        assert!(result.contains('0'));
    }

    #[test]
    fn test_log() {
        let result = transform("Math.log(1)");
        assert!(result.contains('0'));
    }

    #[test]
    fn test_exp() {
        let result = transform("Math.exp(0)");
        assert!(result.contains('1'));
    }

    #[test]
    fn test_hypot() {
        let result = transform("Math.hypot(3, 4)");
        assert!(result.contains('5'));
    }

    #[test]
    fn test_imul() {
        let result = transform("Math.imul(2, 3)");
        assert!(result.contains('6'));
    }

    #[test]
    fn test_clz32() {
        let result = transform("Math.clz32(1)");
        assert!(result.contains("31"));
    }

    #[test]
    fn test_fround() {
        let result = transform("Math.fround(1.5)");
        assert!(result.contains("1.5"));
    }
}
