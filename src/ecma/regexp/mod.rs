//! RegExp method passes.
//!
//! - `Test` — `/a/.test("abc")` → `true`

use oxc::ast::ast::Expression;
use oxc::span::SPAN;
use oxc_traverse::TraverseCtx;

use crate::utils::literal;

// ============================================================================
// Individual Passes
// ============================================================================

/// Evaluates RegExp.test: `/a/.test("abc")` → `true`
#[derive(Default)]
pub struct Test;

impl Test {
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
        if member.property.name.as_str() != "test" {
            return 0;
        }

        // Get the regex
        let Expression::RegExpLiteral(regex) = &member.object else {
            return 0;
        };

        // Get the test string
        let Some(test_str) = call
            .arguments
            .first()
            .and_then(|a| a.as_expression())
            .and_then(literal::string)
        else {
            return 0;
        };

        // Try to match using Rust's regex
        let pattern = regex.regex.pattern.text.as_str();
        let flags = regex.regex.flags;

        // Build regex with flags
        let mut regex_str = String::new();
        if flags.contains(oxc::ast::ast::RegExpFlags::I) {
            regex_str.push_str("(?i)");
        }
        regex_str.push_str(pattern);

        let Ok(re) = regex::Regex::new(&regex_str) else {
            return 0;
        };
        let result = re.is_match(test_str);

        *expr = ctx.ast.expression_boolean_literal(SPAN, result);
        1
    }
}

// ============================================================================
// RegExpGroup
// ============================================================================

/// Group of all RegExp passes.
#[derive(Default)]
pub struct RegExpGroup {
    test: Test,
}

impl RegExpGroup {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        self.test.transform(expr, ctx)
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

        struct Visitor(RegExpGroup);
        impl<'a> oxc_traverse::Traverse<'a, ()> for Visitor {
            fn exit_expression(
                &mut self,
                expr: &mut Expression<'a>,
                ctx: &mut TraverseCtx<'a, ()>,
            ) {
                self.0.transform(expr, ctx);
            }
        }

        let mut visitor = Visitor(RegExpGroup::new());
        traverse_mut(&mut visitor, &allocator, &mut program, scoping, ());
        codegen(&program)
    }

    #[test]
    fn test_regex_test_true() {
        let result = transform("/a/.test(\"abc\")");
        assert!(result.contains("true"));
    }

    #[test]
    fn test_regex_test_false() {
        let result = transform("/x/.test(\"abc\")");
        assert!(result.contains("false"));
    }

    #[test]
    fn test_regex_test_case_insensitive() {
        let result = transform("/A/i.test(\"abc\")");
        assert!(result.contains("true"));
    }
}
