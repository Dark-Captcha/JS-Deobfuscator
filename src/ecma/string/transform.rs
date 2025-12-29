//! String transform passes.
//!
//! - `ToLowerCase` — `"ABC".toLowerCase()` → `"abc"`
//! - `ToUpperCase` — `"abc".toUpperCase()` → `"ABC"`
//! - `Trim` — `" x ".trim()` → `"x"`
//! - `TrimStart` — `" x".trimStart()` → `"x"`
//! - `TrimEnd` — `"x ".trimEnd()` → `"x"`

use oxc::ast::ast::Expression;
use oxc::span::SPAN;
use oxc_traverse::TraverseCtx;

use crate::utils::literal;

// ============================================================================
// Individual Passes
// ============================================================================

/// Evaluates toLowerCase: `"ABC".toLowerCase()` → `"abc"`
#[derive(Default)]
pub struct ToLowerCase;

impl ToLowerCase {
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
        if member.property.name.as_str() != "toLowerCase" {
            return 0;
        }
        if !call.arguments.is_empty() {
            return 0;
        }

        let Some(s) = literal::string(&member.object) else {
            return 0;
        };

        let result = s.to_lowercase();
        *expr = ctx
            .ast
            .expression_string_literal(SPAN, ctx.ast.atom(&result), None);
        1
    }
}

/// Evaluates toUpperCase: `"abc".toUpperCase()` → `"ABC"`
#[derive(Default)]
pub struct ToUpperCase;

impl ToUpperCase {
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
        if member.property.name.as_str() != "toUpperCase" {
            return 0;
        }
        if !call.arguments.is_empty() {
            return 0;
        }

        let Some(s) = literal::string(&member.object) else {
            return 0;
        };

        let result = s.to_uppercase();
        *expr = ctx
            .ast
            .expression_string_literal(SPAN, ctx.ast.atom(&result), None);
        1
    }
}

/// Evaluates trim: `" x ".trim()` → `"x"`
#[derive(Default)]
pub struct Trim;

impl Trim {
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
        if member.property.name.as_str() != "trim" {
            return 0;
        }
        if !call.arguments.is_empty() {
            return 0;
        }

        let Some(s) = literal::string(&member.object) else {
            return 0;
        };

        let result = s.trim();
        *expr = ctx
            .ast
            .expression_string_literal(SPAN, ctx.ast.atom(result), None);
        1
    }
}

/// Evaluates trimStart: `" x".trimStart()` → `"x"`
#[derive(Default)]
pub struct TrimStart;

impl TrimStart {
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
        let name = member.property.name.as_str();
        if name != "trimStart" && name != "trimLeft" {
            return 0;
        }
        if !call.arguments.is_empty() {
            return 0;
        }

        let Some(s) = literal::string(&member.object) else {
            return 0;
        };

        let result = s.trim_start();
        *expr = ctx
            .ast
            .expression_string_literal(SPAN, ctx.ast.atom(result), None);
        1
    }
}

/// Evaluates trimEnd: `"x ".trimEnd()` → `"x"`
#[derive(Default)]
pub struct TrimEnd;

impl TrimEnd {
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
        let name = member.property.name.as_str();
        if name != "trimEnd" && name != "trimRight" {
            return 0;
        }
        if !call.arguments.is_empty() {
            return 0;
        }

        let Some(s) = literal::string(&member.object) else {
            return 0;
        };

        let result = s.trim_end();
        *expr = ctx
            .ast
            .expression_string_literal(SPAN, ctx.ast.atom(result), None);
        1
    }
}

// ============================================================================
// Transform Group
// ============================================================================

/// Group of all string transform passes.
#[derive(Default)]
pub struct Transform {
    to_lower_case: ToLowerCase,
    to_upper_case: ToUpperCase,
    trim: Trim,
    trim_start: TrimStart,
    trim_end: TrimEnd,
}

impl Transform {
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
        mods += self.to_lower_case.transform(expr, ctx);
        mods += self.to_upper_case.transform(expr, ctx);
        mods += self.trim.transform(expr, ctx);
        mods += self.trim_start.transform(expr, ctx);
        mods += self.trim_end.transform(expr, ctx);
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

    fn transform_code(source: &str) -> String {
        let allocator = Allocator::default();
        let mut program = parse_program(&allocator, source);
        let scoping = SemanticBuilder::new()
            .build(&program)
            .semantic
            .into_scoping();

        struct Visitor(Transform);
        impl<'a> oxc_traverse::Traverse<'a, ()> for Visitor {
            fn exit_expression(
                &mut self,
                expr: &mut Expression<'a>,
                ctx: &mut TraverseCtx<'a, ()>,
            ) {
                self.0.transform(expr, ctx);
            }
        }

        let mut visitor = Visitor(Transform::new());
        traverse_mut(&mut visitor, &allocator, &mut program, scoping, ());
        codegen(&program)
    }

    #[test]
    fn test_to_lower_case() {
        let result = transform_code("\"ABC\".toLowerCase()");
        assert!(result.contains("\"abc\""));
    }

    #[test]
    fn test_to_upper_case() {
        let result = transform_code("\"abc\".toUpperCase()");
        assert!(result.contains("\"ABC\""));
    }

    #[test]
    fn test_trim() {
        let result = transform_code("\" x \".trim()");
        assert!(result.contains("\"x\""));
    }

    #[test]
    fn test_trim_start() {
        let result = transform_code("\" x\".trimStart()");
        assert!(result.contains("\"x\""));
    }

    #[test]
    fn test_trim_end() {
        let result = transform_code("\"x \".trimEnd()");
        assert!(result.contains("\"x\""));
    }
}
