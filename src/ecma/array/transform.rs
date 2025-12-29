//! Array transform passes.
//!
//! - `Join` — `[1,2,3].join("-")` → `"1-2-3"`
//! - `ToString` — `[1,2,3].toString()` → `"1,2,3"`
//! - `Reverse` — `[1,2,3].reverse()` → `[3,2,1]` (literal arrays only)
//! - `Flat` — `[[1],[2]].flat()` → `[1,2]`

use oxc::ast::ast::{Argument, ArrayExpressionElement, Expression};
use oxc::span::SPAN;
use oxc_traverse::TraverseCtx;

use crate::utils::literal::{self, LiteralValue};

// ============================================================================
// Individual Passes
// ============================================================================

/// Evaluates join: `[1,2,3].join("-")` → `"1-2-3"`
#[derive(Default)]
pub struct Join;

impl Join {
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
        if member.property.name.as_str() != "join" {
            return 0;
        }

        let Some(elements) = literal::array_elements(&member.object) else {
            return 0;
        };

        // Get separator (default is ",")
        let separator = call
            .arguments
            .first()
            .and_then(get_string_arg)
            .unwrap_or(",");

        let mut parts = Vec::new();
        for elem in elements {
            match LiteralValue::from_expr(elem) {
                Some(LiteralValue::Number(n)) => parts.push(literal::number_to_string(n)),
                Some(LiteralValue::String(s)) => parts.push(s),
                Some(LiteralValue::Boolean(b)) => parts.push(b.to_string()),
                Some(LiteralValue::Null) | Some(LiteralValue::Undefined) => {
                    parts.push(String::new())
                }
                None => return 0, // Can't evaluate non-literal
            }
        }

        let result = parts.join(separator);
        *expr = ctx
            .ast
            .expression_string_literal(SPAN, ctx.ast.atom(&result), None);
        1
    }
}

/// Evaluates toString: `[1,2,3].toString()` → `"1,2,3"`
#[derive(Default)]
pub struct ToString;

impl ToString {
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
        if member.property.name.as_str() != "toString" {
            return 0;
        }

        let Some(elements) = literal::array_elements(&member.object) else {
            return 0;
        };

        let mut parts = Vec::new();
        for elem in elements {
            match LiteralValue::from_expr(elem) {
                Some(LiteralValue::Number(n)) => parts.push(literal::number_to_string(n)),
                Some(LiteralValue::String(s)) => parts.push(s),
                Some(LiteralValue::Boolean(b)) => parts.push(b.to_string()),
                Some(LiteralValue::Null) | Some(LiteralValue::Undefined) => {
                    parts.push(String::new())
                }
                None => return 0,
            }
        }

        let result = parts.join(",");
        *expr = ctx
            .ast
            .expression_string_literal(SPAN, ctx.ast.atom(&result), None);
        1
    }
}

/// Evaluates reverse on literal arrays: `[1,2,3].reverse()` → `[3,2,1]`
/// Note: Only works on literal arrays, returns new array (doesn't mutate)
#[derive(Default)]
pub struct Reverse;

impl Reverse {
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
        if member.property.name.as_str() != "reverse" {
            return 0;
        }

        let Some(elements) = literal::array_elements(&member.object) else {
            return 0;
        };

        // Collect literal values in reverse order
        let mut reversed_values = Vec::new();
        for elem in elements.iter().rev() {
            if let Some(value) = LiteralValue::from_expr(elem) {
                reversed_values.push(value);
            } else {
                return 0; // Can't reverse non-literal elements
            }
        }

        // Build new array
        let mut new_elements = ctx.ast.vec();
        for value in reversed_values {
            let elem_expr = value.into_expr(&ctx.ast);
            new_elements.push(ArrayExpressionElement::from(elem_expr));
        }

        *expr = ctx.ast.expression_array(SPAN, new_elements);
        1
    }
}

/// Evaluates flat: `[[1],[2]].flat()` → `[1,2]`
#[derive(Default)]
pub struct Flat;

impl Flat {
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
        if member.property.name.as_str() != "flat" {
            return 0;
        }

        let Expression::ArrayExpression(arr) = &member.object else {
            return 0;
        };

        // Get depth (default is 1)
        let depth = call
            .arguments
            .first()
            .and_then(get_number_arg)
            .unwrap_or(1.0) as usize;

        // Collect flattened values
        let mut flattened = Vec::new();
        if !self.flatten_array(&arr.elements, depth, &mut flattened) {
            return 0;
        }

        // Build new array
        let mut new_elements = ctx.ast.vec();
        for value in flattened {
            let elem_expr = value.into_expr(&ctx.ast);
            new_elements.push(ArrayExpressionElement::from(elem_expr));
        }

        *expr = ctx.ast.expression_array(SPAN, new_elements);
        1
    }

    fn flatten_array(
        &self,
        elements: &oxc::allocator::Vec<ArrayExpressionElement>,
        depth: usize,
        result: &mut Vec<LiteralValue>,
    ) -> bool {
        for elem in elements {
            match elem {
                ArrayExpressionElement::SpreadElement(_) => return false,
                ArrayExpressionElement::Elision(_) => {
                    result.push(LiteralValue::Undefined);
                }
                _ => {
                    if let Some(elem_expr) = elem.as_expression() {
                        if depth > 0
                            && let Expression::ArrayExpression(nested) = elem_expr
                        {
                            if !self.flatten_array(&nested.elements, depth - 1, result) {
                                return false;
                            }
                            continue;
                        }
                        if let Some(value) = LiteralValue::from_expr(elem_expr) {
                            result.push(value);
                        } else {
                            return false;
                        }
                    }
                }
            }
        }
        true
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
// Transform Group
// ============================================================================

/// Group of all array transform passes.
#[derive(Default)]
pub struct Transform {
    join: Join,
    to_string: ToString,
    reverse: Reverse,
    flat: Flat,
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
        mods += self.join.transform(expr, ctx);
        mods += self.to_string.transform(expr, ctx);
        mods += self.reverse.transform(expr, ctx);
        mods += self.flat.transform(expr, ctx);
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
    fn test_join() {
        let result = transform("[1, 2, 3].join(\"-\")");
        assert!(result.contains("\"1-2-3\""));
    }

    #[test]
    fn test_join_default() {
        let result = transform("[1, 2, 3].join()");
        assert!(result.contains("\"1,2,3\""));
    }

    #[test]
    fn test_to_string() {
        let result = transform("[1, 2, 3].toString()");
        assert!(result.contains("\"1,2,3\""));
    }

    #[test]
    fn test_reverse() {
        let result = transform("[1, 2, 3].reverse()");
        // Output may be formatted with newlines, check for values in order
        let pos_3 = result.find('3').unwrap_or(0);
        let pos_2 = result.find('2').unwrap_or(0);
        let pos_1 = result.find('1').unwrap_or(0);
        assert!(
            pos_3 < pos_2 && pos_2 < pos_1,
            "Expected [3, 2, 1] order, got: {}",
            result
        );
    }

    #[test]
    fn test_flat() {
        let result = transform("[[1], [2]].flat()");
        assert!(result.contains("[1, 2]") || result.contains("[1,2]"));
    }
}
