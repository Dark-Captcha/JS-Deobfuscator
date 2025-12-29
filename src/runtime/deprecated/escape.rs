//! Deprecated escape/unescape passes.
//!
//! - `Escape` — `escape(" ")` → `"%20"`
//! - `Unescape` — `unescape("%20")` → `" "`
//!
//! # Availability
//!
//! - Browser: Available (deprecated)
//! - Node.js: Not available natively
//!
//! # Notes
//!
//! - These are deprecated but still used by obfuscators
//! - Different from `encodeURI`/`decodeURI` (which are ES standard)
//! - `escape` doesn't encode: `@*_+-./`
//! - `unescape` handles `%XX` and `%uXXXX` patterns

use oxc::ast::ast::Expression;
use oxc_traverse::TraverseCtx;

use crate::utils::literal;

// ============================================================================
// Constants
// ============================================================================

/// Characters that escape() does NOT encode.
const ESCAPE_SAFE: &str = "@*_+-./";

// ============================================================================
// Individual Passes
// ============================================================================

/// Evaluates `escape(string)` — URL escape (deprecated).
///
/// # Example
/// ```ignore
/// escape(" ") → "%20"
/// escape("hello") → "hello"
/// escape("héllo") → "h%E9llo"
/// escape("日本") → "%u65E5%u672C"
/// ```
#[derive(Default)]
pub struct Escape;

impl Escape {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Expression::CallExpression(call) = expr else {
            return 0;
        };

        // Check for escape(arg)
        let Expression::Identifier(callee) = &call.callee else {
            return 0;
        };
        if callee.name != "escape" {
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

        // Encode the string
        let encoded = js_escape(input);

        *expr = literal::make_string(&encoded, &ctx.ast);
        1
    }
}

/// Evaluates `unescape(string)` — URL unescape (deprecated).
///
/// # Example
/// ```ignore
/// unescape("%20") → " "
/// unescape("hello") → "hello"
/// unescape("h%E9llo") → "héllo"
/// unescape("%u65E5%u672C") → "日本"
/// ```
#[derive(Default)]
pub struct Unescape;

impl Unescape {
    #[inline]
    pub fn transform<'a>(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) -> usize {
        let Expression::CallExpression(call) = expr else {
            return 0;
        };

        // Check for unescape(arg)
        let Expression::Identifier(callee) = &call.callee else {
            return 0;
        };
        if callee.name != "unescape" {
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

        // Decode the string
        let Some(decoded) = js_unescape(input) else {
            return 0;
        };

        *expr = literal::make_string(&decoded, &ctx.ast);
        1
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// JavaScript escape() implementation.
fn js_escape(input: &str) -> String {
    let mut result = String::with_capacity(input.len() * 3);

    for c in input.chars() {
        let code = c as u32;

        if c.is_ascii_alphanumeric() || ESCAPE_SAFE.contains(c) {
            // Safe characters pass through
            result.push(c);
        } else if code <= 0xFF {
            // Latin-1 characters: %XX
            result.push_str(&format!("%{:02X}", code));
        } else {
            // Unicode characters: %uXXXX
            result.push_str(&format!("%u{:04X}", code));
        }
    }

    result
}

/// JavaScript unescape() implementation.
///
/// Returns `None` if the input contains invalid escape sequences.
fn js_unescape(input: &str) -> Option<String> {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            // Check for %uXXXX (Unicode escape)
            if chars.peek() == Some(&'u') {
                chars.next(); // consume 'u'

                // Read 4 hex digits
                let mut hex = String::with_capacity(4);
                for _ in 0..4 {
                    match chars.next() {
                        Some(h) if h.is_ascii_hexdigit() => hex.push(h),
                        _ => {
                            // Invalid sequence, output literally
                            result.push('%');
                            result.push('u');
                            result.push_str(&hex);
                            continue;
                        }
                    }
                }

                if let Ok(code) = u32::from_str_radix(&hex, 16) {
                    if let Some(decoded) = char::from_u32(code) {
                        result.push(decoded);
                    } else {
                        // Invalid Unicode code point
                        result.push('%');
                        result.push('u');
                        result.push_str(&hex);
                    }
                }
            } else {
                // Check for %XX (Latin-1 escape)
                let mut hex = String::with_capacity(2);
                for _ in 0..2 {
                    match chars.peek() {
                        Some(&h) if h.is_ascii_hexdigit() => {
                            hex.push(h);
                            chars.next();
                        }
                        _ => break,
                    }
                }

                if hex.len() == 2 {
                    if let Ok(code) = u8::from_str_radix(&hex, 16) {
                        result.push(code as char);
                    }
                } else {
                    // Not enough hex digits, output literally
                    result.push('%');
                    result.push_str(&hex);
                }
            }
        } else {
            result.push(c);
        }
    }

    Some(result)
}

// ============================================================================
// Deprecated Group
// ============================================================================

/// Group of all deprecated API passes.
#[derive(Default)]
pub struct DeprecatedGroup {
    escape: Escape,
    unescape: Unescape,
}

impl DeprecatedGroup {
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
        mods += self.escape.transform(expr, ctx);
        mods += self.unescape.transform(expr, ctx);
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

        struct Visitor(DeprecatedGroup);
        impl<'a> oxc_traverse::Traverse<'a, ()> for Visitor {
            fn exit_expression(
                &mut self,
                expr: &mut Expression<'a>,
                ctx: &mut TraverseCtx<'a, ()>,
            ) {
                self.0.transform(expr, ctx);
            }
        }

        let mut visitor = Visitor(DeprecatedGroup::new());
        traverse_mut(&mut visitor, &allocator, &mut program, scoping, ());
        codegen(&program)
    }

    #[test]
    fn test_escape_space() {
        // Use semicolon prefix to avoid directive interpretation
        assert_eq!(transform(";escape(\" \")"), ";\n\"%20\";\n");
    }

    #[test]
    fn test_escape_safe_chars() {
        // Safe characters should not be encoded
        assert_eq!(transform(";escape(\"hello\")"), ";\n\"hello\";\n");
        assert_eq!(transform(";escape(\"@*_+-./\")"), ";\n\"@*_+-./\";\n");
    }

    #[test]
    fn test_escape_latin1() {
        // Latin-1 characters use %XX format
        assert_eq!(transform(";escape(\"é\")"), ";\n\"%E9\";\n");
    }

    #[test]
    fn test_escape_unicode() {
        // Unicode characters use %uXXXX format
        assert_eq!(transform(";escape(\"日\")"), ";\n\"%u65E5\";\n");
    }

    #[test]
    fn test_escape_mixed() {
        assert_eq!(transform(";escape(\"a b\")"), ";\n\"a%20b\";\n");
    }

    #[test]
    fn test_unescape_space() {
        assert_eq!(transform(";unescape(\"%20\")"), ";\n\" \";\n");
    }

    #[test]
    fn test_unescape_latin1() {
        assert_eq!(transform(";unescape(\"%E9\")"), ";\n\"é\";\n");
    }

    #[test]
    fn test_unescape_unicode() {
        assert_eq!(transform(";unescape(\"%u65E5\")"), ";\n\"日\";\n");
    }

    #[test]
    fn test_unescape_mixed() {
        assert_eq!(transform(";unescape(\"a%20b\")"), ";\n\"a b\";\n");
    }

    #[test]
    fn test_unescape_passthrough() {
        // Non-encoded characters pass through
        assert_eq!(transform(";unescape(\"hello\")"), ";\n\"hello\";\n");
    }

    #[test]
    fn test_roundtrip() {
        // unescape(escape("test")) should work
        let result = transform(";unescape(escape(\"test\"))");
        assert_eq!(result, ";\n\"test\";\n");
    }

    #[test]
    fn test_escape_empty() {
        assert_eq!(transform(";escape(\"\")"), ";\n\"\";\n");
    }

    #[test]
    fn test_unescape_empty() {
        assert_eq!(transform(";unescape(\"\")"), ";\n\"\";\n");
    }
}
