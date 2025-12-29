//! String access passes.
//!
//! - `CharAt` — `"abc".charAt(1)` → `"b"`
//! - `CharCodeAt` — `"A".charCodeAt(0)` → `65`
//! - `At` — `"abc".at(-1)` → `"c"`
//! - `Length` — `"abc".length` → `3`
//! - `BracketAccess` — `"abc"[1]` → `"b"`

use oxc::ast::ast::{Argument, Expression, NumberBase};
use oxc::span::SPAN;
use oxc_traverse::TraverseCtx;

use crate::utils::literal;

// ============================================================================
// Individual Passes
// ============================================================================

/// Evaluates string charAt: `"abc".charAt(1)` → `"b"`
#[derive(Default)]
pub struct CharAt;

impl CharAt {
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
        if member.property.name.as_str() != "charAt" {
            return 0;
        }

        let Some(s) = literal::string(&member.object) else {
            return 0;
        };
        let idx = call
            .arguments
            .first()
            .and_then(|a| get_number_arg(a))
            .unwrap_or(0.0) as usize;

        let chars: Vec<char> = s.chars().collect();
        if idx >= chars.len() {
            *expr = ctx
                .ast
                .expression_string_literal(SPAN, ctx.ast.atom(""), None);
            return 1;
        }

        let result = chars[idx].to_string();
        *expr = ctx
            .ast
            .expression_string_literal(SPAN, ctx.ast.atom(&result), None);
        1
    }
}

/// Evaluates string charCodeAt: `"A".charCodeAt(0)` → `65`
#[derive(Default)]
pub struct CharCodeAt;

impl CharCodeAt {
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
        if member.property.name.as_str() != "charCodeAt" {
            return 0;
        }

        let Some(s) = literal::string(&member.object) else {
            return 0;
        };
        let idx = call
            .arguments
            .first()
            .and_then(|a| get_number_arg(a))
            .unwrap_or(0.0) as usize;

        let chars: Vec<char> = s.chars().collect();
        if idx >= chars.len() {
            // Returns NaN for out of bounds - skip transformation
            return 0;
        }

        let code = chars[idx] as u32;
        *expr = ctx
            .ast
            .expression_numeric_literal(SPAN, code as f64, None, NumberBase::Decimal);
        1
    }
}

/// Evaluates string at: `"abc".at(-1)` → `"c"`
#[derive(Default)]
pub struct At;

impl At {
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
        if member.property.name.as_str() != "at" {
            return 0;
        }

        let Some(s) = literal::string(&member.object) else {
            return 0;
        };
        let Some(idx) = call.arguments.first().and_then(|a| get_number_arg(a)) else {
            return 0;
        };

        let chars: Vec<char> = s.chars().collect();
        let len = chars.len() as i64;
        let idx = idx as i64;

        let actual_idx = if idx < 0 { len + idx } else { idx };
        if actual_idx < 0 || actual_idx >= len {
            // Returns undefined for out of bounds - skip
            return 0;
        }

        let result = chars[actual_idx as usize].to_string();
        *expr = ctx
            .ast
            .expression_string_literal(SPAN, ctx.ast.atom(&result), None);
        1
    }
}

/// Evaluates string length: `"abc".length` → `3`
#[derive(Default)]
pub struct Length;

impl Length {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Expression::StaticMemberExpression(member) = expr else {
            return 0;
        };
        if member.property.name.as_str() != "length" {
            return 0;
        }

        let Some(s) = literal::string(&member.object) else {
            return 0;
        };

        // JavaScript string length is UTF-16 code units, not chars
        // For ASCII, this is the same as chars().count()
        let len = s.encode_utf16().count();
        *expr = ctx
            .ast
            .expression_numeric_literal(SPAN, len as f64, None, NumberBase::Decimal);
        1
    }
}

/// Evaluates bracket access: `"abc"[1]` → `"b"`
#[derive(Default)]
pub struct BracketAccess;

impl BracketAccess {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Expression::ComputedMemberExpression(member) = expr else {
            return 0;
        };

        let Some(s) = literal::string(&member.object) else {
            return 0;
        };
        let Some(idx) = literal::number(&member.expression) else {
            return 0;
        };

        if idx < 0.0 || idx.fract() != 0.0 {
            return 0;
        }
        let idx = idx as usize;

        let chars: Vec<char> = s.chars().collect();
        if idx >= chars.len() {
            // Returns undefined for out of bounds - skip
            return 0;
        }

        let result = chars[idx].to_string();
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
// Access Group
// ============================================================================

/// Group of all string access passes.
#[derive(Default)]
pub struct Access {
    char_at: CharAt,
    char_code_at: CharCodeAt,
    at: At,
    length: Length,
    bracket_access: BracketAccess,
}

impl Access {
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
        mods += self.char_at.transform(expr, ctx);
        mods += self.char_code_at.transform(expr, ctx);
        mods += self.at.transform(expr, ctx);
        mods += self.length.transform(expr, ctx);
        mods += self.bracket_access.transform(expr, ctx);
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

        struct Visitor(Access);
        impl<'a> oxc_traverse::Traverse<'a, ()> for Visitor {
            fn exit_expression(
                &mut self,
                expr: &mut Expression<'a>,
                ctx: &mut TraverseCtx<'a, ()>,
            ) {
                self.0.transform(expr, ctx);
            }
        }

        let mut visitor = Visitor(Access::new());
        traverse_mut(&mut visitor, &allocator, &mut program, scoping, ());
        codegen(&program)
    }

    #[test]
    fn test_char_at() {
        let result = transform("\"abc\".charAt(1)");
        assert!(result.contains("\"b\""));
    }

    #[test]
    fn test_char_code_at() {
        assert_eq!(transform("\"A\".charCodeAt(0)"), "65;\n");
    }

    #[test]
    fn test_at_positive() {
        let result = transform("\"abc\".at(1)");
        assert!(result.contains("\"b\""));
    }

    #[test]
    fn test_at_negative() {
        let result = transform("\"abc\".at(-1)");
        assert!(result.contains("\"c\""));
    }

    #[test]
    fn test_length() {
        assert_eq!(transform("\"abc\".length"), "3;\n");
    }

    #[test]
    fn test_bracket_access() {
        let result = transform("\"abc\"[1]");
        assert!(result.contains("\"b\""));
    }
}
