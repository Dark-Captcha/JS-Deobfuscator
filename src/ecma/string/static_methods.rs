//! String static method passes.
//!
//! - `FromCharCode` — `String.fromCharCode(65)` → `"A"`
//! - `FromCodePoint` — `String.fromCodePoint(128512)` → `"😀"`

use oxc::ast::ast::{Argument, Expression};
use oxc::span::SPAN;
use oxc_traverse::TraverseCtx;

use crate::utils::literal;

// ============================================================================
// Individual Passes
// ============================================================================

/// Evaluates String.fromCharCode: `String.fromCharCode(65)` → `"A"`
#[derive(Default)]
pub struct FromCharCode;

impl FromCharCode {
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
        if member.property.name.as_str() != "fromCharCode" {
            return 0;
        }

        // Check for String.fromCharCode
        let Expression::Identifier(obj) = &member.object else {
            return 0;
        };
        if obj.name.as_str() != "String" {
            return 0;
        }

        let mut result = String::new();
        for arg in &call.arguments {
            let Some(code) = get_number_arg(arg) else {
                return 0;
            };
            // JavaScript uses UTF-16, so we mask to 16 bits
            let code = (code as u32) & 0xFFFF;
            result.push(char::from_u32(code).unwrap_or('\u{FFFD}'));
        }

        *expr = ctx
            .ast
            .expression_string_literal(SPAN, ctx.ast.atom(&result), None);
        1
    }
}

/// Evaluates String.fromCodePoint: `String.fromCodePoint(128512)` → `"😀"`
#[derive(Default)]
pub struct FromCodePoint;

impl FromCodePoint {
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
        if member.property.name.as_str() != "fromCodePoint" {
            return 0;
        }

        // Check for String.fromCodePoint
        let Expression::Identifier(obj) = &member.object else {
            return 0;
        };
        if obj.name.as_str() != "String" {
            return 0;
        }

        let mut result = String::new();
        for arg in &call.arguments {
            let Some(code) = get_number_arg(arg) else {
                return 0;
            };
            let code = code as u32;
            // fromCodePoint throws for invalid code points
            let Some(c) = char::from_u32(code) else {
                return 0;
            };
            result.push(c);
        }

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

// ============================================================================
// StaticMethods Group
// ============================================================================

/// Group of all String static method passes.
#[derive(Default)]
pub struct StaticMethods {
    from_char_code: FromCharCode,
    from_code_point: FromCodePoint,
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
        mods += self.from_char_code.transform(expr, ctx);
        mods += self.from_code_point.transform(expr, ctx);
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
    fn test_from_char_code() {
        let result = transform("String.fromCharCode(65)");
        assert!(result.contains("\"A\""));
    }

    #[test]
    fn test_from_char_code_multiple() {
        let result = transform("String.fromCharCode(72, 105)");
        assert!(result.contains("\"Hi\""));
    }

    #[test]
    fn test_from_code_point() {
        let result = transform("String.fromCodePoint(65)");
        assert!(result.contains("\"A\""));
    }

    #[test]
    fn test_from_code_point_emoji() {
        let result = transform("String.fromCodePoint(128512)");
        assert!(result.contains("😀") || result.contains("\\u"));
    }
}
