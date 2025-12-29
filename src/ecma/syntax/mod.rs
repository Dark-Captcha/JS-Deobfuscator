//! Syntax simplification passes.
//!
//! - `ParenthesesUnwrap` — `(x)` → `x`
//! - `SequenceSimplify` — `(a, b, c)` → `c` (if a,b are side-effect free)

use oxc::allocator::CloneIn;
use oxc::ast::ast::Expression;
use oxc_traverse::TraverseCtx;

use crate::utils::side_effects;

// ============================================================================
// Individual Passes
// ============================================================================

/// Unwraps unnecessary parentheses: `(x)` → `x`
#[derive(Default)]
pub struct ParenthesesUnwrap;

impl ParenthesesUnwrap {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        _ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Expression::ParenthesizedExpression(paren) = expr else {
            return 0;
        };

        // Clone the inner expression
        let inner = paren.expression.clone_in(_ctx.ast.allocator);
        *expr = inner;
        1
    }
}

/// Simplifies sequence expressions: `(a, b, c)` → `c` if a,b are pure
#[derive(Default)]
pub struct SequenceSimplify;

impl SequenceSimplify {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Expression::SequenceExpression(seq) = expr else {
            return 0;
        };

        if seq.expressions.len() <= 1 {
            return 0;
        }

        // Check if all but last expression are side-effect free
        let last_idx = seq.expressions.len() - 1;
        for (i, e) in seq.expressions.iter().enumerate() {
            if i < last_idx && !side_effects::is_side_effect_free(e) {
                return 0;
            }
        }

        // Replace with last expression
        let last = seq.expressions[last_idx].clone_in(ctx.ast.allocator);
        *expr = last;
        1
    }
}

// ============================================================================
// SyntaxGroup
// ============================================================================

/// Group of all syntax simplification passes.
#[derive(Default)]
pub struct SyntaxGroup {
    parentheses_unwrap: ParenthesesUnwrap,
    sequence_simplify: SequenceSimplify,
}

impl SyntaxGroup {
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
        mods += self.parentheses_unwrap.transform(expr, ctx);
        mods += self.sequence_simplify.transform(expr, ctx);
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

        struct Visitor(SyntaxGroup);
        impl<'a> oxc_traverse::Traverse<'a, ()> for Visitor {
            fn exit_expression(
                &mut self,
                expr: &mut Expression<'a>,
                ctx: &mut TraverseCtx<'a, ()>,
            ) {
                self.0.transform(expr, ctx);
            }
        }

        let mut visitor = Visitor(SyntaxGroup::new());
        traverse_mut(&mut visitor, &allocator, &mut program, scoping, ());
        codegen(&program)
    }

    #[test]
    fn test_parentheses_unwrap() {
        let result = transform("(42)");
        assert!(result.contains("42") && !result.contains('('));
    }

    #[test]
    fn test_sequence_simplify() {
        let result = transform("(1, 2, 3)");
        assert!(result.contains('3'));
    }

    #[test]
    fn test_sequence_with_side_effects_unchanged() {
        // foo() has side effects, so sequence should not be simplified
        let result = transform("(foo(), 2)");
        assert!(result.contains("foo()"));
    }
}
