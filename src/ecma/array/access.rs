//! Array access passes.
//!
//! - `At` — `[1,2,3].at(-1)` → `3`
//! - `Length` — `[1,2,3].length` → `3`
//! - `BracketAccess` — `[1,2,3][1]` → `2`

use oxc::ast::ast::{Argument, Expression};
use oxc_traverse::TraverseCtx;

use crate::utils::literal;

// ============================================================================
// Individual Passes
// ============================================================================

/// Evaluates at: `[1,2,3].at(-1)` → `3`
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

        let Some(elements) = literal::array_elements(&member.object) else {
            return 0;
        };
        let Some(index) = call.arguments.first().and_then(get_number_arg) else {
            return 0;
        };

        let len = elements.len() as i64;
        let idx = index as i64;
        let actual_idx = if idx < 0 { len + idx } else { idx };

        if actual_idx < 0 || actual_idx >= len {
            *expr = literal::make_undefined(&ctx.ast);
            return 1;
        }

        let element = elements[actual_idx as usize];
        if let Some(value) = literal::LiteralValue::from_expr(element) {
            *expr = value.into_expr(&ctx.ast);
            return 1;
        }
        0
    }
}

/// Evaluates length: `[1,2,3].length` → `3`
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

        let Some(elements) = literal::array_elements(&member.object) else {
            return 0;
        };
        let len = elements.len() as f64;

        *expr = literal::make_number(len, &ctx.ast);
        1
    }
}

/// Evaluates bracket access: `[1,2,3][1]` → `2`
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

        let Some(elements) = literal::array_elements(&member.object) else {
            return 0;
        };
        let Some(index) = literal::number(&member.expression) else {
            return 0;
        };

        let idx = index as usize;
        if idx >= elements.len() {
            *expr = literal::make_undefined(&ctx.ast);
            return 1;
        }

        let element = elements[idx];
        if let Some(value) = literal::LiteralValue::from_expr(element) {
            *expr = value.into_expr(&ctx.ast);
            return 1;
        }
        0
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

/// Group of all array access passes.
#[derive(Default)]
pub struct Access {
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
    fn test_at_positive() {
        let result = transform("[1, 2, 3].at(1)");
        assert!(result.contains('2'));
    }

    #[test]
    fn test_at_negative() {
        let result = transform("[1, 2, 3].at(-1)");
        assert!(result.contains('3'));
    }

    #[test]
    fn test_length() {
        let result = transform("[1, 2, 3].length");
        assert!(result.contains('3'));
    }

    #[test]
    fn test_bracket_access() {
        let result = transform("[1, 2, 3][1]");
        assert!(result.contains('2'));
    }

    #[test]
    fn test_bracket_access_string() {
        let result = transform("[\"a\", \"b\", \"c\"][0]");
        assert!(result.contains("\"a\""));
    }
}
