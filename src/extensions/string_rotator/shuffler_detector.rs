//! Phase 1: Find shuffler IIFE, extract symbols, and remove IIFE statement.
//!
//! Validation:
//! - push/shift pattern >= 2 (try + catch)
//! - parseInt pattern >= 2 (checksum calculation)
//!
//! Extracts:
//! - array_symbol_id: from first argument
//! - accessor_symbol_id: from first parseInt(accessor(...)) pattern
//! - code: for V8 execution
//!
//! Removes: the shuffler IIFE statement

use oxc::ast::ast::*;
use oxc::ast_visit::{Visit, walk};
use oxc::semantic::{Scoping, SymbolId};
use oxc::span::SPAN;
use oxc_traverse::{Traverse, TraverseCtx};
use rustc_hash::FxHashMap;

use crate::utils::codegen::call_to_code;
use crate::utils::scoping::get_reference_symbol;

// ============================================================================
// Types
// ============================================================================

pub type SystemId = usize;

#[derive(Debug, Clone)]
pub struct ShufflerInfo {
    pub array_symbol_id: SymbolId,
    pub accessor_symbol_id: SymbolId,
    pub code: String,
}

pub type ShufflerMap = FxHashMap<SystemId, ShufflerInfo>;

// ============================================================================
// ShufflerDetector
// ============================================================================

#[derive(Default)]
pub struct ShufflerDetector {
    pub results: ShufflerMap,
    counter: SystemId,
}

impl<'a> Traverse<'a, ()> for ShufflerDetector {
    fn exit_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a, ()>) {
        let Statement::ExpressionStatement(expr_stmt) = stmt else {
            return;
        };

        // Collect all IIFE calls from the expression
        let mut calls = Vec::new();
        Self::collect_iife_calls(&expr_stmt.expression, &mut calls);

        if calls.is_empty() {
            return;
        }

        let mut any_detected = false;
        for call in calls {
            if let Some(info) = Self::detect_shuffler_call(call, ctx.scoping()) {
                let id = self.counter;
                self.counter += 1;
                self.results.insert(id, info);
                any_detected = true;
            }
        }

        // Remove the statement if any shuffler was detected
        if any_detected {
            *stmt = ctx.ast.statement_empty(SPAN);
        }
    }
}

impl ShufflerDetector {
    /// Recursively collect IIFE CallExpressions.
    /// Handles: !(fn)(), (fn)(), (fn1)(), (fn2)(), etc.
    fn collect_iife_calls<'a>(expr: &'a Expression<'a>, calls: &mut Vec<&'a CallExpression<'a>>) {
        match expr {
            // !(function(){})() or +(function(){})() etc.
            Expression::UnaryExpression(u) => {
                if let Expression::CallExpression(c) = &u.argument
                    && Self::is_iife(c) {
                        calls.push(c.as_ref());
                    }
            }
            // (function(){})()
            Expression::CallExpression(c) => {
                if Self::is_iife(c) {
                    calls.push(c.as_ref());
                }
            }
            // (fn1)(), (fn2)() — SequenceExpression
            Expression::SequenceExpression(seq) => {
                for sub_expr in &seq.expressions {
                    Self::collect_iife_calls(sub_expr, calls);
                }
            }
            // ((function(){}))() — ParenthesizedExpression
            Expression::ParenthesizedExpression(p) => {
                Self::collect_iife_calls(&p.expression, calls);
            }
            _ => {}
        }
    }

    /// Check if a CallExpression is an IIFE (callee is FunctionExpression)
    fn is_iife(call: &CallExpression) -> bool {
        let callee = match &call.callee {
            Expression::ParenthesizedExpression(p) => &p.expression,
            e => e,
        };
        matches!(callee, Expression::FunctionExpression(_))
    }

    /// Detect shuffler IIFE pattern from a call expression.
    fn detect_shuffler_call(call: &CallExpression, scoping: &Scoping) -> Option<ShufflerInfo> {
        // Callee must be FunctionExpression (possibly parenthesized)
        let callee = match &call.callee {
            Expression::ParenthesizedExpression(p) => &p.expression,
            e => e,
        };
        let Expression::FunctionExpression(func) = callee else {
            return None;
        };

        // Get array symbol from first argument
        let Argument::Identifier(array_id) = call.arguments.first()? else {
            return None;
        };
        let array_symbol_id = get_reference_symbol(scoping, array_id)?;

        // Validate body and extract accessor symbol
        let body = func.body.as_ref()?;
        let mut validator = ShufflerValidator::new(scoping);
        validator.visit_function_body(body);

        // Validation: push/shift >= 2, parseInt >= 2
        if validator.push_shift_count < 2 {
            return None;
        }
        if validator.parse_int_count < 2 {
            return None;
        }

        // Must have found accessor symbol
        let accessor_symbol_id = validator.accessor_symbol_id?;

        Some(ShufflerInfo {
            array_symbol_id,
            accessor_symbol_id,
            code: call_to_code(call),
        })
    }
}

// ============================================================================
// Shuffler Validation & Accessor Extraction
// ============================================================================

struct ShufflerValidator<'s> {
    scoping: &'s Scoping,
    push_shift_count: usize,
    parse_int_count: usize,
    accessor_symbol_id: Option<SymbolId>,
}

impl<'s> ShufflerValidator<'s> {
    fn new(scoping: &'s Scoping) -> Self {
        Self {
            scoping,
            push_shift_count: 0,
            parse_int_count: 0,
            accessor_symbol_id: None,
        }
    }
}

impl<'a> Visit<'a> for ShufflerValidator<'_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        // Check: J.push(J.shift())
        let is_push = match &call.callee {
            Expression::StaticMemberExpression(m) => m.property.name == "push",
            Expression::ComputedMemberExpression(m) => {
                matches!(&m.expression, Expression::StringLiteral(s) if s.value == "push")
            }
            _ => false,
        };

        if is_push
            && call.arguments.len() == 1
            && let Argument::CallExpression(inner) = &call.arguments[0]
        {
            let is_shift = match &inner.callee {
                Expression::StaticMemberExpression(m) => m.property.name == "shift",
                Expression::ComputedMemberExpression(m) => {
                    matches!(&m.expression, Expression::StringLiteral(s) if s.value == "shift")
                }
                _ => false,
            };
            if is_shift {
                self.push_shift_count += 1;
            }
        }

        // Check: parseInt(accessor(...))
        if let Expression::Identifier(id) = &call.callee
            && id.name == "parseInt"
            && !call.arguments.is_empty()
        {
            self.parse_int_count += 1;

            // Extract accessor symbol from first parseInt call
            if self.accessor_symbol_id.is_none()
                && let Argument::CallExpression(inner_call) = &call.arguments[0]
                && let Expression::Identifier(accessor_id) = &inner_call.callee
            {
                self.accessor_symbol_id = get_reference_symbol(self.scoping, accessor_id);
            }
        }

        walk::walk_call_expression(self, call);
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::testing::parse_program;
    use oxc::allocator::Allocator;
    use oxc::codegen::Codegen;
    use oxc::semantic::SemanticBuilder;
    use oxc_traverse::traverse_mut;

    fn detect(source: &str) -> (ShufflerMap, String) {
        let alloc = Allocator::default();
        let mut program = parse_program(&alloc, source);
        let scoping = SemanticBuilder::new()
            .build(&program)
            .semantic
            .into_scoping();
        let mut detector = ShufflerDetector::default();
        traverse_mut(&mut detector, &alloc, &mut program, scoping, ());
        let output = Codegen::new().build(&program).code;
        (detector.results, output)
    }

    #[test]
    fn test_valid_shuffler() {
        let code = r#"
            function m() { return ["a", "b"]; }
            function G(x) { return m()[x]; }
            !(function(p, g) {
                const J = p();
                while (!![]) {
                    try {
                        if (parseInt(G(1)) / 1 + parseInt(G(2)) / 2 === g) break;
                        else J.push(J.shift());
                    } catch(Q) { J.push(J.shift()); }
                }
            })(m, 0x44598);
        "#;
        let (results, output) = detect(code);
        assert_eq!(results.len(), 1);
        // IIFE should be removed
        assert!(!output.contains("while"));
    }

    #[test]
    fn test_no_unary_prefix() {
        let code = r#"
            function m() { return ["a"]; }
            function G(x) { return m()[x]; }
            (function(p) {
                var J = p();
                for (;;) try {
                    if (123 === parseInt(G(1)) / 1 + parseInt(G(2)) / 2) break;
                    J.push(J.shift());
                } catch(e) { J.push(J.shift()); }
            })(m);
        "#;
        let (results, output) = detect(code);
        assert_eq!(results.len(), 1);
        assert!(!output.contains("for (;;)"));
    }

    #[test]
    fn test_missing_push_shift() {
        let code = r#"
            function m() { return ["a"]; }
            function G(x) { return m()[x]; }
            !(function(p, g) {
                if (parseInt(G(1)) / 1 + parseInt(G(2)) / 2 === g) {}
            })(m, 0x44598);
        "#;
        let (results, _) = detect(code);
        assert!(results.is_empty());
    }

    #[test]
    fn test_missing_parseint() {
        let code = r#"
            function m() { return ["a"]; }
            !(function(p, g) {
                const J = p();
                J.push(J.shift());
                J.push(J.shift());
            })(m, 0x44598);
        "#;
        let (results, _) = detect(code);
        assert!(results.is_empty());
    }
}
