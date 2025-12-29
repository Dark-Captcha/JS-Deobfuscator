//! URI encoding/decoding passes.
//!
//! - `EncodeURI` — `encodeURI(" ")` → `"%20"`
//! - `DecodeURI` — `decodeURI("%20")` → `" "`
//! - `EncodeURIComponent` — `encodeURIComponent("&")` → `"%26"`
//! - `DecodeURIComponent` — `decodeURIComponent("%26")` → `"&"`

use oxc::ast::ast::Expression;
use oxc::span::SPAN;
use oxc_traverse::TraverseCtx;

use crate::utils::literal;

// Characters NOT encoded by encodeURI
const URI_UNESCAPED: &str =
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_.!~*'()";
const URI_RESERVED: &str = ";/?:@&=+$,#";

// Characters NOT encoded by encodeURIComponent
const COMPONENT_UNESCAPED: &str =
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_.!~*'()";

// ============================================================================
// Individual Passes
// ============================================================================

/// Evaluates encodeURI
#[derive(Default)]
pub struct EncodeURI;

impl EncodeURI {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Expression::CallExpression(call) = expr else {
            return 0;
        };
        let Expression::Identifier(callee) = &call.callee else {
            return 0;
        };
        if callee.name.as_str() != "encodeURI" {
            return 0;
        }

        let Some(s) = call
            .arguments
            .first()
            .and_then(|a| a.as_expression())
            .and_then(literal::string)
        else {
            return 0;
        };

        let result = encode_uri(s);
        *expr = ctx
            .ast
            .expression_string_literal(SPAN, ctx.ast.atom(&result), None);
        1
    }
}

/// Evaluates decodeURI
#[derive(Default)]
pub struct DecodeURI;

impl DecodeURI {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Expression::CallExpression(call) = expr else {
            return 0;
        };
        let Expression::Identifier(callee) = &call.callee else {
            return 0;
        };
        if callee.name.as_str() != "decodeURI" {
            return 0;
        }

        let Some(s) = call
            .arguments
            .first()
            .and_then(|a| a.as_expression())
            .and_then(literal::string)
        else {
            return 0;
        };

        let Some(result) = decode_uri(s, true) else {
            return 0;
        };
        *expr = ctx
            .ast
            .expression_string_literal(SPAN, ctx.ast.atom(&result), None);
        1
    }
}

/// Evaluates encodeURIComponent
#[derive(Default)]
pub struct EncodeURIComponent;

impl EncodeURIComponent {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Expression::CallExpression(call) = expr else {
            return 0;
        };
        let Expression::Identifier(callee) = &call.callee else {
            return 0;
        };
        if callee.name.as_str() != "encodeURIComponent" {
            return 0;
        }

        let Some(s) = call
            .arguments
            .first()
            .and_then(|a| a.as_expression())
            .and_then(literal::string)
        else {
            return 0;
        };

        let result = encode_uri_component(s);
        *expr = ctx
            .ast
            .expression_string_literal(SPAN, ctx.ast.atom(&result), None);
        1
    }
}

/// Evaluates decodeURIComponent
#[derive(Default)]
pub struct DecodeURIComponent;

impl DecodeURIComponent {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Expression::CallExpression(call) = expr else {
            return 0;
        };
        let Expression::Identifier(callee) = &call.callee else {
            return 0;
        };
        if callee.name.as_str() != "decodeURIComponent" {
            return 0;
        }

        let Some(s) = call
            .arguments
            .first()
            .and_then(|a| a.as_expression())
            .and_then(literal::string)
        else {
            return 0;
        };

        let Some(result) = decode_uri(s, false) else {
            return 0;
        };
        *expr = ctx
            .ast
            .expression_string_literal(SPAN, ctx.ast.atom(&result), None);
        1
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn encode_uri(s: &str) -> String {
    let mut result = String::with_capacity(s.len() * 3);
    for c in s.chars() {
        if URI_UNESCAPED.contains(c) || URI_RESERVED.contains(c) {
            result.push(c);
        } else {
            for byte in c.to_string().as_bytes() {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}

fn encode_uri_component(s: &str) -> String {
    let mut result = String::with_capacity(s.len() * 3);
    for c in s.chars() {
        if COMPONENT_UNESCAPED.contains(c) {
            result.push(c);
        } else {
            for byte in c.to_string().as_bytes() {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}

fn decode_uri(s: &str, preserve_reserved: bool) -> Option<String> {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if hex.len() != 2 {
                return None;
            }
            let byte = u8::from_str_radix(&hex, 16).ok()?;
            let decoded = byte as char;

            if preserve_reserved && URI_RESERVED.contains(decoded) {
                result.push('%');
                result.push_str(&hex);
            } else {
                result.push(decoded);
            }
        } else {
            result.push(c);
        }
    }
    Some(result)
}

// ============================================================================
// UriGroup
// ============================================================================

/// Group of all URI passes.
#[derive(Default)]
pub struct UriGroup {
    encode_uri: EncodeURI,
    decode_uri: DecodeURI,
    encode_uri_component: EncodeURIComponent,
    decode_uri_component: DecodeURIComponent,
}

impl UriGroup {
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
        mods += self.encode_uri.transform(expr, ctx);
        mods += self.decode_uri.transform(expr, ctx);
        mods += self.encode_uri_component.transform(expr, ctx);
        mods += self.decode_uri_component.transform(expr, ctx);
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

        struct Visitor(UriGroup);
        impl<'a> oxc_traverse::Traverse<'a, ()> for Visitor {
            fn exit_expression(
                &mut self,
                expr: &mut Expression<'a>,
                ctx: &mut TraverseCtx<'a, ()>,
            ) {
                self.0.transform(expr, ctx);
            }
        }

        let mut visitor = Visitor(UriGroup::new());
        traverse_mut(&mut visitor, &allocator, &mut program, scoping, ());
        codegen(&program)
    }

    #[test]
    fn test_encode_uri() {
        let result = transform("encodeURI(\"hello world\")");
        assert!(result.contains("hello%20world"));
    }

    #[test]
    fn test_decode_uri() {
        let result = transform("decodeURI(\"hello%20world\")");
        assert!(result.contains("hello world"));
    }

    #[test]
    fn test_encode_uri_component() {
        let result = transform("encodeURIComponent(\"a&b\")");
        assert!(result.contains("a%26b"));
    }

    #[test]
    fn test_decode_uri_component() {
        let result = transform("decodeURIComponent(\"a%26b\")");
        assert!(result.contains("a&b"));
    }
}
