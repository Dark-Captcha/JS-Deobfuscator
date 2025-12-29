//! Literal value representation and operations.
//!
//! This module provides `LiteralValue` — the **single source of truth** for
//! working with JavaScript literal values in the deobfuscator.
//!
//! # Design
//!
//! All literal-related operations go through this module:
//! - Extraction from AST expressions
//! - Conversion back to AST expressions
//! - Truthiness evaluation
//! - Type checking
//! - Array operations

use oxc::ast::AstBuilder;
use oxc::ast::ast::{ArrayExpressionElement, Expression, NumberBase, UnaryOperator};
use oxc::span::SPAN;

// ============================================================================
// LiteralValue Enum
// ============================================================================

/// Compact literal value storage (lifetime-free).
///
/// Use this when you need to store literal values across operations,
/// such as in constant propagation.
#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    Number(f64),
    String(String),
    Boolean(bool),
    Null,
    Undefined,
}

impl LiteralValue {
    /// Extract a full `LiteralValue` from an expression.
    pub fn from_expr(expr: &Expression) -> Option<Self> {
        match expr {
            Expression::NumericLiteral(lit) => Some(Self::Number(lit.value)),
            Expression::StringLiteral(lit) => Some(Self::String(lit.value.to_string())),
            Expression::BooleanLiteral(lit) => Some(Self::Boolean(lit.value)),
            Expression::NullLiteral(_) => Some(Self::Null),
            Expression::Identifier(ident) if ident.name == "undefined" => Some(Self::Undefined),
            Expression::UnaryExpression(unary) => match unary.operator {
                UnaryOperator::UnaryNegation => {
                    Self::from_expr(&unary.argument).and_then(|v| match v {
                        Self::Number(n) => Some(Self::Number(-n)),
                        _ => None,
                    })
                }
                UnaryOperator::UnaryPlus => {
                    Self::from_expr(&unary.argument).and_then(|v| match v {
                        Self::Number(n) => Some(Self::Number(n)),
                        _ => None,
                    })
                }
                _ => None,
            },
            Expression::ParenthesizedExpression(paren) => Self::from_expr(&paren.expression),
            _ => None,
        }
    }

    /// Convert to an AST expression.
    pub fn into_expr<'a>(self, ast: &AstBuilder<'a>) -> Expression<'a> {
        match self {
            Self::Number(n) => make_number(n, ast),
            Self::String(s) => ast.expression_string_literal(SPAN, ast.atom(&s), None),
            Self::Boolean(b) => ast.expression_boolean_literal(SPAN, b),
            Self::Null => ast.expression_null_literal(SPAN),
            Self::Undefined => {
                let ident = ast.alloc_identifier_reference(SPAN, ast.atom("undefined"));
                Expression::Identifier(ident)
            }
        }
    }

    /// Check if this value is falsy.
    pub fn is_falsy(&self) -> bool {
        match self {
            Self::Boolean(false) => true,
            Self::Number(n) => *n == 0.0 || n.is_nan(),
            Self::String(s) => s.is_empty(),
            Self::Null | Self::Undefined => true,
            _ => false,
        }
    }

    /// Check if this value is truthy.
    #[inline]
    pub fn is_truthy(&self) -> bool {
        !self.is_falsy()
    }

    /// Get the JavaScript typeof result.
    pub fn type_of(&self) -> &'static str {
        match self {
            Self::Number(_) => "number",
            Self::String(_) => "string",
            Self::Boolean(_) => "boolean",
            Self::Null => "object",
            Self::Undefined => "undefined",
        }
    }
}

// ============================================================================
// Number Operations
// ============================================================================

/// Extract a number from an expression.
///
/// Handles: `NumericLiteral`, unary `+`/`-`, parentheses.
#[inline]
pub fn number(expr: &Expression) -> Option<f64> {
    match expr {
        Expression::NumericLiteral(lit) => Some(lit.value),
        Expression::UnaryExpression(unary) => match unary.operator {
            UnaryOperator::UnaryNegation => number(&unary.argument).map(|v| -v),
            UnaryOperator::UnaryPlus => number(&unary.argument),
            _ => None,
        },
        Expression::ParenthesizedExpression(paren) => number(&paren.expression),
        _ => None,
    }
}

/// Create a number literal expression.
///
/// Handles negative numbers by creating unary negation.
pub fn make_number<'a>(value: f64, ast: &AstBuilder<'a>) -> Expression<'a> {
    if value.is_sign_negative() && value != 0.0 {
        let inner = ast.expression_numeric_literal(SPAN, value.abs(), None, NumberBase::Decimal);
        ast.expression_unary(SPAN, UnaryOperator::UnaryNegation, inner)
    } else if value == 0.0 && value.is_sign_negative() {
        let inner = ast.expression_numeric_literal(SPAN, 0.0, None, NumberBase::Decimal);
        ast.expression_unary(SPAN, UnaryOperator::UnaryNegation, inner)
    } else {
        ast.expression_numeric_literal(SPAN, value, None, NumberBase::Decimal)
    }
}

// ============================================================================
// String Operations
// ============================================================================

/// Extract a string from an expression (returns reference, no allocation).
#[inline]
pub fn string<'a>(expr: &'a Expression<'a>) -> Option<&'a str> {
    match expr {
        Expression::StringLiteral(lit) => Some(lit.value.as_str()),
        Expression::ParenthesizedExpression(paren) => string(&paren.expression),
        _ => None,
    }
}

/// Create a string literal expression.
#[inline]
pub fn make_string<'a>(value: &str, ast: &AstBuilder<'a>) -> Expression<'a> {
    ast.expression_string_literal(SPAN, ast.atom(value), None)
}

// ============================================================================
// Boolean Operations
// ============================================================================

/// Extract a boolean from an expression.
#[inline]
pub fn boolean(expr: &Expression) -> Option<bool> {
    match expr {
        Expression::BooleanLiteral(lit) => Some(lit.value),
        Expression::ParenthesizedExpression(paren) => boolean(&paren.expression),
        _ => None,
    }
}

/// Create a boolean literal expression.
#[inline]
pub fn make_boolean<'a>(value: bool, ast: &AstBuilder<'a>) -> Expression<'a> {
    ast.expression_boolean_literal(SPAN, value)
}

// ============================================================================
// Null/Undefined Operations
// ============================================================================

/// Create a null literal expression.
#[inline]
pub fn make_null<'a>(ast: &AstBuilder<'a>) -> Expression<'a> {
    ast.expression_null_literal(SPAN)
}

/// Create an undefined identifier expression.
#[inline]
pub fn make_undefined<'a>(ast: &AstBuilder<'a>) -> Expression<'a> {
    let ident = ast.alloc_identifier_reference(SPAN, ast.atom("undefined"));
    Expression::Identifier(ident)
}

// ============================================================================
// Truthiness
// ============================================================================

/// Get the truthiness of an expression if statically determinable.
///
/// Returns `Some(true)` for truthy, `Some(false)` for falsy, `None` if unknown.
///
/// # Falsy values in JavaScript
/// - `false`, `0`, `-0`, `NaN`, `""`, `null`, `undefined`
pub fn is_truthy(expr: &Expression) -> Option<bool> {
    match expr {
        Expression::BooleanLiteral(lit) => Some(lit.value),
        Expression::NumericLiteral(lit) => Some(lit.value != 0.0 && !lit.value.is_nan()),
        Expression::StringLiteral(lit) => Some(!lit.value.is_empty()),
        Expression::NullLiteral(_) => Some(false),
        Expression::Identifier(ident) if ident.name == "undefined" => Some(false),
        Expression::Identifier(ident) if ident.name == "NaN" => Some(false),
        Expression::ArrayExpression(_) | Expression::ObjectExpression(_) => Some(true),
        Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_) => Some(true),
        Expression::UnaryExpression(unary) if unary.operator == UnaryOperator::LogicalNot => {
            is_truthy(&unary.argument).map(|v| !v)
        }
        Expression::UnaryExpression(unary) if unary.operator == UnaryOperator::Void => Some(false),
        Expression::ParenthesizedExpression(paren) => is_truthy(&paren.expression),
        Expression::SequenceExpression(seq) => seq.expressions.last().and_then(is_truthy),
        _ => None,
    }
}

// ============================================================================
// Array Operations
// ============================================================================

/// Extract array elements from an expression.
///
/// Returns `None` if expression is not an array or contains spread/holes.
pub fn array_elements<'a>(expr: &'a Expression<'a>) -> Option<Vec<&'a Expression<'a>>> {
    match expr {
        Expression::ArrayExpression(arr) => {
            let mut elements = Vec::with_capacity(arr.elements.len());
            for elem in &arr.elements {
                match elem {
                    ArrayExpressionElement::SpreadElement(_) => return None,
                    ArrayExpressionElement::Elision(_) => return None,
                    _ => {
                        if let Some(e) = elem.as_expression() {
                            elements.push(e);
                        } else {
                            return None;
                        }
                    }
                }
            }
            Some(elements)
        }
        Expression::ParenthesizedExpression(paren) => array_elements(&paren.expression),
        _ => None,
    }
}

/// Create an array literal expression from elements.
pub fn make_array<'a>(
    elements: impl IntoIterator<Item = Expression<'a>>,
    ast: &AstBuilder<'a>,
) -> Expression<'a> {
    let elems: oxc::allocator::Vec<'a, ArrayExpressionElement<'a>> =
        ast.vec_from_iter(elements.into_iter().map(ArrayExpressionElement::from));
    ast.expression_array(SPAN, elems)
}

// ============================================================================
// Type Checking
// ============================================================================

/// Check if an expression is a literal value.
pub fn is_literal(expr: &Expression) -> bool {
    match expr {
        Expression::NumericLiteral(_)
        | Expression::StringLiteral(_)
        | Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_) => true,
        Expression::Identifier(id) if id.name == "undefined" => true,
        Expression::UnaryExpression(u)
            if matches!(
                u.operator,
                UnaryOperator::UnaryNegation | UnaryOperator::UnaryPlus
            ) =>
        {
            is_literal(&u.argument)
        }
        Expression::ParenthesizedExpression(paren) => is_literal(&paren.expression),
        Expression::ArrayExpression(_) => is_literal_array(expr),
        _ => false,
    }
}

/// Check if all array elements are literals.
pub fn is_literal_array(expr: &Expression) -> bool {
    array_elements(expr).is_some_and(|elems| elems.iter().all(|e| is_literal(e)))
}

// ============================================================================
// Formatting
// ============================================================================

/// Format a number as JavaScript would.
///
/// Handles NaN, Infinity, integers, and floats.
pub fn number_to_string(n: f64) -> String {
    if n.is_nan() {
        "NaN".to_string()
    } else if n.is_infinite() {
        if n.is_sign_positive() {
            "Infinity".to_string()
        } else {
            "-Infinity".to_string()
        }
    } else if n == 0.0 {
        "0".to_string()
    } else if n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        format!("{n}")
    }
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
    fn test_number() {
        let alloc = Allocator::default();
        assert_eq!(number(&parse_expr(&alloc, "42")), Some(42.0));
        assert_eq!(number(&parse_expr(&alloc, "-5")), Some(-5.0));
        assert_eq!(number(&parse_expr(&alloc, "+3")), Some(3.0));
        assert_eq!(number(&parse_expr(&alloc, "(42)")), Some(42.0));
        assert_eq!(number(&parse_expr(&alloc, "\"x\"")), None);
    }

    #[test]
    fn test_string() {
        let alloc = Allocator::default();
        assert_eq!(string(&parse_expr(&alloc, "\"hello\"")), Some("hello"));
        assert_eq!(string(&parse_expr(&alloc, "(\"world\")")), Some("world"));
        assert_eq!(string(&parse_expr(&alloc, "42")), None);
    }

    #[test]
    fn test_boolean() {
        let alloc = Allocator::default();
        assert_eq!(boolean(&parse_expr(&alloc, "true")), Some(true));
        assert_eq!(boolean(&parse_expr(&alloc, "false")), Some(false));
        assert_eq!(boolean(&parse_expr(&alloc, "(true)")), Some(true));
        assert_eq!(boolean(&parse_expr(&alloc, "42")), None);
    }

    #[test]
    fn test_is_truthy() {
        let alloc = Allocator::default();
        assert_eq!(is_truthy(&parse_expr(&alloc, "true")), Some(true));
        assert_eq!(is_truthy(&parse_expr(&alloc, "1")), Some(true));
        assert_eq!(is_truthy(&parse_expr(&alloc, "\"x\"")), Some(true));
        assert_eq!(is_truthy(&parse_expr(&alloc, "[]")), Some(true));
        assert_eq!(is_truthy(&parse_expr(&alloc, "false")), Some(false));
        assert_eq!(is_truthy(&parse_expr(&alloc, "0")), Some(false));
        assert_eq!(is_truthy(&parse_expr(&alloc, "\"\"")), Some(false));
        assert_eq!(is_truthy(&parse_expr(&alloc, "null")), Some(false));
        assert_eq!(is_truthy(&parse_expr(&alloc, "x")), None);
    }

    #[test]
    fn test_literal_value_roundtrip() {
        let alloc = Allocator::default();
        let ast = AstBuilder::new(&alloc);

        let expr = LiteralValue::Number(42.0).into_expr(&ast);
        assert_eq!(number(&expr), Some(42.0));

        let expr = LiteralValue::Number(-5.0).into_expr(&ast);
        assert_eq!(number(&expr), Some(-5.0));

        let expr = LiteralValue::String("hello".to_string()).into_expr(&ast);
        assert_eq!(string(&expr), Some("hello"));
    }

    #[test]
    fn test_array_elements() {
        let alloc = Allocator::default();
        let expr = parse_expr(&alloc, "[1, 2, 3]");
        let elems = array_elements(&expr).unwrap();
        assert_eq!(elems.len(), 3);
        assert_eq!(number(elems[0]), Some(1.0));
    }

    #[test]
    fn test_is_literal() {
        let alloc = Allocator::default();
        assert!(is_literal(&parse_expr(&alloc, "42")));
        assert!(is_literal(&parse_expr(&alloc, "\"x\"")));
        assert!(is_literal(&parse_expr(&alloc, "true")));
        assert!(is_literal(&parse_expr(&alloc, "null")));
        assert!(is_literal(&parse_expr(&alloc, "[1, 2]")));
        assert!(!is_literal(&parse_expr(&alloc, "x")));
        assert!(!is_literal(&parse_expr(&alloc, "foo()")));
    }

    #[test]
    fn test_number_to_string() {
        assert_eq!(number_to_string(42.0), "42");
        assert_eq!(number_to_string(-5.0), "-5");
        assert_eq!(number_to_string(3.15), "3.15");
        assert_eq!(number_to_string(0.0), "0");
        assert_eq!(number_to_string(f64::NAN), "NaN");
        assert_eq!(number_to_string(f64::INFINITY), "Infinity");
        assert_eq!(number_to_string(f64::NEG_INFINITY), "-Infinity");
    }
}
