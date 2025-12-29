//! Phase 5: Inline decoded strings and remove rotator components.

use oxc::ast::ast::*;
use oxc::semantic::SymbolId;
use oxc::span::SPAN;
use oxc_traverse::{Traverse, TraverseCtx};
use rustc_hash::FxHashSet;

use crate::extensions::string_rotator::DecodedMap;
use crate::utils::codegen::call_to_code;
use crate::utils::literal::is_literal;
use crate::utils::scoping::get_reference_symbol;

// ============================================================================
// StringInliner
// ============================================================================

/// Replaces accessor calls with decoded string literals.
pub struct StringInliner {
    accessor_symbol_id: SymbolId,
    decoded: DecodedMap,
    pub replaced_count: usize,
}

impl StringInliner {
    pub fn new(accessor_symbol_id: SymbolId, decoded: DecodedMap) -> Self {
        Self {
            accessor_symbol_id,
            decoded,
            replaced_count: 0,
        }
    }
}

impl<'a> Traverse<'a, ()> for StringInliner {
    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a, ()>) {
        let Expression::CallExpression(call) = expr else {
            return;
        };

        // Check if callee is identifier matching accessor symbol
        let Expression::Identifier(id) = &call.callee else {
            return;
        };
        let Some(symbol_id) = get_reference_symbol(ctx.scoping(), id) else {
            return;
        };

        if symbol_id != self.accessor_symbol_id {
            return;
        }

        // Only replace calls with literal arguments
        if !call
            .arguments
            .iter()
            .all(|a| a.as_expression().is_some_and(is_literal))
        {
            return;
        }

        // Look up decoded value
        let call_code = call_to_code(call);
        let Some(decoded_str) = self.decoded.get(&call_code) else {
            return;
        };

        // Replace with string literal
        let ast = ctx.ast;
        *expr = ast.expression_string_literal(SPAN, ast.atom(decoded_str), None);
        self.replaced_count += 1;
    }
}

// ============================================================================
// ComponentRemover
// ============================================================================

/// Removes array and accessor functions by symbol ID.
pub struct ComponentRemover {
    symbols_to_remove: FxHashSet<SymbolId>,
    pub removed_count: usize,
}

impl ComponentRemover {
    pub fn new(symbols: impl IntoIterator<Item = SymbolId>) -> Self {
        Self {
            symbols_to_remove: symbols.into_iter().collect(),
            removed_count: 0,
        }
    }
}

impl<'a> Traverse<'a, ()> for ComponentRemover {
    fn exit_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a, ()>) {
        let Statement::FunctionDeclaration(func) = stmt else {
            return;
        };

        if let Some(id) = &func.id
            && let Some(symbol_id) = id.symbol_id.get()
            && self.symbols_to_remove.contains(&symbol_id)
        {
            *stmt = ctx.ast.statement_empty(SPAN);
            self.removed_count += 1;
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extensions::string_rotator::{CallCollector, ComponentCollector, ShufflerDetector};
    use crate::utils::testing::parse_program;
    use oxc::allocator::Allocator;
    use oxc::codegen::Codegen;
    use oxc::semantic::SemanticBuilder;
    use oxc_traverse::traverse_mut;

    fn codegen(program: &oxc::ast::ast::Program) -> String {
        Codegen::new().build(program).code
    }

    #[test]
    fn test_inline_strings() {
        let code = r#"
            function arr() { return ["hello", "world"]; }
            function get(i) { return arr()[i]; }
            console.log(get(0));
            console.log(get(1));
        "#;

        let alloc = Allocator::default();
        let mut program = parse_program(&alloc, code);

        // Create decoded map manually (simulating Phase 4)
        let mut decoded = DecodedMap::default();
        decoded.insert("get(0)".to_string(), "hello".to_string());
        decoded.insert("get(1)".to_string(), "world".to_string());

        // Get accessor symbol
        let scoping = SemanticBuilder::new()
            .build(&program)
            .semantic
            .into_scoping();
        let mut detector = ShufflerDetector::default();
        traverse_mut(&mut detector, &alloc, &mut program, scoping, ());

        // For this test, manually find the 'get' function symbol
        // Since there's no shuffler, we need to find it differently
        let scoping = SemanticBuilder::new()
            .build(&program)
            .semantic
            .into_scoping();

        // Find 'get' symbol by name
        let get_symbol = scoping
            .symbol_ids()
            .find(|&id| scoping.symbol_name(id) == "get")
            .unwrap();

        // Phase 5: Inline
        let mut inliner = StringInliner::new(get_symbol, decoded);
        let scoping = SemanticBuilder::new()
            .build(&program)
            .semantic
            .into_scoping();
        traverse_mut(&mut inliner, &alloc, &mut program, scoping, ());

        assert_eq!(inliner.replaced_count, 2);

        let output = codegen(&program);
        assert!(output.contains("\"hello\""));
        assert!(output.contains("\"world\""));
        assert!(!output.contains("get(0)"));
        assert!(!output.contains("get(1)"));
    }

    #[test]
    fn test_remove_components() {
        let code = r#"
            function arr() { return ["a"]; }
            function get(i) { return arr()[i]; }
            console.log("keep");
        "#;

        let alloc = Allocator::default();
        let mut program = parse_program(&alloc, code);

        // Find symbols
        let scoping = SemanticBuilder::new()
            .build(&program)
            .semantic
            .into_scoping();
        let arr_symbol = scoping
            .symbol_ids()
            .find(|&id| scoping.symbol_name(id) == "arr")
            .unwrap();
        let get_symbol = scoping
            .symbol_ids()
            .find(|&id| scoping.symbol_name(id) == "get")
            .unwrap();

        // Remove
        let mut remover = ComponentRemover::new([arr_symbol, get_symbol]);
        let scoping = SemanticBuilder::new()
            .build(&program)
            .semantic
            .into_scoping();
        traverse_mut(&mut remover, &alloc, &mut program, scoping, ());

        assert_eq!(remover.removed_count, 2);

        let output = codegen(&program);
        assert!(!output.contains("function arr"));
        assert!(!output.contains("function get"));
        assert!(output.contains("\"keep\""));
    }

    #[test]
    fn test_full_pipeline_no_v8() {
        // Test the full pipeline without V8 execution (mock decoded values)
        let code = r#"
            function m() { return ["hello", "world", "test"]; }
            function G(x) { return m()[x]; }
            (function(p, g) {
                const J = p();
                while (!![]) {
                    try {
                        if (parseInt(G(1)) / 1 + parseInt(G(2)) / 2 === g) break;
                        else J.push(J.shift());
                    } catch(Q) { J.push(J.shift()); }
                }
            })(m, 123);
            console.log(G(0));
            console.log(G(1));
        "#;

        let alloc = Allocator::default();
        let mut program = parse_program(&alloc, code);

        // Phase 1: Detect
        let scoping = SemanticBuilder::new()
            .build(&program)
            .semantic
            .into_scoping();
        let mut detector = ShufflerDetector::default();
        traverse_mut(&mut detector, &alloc, &mut program, scoping, ());

        assert_eq!(detector.results.len(), 1);
        let info = &detector.results[&0];

        // Phase 2: Collect components
        let targets = vec![info.array_symbol_id, info.accessor_symbol_id];
        let scoping = SemanticBuilder::new()
            .build(&program)
            .semantic
            .into_scoping();
        let mut collector = ComponentCollector::new(targets.clone());
        traverse_mut(&mut collector, &alloc, &mut program, scoping, ());

        // Phase 3: Collect calls
        let scoping = SemanticBuilder::new()
            .build(&program)
            .semantic
            .into_scoping();
        let mut call_collector = CallCollector::new(info.accessor_symbol_id);
        traverse_mut(&mut call_collector, &alloc, &mut program, scoping, ());

        // Phase 4: Mock decoded values (skip V8 to avoid infinite loop)
        let mut decoded = DecodedMap::default();
        decoded.insert("G(0)".to_string(), "hello".to_string());
        decoded.insert("G(1)".to_string(), "world".to_string());
        decoded.insert("G(2)".to_string(), "test".to_string());

        // Phase 5a: Inline strings
        let scoping = SemanticBuilder::new()
            .build(&program)
            .semantic
            .into_scoping();
        let mut inliner = StringInliner::new(info.accessor_symbol_id, decoded);
        traverse_mut(&mut inliner, &alloc, &mut program, scoping, ());

        assert!(inliner.replaced_count >= 2);

        // Phase 5b: Remove functions (IIFE already removed in Phase 1)
        let scoping = SemanticBuilder::new()
            .build(&program)
            .semantic
            .into_scoping();
        let mut remover = ComponentRemover::new(targets);
        traverse_mut(&mut remover, &alloc, &mut program, scoping, ());

        let output = codegen(&program);

        // Verify strings are inlined
        assert!(output.contains("\"hello\""));
        assert!(output.contains("\"world\""));
        // Verify functions are removed
        assert!(!output.contains("function m()"));
        assert!(!output.contains("function G("));
        // Verify IIFE is removed (by Phase 1)
        assert!(!output.contains("while"));
    }
}
