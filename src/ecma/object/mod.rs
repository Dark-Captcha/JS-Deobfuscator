//! Object static method passes.
//!
//! - `Keys` — `Object.keys({a:1})` → `["a"]`
//! - `Values` — `Object.values({a:1})` → `[1]`
//! - `Entries` — `Object.entries({a:1})` → `[["a",1]]`

use oxc::ast::ast::{ArrayExpressionElement, Expression, ObjectPropertyKind, PropertyKey};
use oxc::span::SPAN;
use oxc_traverse::TraverseCtx;

use crate::utils::literal::LiteralValue;

// ============================================================================
// Individual Passes
// ============================================================================

/// Evaluates Object.keys: `Object.keys({a:1})` → `["a"]`
#[derive(Default)]
pub struct Keys;

impl Keys {
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
        if member.property.name.as_str() != "keys" {
            return 0;
        }

        let Expression::Identifier(obj) = &member.object else {
            return 0;
        };
        if obj.name.as_str() != "Object" {
            return 0;
        }

        let Some(arg) = call.arguments.first().and_then(|a| a.as_expression()) else {
            return 0;
        };
        let Expression::ObjectExpression(obj_expr) = arg else {
            return 0;
        };

        let mut keys = ctx.ast.vec();
        for prop in &obj_expr.properties {
            let ObjectPropertyKind::ObjectProperty(p) = prop else {
                return 0;
            };
            let key_str = match &p.key {
                PropertyKey::StaticIdentifier(id) => id.name.to_string(),
                PropertyKey::StringLiteral(s) => s.value.to_string(),
                _ => return 0,
            };
            let key_expr = ctx
                .ast
                .expression_string_literal(SPAN, ctx.ast.atom(&key_str), None);
            keys.push(ArrayExpressionElement::from(key_expr));
        }

        *expr = ctx.ast.expression_array(SPAN, keys);
        1
    }
}

/// Evaluates Object.values: `Object.values({a:1})` → `[1]`
#[derive(Default)]
pub struct Values;

impl Values {
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
        if member.property.name.as_str() != "values" {
            return 0;
        }

        let Expression::Identifier(obj) = &member.object else {
            return 0;
        };
        if obj.name.as_str() != "Object" {
            return 0;
        }

        let Some(arg) = call.arguments.first().and_then(|a| a.as_expression()) else {
            return 0;
        };
        let Expression::ObjectExpression(obj_expr) = arg else {
            return 0;
        };

        let mut values = ctx.ast.vec();
        for prop in &obj_expr.properties {
            let ObjectPropertyKind::ObjectProperty(p) = prop else {
                return 0;
            };
            let Some(value) = LiteralValue::from_expr(&p.value) else {
                return 0;
            };
            values.push(ArrayExpressionElement::from(value.into_expr(&ctx.ast)));
        }

        *expr = ctx.ast.expression_array(SPAN, values);
        1
    }
}

/// Evaluates Object.entries: `Object.entries({a:1})` → `[["a",1]]`
#[derive(Default)]
pub struct Entries;

impl Entries {
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
        if member.property.name.as_str() != "entries" {
            return 0;
        }

        let Expression::Identifier(obj) = &member.object else {
            return 0;
        };
        if obj.name.as_str() != "Object" {
            return 0;
        }

        let Some(arg) = call.arguments.first().and_then(|a| a.as_expression()) else {
            return 0;
        };
        let Expression::ObjectExpression(obj_expr) = arg else {
            return 0;
        };

        let mut entries = ctx.ast.vec();
        for prop in &obj_expr.properties {
            let ObjectPropertyKind::ObjectProperty(p) = prop else {
                return 0;
            };

            let key_str = match &p.key {
                PropertyKey::StaticIdentifier(id) => id.name.to_string(),
                PropertyKey::StringLiteral(s) => s.value.to_string(),
                _ => return 0,
            };
            let Some(value) = LiteralValue::from_expr(&p.value) else {
                return 0;
            };

            let mut pair = ctx.ast.vec();
            pair.push(ArrayExpressionElement::from(
                ctx.ast
                    .expression_string_literal(SPAN, ctx.ast.atom(&key_str), None),
            ));
            pair.push(ArrayExpressionElement::from(value.into_expr(&ctx.ast)));

            entries.push(ArrayExpressionElement::from(
                ctx.ast.expression_array(SPAN, pair),
            ));
        }

        *expr = ctx.ast.expression_array(SPAN, entries);
        1
    }
}

// ============================================================================
// ObjectGroup
// ============================================================================

/// Group of all Object passes.
#[derive(Default)]
pub struct ObjectGroup {
    keys: Keys,
    values: Values,
    entries: Entries,
}

impl ObjectGroup {
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
        mods += self.keys.transform(expr, ctx);
        mods += self.values.transform(expr, ctx);
        mods += self.entries.transform(expr, ctx);
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

        struct Visitor(ObjectGroup);
        impl<'a> oxc_traverse::Traverse<'a, ()> for Visitor {
            fn exit_expression(
                &mut self,
                expr: &mut Expression<'a>,
                ctx: &mut TraverseCtx<'a, ()>,
            ) {
                self.0.transform(expr, ctx);
            }
        }

        let mut visitor = Visitor(ObjectGroup::new());
        traverse_mut(&mut visitor, &allocator, &mut program, scoping, ());
        codegen(&program)
    }

    #[test]
    fn test_keys() {
        let result = transform("Object.keys({a: 1, b: 2})");
        assert!(result.contains("\"a\"") && result.contains("\"b\""));
    }

    #[test]
    fn test_values() {
        let result = transform("Object.values({a: 1, b: 2})");
        assert!(result.contains('1') && result.contains('2'));
    }

    #[test]
    fn test_entries() {
        let result = transform("Object.entries({a: 1})");
        assert!(result.contains("\"a\"") && result.contains('1'));
    }
}
