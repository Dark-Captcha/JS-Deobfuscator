//! Test helpers for transformation passes.
//!
//! Provides utilities for writing concise pass tests.

use oxc::allocator::Allocator;
use oxc::ast::ast::{Expression, Program};
use oxc::codegen::Codegen;
use oxc::parser::Parser;
use oxc::span::SourceType;

// ============================================================================
// Parsing Helpers
// ============================================================================

/// Parse an expression from source string.
///
/// # Panics
///
/// Panics if parsing fails.
pub fn parse_expr<'a>(allocator: &'a Allocator, source: &'a str) -> Expression<'a> {
    Parser::new(allocator, source, SourceType::mjs())
        .parse_expression()
        .expect("Failed to parse expression")
}

/// Parse a program from source string.
///
/// # Panics
///
/// Panics if parsing fails.
pub fn parse_program<'a>(allocator: &'a Allocator, source: &'a str) -> Program<'a> {
    let ret = Parser::new(allocator, source, SourceType::mjs()).parse();
    if ret.panicked {
        panic!("Parser panicked");
    }
    if !ret.errors.is_empty() {
        panic!("Parse errors: {:?}", ret.errors);
    }
    ret.program
}

// ============================================================================
// Code Generation Helpers
// ============================================================================

/// Generate code from a program.
pub fn codegen(program: &Program<'_>) -> String {
    Codegen::new().build(program).code
}

/// Generate code from a program (minified).
pub fn codegen_minified(program: &Program<'_>) -> String {
    use oxc::codegen::CodegenOptions;
    Codegen::new()
        .with_options(CodegenOptions::minify())
        .build(program)
        .code
}

// ============================================================================
// Normalization Helpers
// ============================================================================

/// Normalize code for comparison.
///
/// Removes whitespace differences for easier assertion.
pub fn normalize(code: &str) -> String {
    // Parse and regenerate to normalize
    let allocator = Allocator::default();
    let program = parse_program(&allocator, code);
    codegen_minified(&program)
}

// ============================================================================
// Assertion Helpers
// ============================================================================

/// Assert that two code strings are semantically equivalent.
///
/// Normalizes both strings before comparison.
pub fn assert_code_eq(actual: &str, expected: &str) {
    let actual_norm = normalize(actual);
    let expected_norm = normalize(expected);
    assert_eq!(
        actual_norm, expected_norm,
        "\nActual:\n{actual}\n\nExpected:\n{expected}\n"
    );
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_expr() {
        let alloc = Allocator::default();
        let expr = parse_expr(&alloc, "1 + 2");
        assert!(matches!(expr, Expression::BinaryExpression(_)));
    }

    #[test]
    fn test_parse_program() {
        let alloc = Allocator::default();
        let program = parse_program(&alloc, "const x = 1;");
        assert_eq!(program.body.len(), 1);
    }

    #[test]
    fn test_normalize() {
        let code1 = "const   x  =  1 ;";
        let code2 = "const x=1;";
        assert_eq!(normalize(code1), normalize(code2));
    }

    #[test]
    fn test_assert_code_eq() {
        assert_code_eq("const x = 1;", "const x=1;");
    }
}
