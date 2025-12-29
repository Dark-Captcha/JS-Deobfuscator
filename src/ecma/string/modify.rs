//! String modification passes.
//!
//! - `Concat` — `"a".concat("b")` → `"ab"`
//! - `Repeat` — `"ab".repeat(3)` → `"ababab"`
//! - `Replace` — `"hello".replace("l", "x")` → `"hexlo"`
//! - `ReplaceAll` — `"hello".replaceAll("l", "x")` → `"hexxo"`
//! - `PadStart` — `"5".padStart(2, "0")` → `"05"`
//! - `PadEnd` — `"5".padEnd(2, "0")` → `"50"`

use oxc::ast::ast::{Argument, Expression};
use oxc::span::SPAN;
use oxc_traverse::TraverseCtx;

use crate::utils::literal;

// ============================================================================
// Individual Passes
// ============================================================================

/// Evaluates concat: `"a".concat("b")` → `"ab"`
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

        let Some(s) = literal::string(&member.object) else {
            return 0;
        };

        let mut result = s.to_string();
        for arg in &call.arguments {
            let Some(part) = get_string_arg(arg) else {
                return 0;
            };
            result.push_str(part);
        }

        *expr = ctx
            .ast
            .expression_string_literal(SPAN, ctx.ast.atom(&result), None);
        1
    }
}

/// Evaluates repeat: `"ab".repeat(3)` → `"ababab"`
#[derive(Default)]
pub struct Repeat;

impl Repeat {
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
        if member.property.name.as_str() != "repeat" {
            return 0;
        }

        let Some(s) = literal::string(&member.object) else {
            return 0;
        };
        let Some(count) = call.arguments.first().and_then(|a| get_number_arg(a)) else {
            return 0;
        };

        if count < 0.0 || count.is_nan() || count.is_infinite() {
            return 0;
        }
        let count = count as usize;

        // Limit result size for safety
        if s.len() * count > 10000 {
            return 0;
        }

        let result = s.repeat(count);
        *expr = ctx
            .ast
            .expression_string_literal(SPAN, ctx.ast.atom(&result), None);
        1
    }
}

/// Evaluates replace: `"hello".replace("l", "x")` → `"hexlo"`
#[derive(Default)]
pub struct Replace;

impl Replace {
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
        if member.property.name.as_str() != "replace" {
            return 0;
        }

        let Some(s) = literal::string(&member.object) else {
            return 0;
        };
        let Some(search) = call.arguments.first().and_then(|a| get_string_arg(a)) else {
            return 0;
        };
        let Some(replacement) = call.arguments.get(1).and_then(|a| get_string_arg(a)) else {
            return 0;
        };

        // Only replace first occurrence
        let result = s.replacen(search, replacement, 1);
        *expr = ctx
            .ast
            .expression_string_literal(SPAN, ctx.ast.atom(&result), None);
        1
    }
}

/// Evaluates replaceAll: `"hello".replaceAll("l", "x")` → `"hexxo"`
#[derive(Default)]
pub struct ReplaceAll;

impl ReplaceAll {
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
        if member.property.name.as_str() != "replaceAll" {
            return 0;
        }

        let Some(s) = literal::string(&member.object) else {
            return 0;
        };
        let Some(search) = call.arguments.first().and_then(|a| get_string_arg(a)) else {
            return 0;
        };
        let Some(replacement) = call.arguments.get(1).and_then(|a| get_string_arg(a)) else {
            return 0;
        };

        let result = s.replace(search, replacement);
        *expr = ctx
            .ast
            .expression_string_literal(SPAN, ctx.ast.atom(&result), None);
        1
    }
}

/// Evaluates padStart: `"5".padStart(2, "0")` → `"05"`
#[derive(Default)]
pub struct PadStart;

impl PadStart {
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
        if member.property.name.as_str() != "padStart" {
            return 0;
        }

        let Some(s) = literal::string(&member.object) else {
            return 0;
        };
        let Some(target_len) = call.arguments.first().and_then(|a| get_number_arg(a)) else {
            return 0;
        };

        let target_len = target_len.max(0.0) as usize;
        if target_len > 10000 {
            return 0;
        } // Safety limit

        let pad_str = call
            .arguments
            .get(1)
            .and_then(|a| get_string_arg(a))
            .unwrap_or(" ");

        if pad_str.is_empty() || s.len() >= target_len {
            *expr = ctx
                .ast
                .expression_string_literal(SPAN, ctx.ast.atom(s), None);
            return 1;
        }

        let pad_len = target_len - s.len();
        let mut result = String::with_capacity(target_len);
        let mut remaining = pad_len;
        while remaining > 0 {
            let take = remaining.min(pad_str.len());
            result.push_str(&pad_str[..take]);
            remaining -= take;
        }
        result.push_str(s);

        *expr = ctx
            .ast
            .expression_string_literal(SPAN, ctx.ast.atom(&result), None);
        1
    }
}

/// Evaluates padEnd: `"5".padEnd(2, "0")` → `"50"`
#[derive(Default)]
pub struct PadEnd;

impl PadEnd {
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
        if member.property.name.as_str() != "padEnd" {
            return 0;
        }

        let Some(s) = literal::string(&member.object) else {
            return 0;
        };
        let Some(target_len) = call.arguments.first().and_then(|a| get_number_arg(a)) else {
            return 0;
        };

        let target_len = target_len.max(0.0) as usize;
        if target_len > 10000 {
            return 0;
        } // Safety limit

        let pad_str = call
            .arguments
            .get(1)
            .and_then(|a| get_string_arg(a))
            .unwrap_or(" ");

        if pad_str.is_empty() || s.len() >= target_len {
            *expr = ctx
                .ast
                .expression_string_literal(SPAN, ctx.ast.atom(s), None);
            return 1;
        }

        let pad_len = target_len - s.len();
        let mut result = s.to_string();
        let mut remaining = pad_len;
        while remaining > 0 {
            let take = remaining.min(pad_str.len());
            result.push_str(&pad_str[..take]);
            remaining -= take;
        }

        *expr = ctx
            .ast
            .expression_string_literal(SPAN, ctx.ast.atom(&result), None);
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
// Modify Group
// ============================================================================

/// Group of all string modification passes.
#[derive(Default)]
pub struct Modify {
    concat: Concat,
    repeat: Repeat,
    replace: Replace,
    replace_all: ReplaceAll,
    pad_start: PadStart,
    pad_end: PadEnd,
}

impl Modify {
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
        mods += self.concat.transform(expr, ctx);
        mods += self.repeat.transform(expr, ctx);
        mods += self.replace.transform(expr, ctx);
        mods += self.replace_all.transform(expr, ctx);
        mods += self.pad_start.transform(expr, ctx);
        mods += self.pad_end.transform(expr, ctx);
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

        struct Visitor(Modify);
        impl<'a> oxc_traverse::Traverse<'a, ()> for Visitor {
            fn exit_expression(
                &mut self,
                expr: &mut Expression<'a>,
                ctx: &mut TraverseCtx<'a, ()>,
            ) {
                self.0.transform(expr, ctx);
            }
        }

        let mut visitor = Visitor(Modify::new());
        traverse_mut(&mut visitor, &allocator, &mut program, scoping, ());
        codegen(&program)
    }

    #[test]
    fn test_concat() {
        let result = transform("\"a\".concat(\"b\", \"c\")");
        assert!(result.contains("\"abc\""));
    }

    #[test]
    fn test_repeat() {
        let result = transform("\"ab\".repeat(3)");
        assert!(result.contains("\"ababab\""));
    }

    #[test]
    fn test_replace() {
        let result = transform("\"hello\".replace(\"l\", \"x\")");
        assert!(result.contains("\"hexlo\""));
    }

    #[test]
    fn test_replace_all() {
        let result = transform("\"hello\".replaceAll(\"l\", \"x\")");
        assert!(result.contains("\"hexxo\""));
    }

    #[test]
    fn test_pad_start() {
        let result = transform("\"5\".padStart(2, \"0\")");
        assert!(result.contains("\"05\""));
    }

    #[test]
    fn test_pad_end() {
        let result = transform("\"5\".padEnd(2, \"0\")");
        assert!(result.contains("\"50\""));
    }
}
