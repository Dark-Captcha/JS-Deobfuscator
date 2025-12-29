//! Scoping query utilities.
//!
//! Helpers for symbol resolution and reference tracking.

use oxc::ast::ast::{BindingIdentifier, IdentifierReference};
use oxc::semantic::{Scoping, SymbolId};

// ============================================================================
// Symbol Resolution
// ============================================================================

/// Get symbol ID from an identifier reference.
///
/// Returns `None` for:
/// - Global variables (not declared in scope)
/// - Unresolved references
///
/// # Example
///
/// ```ignore
/// // For: const x = 1; console.log(x);
/// // The 'x' in console.log(x) resolves to the symbol for 'const x'
/// let symbol_id = get_reference_symbol(scoping, &x_ident);
/// ```
pub fn get_reference_symbol(scoping: &Scoping, ident: &IdentifierReference) -> Option<SymbolId> {
    let ref_id = ident.reference_id.get()?;
    scoping.get_reference(ref_id).symbol_id()
}

/// Get symbol ID from a binding identifier.
///
/// # Example
///
/// ```ignore
/// // For: const x = 1;
/// // The 'x' in the declaration has a symbol
/// let symbol_id = get_binding_symbol(&x_binding);
/// ```
pub fn get_binding_symbol(binding: &BindingIdentifier) -> Option<SymbolId> {
    binding.symbol_id.get()
}

// ============================================================================
// Reference Queries
// ============================================================================

/// Check if symbol has any write references.
///
/// A write reference is an assignment to the variable.
///
/// # Example
///
/// ```ignore
/// // const x = 1; x = 2;  // has_writes = true
/// // const x = 1; use(x); // has_writes = false
/// ```
pub fn has_writes(scoping: &Scoping, symbol_id: SymbolId) -> bool {
    scoping
        .get_resolved_references(symbol_id)
        .any(|r| r.flags().is_write())
}

/// Check if symbol has any read references.
///
/// # Example
///
/// ```ignore
/// // const x = 1; use(x); // has_reads = true
/// // const x = 1;         // has_reads = false
/// ```
pub fn has_reads(scoping: &Scoping, symbol_id: SymbolId) -> bool {
    scoping
        .get_resolved_references(symbol_id)
        .any(|r| r.flags().is_read())
}

/// Count read references to a symbol.
pub fn count_reads(scoping: &Scoping, symbol_id: SymbolId) -> usize {
    scoping
        .get_resolved_references(symbol_id)
        .filter(|r| r.flags().is_read())
        .count()
}

/// Count write references to a symbol.
pub fn count_writes(scoping: &Scoping, symbol_id: SymbolId) -> usize {
    scoping
        .get_resolved_references(symbol_id)
        .filter(|r| r.flags().is_write())
        .count()
}

/// Count all references to a symbol.
pub fn count_references(scoping: &Scoping, symbol_id: SymbolId) -> usize {
    scoping.get_resolved_references(symbol_id).count()
}

// ============================================================================
// Symbol Queries
// ============================================================================

/// Check if symbol is a const variable.
pub fn is_const(scoping: &Scoping, symbol_id: SymbolId) -> bool {
    scoping.symbol_flags(symbol_id).is_const_variable()
}

/// Check if symbol is safe to inline (const or no writes).
///
/// A symbol is safe to inline if:
/// - It's declared with `const`, OR
/// - It has no write references after declaration
pub fn is_safe_to_inline(scoping: &Scoping, symbol_id: SymbolId) -> bool {
    is_const(scoping, symbol_id) || !has_writes(scoping, symbol_id)
}
