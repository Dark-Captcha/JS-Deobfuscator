//! Side effect analysis.
//!
//! Determines whether expressions have side effects that would prevent
//! safe removal or duplication.

use oxc::ast::ast::{
    ArrayExpressionElement, Expression, ObjectPropertyKind, PropertyKey, UnaryOperator,
};

// ============================================================================
// Side Effect Analysis
// ============================================================================

/// Check if an expression is side-effect free.
///
/// Side-effect free expressions can be safely removed or duplicated
/// without changing program behavior.
///
/// # Side-effect free
///
/// - Literals (numbers, strings, booleans, null)
/// - Identifiers (reading a variable)
/// - Pure operators (arithmetic, comparison, logical)
/// - Array/object literals (if all elements are side-effect free)
/// - Function expressions (defining, not calling)
///
/// # Has side effects
///
/// - Function calls
/// - Assignments
/// - `delete` operator
/// - `new` expressions
/// - Property access on potentially null/undefined (can throw)
///
/// # Example
///
/// ```ignore
/// is_side_effect_free(&expr_1_plus_2) // true
/// is_side_effect_free(&expr_foo_call) // false
/// is_side_effect_free(&expr_x_assign) // false
/// ```
pub fn is_side_effect_free(expr: &Expression) -> bool {
    match expr {
        // Literals are always side-effect free
        Expression::NumericLiteral(_)
        | Expression::StringLiteral(_)
        | Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::BigIntLiteral(_)
        | Expression::RegExpLiteral(_) => true,

        // Identifiers (reading a variable has no side effect)
        Expression::Identifier(_) => true,

        // This expression
        Expression::ThisExpression(_) => true,

        // Function/class expressions (not calls)
        Expression::FunctionExpression(_)
        | Expression::ArrowFunctionExpression(_)
        | Expression::ClassExpression(_) => true,

        // Template literals without tag
        Expression::TemplateLiteral(t) => t.expressions.iter().all(is_side_effect_free),

        // Unary expressions (except delete)
        Expression::UnaryExpression(u) => {
            u.operator != UnaryOperator::Delete && is_side_effect_free(&u.argument)
        }

        // Binary expressions
        Expression::BinaryExpression(b) => {
            is_side_effect_free(&b.left) && is_side_effect_free(&b.right)
        }

        // Logical expressions
        Expression::LogicalExpression(l) => {
            is_side_effect_free(&l.left) && is_side_effect_free(&l.right)
        }

        // Conditional expressions
        Expression::ConditionalExpression(c) => {
            is_side_effect_free(&c.test)
                && is_side_effect_free(&c.consequent)
                && is_side_effect_free(&c.alternate)
        }

        // Sequence expressions
        Expression::SequenceExpression(s) => s.expressions.iter().all(is_side_effect_free),

        // Parenthesized
        Expression::ParenthesizedExpression(p) => is_side_effect_free(&p.expression),

        // Array expressions (if all elements are side-effect free)
        Expression::ArrayExpression(arr) => arr.elements.iter().all(|el| match el {
            ArrayExpressionElement::SpreadElement(s) => is_side_effect_free(&s.argument),
            ArrayExpressionElement::Elision(_) => true,
            _ => el.as_expression().is_some_and(is_side_effect_free),
        }),

        // Object expressions (if all properties are side-effect free)
        Expression::ObjectExpression(obj) => obj.properties.iter().all(|prop| match prop {
            ObjectPropertyKind::ObjectProperty(p) => {
                // Computed keys might have side effects
                let key_safe = matches!(
                    &p.key,
                    PropertyKey::StaticIdentifier(_)
                        | PropertyKey::StringLiteral(_)
                        | PropertyKey::NumericLiteral(_)
                );
                key_safe && is_side_effect_free(&p.value)
            }
            ObjectPropertyKind::SpreadProperty(s) => is_side_effect_free(&s.argument),
        }),

        // Member expressions (reading property)
        // Note: This is aggressive — property access CAN throw if object is null/undefined
        // But for deobfuscation, we treat it as side-effect free
        Expression::StaticMemberExpression(m) => !m.optional && is_side_effect_free(&m.object),
        Expression::ComputedMemberExpression(m) => {
            !m.optional && is_side_effect_free(&m.object) && is_side_effect_free(&m.expression)
        }

        // Everything else potentially has side effects
        // Calls, assignments, new, await, yield, etc.
        _ => false,
    }
}

/// Conservative check — may return true even if expression is pure.
///
/// Use this when you need to be certain about side effects.
pub fn may_have_side_effects(expr: &Expression) -> bool {
    !is_side_effect_free(expr)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use oxc::allocator::Allocator;
    use oxc::parser::Parser;
    use oxc::span::SourceType;

    fn parse_expr<'a>(allocator: &'a Allocator, source: &'a str) -> Expression<'a> {
        Parser::new(allocator, source, SourceType::mjs())
            .parse_expression()
            .unwrap()
    }

    #[test]
    fn test_literals_are_side_effect_free() {
        let allocator = Allocator::default();

        assert!(is_side_effect_free(&parse_expr(&allocator, "42")));
        assert!(is_side_effect_free(&parse_expr(&allocator, "\"hello\"")));
        assert!(is_side_effect_free(&parse_expr(&allocator, "true")));
        assert!(is_side_effect_free(&parse_expr(&allocator, "null")));
    }

    #[test]
    fn test_identifiers_are_side_effect_free() {
        let allocator = Allocator::default();

        assert!(is_side_effect_free(&parse_expr(&allocator, "x")));
        assert!(is_side_effect_free(&parse_expr(&allocator, "foo")));
    }

    #[test]
    fn test_operators_are_side_effect_free() {
        let allocator = Allocator::default();

        assert!(is_side_effect_free(&parse_expr(&allocator, "1 + 2")));
        assert!(is_side_effect_free(&parse_expr(&allocator, "a && b")));
        assert!(is_side_effect_free(&parse_expr(&allocator, "x ? y : z")));
        assert!(is_side_effect_free(&parse_expr(&allocator, "-5")));
        assert!(is_side_effect_free(&parse_expr(&allocator, "!true")));
    }

    #[test]
    fn test_calls_have_side_effects() {
        let allocator = Allocator::default();

        assert!(!is_side_effect_free(&parse_expr(&allocator, "foo()")));
        assert!(!is_side_effect_free(&parse_expr(
            &allocator,
            "console.log(x)"
        )));
    }

    #[test]
    fn test_assignments_have_side_effects() {
        let allocator = Allocator::default();

        assert!(!is_side_effect_free(&parse_expr(&allocator, "x = 1")));
        assert!(!is_side_effect_free(&parse_expr(&allocator, "x += 1")));
    }

    #[test]
    fn test_delete_has_side_effects() {
        let allocator = Allocator::default();

        assert!(!is_side_effect_free(&parse_expr(&allocator, "delete x.y")));
    }

    #[test]
    fn test_arrays_and_objects() {
        let allocator = Allocator::default();

        assert!(is_side_effect_free(&parse_expr(&allocator, "[1, 2, 3]")));
        assert!(is_side_effect_free(&parse_expr(&allocator, "{a: 1, b: 2}")));
        assert!(!is_side_effect_free(&parse_expr(&allocator, "[foo()]")));
    }
}
