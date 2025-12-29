//! String extraction passes.
//!
//! - `Slice` — `"hello".slice(1, 3)` → `"el"`
//! - `Substring` — `"hello".substring(1, 3)` → `"el"`
//! - `Substr` — `"hello".substr(1, 2)` → `"el"`
//! - `Split` — `"a,b".split(",")` → `["a", "b"]`

use oxc::ast::ast::{Argument, ArrayExpressionElement, Expression};
use oxc::span::SPAN;
use oxc_traverse::TraverseCtx;

use crate::utils::literal;

// ============================================================================
// Individual Passes
// ============================================================================

/// Evaluates slice: `"hello".slice(1, 3)` → `"el"`
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

        let Some(s) = literal::string(&member.object) else {
            return 0;
        };
        let Some(start) = call.arguments.first().and_then(|a| get_number_arg(a)) else {
            return 0;
        };

        let end = call.arguments.get(1).and_then(|a| get_number_arg(a));

        let chars: Vec<char> = s.chars().collect();
        let len = chars.len() as i64;

        let start_idx = normalize_index(start as i64, len);
        let end_idx = match end {
            Some(e) => normalize_index(e as i64, len),
            None => len as usize,
        };

        if start_idx >= end_idx {
            *expr = ctx
                .ast
                .expression_string_literal(SPAN, ctx.ast.atom(""), None);
            return 1;
        }

        let result: String = chars[start_idx..end_idx].iter().collect();
        *expr = ctx
            .ast
            .expression_string_literal(SPAN, ctx.ast.atom(&result), None);
        1
    }
}

/// Evaluates substring: `"hello".substring(1, 3)` → `"el"`
#[derive(Default)]
pub struct Substring;

impl Substring {
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
        if member.property.name.as_str() != "substring" {
            return 0;
        }

        let Some(s) = literal::string(&member.object) else {
            return 0;
        };
        let Some(start) = call.arguments.first().and_then(|a| get_number_arg(a)) else {
            return 0;
        };

        let end = call.arguments.get(1).and_then(|a| get_number_arg(a));

        let chars: Vec<char> = s.chars().collect();
        let len = chars.len();

        // substring clamps to 0 and swaps if start > end
        let mut start_idx = start.max(0.0) as usize;
        let mut end_idx = match end {
            Some(e) => e.max(0.0) as usize,
            None => len,
        };

        start_idx = start_idx.min(len);
        end_idx = end_idx.min(len);

        if start_idx > end_idx {
            std::mem::swap(&mut start_idx, &mut end_idx);
        }

        let result: String = chars[start_idx..end_idx].iter().collect();
        *expr = ctx
            .ast
            .expression_string_literal(SPAN, ctx.ast.atom(&result), None);
        1
    }
}

/// Evaluates substr: `"hello".substr(1, 2)` → `"el"`
#[derive(Default)]
pub struct Substr;

impl Substr {
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
        if member.property.name.as_str() != "substr" {
            return 0;
        }

        let Some(s) = literal::string(&member.object) else {
            return 0;
        };
        let Some(start) = call.arguments.first().and_then(|a| get_number_arg(a)) else {
            return 0;
        };

        let length = call.arguments.get(1).and_then(|a| get_number_arg(a));

        let chars: Vec<char> = s.chars().collect();
        let len = chars.len() as i64;

        let start_idx = if start < 0.0 {
            (len + start as i64).max(0) as usize
        } else {
            (start as usize).min(len as usize)
        };

        let count = match length {
            Some(l) if l < 0.0 => 0,
            Some(l) => l as usize,
            None => len as usize,
        };

        let end_idx = (start_idx + count).min(len as usize);
        let result: String = chars[start_idx..end_idx].iter().collect();
        *expr = ctx
            .ast
            .expression_string_literal(SPAN, ctx.ast.atom(&result), None);
        1
    }
}

/// Evaluates split: `"a,b".split(",")` → `["a", "b"]`
#[derive(Default)]
pub struct Split;

impl Split {
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
        if member.property.name.as_str() != "split" {
            return 0;
        }

        let Some(s) = literal::string(&member.object) else {
            return 0;
        };
        let Some(separator) = call.arguments.first().and_then(|a| get_string_arg(a)) else {
            return 0;
        };

        let limit = call
            .arguments
            .get(1)
            .and_then(|a| get_number_arg(a))
            .map(|n| n as usize);

        let parts: Vec<&str> = if separator.is_empty() {
            s.char_indices()
                .map(|(i, c)| &s[i..i + c.len_utf8()])
                .collect()
        } else {
            s.split(separator).collect()
        };

        let parts: Vec<&str> = match limit {
            Some(0) => vec![],
            Some(n) => parts.into_iter().take(n).collect(),
            None => parts,
        };

        // Limit array size for safety
        if parts.len() > 1000 {
            return 0;
        }

        let mut elements = ctx.ast.vec();
        for part in parts {
            let lit = ctx
                .ast
                .expression_string_literal(SPAN, ctx.ast.atom(part), None);
            elements.push(ArrayExpressionElement::from(lit));
        }

        *expr = ctx.ast.expression_array(SPAN, elements);
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

fn get_string_arg<'a>(arg: &'a Argument<'a>) -> Option<&'a str> {
    match arg {
        Argument::StringLiteral(lit) => Some(lit.value.as_str()),
        _ => arg.as_expression().and_then(literal::string),
    }
}

fn normalize_index(idx: i64, len: i64) -> usize {
    if idx < 0 {
        (len + idx).max(0) as usize
    } else {
        (idx as usize).min(len as usize)
    }
}

// ============================================================================
// Extract Group
// ============================================================================

/// Group of all string extraction passes.
#[derive(Default)]
pub struct Extract {
    slice: Slice,
    substring: Substring,
    substr: Substr,
    split: Split,
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
        mods += self.substring.transform(expr, ctx);
        mods += self.substr.transform(expr, ctx);
        mods += self.split.transform(expr, ctx);
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
        let result = transform("\"hello\".slice(1, 3)");
        assert!(result.contains("\"el\""));
    }

    #[test]
    fn test_slice_negative() {
        let result = transform("\"hello\".slice(-2)");
        assert!(result.contains("\"lo\""));
    }

    #[test]
    fn test_substring() {
        let result = transform("\"hello\".substring(1, 3)");
        assert!(result.contains("\"el\""));
    }

    #[test]
    fn test_substr() {
        let result = transform("\"hello\".substr(1, 2)");
        assert!(result.contains("\"el\""));
    }

    #[test]
    fn test_split() {
        let result = transform("\"a,b,c\".split(\",\")");
        assert!(result.contains("\"a\""));
        assert!(result.contains("\"b\""));
        assert!(result.contains("\"c\""));
    }
}
