//! Phase 2: Collect array and accessor function code by symbol_id.

use oxc::ast::ast::*;
use oxc::semantic::SymbolId;
use oxc_traverse::{Traverse, TraverseCtx};
use rustc_hash::FxHashMap;

use crate::utils::codegen::stmt_to_code;
use crate::utils::scoping::get_binding_symbol;

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Clone)]
pub struct FunctionCode {
    pub code: String,
}

/// Maps SymbolId -> function code
pub type FunctionCodeMap = FxHashMap<SymbolId, FunctionCode>;

// ============================================================================
// ComponentCollector
// ============================================================================

/// Collects function code for specified symbol IDs.
pub struct ComponentCollector {
    /// Symbol IDs to look for
    target_symbols: Vec<SymbolId>,
    /// Collected function code
    pub results: FunctionCodeMap,
}

impl ComponentCollector {
    pub fn new(target_symbols: Vec<SymbolId>) -> Self {
        Self {
            target_symbols,
            results: FxHashMap::default(),
        }
    }
}

impl<'a> Traverse<'a, ()> for ComponentCollector {
    fn exit_statement(&mut self, stmt: &mut Statement<'a>, _ctx: &mut TraverseCtx<'a, ()>) {
        // Match FunctionDeclaration
        let Statement::FunctionDeclaration(func) = stmt else {
            return;
        };

        // Get binding symbol
        let Some(id) = &func.id else { return };
        let Some(symbol_id) = get_binding_symbol(id) else {
            return;
        };

        // Check if this is a target symbol
        if !self.target_symbols.contains(&symbol_id) {
            return;
        }

        // Already collected
        if self.results.contains_key(&symbol_id) {
            return;
        }

        self.results.insert(
            symbol_id,
            FunctionCode {
                code: stmt_to_code(stmt),
            },
        );
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extensions::string_rotator::ShufflerDetector;
    use crate::utils::testing::parse_program;
    use oxc::allocator::Allocator;
    use oxc::semantic::SemanticBuilder;
    use oxc_traverse::traverse_mut;

    #[test]
    fn test_collects_functions() {
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

        let alloc = Allocator::default();
        let mut program = parse_program(&alloc, code);

        // Phase 1: Detect shufflers
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
        let mut collector = ComponentCollector::new(targets);
        let scoping = SemanticBuilder::new()
            .build(&program)
            .semantic
            .into_scoping();
        traverse_mut(&mut collector, &alloc, &mut program, scoping, ());

        assert_eq!(collector.results.len(), 2);
        assert!(collector.results.contains_key(&info.array_symbol_id));
        assert!(collector.results.contains_key(&info.accessor_symbol_id));

        // Check array function code
        let array_code = &collector.results[&info.array_symbol_id].code;
        assert!(array_code.contains("function m()"));
        assert!(array_code.contains(r#"["a", "b"]"#));

        // Check accessor function code
        let accessor_code = &collector.results[&info.accessor_symbol_id].code;
        assert!(accessor_code.contains("function G("));
        assert!(accessor_code.contains("m()[x]"));
    }
}
