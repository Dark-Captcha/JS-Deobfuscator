//! Number static method passes.
//!
//! - `IsNaN` — `Number.isNaN(NaN)` → `true`
//! - `IsFinite` — `Number.isFinite(1)` → `true`
//! - `IsInteger` — `Number.isInteger(1)` → `true`
//! - `IsSafeInteger` — `Number.isSafeInteger(1)` → `true`

use oxc::ast::ast::{Argument, Expression};
use oxc::span::SPAN;
use oxc_traverse::TraverseCtx;

use crate::utils::literal;

// ============================================================================
// Individual Passes
// ============================================================================

/// Evaluates Number.isNaN: `Number.isNaN(NaN)` → `true`
/// Note: Number.isNaN is stricter than global isNaN - only returns true for actual NaN
#[derive(Default)]
pub struct IsNaN;

impl IsNaN {
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
        if member.property.name.as_str() != "isNaN" {
            return 0;
        }

        let Expression::Identifier(obj) = &member.object else {
            return 0;
        };
        if obj.name.as_str() != "Number" {
            return 0;
        }

        let Some(arg) = call.arguments.first() else {
            return 0;
        };

        // Check for NaN identifier
        if let Some(Expression::Identifier(ident)) = arg.as_expression()
            && ident.name.as_str() == "NaN"
        {
            *expr = ctx.ast.expression_boolean_literal(SPAN, true);
            return 1;
        }

        // Check for numeric literal
        let Some(n) = get_number_arg(arg) else {
            return 0;
        };
        let result = n.is_nan();
        *expr = ctx.ast.expression_boolean_literal(SPAN, result);
        1
    }
}

/// Evaluates Number.isFinite: `Number.isFinite(1)` → `true`
#[derive(Default)]
pub struct IsFinite;

impl IsFinite {
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
        if member.property.name.as_str() != "isFinite" {
            return 0;
        }

        let Expression::Identifier(obj) = &member.object else {
            return 0;
        };
        if obj.name.as_str() != "Number" {
            return 0;
        }

        let Some(arg) = call.arguments.first() else {
            return 0;
        };

        // Check for Infinity/NaN identifiers
        if let Some(Expression::Identifier(ident)) = arg.as_expression() {
            match ident.name.as_str() {
                "Infinity" | "NaN" => {
                    *expr = ctx.ast.expression_boolean_literal(SPAN, false);
                    return 1;
                }
                _ => {}
            }
        }

        let Some(n) = get_number_arg(arg) else {
            return 0;
        };
        let result = n.is_finite();
        *expr = ctx.ast.expression_boolean_literal(SPAN, result);
        1
    }
}

/// Evaluates Number.isInteger: `Number.isInteger(1)` → `true`
#[derive(Default)]
pub struct IsInteger;

impl IsInteger {
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
        if member.property.name.as_str() != "isInteger" {
            return 0;
        }

        let Expression::Identifier(obj) = &member.object else {
            return 0;
        };
        if obj.name.as_str() != "Number" {
            return 0;
        }

        let Some(n) = call.arguments.first().and_then(get_number_arg) else {
            return 0;
        };
        let result = n.is_finite() && n.fract() == 0.0;
        *expr = ctx.ast.expression_boolean_literal(SPAN, result);
        1
    }
}

/// Evaluates Number.isSafeInteger: `Number.isSafeInteger(1)` → `true`
#[derive(Default)]
pub struct IsSafeInteger;

impl IsSafeInteger {
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
        if member.property.name.as_str() != "isSafeInteger" {
            return 0;
        }

        let Expression::Identifier(obj) = &member.object else {
            return 0;
        };
        if obj.name.as_str() != "Number" {
            return 0;
        }

        let Some(n) = call.arguments.first().and_then(get_number_arg) else {
            return 0;
        };

        // Safe integers are in range -(2^53 - 1) to 2^53 - 1
        const MAX_SAFE: f64 = 9007199254740991.0; // 2^53 - 1
        let result = n.is_finite() && n.fract() == 0.0 && n.abs() <= MAX_SAFE;
        *expr = ctx.ast.expression_boolean_literal(SPAN, result);
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
// StaticMethods Group
// ============================================================================

/// Group of all Number static method passes.
#[derive(Default)]
pub struct StaticMethods {
    is_nan: IsNaN,
    is_finite: IsFinite,
    is_integer: IsInteger,
    is_safe_integer: IsSafeInteger,
}

impl StaticMethods {
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
        mods += self.is_nan.transform(expr, ctx);
        mods += self.is_finite.transform(expr, ctx);
        mods += self.is_integer.transform(expr, ctx);
        mods += self.is_safe_integer.transform(expr, ctx);
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

        struct Visitor(StaticMethods);
        impl<'a> oxc_traverse::Traverse<'a, ()> for Visitor {
            fn exit_expression(
                &mut self,
                expr: &mut Expression<'a>,
                ctx: &mut TraverseCtx<'a, ()>,
            ) {
                self.0.transform(expr, ctx);
            }
        }

        let mut visitor = Visitor(StaticMethods::new());
        traverse_mut(&mut visitor, &allocator, &mut program, scoping, ());
        codegen(&program)
    }

    #[test]
    fn test_is_nan_true() {
        let result = transform("Number.isNaN(NaN)");
        assert!(result.contains("true"));
    }

    #[test]
    fn test_is_nan_false() {
        let result = transform("Number.isNaN(42)");
        assert!(result.contains("false"));
    }

    #[test]
    fn test_is_finite_true() {
        let result = transform("Number.isFinite(42)");
        assert!(result.contains("true"));
    }

    #[test]
    fn test_is_finite_false() {
        let result = transform("Number.isFinite(Infinity)");
        assert!(result.contains("false"));
    }

    #[test]
    fn test_is_integer_true() {
        let result = transform("Number.isInteger(42)");
        assert!(result.contains("true"));
    }

    #[test]
    fn test_is_integer_false() {
        let result = transform("Number.isInteger(3.14)");
        assert!(result.contains("false"));
    }

    #[test]
    fn test_is_safe_integer_true() {
        let result = transform("Number.isSafeInteger(42)");
        assert!(result.contains("true"));
    }

    #[test]
    fn test_is_safe_integer_false() {
        let result = transform("Number.isSafeInteger(9007199254740992)");
        assert!(result.contains("false"));
    }
}
