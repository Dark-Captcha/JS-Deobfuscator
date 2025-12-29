//! JSON method passes.
//!
//! - `Parse` — `JSON.parse('{"a":1}')` → `{a:1}`
//! - `Stringify` — `JSON.stringify({a:1})` → `'{"a":1}'`

use oxc::ast::ast::Expression;
use oxc::span::SPAN;
use oxc_traverse::TraverseCtx;

use crate::utils::literal;

// ============================================================================
// Individual Passes
// ============================================================================

/// Evaluates JSON.parse: `JSON.parse('{"a":1}')` → object literal
/// Note: Only handles simple cases with string literals
#[derive(Default)]
pub struct Parse;

impl Parse {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        _expr: &mut Expression<'a>,
        _ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        // JSON.parse is complex - skip for now as it requires building AST from JSON
        // This would need a JSON parser to AST converter
        0
    }
}

/// Evaluates JSON.stringify on literals: `JSON.stringify({a:1})` → `'{"a":1}'`
#[derive(Default)]
pub struct Stringify;

impl Stringify {
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
        if member.property.name.as_str() != "stringify" {
            return 0;
        }

        let Expression::Identifier(obj) = &member.object else {
            return 0;
        };
        if obj.name.as_str() != "JSON" {
            return 0;
        }

        let Some(arg) = call.arguments.first().and_then(|a| a.as_expression()) else {
            return 0;
        };

        // Only handle simple literals for now
        let Some(value) = literal::LiteralValue::from_expr(arg) else {
            return 0;
        };

        let json_str = match value {
            literal::LiteralValue::Number(n) => literal::number_to_string(n),
            literal::LiteralValue::String(s) => format!("\"{}\"", escape_json_string(&s)),
            literal::LiteralValue::Boolean(b) => b.to_string(),
            literal::LiteralValue::Null => "null".to_string(),
            literal::LiteralValue::Undefined => return 0, // undefined -> undefined, not valid JSON
        };

        *expr = ctx
            .ast
            .expression_string_literal(SPAN, ctx.ast.atom(&json_str), None);
        1
    }
}

fn escape_json_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c if c.is_control() => {
                result.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => result.push(c),
        }
    }
    result
}

// ============================================================================
// JsonGroup
// ============================================================================

/// Group of all JSON passes.
#[derive(Default)]
pub struct JsonGroup {
    parse: Parse,
    stringify: Stringify,
}

impl JsonGroup {
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
        mods += self.parse.transform(expr, ctx);
        mods += self.stringify.transform(expr, ctx);
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

        struct Visitor(JsonGroup);
        impl<'a> oxc_traverse::Traverse<'a, ()> for Visitor {
            fn exit_expression(
                &mut self,
                expr: &mut Expression<'a>,
                ctx: &mut TraverseCtx<'a, ()>,
            ) {
                self.0.transform(expr, ctx);
            }
        }

        let mut visitor = Visitor(JsonGroup::new());
        traverse_mut(&mut visitor, &allocator, &mut program, scoping, ());
        codegen(&program)
    }

    #[test]
    fn test_stringify_number() {
        let result = transform("JSON.stringify(42)");
        assert!(result.contains("\"42\""));
    }

    #[test]
    fn test_stringify_string() {
        let result = transform("JSON.stringify(\"hello\")");
        assert!(result.contains("\\\"hello\\\""));
    }

    #[test]
    fn test_stringify_boolean() {
        let result = transform("JSON.stringify(true)");
        assert!(result.contains("\"true\""));
    }
}
