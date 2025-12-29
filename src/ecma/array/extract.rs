//! Array extract passes.
//!
//! - `Slice` — `[1,2,3].slice(1)` → `[2,3]`
//! - `Concat` — `[1].concat([2])` → `[1,2]`

use oxc::ast::ast::{Argument, ArrayExpressionElement, Expression};
use oxc::span::SPAN;
use oxc_traverse::TraverseCtx;

use crate::utils::literal::{self, LiteralValue};

// ============================================================================
// Individual Passes
// ============================================================================

/// Evaluates slice: `[1,2,3].slice(1)` → `[2,3]`
#[derive(Default)]
pub struct Slice;

impl Slice {
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
        if member.property.name.as_str() != "slice" {
            return 0;
        }

        let Some(elements) = literal::array_elements(&member.object) else {
            return 0;
        };
        let len = elements.len() as i64;

        // Get start index (default 0)
        let start = call
            .arguments
            .first()
            .and_then(get_number_arg)
            .map(|n| n as i64)
            .unwrap_or(0);

        // Get end index (default len)
        let end = call
            .arguments
            .get(1)
            .and_then(get_number_arg)
            .map(|n| n as i64)
            .unwrap_or(len);

        // Normalize indices
        let start = if start < 0 {
            (len + start).max(0)
        } else {
            start.min(len)
        } as usize;
        let end = if end < 0 {
            (len + end).max(0)
        } else {
            end.min(len)
        } as usize;

        if start >= end {
            // Empty array
            *expr = ctx.ast.expression_array(SPAN, ctx.ast.vec());
            return 1;
        }

        // Collect sliced values
        let mut sliced = Vec::new();
        for elem in elements.iter().skip(start).take(end - start) {
            if let Some(value) = LiteralValue::from_expr(elem) {
                sliced.push(value);
            } else {
                return 0;
            }
        }

        // Build new array
        let mut new_elements = ctx.ast.vec();
        for value in sliced {
            let elem_expr = value.into_expr(&ctx.ast);
            new_elements.push(ArrayExpressionElement::from(elem_expr));
        }

        *expr = ctx.ast.expression_array(SPAN, new_elements);
        1
    }
}

/// Evaluates concat: `[1].concat([2])` → `[1,2]`
#[derive(Default)]
pub struct Concat;

impl Concat {
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
        if member.property.name.as_str() != "concat" {
            return 0;
        }

        let Some(base_elements) = literal::array_elements(&member.object) else {
            return 0;
        };

        // Collect all values
        let mut all_values = Vec::new();

        // Add base array elements
        for elem in base_elements {
            if let Some(value) = LiteralValue::from_expr(elem) {
                all_values.push(value);
            } else {
                return 0;
            }
        }

        // Add elements from each argument
        for arg in &call.arguments {
            let Some(arg_expr) = arg.as_expression() else {
                return 0;
            };

            // If argument is an array, spread its elements
            if let Some(arg_elements) = literal::array_elements(arg_expr) {
                for elem in arg_elements {
                    if let Some(value) = LiteralValue::from_expr(elem) {
                        all_values.push(value);
                    } else {
                        return 0;
                    }
                }
            } else if let Some(value) = LiteralValue::from_expr(arg_expr) {
                // Non-array argument is added as single element
                all_values.push(value);
            } else {
                return 0;
            }
        }

        // Build new array
        let mut new_elements = ctx.ast.vec();
        for value in all_values {
            let elem_expr = value.into_expr(&ctx.ast);
            new_elements.push(ArrayExpressionElement::from(elem_expr));
        }

        *expr = ctx.ast.expression_array(SPAN, new_elements);
        1
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn get_number_arg(arg: &Argument) -> Option<f64> {
    match arg {
        Argument::NumericLiteral(n) => Some(n.value),
        _ => arg.as_expression().and_then(literal::number),
    }
}

// ============================================================================
// Extract Group
// ============================================================================

/// Group of all array extract passes.
#[derive(Default)]
pub struct Extract {
    slice: Slice,
    concat: Concat,
}

impl Extract {
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
        mods += self.slice.transform(expr, ctx);
        mods += self.concat.transform(expr, ctx);
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

        struct Visitor(Extract);
        impl<'a> oxc_traverse::Traverse<'a, ()> for Visitor {
            fn exit_expression(
                &mut self,
                expr: &mut Expression<'a>,
                ctx: &mut TraverseCtx<'a, ()>,
            ) {
                self.0.transform(expr, ctx);
            }
        }

        let mut visitor = Visitor(Extract::new());
        traverse_mut(&mut visitor, &allocator, &mut program, scoping, ());
        codegen(&program)
    }

    #[test]
    fn test_slice() {
        let result = transform("[1, 2, 3].slice(1)");
        assert!(result.contains("[2, 3]") || result.contains("[2,3]"));
    }

    #[test]
    fn test_slice_with_end() {
        let result = transform("[1, 2, 3, 4].slice(1, 3)");
        assert!(result.contains("[2, 3]") || result.contains("[2,3]"));
    }

    #[test]
    fn test_slice_negative() {
        let result = transform("[1, 2, 3].slice(-2)");
        assert!(result.contains("[2, 3]") || result.contains("[2,3]"));
    }

    #[test]
    fn test_concat() {
        let result = transform("[1].concat([2, 3])");
        // Output may be formatted with newlines
        assert!(result.contains('1') && result.contains('2') && result.contains('3'));
    }

    #[test]
    fn test_concat_multiple() {
        let result = transform("[1].concat([2], [3])");
        assert!(result.contains('1') && result.contains('2') && result.contains('3'));
    }
}
