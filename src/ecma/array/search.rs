//! Array search passes.
//!
//! - `IndexOf` — `[1,2,3].indexOf(2)` → `1`
//! - `LastIndexOf` — `[1,2,1].lastIndexOf(1)` → `2`
//! - `Includes` — `[1,2,3].includes(2)` → `true`

use oxc::ast::ast::{Argument, Expression};
use oxc::span::SPAN;
use oxc_traverse::TraverseCtx;

use crate::utils::literal::{self, LiteralValue};

// ============================================================================
// Individual Passes
// ============================================================================

/// Evaluates indexOf: `[1,2,3].indexOf(2)` → `1`
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

        let Some(elements) = literal::array_elements(&member.object) else {
            return 0;
        };
        let Some(search_value) = call.arguments.first().and_then(get_literal_arg) else {
            return 0;
        };

        // Get optional fromIndex
        let from_index = call
            .arguments
            .get(1)
            .and_then(get_number_arg)
            .map(|n| n as i64)
            .unwrap_or(0);

        let len = elements.len() as i64;
        let start = if from_index < 0 {
            (len + from_index).max(0) as usize
        } else {
            from_index as usize
        };

        for (i, elem) in elements.iter().enumerate().skip(start) {
            if let Some(elem_value) = LiteralValue::from_expr(elem)
                && values_equal(&elem_value, &search_value)
            {
                *expr = literal::make_number(i as f64, &ctx.ast);
                return 1;
            }
        }

        *expr = literal::make_number(-1.0, &ctx.ast);
        1
    }
}

/// Evaluates lastIndexOf: `[1,2,1].lastIndexOf(1)` → `2`
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

        let Some(elements) = literal::array_elements(&member.object) else {
            return 0;
        };
        let Some(search_value) = call.arguments.first().and_then(get_literal_arg) else {
            return 0;
        };

        // Get optional fromIndex (defaults to len - 1)
        let len = elements.len() as i64;
        let from_index = call
            .arguments
            .get(1)
            .and_then(get_number_arg)
            .map(|n| n as i64)
            .unwrap_or(len - 1);

        let end = if from_index < 0 {
            (len + from_index) as usize
        } else {
            from_index.min(len - 1) as usize
        };

        for i in (0..=end).rev() {
            if i >= elements.len() {
                continue;
            }
            if let Some(elem_value) = LiteralValue::from_expr(elements[i])
                && values_equal(&elem_value, &search_value)
            {
                *expr = literal::make_number(i as f64, &ctx.ast);
                return 1;
            }
        }

        *expr = literal::make_number(-1.0, &ctx.ast);
        1
    }
}

/// Evaluates includes: `[1,2,3].includes(2)` → `true`
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

        let Some(elements) = literal::array_elements(&member.object) else {
            return 0;
        };
        let Some(search_value) = call.arguments.first().and_then(get_literal_arg) else {
            return 0;
        };

        // Get optional fromIndex
        let from_index = call
            .arguments
            .get(1)
            .and_then(get_number_arg)
            .map(|n| n as i64)
            .unwrap_or(0);

        let len = elements.len() as i64;
        let start = if from_index < 0 {
            (len + from_index).max(0) as usize
        } else {
            from_index as usize
        };

        for elem in elements.iter().skip(start) {
            if let Some(elem_value) = LiteralValue::from_expr(elem)
                && values_equal(&elem_value, &search_value)
            {
                *expr = ctx.ast.expression_boolean_literal(SPAN, true);
                return 1;
            }
        }

        *expr = ctx.ast.expression_boolean_literal(SPAN, false);
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

fn get_literal_arg(arg: &Argument) -> Option<LiteralValue> {
    arg.as_expression().and_then(LiteralValue::from_expr)
}

fn values_equal(a: &LiteralValue, b: &LiteralValue) -> bool {
    match (a, b) {
        (LiteralValue::Number(x), LiteralValue::Number(y)) => {
            // Handle NaN
            if x.is_nan() && y.is_nan() {
                return false;
            }
            (x - y).abs() < f64::EPSILON
        }
        (LiteralValue::String(x), LiteralValue::String(y)) => x == y,
        (LiteralValue::Boolean(x), LiteralValue::Boolean(y)) => x == y,
        (LiteralValue::Null, LiteralValue::Null) => true,
        (LiteralValue::Undefined, LiteralValue::Undefined) => true,
        _ => false,
    }
}

// ============================================================================
// Search Group
// ============================================================================

/// Group of all array search passes.
#[derive(Default)]
pub struct Search {
    index_of: IndexOf,
    last_index_of: LastIndexOf,
    includes: Includes,
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
        let result = transform("[1, 2, 3].indexOf(2)");
        assert!(result.contains('1'));
    }

    #[test]
    fn test_index_of_not_found() {
        let result = transform("[1, 2, 3].indexOf(5)");
        assert!(result.contains("-1"));
    }

    #[test]
    fn test_last_index_of() {
        let result = transform("[1, 2, 1].lastIndexOf(1)");
        assert!(result.contains('2'));
    }

    #[test]
    fn test_includes_true() {
        let result = transform("[1, 2, 3].includes(2)");
        assert!(result.contains("true"));
    }

    #[test]
    fn test_includes_false() {
        let result = transform("[1, 2, 3].includes(5)");
        assert!(result.contains("false"));
    }
}
