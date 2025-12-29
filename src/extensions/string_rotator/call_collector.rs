//! Phase 3: Collect all calls to accessor function by symbol_id.

use oxc::ast::ast::*;
use oxc::semantic::SymbolId;
use oxc_traverse::{Traverse, TraverseCtx};
use rustc_hash::FxHashSet;

use crate::utils::codegen::call_to_code;
use crate::utils::literal::is_literal;
use crate::utils::scoping::get_reference_symbol;

// ============================================================================
// CallCollector
// ============================================================================

/// Collects all unique call expressions to specified accessor symbol.
pub struct CallCollector {
    accessor_symbol_id: SymbolId,
    /// Unique call codes (deduplicated)
    pub calls: FxHashSet<String>,
}

impl CallCollector {
    pub fn new(accessor_symbol_id: SymbolId) -> Self {
        Self {
            accessor_symbol_id,
            calls: FxHashSet::default(),
        }
    }
}

impl<'a> Traverse<'a, ()> for CallCollector {
    fn exit_call_expression(
        &mut self,
        call: &mut CallExpression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) {
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

        // Only collect calls with literal arguments (can be evaluated)
        if !call
            .arguments
            .iter()
            .all(|a| a.as_expression().is_some_and(is_literal))
        {
            return;
        }

        self.calls.insert(call_to_code(call));
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
    fn test_collects_calls() {
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
            
            // Usage calls
            console.log(G(0));
            console.log(G(1));
            console.log(G(0));  // duplicate
            console.log(G(-5));
        "#;

        let alloc = Allocator::default();
        let mut program = parse_program(&alloc, code);

        // Phase 1: Detect shufflers (removes IIFE, so G(1), G(2) inside are gone)
        let scoping = SemanticBuilder::new()
            .build(&program)
            .semantic
            .into_scoping();
        let mut detector = ShufflerDetector::default();
        traverse_mut(&mut detector, &alloc, &mut program, scoping, ());

        let info = &detector.results[&0];

        // Phase 3: Collect calls (only from remaining code, not IIFE)
        let scoping = SemanticBuilder::new()
            .build(&program)
            .semantic
            .into_scoping();
        let mut collector = CallCollector::new(info.accessor_symbol_id);
        traverse_mut(&mut collector, &alloc, &mut program, scoping, ());

        // Should have 3 unique calls: G(0), G(1), G(-5) - IIFE calls removed in Phase 1
        assert_eq!(collector.calls.len(), 3);
        assert!(collector.calls.contains("G(0)"));
        assert!(collector.calls.contains("G(1)"));
        assert!(collector.calls.contains("G(-5)"));
    }

    #[test]
    fn test_skips_non_literal_args() {
        let code = r#"
            function m() { return ["a"]; }
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
            
            var x = 5;
            G(x);      // non-literal, skip
            G(1 + 2);  // non-literal, skip
            G(0);      // literal, collect
        "#;

        let alloc = Allocator::default();
        let mut program = parse_program(&alloc, code);

        // Phase 1: Detect shufflers (removes IIFE)
        let scoping = SemanticBuilder::new()
            .build(&program)
            .semantic
            .into_scoping();
        let mut detector = ShufflerDetector::default();
        traverse_mut(&mut detector, &alloc, &mut program, scoping, ());

        let info = &detector.results[&0];

        let scoping = SemanticBuilder::new()
            .build(&program)
            .semantic
            .into_scoping();
        let mut collector = CallCollector::new(info.accessor_symbol_id);
        traverse_mut(&mut collector, &alloc, &mut program, scoping, ());

        // Only G(0) - skips G(x) and G(1+2), IIFE calls removed in Phase 1
        assert_eq!(collector.calls.len(), 1);
        assert!(collector.calls.contains("G(0)"));
    }
}
