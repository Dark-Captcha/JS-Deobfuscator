//! Unary operator passes.
//!
//! - `Not` — `!true` → `false`
//! - `Typeof` — `typeof "x"` → `"string"`
//! - `Void` — `void 0` → `undefined`
//! - `UnaryPlus` — `+"42"` → `42`, `+[]` → `0`
//! - `UnaryMinus` — `-(-5)` → `5`
//! - `NotCoerce` — `![]` → `false`, `!![]` → `true`

use oxc::ast::ast::{Expression, UnaryOperator};
use oxc::span::SPAN;
use oxc_traverse::TraverseCtx;

use crate::utils::literal;

// ============================================================================
// Individual Passes
// ============================================================================

/// Evaluates boolean NOT: `!true` → `false`
#[derive(Default)]
pub struct Not;

impl Not {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Expression::UnaryExpression(unary) = expr else {
            return 0;
        };
        if unary.operator != UnaryOperator::LogicalNot {
            return 0;
        }

        let Some(value) = literal::boolean(&unary.argument) else {
            return 0;
        };

        *expr = ctx.ast.expression_boolean_literal(SPAN, !value);
        1
    }
}

/// Evaluates typeof on literals: `typeof "x"` → `"string"`
#[derive(Default)]
pub struct Typeof;

impl Typeof {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Expression::UnaryExpression(unary) = expr else {
            return 0;
        };
        if unary.operator != UnaryOperator::Typeof {
            return 0;
        }

        let Some(type_str) = get_typeof(&unary.argument) else {
            return 0;
        };

        *expr = ctx
            .ast
            .expression_string_literal(SPAN, ctx.ast.atom(type_str), None);
        1
    }
}

/// Converts `void 0` to `undefined`
#[derive(Default)]
pub struct Void;

impl Void {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Expression::UnaryExpression(unary) = expr else {
            return 0;
        };
        if unary.operator != UnaryOperator::Void {
            return 0;
        }

        // Only transform if argument is side-effect-free (numeric literal)
        if literal::number(&unary.argument).is_none() {
            return 0;
        }

        *expr = Expression::Identifier(ctx.ast.alloc_identifier_reference(SPAN, "undefined"));
        1
    }
}

/// Evaluates unary plus for type coercion: `+"42"` → `42`, `+[]` → `0`
#[derive(Default)]
pub struct UnaryPlus;

impl UnaryPlus {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Expression::UnaryExpression(unary) = expr else {
            return 0;
        };
        if unary.operator != UnaryOperator::UnaryPlus {
            return 0;
        }

        let result = match &unary.argument {
            // +[] → 0
            Expression::ArrayExpression(arr) if arr.elements.is_empty() => 0.0,

            // +[n] → n (single numeric element)
            Expression::ArrayExpression(arr) if arr.elements.len() == 1 => {
                if let Some(elem) = arr.elements.first() {
                    if let Some(e) = elem.as_expression() {
                        if let Some(n) = literal::number(e) {
                            n
                        } else if let Some(s) = literal::string(e) {
                            s.trim().parse::<f64>().unwrap_or(f64::NAN)
                        } else {
                            return 0;
                        }
                    } else {
                        return 0;
                    }
                } else {
                    return 0;
                }
            }

            // +"" → 0, +"123" → 123
            Expression::StringLiteral(s) => {
                let trimmed = s.value.trim();
                if trimmed.is_empty() {
                    0.0
                } else {
                    trimmed.parse::<f64>().unwrap_or(f64::NAN)
                }
            }

            // +true → 1, +false → 0
            Expression::BooleanLiteral(b) => {
                if b.value {
                    1.0
                } else {
                    0.0
                }
            }

            // +null → 0
            Expression::NullLiteral(_) => 0.0,

            // Already a number - just unwrap
            Expression::NumericLiteral(n) => n.value,

            _ => return 0,
        };

        // Skip NaN results (can't represent cleanly)
        if result.is_nan() {
            return 0;
        }

        *expr = literal::make_number(result, &ctx.ast);
        1
    }
}

/// Simplifies double negation: `-(-5)` → `5`
#[derive(Default)]
pub struct UnaryMinus;

impl UnaryMinus {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Expression::UnaryExpression(unary) = expr else {
            return 0;
        };
        if unary.operator != UnaryOperator::UnaryNegation {
            return 0;
        }

        // Check for double negation: -(-x)
        if let Expression::UnaryExpression(inner) = &unary.argument
            && inner.operator == UnaryOperator::UnaryNegation
            && let Some(n) = literal::number(&inner.argument)
        {
            *expr = literal::make_number(n, &ctx.ast);
            return 1;
        }

        0
    }
}

/// Coerces values with NOT: `![]` → `false`, `!![]` → `true`
#[derive(Default)]
pub struct NotCoerce;

impl NotCoerce {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Expression::UnaryExpression(unary) = expr else {
            return 0;
        };
        if unary.operator != UnaryOperator::LogicalNot {
            return 0;
        }

        // Get truthiness of argument
        let Some(truthy) = literal::is_truthy(&unary.argument) else {
            return 0;
        };

        *expr = ctx.ast.expression_boolean_literal(SPAN, !truthy);
        1
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn get_typeof(expr: &Expression) -> Option<&'static str> {
    match expr {
        Expression::StringLiteral(_) => Some("string"),
        Expression::NumericLiteral(_) => Some("number"),
        Expression::BooleanLiteral(_) => Some("boolean"),
        Expression::NullLiteral(_) => Some("object"), // typeof null === "object"
        Expression::Identifier(id) if id.name == "undefined" => Some("undefined"),
        Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_) => {
            Some("function")
        }
        Expression::ArrayExpression(_) | Expression::ObjectExpression(_) => Some("object"),
        Expression::BigIntLiteral(_) => Some("bigint"),
        Expression::ParenthesizedExpression(p) => get_typeof(&p.expression),
        Expression::UnaryExpression(u) => match u.operator {
            UnaryOperator::UnaryNegation | UnaryOperator::UnaryPlus => Some("number"),
            UnaryOperator::LogicalNot => Some("boolean"),
            UnaryOperator::Void => Some("undefined"),
            _ => None,
        },
        _ => None,
    }
}

// ============================================================================
// Unary Group
// ============================================================================

/// Group of all unary passes.
#[derive(Default)]
pub struct Unary {
    not: Not,
    typeof_: Typeof,
    void_: Void,
    unary_plus: UnaryPlus,
    unary_minus: UnaryMinus,
    not_coerce: NotCoerce,
}

impl Unary {
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
        mods += self.not.transform(expr, ctx);
        mods += self.typeof_.transform(expr, ctx);
        mods += self.void_.transform(expr, ctx);
        mods += self.unary_plus.transform(expr, ctx);
        mods += self.unary_minus.transform(expr, ctx);
        mods += self.not_coerce.transform(expr, ctx);
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

        struct Visitor(Unary);
        impl<'a> oxc_traverse::Traverse<'a, ()> for Visitor {
            fn exit_expression(
                &mut self,
                expr: &mut Expression<'a>,
                ctx: &mut TraverseCtx<'a, ()>,
            ) {
                self.0.transform(expr, ctx);
            }
        }

        let mut visitor = Visitor(Unary::new());
        traverse_mut(&mut visitor, &allocator, &mut program, scoping, ());
        codegen(&program)
    }

    #[test]
    fn test_not() {
        assert_eq!(transform("!true"), "false;\n");
        assert_eq!(transform("!false"), "true;\n");
    }

    #[test]
    fn test_typeof() {
        // Note: codegen may add parentheses around string literals
        let result = transform("typeof \"x\"");
        assert!(result.contains("\"string\""));

        let result = transform("typeof 42");
        assert!(result.contains("\"number\""));

        let result = transform("typeof true");
        assert!(result.contains("\"boolean\""));

        let result = transform("typeof null");
        assert!(result.contains("\"object\""));

        let result = transform("typeof undefined");
        assert!(result.contains("\"undefined\""));
    }

    #[test]
    fn test_void() {
        assert_eq!(transform("void 0"), "undefined;\n");
        assert_eq!(transform("void 1"), "undefined;\n");
    }

    #[test]
    fn test_unary_plus() {
        assert_eq!(transform("+[]"), "0;\n");
        assert_eq!(transform("+\"\""), "0;\n");
        assert_eq!(transform("+true"), "1;\n");
        assert_eq!(transform("+false"), "0;\n");
        assert_eq!(transform("+null"), "0;\n");
    }

    #[test]
    fn test_unary_plus_string() {
        assert_eq!(transform("+\"42\""), "42;\n");
        assert_eq!(transform("+\"3.14\""), "3.14;\n");
    }

    #[test]
    fn test_not_coerce() {
        assert_eq!(transform("![]"), "false;\n");
        assert_eq!(transform("!\"\""), "true;\n");
        assert_eq!(transform("!0"), "true;\n");
        assert_eq!(transform("!1"), "false;\n");
    }
}
