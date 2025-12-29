//! String search passes.
//!
//! - `IndexOf` — `"hello".indexOf("l")` → `2`
//! - `LastIndexOf` — `"hello".lastIndexOf("l")` → `3`
//! - `Includes` — `"hello".includes("ell")` → `true`
//! - `StartsWith` — `"hello".startsWith("he")` → `true`
//! - `EndsWith` — `"hello".endsWith("lo")` → `true`

use oxc::ast::ast::{Argument, Expression, NumberBase};
use oxc::span::SPAN;
use oxc_traverse::TraverseCtx;

use crate::utils::literal;

// ============================================================================
// Individual Passes
// ============================================================================

/// Evaluates indexOf: `"hello".indexOf("l")` → `2`
#[derive(Default)]
pub struct IndexOf;

impl IndexOf {
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
        if member.property.name.as_str() != "indexOf" {
            return 0;
        }

        let Some(s) = literal::string(&member.object) else {
            return 0;
        };
        let Some(search) = call.arguments.first().and_then(|a| get_string_arg(a)) else {
            return 0;
        };

        let start = call
            .arguments
            .get(1)
            .and_then(|a| get_number_arg(a))
            .unwrap_or(0.0)
            .max(0.0) as usize;

        let result = if start >= s.len() {
            -1
        } else {
            s[start..]
                .find(search)
                .map(|i| (i + start) as i64)
                .unwrap_or(-1)
        };

        *expr = ctx
            .ast
            .expression_numeric_literal(SPAN, result as f64, None, NumberBase::Decimal);
        1
    }
}

/// Evaluates lastIndexOf: `"hello".lastIndexOf("l")` → `3`
#[derive(Default)]
pub struct LastIndexOf;

impl LastIndexOf {
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
        if member.property.name.as_str() != "lastIndexOf" {
            return 0;
        }

        let Some(s) = literal::string(&member.object) else {
            return 0;
        };
        let Some(search) = call.arguments.first().and_then(|a| get_string_arg(a)) else {
            return 0;
        };

        let result = s.rfind(search).map(|i| i as i64).unwrap_or(-1);
        *expr = ctx
            .ast
            .expression_numeric_literal(SPAN, result as f64, None, NumberBase::Decimal);
        1
    }
}

/// Evaluates includes: `"hello".includes("ell")` → `true`
#[derive(Default)]
pub struct Includes;

impl Includes {
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
        if member.property.name.as_str() != "includes" {
            return 0;
        }

        let Some(s) = literal::string(&member.object) else {
            return 0;
        };
        let Some(search) = call.arguments.first().and_then(|a| get_string_arg(a)) else {
            return 0;
        };

        let start = call
            .arguments
            .get(1)
            .and_then(|a| get_number_arg(a))
            .unwrap_or(0.0)
            .max(0.0) as usize;

        let result = if start >= s.len() {
            search.is_empty()
        } else {
            s[start..].contains(search)
        };

        *expr = ctx.ast.expression_boolean_literal(SPAN, result);
        1
    }
}

/// Evaluates startsWith: `"hello".startsWith("he")` → `true`
#[derive(Default)]
pub struct StartsWith;

impl StartsWith {
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
        if member.property.name.as_str() != "startsWith" {
            return 0;
        }

        let Some(s) = literal::string(&member.object) else {
            return 0;
        };
        let Some(search) = call.arguments.first().and_then(|a| get_string_arg(a)) else {
            return 0;
        };

        let start = call
            .arguments
            .get(1)
            .and_then(|a| get_number_arg(a))
            .unwrap_or(0.0)
            .max(0.0) as usize;

        let result = if start >= s.len() {
            search.is_empty()
        } else {
            s[start..].starts_with(search)
        };

        *expr = ctx.ast.expression_boolean_literal(SPAN, result);
        1
    }
}

/// Evaluates endsWith: `"hello".endsWith("lo")` → `true`
#[derive(Default)]
pub struct EndsWith;

impl EndsWith {
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
        if member.property.name.as_str() != "endsWith" {
            return 0;
        }

        let Some(s) = literal::string(&member.object) else {
            return 0;
        };
        let Some(search) = call.arguments.first().and_then(|a| get_string_arg(a)) else {
            return 0;
        };

        let result = match call.arguments.get(1).and_then(|a| get_number_arg(a)) {
            Some(end) => {
                let end = (end.max(0.0) as usize).min(s.len());
                s[..end].ends_with(search)
            }
            None => s.ends_with(search),
        };

        *expr = ctx.ast.expression_boolean_literal(SPAN, result);
        1
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn get_string_arg<'a>(arg: &'a Argument<'a>) -> Option<&'a str> {
    match arg {
        Argument::StringLiteral(lit) => Some(lit.value.as_str()),
        _ => arg.as_expression().and_then(literal::string),
    }
}

fn get_number_arg(arg: &Argument) -> Option<f64> {
    match arg {
        Argument::NumericLiteral(n) => Some(n.value),
        _ => arg.as_expression().and_then(literal::number),
    }
}

// ============================================================================
// Search Group
// ============================================================================

/// Group of all string search passes.
#[derive(Default)]
pub struct Search {
    index_of: IndexOf,
    last_index_of: LastIndexOf,
    includes: Includes,
    starts_with: StartsWith,
    ends_with: EndsWith,
}

impl Search {
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
        mods += self.index_of.transform(expr, ctx);
        mods += self.last_index_of.transform(expr, ctx);
        mods += self.includes.transform(expr, ctx);
        mods += self.starts_with.transform(expr, ctx);
        mods += self.ends_with.transform(expr, ctx);
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

        struct Visitor(Search);
        impl<'a> oxc_traverse::Traverse<'a, ()> for Visitor {
            fn exit_expression(
                &mut self,
                expr: &mut Expression<'a>,
                ctx: &mut TraverseCtx<'a, ()>,
            ) {
                self.0.transform(expr, ctx);
            }
        }

        let mut visitor = Visitor(Search::new());
        traverse_mut(&mut visitor, &allocator, &mut program, scoping, ());
        codegen(&program)
    }

    #[test]
    fn test_index_of() {
        assert_eq!(transform("\"hello\".indexOf(\"l\")"), "2;\n");
        assert_eq!(transform("\"hello\".indexOf(\"x\")"), "-1;\n");
    }

    #[test]
    fn test_last_index_of() {
        assert_eq!(transform("\"hello\".lastIndexOf(\"l\")"), "3;\n");
    }

    #[test]
    fn test_includes() {
        assert_eq!(transform("\"hello\".includes(\"ell\")"), "true;\n");
        assert_eq!(transform("\"hello\".includes(\"xyz\")"), "false;\n");
    }

    #[test]
    fn test_starts_with() {
        assert_eq!(transform("\"hello\".startsWith(\"he\")"), "true;\n");
        assert_eq!(transform("\"hello\".startsWith(\"lo\")"), "false;\n");
    }

    #[test]
    fn test_ends_with() {
        assert_eq!(transform("\"hello\".endsWith(\"lo\")"), "true;\n");
        assert_eq!(transform("\"hello\".endsWith(\"he\")"), "false;\n");
    }
}
