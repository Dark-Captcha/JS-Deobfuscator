//! Base64 encoding/decoding passes.
//!
//! - `Atob` — `atob("aGVsbG8=")` → `"hello"`
//! - `Btoa` — `btoa("hello")` → `"aGVsbG8="`
//!
//! # Availability
//!
//! - Browser: Always available
//! - Node.js: Available since v16.0.0
//!
//! # Notes
//!
//! - `atob` decodes Base64 to Latin-1 string (not UTF-8)
//! - `btoa` encodes Latin-1 string to Base64
//! - Invalid Base64 input → pass unchanged (don't error)

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use oxc::ast::ast::Expression;
use oxc_traverse::TraverseCtx;

use crate::utils::literal;

// ============================================================================
// Individual Passes
// ============================================================================

/// Evaluates `atob(base64)` — Base64 decode.
///
/// # Example
/// ```ignore
/// atob("aGVsbG8=") → "hello"
/// atob("SGVsbG8gV29ybGQ=") → "Hello World"
/// ```
#[derive(Default)]
pub struct Atob;

impl Atob {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Expression::CallExpression(call) = expr else {
            return 0;
        };

        // Check for atob(arg)
        let Expression::Identifier(callee) = &call.callee else {
            return 0;
        };
        if callee.name != "atob" {
            return 0;
        }

        // Must have exactly one argument
        if call.arguments.len() != 1 {
            return 0;
        }

        // Get string argument
        let Some(arg) = call.arguments[0].as_expression() else {
            return 0;
        };
        let Some(input) = literal::string(arg) else {
            return 0;
        };

        // Decode Base64
        let Ok(decoded_bytes) = STANDARD.decode(input) else {
            return 0;
        };

        // Convert to Latin-1 string (each byte becomes a char)
        // atob produces a "binary string" where each byte is a code point
        let decoded: String = decoded_bytes.iter().map(|&b| b as char).collect();

        *expr = literal::make_string(&decoded, &ctx.ast);
        1
    }
}

/// Evaluates `btoa(string)` — Base64 encode.
///
/// # Example
/// ```ignore
/// btoa("hello") → "aGVsbG8="
/// btoa("Hello World") → "SGVsbG8gV29ybGQ="
/// ```
#[derive(Default)]
pub struct Btoa;

impl Btoa {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Expression::CallExpression(call) = expr else {
            return 0;
        };

        // Check for btoa(arg)
        let Expression::Identifier(callee) = &call.callee else {
            return 0;
        };
        if callee.name != "btoa" {
            return 0;
        }

        // Must have exactly one argument
        if call.arguments.len() != 1 {
            return 0;
        }

        // Get string argument
        let Some(arg) = call.arguments[0].as_expression() else {
            return 0;
        };
        let Some(input) = literal::string(arg) else {
            return 0;
        };

        // btoa only accepts Latin-1 characters (0x00-0xFF)
        // If any character is > 0xFF, it would throw in JS
        if input.chars().any(|c| c as u32 > 0xFF) {
            return 0;
        }

        // Convert string to bytes (Latin-1 encoding)
        let bytes: Vec<u8> = input.chars().map(|c| c as u8).collect();

        // Encode to Base64
        let encoded = STANDARD.encode(&bytes);

        *expr = literal::make_string(&encoded, &ctx.ast);
        1
    }
}

// ============================================================================
// Encoding Group
// ============================================================================

/// Group of all encoding passes.
#[derive(Default)]
pub struct EncodingGroup {
    atob: Atob,
    btoa: Btoa,
}

impl EncodingGroup {
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
        mods += self.atob.transform(expr, ctx);
        mods += self.btoa.transform(expr, ctx);
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

        struct Visitor(EncodingGroup);
        impl<'a> oxc_traverse::Traverse<'a, ()> for Visitor {
            fn exit_expression(
                &mut self,
                expr: &mut Expression<'a>,
                ctx: &mut TraverseCtx<'a, ()>,
            ) {
                self.0.transform(expr, ctx);
            }
        }

        let mut visitor = Visitor(EncodingGroup::new());
        traverse_mut(&mut visitor, &allocator, &mut program, scoping, ());
        codegen(&program)
    }

    #[test]
    fn test_atob_basic() {
        // Use semicolon prefix to avoid directive interpretation
        assert_eq!(transform(";atob(\"aGVsbG8=\")"), ";\n\"hello\";\n");
    }

    #[test]
    fn test_atob_hello_world() {
        assert_eq!(
            transform(";atob(\"SGVsbG8gV29ybGQ=\")"),
            ";\n\"Hello World\";\n"
        );
    }

    #[test]
    fn test_atob_empty() {
        assert_eq!(transform(";atob(\"\")"), ";\n\"\";\n");
    }

    #[test]
    fn test_atob_invalid_unchanged() {
        // Invalid base64 should remain unchanged
        assert_eq!(transform(";atob(\"!!!\")"), ";\natob(\"!!!\");\n");
    }

    #[test]
    fn test_btoa_basic() {
        assert_eq!(transform(";btoa(\"hello\")"), ";\n\"aGVsbG8=\";\n");
    }

    #[test]
    fn test_btoa_hello_world() {
        assert_eq!(
            transform(";btoa(\"Hello World\")"),
            ";\n\"SGVsbG8gV29ybGQ=\";\n"
        );
    }

    #[test]
    fn test_btoa_empty() {
        assert_eq!(transform(";btoa(\"\")"), ";\n\"\";\n");
    }

    #[test]
    fn test_btoa_binary() {
        // Binary string with bytes 0-255
        assert_eq!(transform(";btoa(\"\\x00\\x01\\x02\")"), ";\n\"AAEC\";\n");
    }

    #[test]
    fn test_roundtrip() {
        // atob(btoa("test")) should work in sequence
        // First btoa transforms, then atob
        let result = transform(";atob(btoa(\"test\"))");
        // After btoa: atob("dGVzdA==")
        // After atob: "test"
        assert_eq!(result, ";\n\"test\";\n");
    }
}
