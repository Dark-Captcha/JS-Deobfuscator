//! Ternary conditional operator pass.
//!
//! - `Ternary` — `true ? a : b` → `a`, `false ? a : b` → `b`

use oxc::allocator::TakeIn;
use oxc::ast::ast::Expression;
use oxc_traverse::TraverseCtx;

use crate::utils::literal;

// ============================================================================
// Ternary Pass
// ============================================================================

/// Simplifies ternary with known condition: `true ? a : b` → `a`
#[derive(Default)]
pub struct Ternary;

impl Ternary {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Expression::ConditionalExpression(cond) = expr else {
            return 0;
        };

        let Some(test_value) = literal::is_truthy(&cond.test) else {
            return 0;
        };

        let replacement = if test_value {
            cond.consequent.take_in(ctx.ast.allocator)
        } else {
            cond.alternate.take_in(ctx.ast.allocator)
        };

        *expr = replacement;
        1
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

        struct Visitor(Ternary);
        impl<'a> oxc_traverse::Traverse<'a, ()> for Visitor {
            fn exit_expression(
                &mut self,
                expr: &mut Expression<'a>,
                ctx: &mut TraverseCtx<'a, ()>,
            ) {
                self.0.transform(expr, ctx);
            }
        }

        let mut visitor = Visitor(Ternary);
        traverse_mut(&mut visitor, &allocator, &mut program, scoping, ());
        codegen(&program)
    }

    #[test]
    fn test_ternary_true() {
        assert_eq!(transform("true ? a : b"), "a;\n");
        assert_eq!(transform("1 ? a : b"), "a;\n");
        assert_eq!(transform("\"x\" ? a : b"), "a;\n");
    }

    #[test]
    fn test_ternary_false() {
        assert_eq!(transform("false ? a : b"), "b;\n");
        assert_eq!(transform("0 ? a : b"), "b;\n");
        assert_eq!(transform("\"\" ? a : b"), "b;\n");
        assert_eq!(transform("null ? a : b"), "b;\n");
    }

    #[test]
    fn test_ternary_unknown() {
        // Unknown condition - should not transform
        assert_eq!(transform("x ? a : b"), "x ? a : b;\n");
    }
}
