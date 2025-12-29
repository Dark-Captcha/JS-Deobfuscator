//! Number instance method passes.
//!
//! - `ToFixed` — `(3.14159).toFixed(2)` → `"3.14"`
//! - `ToString` — `(255).toString(16)` → `"ff"`

use oxc::ast::ast::{Argument, Expression};
use oxc::span::SPAN;
use oxc_traverse::TraverseCtx;

use crate::utils::literal;

// ============================================================================
// Individual Passes
// ============================================================================

/// Evaluates toFixed: `(3.14159).toFixed(2)` → `"3.14"`
#[derive(Default)]
pub struct ToFixed;

impl ToFixed {
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
        if member.property.name.as_str() != "toFixed" {
            return 0;
        }

        let Some(n) = literal::number(&member.object) else {
            return 0;
        };

        // Get digits (default 0)
        let digits = call
            .arguments
            .first()
            .and_then(get_number_arg)
            .unwrap_or(0.0) as usize;

        // toFixed throws for digits > 100
        if digits > 100 {
            return 0;
        }

        // Handle special cases
        if n.is_nan() {
            *expr = ctx
                .ast
                .expression_string_literal(SPAN, ctx.ast.atom("NaN"), None);
            return 1;
        }
        if n.is_infinite() {
            let s = if n.is_sign_positive() {
                "Infinity"
            } else {
                "-Infinity"
            };
            *expr = ctx
                .ast
                .expression_string_literal(SPAN, ctx.ast.atom(s), None);
            return 1;
        }

        let result = format!("{:.prec$}", n, prec = digits);
        *expr = ctx
            .ast
            .expression_string_literal(SPAN, ctx.ast.atom(&result), None);
        1
    }
}

/// Evaluates toString with radix: `(255).toString(16)` → `"ff"`
#[derive(Default)]
pub struct ToString;

impl ToString {
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
        if member.property.name.as_str() != "toString" {
            return 0;
        }

        let Some(n) = literal::number(&member.object) else {
            return 0;
        };

        // Get radix (default 10)
        let radix = call
            .arguments
            .first()
            .and_then(get_number_arg)
            .unwrap_or(10.0) as u32;

        // Valid radix is 2-36
        if !(2..=36).contains(&radix) {
            return 0;
        }

        // Handle special cases
        if n.is_nan() {
            *expr = ctx
                .ast
                .expression_string_literal(SPAN, ctx.ast.atom("NaN"), None);
            return 1;
        }
        if n.is_infinite() {
            let s = if n.is_sign_positive() {
                "Infinity"
            } else {
                "-Infinity"
            };
            *expr = ctx
                .ast
                .expression_string_literal(SPAN, ctx.ast.atom(s), None);
            return 1;
        }

        // For non-base-10, only handle integers
        if radix != 10 {
            if n.fract() != 0.0 {
                return 0;
            }
            let int_val = n as i64;
            let result = if int_val < 0 {
                format!("-{}", to_radix_string(int_val.unsigned_abs(), radix))
            } else {
                to_radix_string(int_val as u64, radix)
            };
            *expr = ctx
                .ast
                .expression_string_literal(SPAN, ctx.ast.atom(&result), None);
            return 1;
        }

        // Base 10
        let result = literal::number_to_string(n);
        *expr = ctx
            .ast
            .expression_string_literal(SPAN, ctx.ast.atom(&result), None);
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

fn to_radix_string(mut n: u64, radix: u32) -> String {
    if n == 0 {
        return "0".to_string();
    }

    const DIGITS: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    let mut result = Vec::new();

    while n > 0 {
        result.push(DIGITS[(n % radix as u64) as usize]);
        n /= radix as u64;
    }

    result.reverse();
    String::from_utf8(result).unwrap()
}

// ============================================================================
// Instance Group
// ============================================================================

/// Group of all Number instance method passes.
#[derive(Default)]
pub struct Instance {
    to_fixed: ToFixed,
    to_string: ToString,
}

impl Instance {
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
        mods += self.to_fixed.transform(expr, ctx);
        mods += self.to_string.transform(expr, ctx);
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

        struct Visitor(Instance);
        impl<'a> oxc_traverse::Traverse<'a, ()> for Visitor {
            fn exit_expression(
                &mut self,
                expr: &mut Expression<'a>,
                ctx: &mut TraverseCtx<'a, ()>,
            ) {
                self.0.transform(expr, ctx);
            }
        }

        let mut visitor = Visitor(Instance::new());
        traverse_mut(&mut visitor, &allocator, &mut program, scoping, ());
        codegen(&program)
    }

    #[test]
    fn test_to_fixed() {
        let result = transform("(3.14159).toFixed(2)");
        assert!(result.contains("\"3.14\""));
    }

    #[test]
    fn test_to_fixed_zero() {
        let result = transform("(3.7).toFixed(0)");
        assert!(result.contains("\"4\""));
    }

    #[test]
    fn test_to_string_base10() {
        let result = transform("(255).toString()");
        assert!(result.contains("\"255\""));
    }

    #[test]
    fn test_to_string_hex() {
        let result = transform("(255).toString(16)");
        assert!(result.contains("\"ff\""));
    }

    #[test]
    fn test_to_string_binary() {
        let result = transform("(5).toString(2)");
        assert!(result.contains("\"101\""));
    }
}
