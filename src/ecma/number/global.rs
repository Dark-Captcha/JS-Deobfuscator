//! Global number function passes.
//!
//! - `GlobalIsNaN` — `isNaN(NaN)` → `true`
//! - `GlobalIsFinite` — `isFinite(1)` → `true`
//! - `ParseInt` — `parseInt("42")` → `42`
//! - `ParseFloat` — `parseFloat("3.14")` → `3.14`

use oxc::ast::ast::{Argument, Expression};
use oxc::span::SPAN;
use oxc_traverse::TraverseCtx;

use crate::utils::literal;

// ============================================================================
// Individual Passes
// ============================================================================

/// Evaluates global isNaN: `isNaN(NaN)` → `true`
/// Note: Global isNaN coerces to number first, unlike Number.isNaN
#[derive(Default)]
pub struct GlobalIsNaN;

impl GlobalIsNaN {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Expression::CallExpression(call) = expr else {
            return 0;
        };
        let Expression::Identifier(callee) = &call.callee else {
            return 0;
        };
        if callee.name.as_str() != "isNaN" {
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

        let Some(n) = get_number_arg(arg) else {
            return 0;
        };
        *expr = ctx.ast.expression_boolean_literal(SPAN, n.is_nan());
        1
    }
}

/// Evaluates global isFinite: `isFinite(1)` → `true`
#[derive(Default)]
pub struct GlobalIsFinite;

impl GlobalIsFinite {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Expression::CallExpression(call) = expr else {
            return 0;
        };
        let Expression::Identifier(callee) = &call.callee else {
            return 0;
        };
        if callee.name.as_str() != "isFinite" {
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
        *expr = ctx.ast.expression_boolean_literal(SPAN, n.is_finite());
        1
    }
}

/// Evaluates parseInt: `parseInt("42")` → `42`
#[derive(Default)]
pub struct ParseInt;

impl ParseInt {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Expression::CallExpression(call) = expr else {
            return 0;
        };
        let Expression::Identifier(callee) = &call.callee else {
            return 0;
        };
        if callee.name.as_str() != "parseInt" {
            return 0;
        }

        let Some(s) = call.arguments.first().and_then(get_string_arg) else {
            return 0;
        };

        // Get radix (default depends on string prefix)
        let radix = call
            .arguments
            .get(1)
            .and_then(get_number_arg)
            .map(|n| n as u32);

        let Some(result) = parse_int_impl(s, radix) else {
            return 0;
        };
        *expr = literal::make_number(result, &ctx.ast);
        1
    }
}

/// Evaluates parseFloat: `parseFloat("3.14")` → `3.14`
#[derive(Default)]
pub struct ParseFloat;

impl ParseFloat {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Expression::CallExpression(call) = expr else {
            return 0;
        };
        let Expression::Identifier(callee) = &call.callee else {
            return 0;
        };
        if callee.name.as_str() != "parseFloat" {
            return 0;
        }

        let Some(s) = call.arguments.first().and_then(get_string_arg) else {
            return 0;
        };

        let trimmed = s.trim_start();
        if trimmed.is_empty() {
            return 0;
        }

        // Try to parse as float
        let result: f64 = match trimmed.parse() {
            Ok(n) => n,
            Err(_) => {
                // Try parsing prefix
                let end = trimmed
                    .find(|c: char| {
                        !c.is_ascii_digit()
                            && c != '.'
                            && c != '-'
                            && c != '+'
                            && c != 'e'
                            && c != 'E'
                    })
                    .unwrap_or(trimmed.len());
                match trimmed[..end].parse() {
                    Ok(n) => n,
                    Err(_) => return 0,
                }
            }
        };

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

fn get_string_arg<'a>(arg: &'a Argument<'a>) -> Option<&'a str> {
    match arg {
        Argument::StringLiteral(lit) => Some(lit.value.as_str()),
        _ => arg.as_expression().and_then(literal::string),
    }
}

fn parse_int_impl(s: &str, radix: Option<u32>) -> Option<f64> {
    let trimmed = s.trim_start();
    if trimmed.is_empty() {
        return None;
    }

    let (negative, rest) = if let Some(s) = trimmed.strip_prefix('-') {
        (true, s)
    } else if let Some(s) = trimmed.strip_prefix('+') {
        (false, s)
    } else {
        (false, trimmed)
    };

    let (radix, rest) = match radix {
        Some(r) if (2..=36).contains(&r) => (r, rest),
        Some(0) | None => {
            if let Some(s) = rest.strip_prefix("0x").or_else(|| rest.strip_prefix("0X")) {
                (16, s)
            } else {
                (10, rest)
            }
        }
        _ => return None,
    };

    // Parse digits
    let end = rest
        .find(|c: char| !c.is_digit(radix))
        .unwrap_or(rest.len());
    if end == 0 {
        return None;
    }

    let value = i64::from_str_radix(&rest[..end], radix).ok()?;
    Some(if negative {
        -(value as f64)
    } else {
        value as f64
    })
}

// ============================================================================
// Global Group
// ============================================================================

/// Group of all global number function passes.
#[derive(Default)]
pub struct Global {
    is_nan: GlobalIsNaN,
    is_finite: GlobalIsFinite,
    parse_int: ParseInt,
    parse_float: ParseFloat,
}

impl Global {
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
        mods += self.parse_int.transform(expr, ctx);
        mods += self.parse_float.transform(expr, ctx);
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

        struct Visitor(Global);
        impl<'a> oxc_traverse::Traverse<'a, ()> for Visitor {
            fn exit_expression(
                &mut self,
                expr: &mut Expression<'a>,
                ctx: &mut TraverseCtx<'a, ()>,
            ) {
                self.0.transform(expr, ctx);
            }
        }

        let mut visitor = Visitor(Global::new());
        traverse_mut(&mut visitor, &allocator, &mut program, scoping, ());
        codegen(&program)
    }

    #[test]
    fn test_global_is_nan() {
        let result = transform("isNaN(NaN)");
        assert!(result.contains("true"));
    }

    #[test]
    fn test_global_is_finite() {
        let result = transform("isFinite(42)");
        assert!(result.contains("true"));
    }

    #[test]
    fn test_parse_int() {
        let result = transform("parseInt(\"42\")");
        assert!(result.contains("42"));
    }

    #[test]
    fn test_parse_int_hex() {
        let result = transform("parseInt(\"0xff\")");
        assert!(result.contains("255"));
    }

    #[test]
    fn test_parse_int_radix() {
        let result = transform("parseInt(\"ff\", 16)");
        assert!(result.contains("255"));
    }

    #[test]
    fn test_parse_float() {
        let result = transform("parseFloat(\"3.14\")");
        assert!(result.contains("3.14"));
    }
}
