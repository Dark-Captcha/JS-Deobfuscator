# OXC Deobfuscation Handbook

**Real-World Patterns for High-Accuracy JavaScript Deobfuscation**

**Version 1.1.0** | **Updated: 2025-12-27** | **Author Experience: 1 Year Production Implementation**

---

## Table of Contents

| #   | Section Title                          | Tier       |
| --- | -------------------------------------- | ---------- |
| 1   | Introduction & Core Principles         | Foundation |
| 2   | Architecture & Pipeline                | Foundation |
| 3   | Memory & Safety Fundamentals           | Foundation |
| 4   | Symbol Resolution & Reference Tracking | Core       |
| 5   | Traversal Strategies                   | Core       |
| 6   | Expression & Statement Manipulation    | Core       |
| 7   | Multi-Pass Transformation Framework    | Pattern    |
| 8   | Module System & Execution Order        | Pattern    |
| 9   | Dynamic Evaluation Integration         | Advanced   |
| 10  | Control Flow & Dead Code Analysis      | Advanced   |
| 11  | Core Deobfuscation Recipes             | Recipe     |
| 12  | Advanced Techniques                    | Recipe     |
| 13  | Anti-Patterns & Pitfalls               | Reference  |
| 14  | Value Representation Patterns          | Reference  |
| 15  | Quick Reference                        | Reference  |
| A   | Appendix A: File Structure             | Appendix   |

---

## 1. Introduction & Core Principles

### 1.1 Purpose

This handbook distills **one year of production-grade experience** building a high-accuracy JavaScript deobfuscator using the OXC (Oxford Compiler Collection) ecosystem in Rust.

It is **not** a general OXC reference. It is a **practical pattern library** focused on:

- Correct, safe AST manipulation
- Multi-pass convergence until fixed point
- Real-world deobfuscation techniques (string arrays, proxy functions, control flow flattening, encoded eval, etc.)
- Avoiding common pitfalls that break semantics or cause infinite loops

The patterns herein have been battle-tested on heavily obfuscated production code and refined for maximum effectiveness.

### 1.2 Core Philosophy

Deobfuscation is an **iterative simplification process**:

1. Normalize structure (formatters)
2. Decode encoded data early (string arrays, number encoding)
3. Propagate constants and inline aliases
4. Simplify control flow and expressions
5. Remove dead code
6. Repeat until no modifications (convergence)

**Key Insight**: Most obfuscation relies on indirection. Effective deobfuscation systematically eliminates indirection while preserving side effects.

### 1.3 Explicit Assumptions

These assumptions underpin all patterns:

1. **Arena Allocation**  
   All AST nodes live in an `Allocator` arena. No individual `Drop`. Memory freed only on `reset()` or drop.

2. **Semantic IDs Preserved**  
   `clone_in` and `take_in` preserve `SymbolId`/`ReferenceId`. Standard `.clone()` is forbidden.

3. **Single Unified Scoping**  
   Symbol table + reference table + scope tree in one structure. Share when possible, rebuild after structural changes.

4. **Traversal Safety**  
   Use `oxc_traverse::Traverse` (not VisitMut) for semantic-aware passes. "Without" types prevent aliasing violations.

5. **Convergence Required**  
   Transformations run in a loop until zero modifications. Max passes limit prevents infinite loops.

6. **Side-Effect Awareness**  
   Never duplicate effectful expressions without temp vars.

### 1.4 Design Goals

- **Correctness**: Preserve exact semantics
- **Safety**: No memory unsafety, no invalid AST states
- **Completeness**: Handle common obfuscation patterns (VM-based, string encoding, control flow flattening)
- **Efficiency**: Single-pass where possible, convergence loop for interdependent transforms
- **Maintainability**: Modular passes with clear responsibilities

### 1.5 Pipeline Overview

```mermaid
graph TD
    A[Source Code] --> B[Parser]
    B --> C[AST + Scoping]
    C --> D[Pre-Formatters (once)]
    D --> E[Deobfuscation Loop<br/>until convergence]
    E --> F[Post-Formatters (once)]
    F --> G[Codegen → Clean JS]
```

---

## 2. Architecture & Pipeline

The deobfuscation system is built around a **multi-pass, convergence-driven pipeline** using OXC's traversal and semantic infrastructure. The architecture ensures maximum simplification while preserving exact semantics.

### 2.1 Full Pipeline Diagram

```mermaid
graph TD
    A[Source Code] --> B[Parser + SemanticBuilder]
    B --> C[AST + Scoping]
    C --> D[Pre-Formatters<br/>(run once)]
    D --> E[Deobfuscation Loop<br/>(until 0 modifications)]
    E --> F[Post-Formatters<br/>(run once)]
    F --> G[Codegen → Clean JavaScript]
    style E fill:#f9f,stroke:#333,stroke-width:2px
```

- **Pre/Post-Formatters**: Structural normalization (brace wrapping, statement splitting, identifier renaming).
- **Deobfuscation Loop**: Core simplification passes — repeated until fixed point.
- **Convergence**: Essential for interdependent transforms (e.g., string decoding enables constant propagation).

### 2.2 Pass Execution Model

All passes implement a common interface:

```rust
pub trait Module: Send + Sync {
    fn name(&self) -> &'static str;
    fn changes_symbols(&self) -> bool;  // Requires scoping rebuild?

    fn transform<'a>(
        &mut self,
        allocator: &'a Allocator,
        program: &mut Program<'a>,
        scoping: Scoping,
    ) -> Result<TransformResult>;
}

pub struct TransformResult {
    pub modifications: usize,
    pub scoping: Scoping,
}
```

### 2.3 Scoping Sharing vs Rebuild Rules

| Situation                              | Action              | Rationale                   |
| -------------------------------------- | ------------------- | --------------------------- |
| Read-only passes (collect info)        | **Share** scoping   | Same SymbolIds valid        |
| No structural changes                  | **Share** scoping   | Reference counts unchanged  |
| Add/remove declarations                | **Rebuild** scoping | SymbolIds may invalidate    |
| Rename identifiers                     | **Rebuild** scoping | Binding lookup changes      |
| Before cleanup pass (unused detection) | **Rebuild** scoping | Need fresh reference counts |

**Pattern**: Share between collect/apply, rebuild before cleanup.

### 2.4 Convergence Loop

```rust
loop {
    let mut total_mods = 0;

    for module in deobfuscators {
        let result = module.transform(allocator, program, scoping)?;
        total_mods += result.modifications;
        scoping = result.scoping;

        if module.changes_symbols() {
            scoping = SemanticBuilder::build(program);  // Rebuild
        }
    }

    if total_mods == 0 { break; }  // Converged
    passes += 1;
    if passes >= max_passes { break; }  // Safety
}
```

- **Max passes**: 20–50 typical for heavy obfuscation.

### 2.5 Module Registry Pattern

```rust
pub struct ModuleRegistry {
    modules: Vec<Box<dyn Module>>,
}

impl ModuleRegistry {
    pub fn formatters() -> Self { /* pre/post */ }
    pub fn deobfuscators() -> Self { /* convergence loop */ }
}
```

### 2.6 Execution Order Rationale

#### Pre-Formatters (once)

1. IdentifierRenamer — normalize names early
2. BraceWrapper — consistent blocks
3. ForVarHoister — hoist loop vars
4. StatementSplitter — split compounds
5. MemberSimplifier — simplify access

#### Deobfuscation Loop (repeated)

1. StringArrayDecoder — earliest (unlocks strings)
2. ConstantPropagator — inline literals
3. AliasInliner — remove aliases
4. MemberSimplifier — repeated simplification
5. ObjectFlattener — inline props
6. ProxyInliner — inline wrappers
7. IifeOptimizer — unwrap IIFEs
8. EvalCallInliner — handle encoded eval
9. SequenceSimplifier — flatten sequences
10. DeadCodeEliminator — remove unused
11. StaticEvaluator — evaluate safe expressions
12. ControlFlowDeflattener — linearize flow (latest)

#### Post-Formatters (once)

Same as pre + StringArrayPostCleaner

**Order Insight**: Data decoding first → propagation → simplification → cleanup.

---

## 3. Memory & Safety Fundamentals

Memory management in OXC is **arena-based** and forms the foundation of all safe transformations. Understanding these fundamentals is non-negotiable—violations lead to lifetime errors, invalid AST states, or memory corruption.

### 3.1 Arena Allocation Model

All AST nodes and related data are allocated in a **`oxc_allocator::Allocator`** (wrapper around `bumpalo::Bump`).

Key characteristics:

- **Bulk allocation**: Fast contiguous chunks
- **No individual Drop**: Memory freed only on `allocator.reset()` or drop
- **Exponential chunk growth**: Stabilizes after warmup
- **No `Drop` types allowed** in arena (compile-time enforced)

**Consequence**: Custom data stored with AST must avoid `Drop` impls.

### 3.2 Allocator Lifecycle

```rust
let mut allocator = Allocator::default();  // Lazy
// or Allocator::with_capacity(1 << 20)

for file in files {
    // Parse and transform
    let ret = Parser::new(&allocator, source, source_type).parse();
    transform(&allocator, &mut ret.program);

    allocator.reset();  // CRITICAL: reuse memory
}
```

- **MUST recycle** with `reset()` — new allocator per file is expensive.
- No reset → unbounded growth.

Query:

- `allocator.used_bytes()`
- `allocator.capacity()`

### 3.3 Core Arena Types

| Type                | Purpose                   | Creation Example                    |
| ------------------- | ------------------------- | ----------------------------------- |
| `Box<'a, T>`        | Single value              | `Box::new_in(value, &allocator)`    |
| `Vec<'a, T>`        | Growable array            | `Vec::new_in(&allocator)`           |
| `HashMap<'a, K, V>` | Fast hash map             | `HashMap::new_in(&allocator)`       |
| `StringBuilder<'a>` | Efficient string building | `StringBuilder::new_in(&allocator)` |

Thread safety: `Sync` if contents are, **not Send** (shared bump).

### 3.4 CloneIn vs TakeIn — Deep Dive

Both **preserve** `SymbolId`/`ReferenceId` — critical for semantics.

| Operation  | Effect             | Use Case                      | Leaves Behind   |
| ---------- | ------------------ | ----------------------------- | --------------- |
| `clone_in` | Deep copy in arena | Storing for multiple uses     | Original intact |
| `take_in`  | Move ownership     | Efficient replacement/rebuild | Dummy value     |

Dummy values:

- `Expression` → `NullLiteral`
- `Statement` → `EmptyStatement`
- `Vec<T>` → empty

**MUST** replace/remove dummy after `take_in`.

#### 3.4.1 Patterns

```rust
// Clone for storage
self.values.insert(symbol_id, init.clone_in(ctx.ast.allocator));

// Take for movement
*expr = cond.consequent.take_in(ctx.ast.allocator);  // Dummy discarded on replace

// Vector rebuild
let mut new_stmts = ctx.ast.vec();
for stmt in stmts.take_in(allocator) {
    if keep(&stmt) { new_stmts.push(stmt); }
}
*stmts = new_stmts;
```

### 3.5 FromIn / IntoIn

Convenience conversions into arena types.

### 3.6 Address Stability

- `Box<T>`: always stable
- `Vec<T>` items: unstable after mutation

### 3.7 AllocatorPool

For multi-threaded:

```rust
let pool = AllocatorPool::new(threads);
let guard = pool.get();  // Auto-reset on drop
```

### 3.8 Critical Safety Rules

**MUST**:

- Recycle allocator.
- Use `clone_in`/`take_in` for AST.
- Handle dummies after `take_in`.

**MUST NOT**:

- Standard `.clone()` on AST nodes.
- `Drop` types in arena.
- Assume `Vec` addresses stable.

---

## 4. Symbol Resolution & Reference Tracking

Accurate symbol resolution and reference tracking are the **cornerstone of safe deobfuscation**. They enable constant propagation, dead code elimination, and alias inlining without breaking semantics.

### 4.1 Key Concepts

- **SymbolId**: Unique ID for a declaration (variable, function, class, import)
- **ReferenceId**: Unique ID for each usage of a name
- **Scoping**: Unified structure containing symbols, references, and scope tree

```
Declaration → SymbolId
Usage       → ReferenceId → (resolved) SymbolId
```

### 4.2 Safe Resolution Patterns (CRITICAL)

**Danger**: Direct `.reference_id()` / `.symbol_id()` can **panic** if not set.

**Safe Pattern**:

```rust
// IdentifierReference → SymbolId
pub fn get_reference_symbol(scoping: &Scoping, reference: &IdentifierReference) -> Option<SymbolId> {
    let ref_id = reference.reference_id.get()?;  // Safe Option
    scoping.get_reference(ref_id).symbol_id()
}

// BindingIdentifier → SymbolId
pub fn get_binding_symbol(binding: &BindingIdentifier) -> Option<SymbolId> {
    binding.symbol_id.get()
}
```

Complete patterns:

- VariableDeclarator → `decl.id.get_binding_identifier().and_then(|b| b.symbol_id.get())`
- Function → `func.id.as_ref().and_then(|id| id.symbol_id.get())`
- CallExpression callee → identifier case only

### 4.3 Reference Counting

```rust
pub fn count_read_references(scoping: &Scoping, symbol_id: SymbolId) -> usize {
    scoping
        .get_resolved_references(symbol_id)
        .filter(|r| r.flags().is_read())
        .count()
}

pub fn count_write_references(scoping: &Scoping, symbol_id: SymbolId) -> usize {
    scoping
        .get_resolved_references(symbol_id)
        .filter(|r| r.flags().is_write())
        .count()
}
```

Derived checks:

- `is_constant()`: no writes
- `is_unused()`: zero reads

### 4.4 Resolution Safety Decision Tree

```
Want to inline identifier?
│
├── Get SymbolId safely (.get()?)
│   └── None → global/unknown → DO NOT inline
│
├── Check writes
│   └── any(is_write) → NOT safe (reassigned)
│
├── (Optional) Check reads after inlining
└── zero reads → safe to remove declaration
```

### 4.5 Global vs Bound References

- `root_unresolved_references()` → HashMap<name, Vec<ReferenceId>>
- Globals (console, Math) appear here.
- **DO NOT** assume constants for globals without context.

### 4.6 Practical Usage in Passes

```rust
// In collector
let has_writes = ctx.scoping()
    .get_resolved_references(symbol_id)
    .any(|r| r.flags().is_write());

if !has_writes {
    // Safe to store for inlining
}
```

### 4.7 Critical Rules

**MUST**:

- Use `.get()` for safe resolution.
- Check writes before inlining.
- Rebuild scoping after declaration changes.

**MUST NOT**:

- Use direct accessors (panic risk).
- Inline globals without verification.

---

## 5. Traversal Strategies

Traversal is the **primary mechanism** for inspecting and modifying the AST. Choosing the correct strategy determines whether a pass can safely access symbols, ancestors, or perform semantic-aware transformations.

### 5.1 The Two Traversal Systems

| System                    | Crate           | Trait      | Context Provided   | Ancestor Access | Symbol/Scoping Access | Typical Use Case          |
| ------------------------- | --------------- | ---------- | ------------------ | --------------- | --------------------- | ------------------------- |
| Root-level (full program) | `oxc_traverse`  | `Traverse` | Full `TraverseCtx` | Yes (§7)        | Yes                   | **Deobfuscation passes**  |
| Path-level (subtree)      | `oxc_ast_visit` | `VisitMut` | None               | No              | No                    | Simple subtree transforms |

**For deobfuscation**: **Always use `oxc_traverse::Traverse`**.

### 5.2 Traverse Implementation Pattern

```rust
struct MyPass {
    modifications: usize,
    // state...
}

impl<'a> Traverse<'a, ()> for MyPass {
    // Top-down: collect info, early bailout
    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a, ()>) { }

    // Bottom-up: perform replacements (children processed)
    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a, ()>) {
        // Safe to replace here
        self.modifications += 1;
    }

    fn enter_statements(&mut self, stmts: &mut ArenaVec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a, ()>) { }
    fn exit_statements(&mut self, stmts: &mut ArenaVec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a, ()>) { }
}
```

**Rule**: Collect in `enter_*`, transform in `exit_*`.

### 5.3 Key Hooks

- `enter/exit_expression` — most common
- `enter/exit_statements` — statement list manipulation
- `enter/exit_function` — function-specific state
- `enter/exit_variable_declarator` — declaration analysis
- `enter/exit_call_expression` — call inlining
- Identifier hooks: `enter_identifier_reference`, `enter_binding_identifier`

### 5.4 TraverseCtx Access Patterns

```rust
// Allocator & AST builder
let allocator = ctx.ast.allocator;
let new_expr = ctx.ast.expression_string_literal(SPAN, ctx.ast.atom("value"), None);

// Scoping
let scoping = ctx.scoping();                // Immutable
let scoping_mut = ctx.scoping_mut();        // Mutable (rare)

// UID generation
let name = ctx.generate_uid_name(&atom);

// Ancestor access (§7)
match ctx.parent() {
    Ancestor::CallExpressionCallee(call) => { /* callee context */ }
    Ancestor::BinaryExpressionLeft(bin) => { bin.right() /* sibling */ }
}
```

### 5.5 Traverse vs VisitMut — Critical Comparison

**VisitMut** (`oxc_ast_visit`):

- No context, no ancestors, no scoping.
- Manual recursion via `walk_mut::*`.
- Suitable only for **syntactic** subtree transforms.

**Anti-Pattern**:

```rust
// WRONG for deobfuscation
impl<'a> VisitMut<'a> for ConstantInliner { ... }  // No symbol access!
```

**Correct**:

```rust
impl<'a> Traverse<'a, ()> for ConstantInliner { ... }  // Full semantic access
```

### 5.6 Manual Recursion Anti-Pattern

```rust
// WRONG — maintenance nightmare
fn substitute(expr: &mut Expression<'a>, subs: &HashMap<...>) {
    match expr {
        Expression::Identifier(id) => { ... }
        Expression::BinaryExpression(bin) => {
            substitute(&mut bin.left, subs);   // Manual!
            substitute(&mut bin.right, subs);
        }
        // Must handle ALL ~50 variants
    }
}

// CORRECT — use VisitMut for subtree
let mut visitor = ParamSubstituter { subs, alloc };
visitor.visit_expression(expr);  // Auto-recursion via walk_mut
```

### 5.7 Critical Rules

**MUST**:

- Use `Traverse` for any semantic-aware pass.
- Perform replacements in `exit_*`.
- Use `ctx.ast` for new nodes.

**MUST NOT**:

- Use `VisitMut` when symbol resolution needed.
- Manually recurse over AST.

---

## 6. Expression & Statement Manipulation

Direct manipulation of expressions and statements is the core of deobfuscation. These patterns ensure safe, semantic-preserving changes.

### 6.1 Expression Creation

Use `ctx.ast` (AstBuilder) for all new nodes.

```rust
// Literals
let str_lit = ctx.ast.expression_string_literal(SPAN, ctx.ast.atom("value"), None);
let num_lit = ctx.ast.expression_numeric_literal(SPAN, 42.0, None, NumberBase::Decimal);
let bool_lit = ctx.ast.expression_boolean_literal(SPAN, true);
let null_lit = ctx.ast.expression_null_literal(SPAN);

// Identifier reference
let ident = ctx.ast.expression_identifier_reference(SPAN, ctx.ast.atom("name"));

// Operations
let binary = ctx.ast.expression_binary(SPAN, left, BinaryOperator::Addition, right);
let unary = ctx.ast.expression_unary(SPAN, UnaryOperator::UnaryNegation, argument);

// Collections
let mut elements = ctx.ast.vec();
elements.push(ArrayExpressionElement::Expression(expr1));
let array = ctx.ast.expression_array(SPAN, elements);
```

### 6.2 Expression Replacement

**Pattern**: Replace in `exit_expression` (children processed).

```rust
fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a, ()>) {
    // Call inlining
    if let Expression::CallExpression(call) = expr {
        if let Some(decoded) = self.try_decode_call(call, ctx) {
            *expr = decoded;
            self.modifications += 1;
        }
    }

    // Constant inlining
    if let Expression::Identifier(ident) = expr {
        if let Some(symbol_id) = get_reference_symbol(ctx.scoping(), ident) {
            if let Some(value) = self.constants.get(&symbol_id) {
                *expr = value.clone_in(ctx.ast.allocator);
                self.modifications += 1;
            }
        }
    }
}
```

### 6.3 Value Extraction Patterns

```rust
// String (handles parens)
fn extract_string(expr: &Expression) -> Option<&str> {
    match expr {
        Expression::StringLiteral(lit) => Some(lit.value.as_str()),
        Expression::ParenthesizedExpression(p) => extract_string(&p.expression),
        _ => None,
    }
}

// Numeric (handles unary negation)
fn extract_numeric(expr: &Expression) -> Option<f64> {
    match expr {
        Expression::NumericLiteral(lit) => Some(lit.value),
        Expression::UnaryExpression(u) if u.operator == UnaryOperator::UnaryNegation => {
            extract_numeric(&u.argument).map(|v| -v)
        }
        Expression::ParenthesizedExpression(p) => extract_numeric(&p.expression),
        _ => None,
    }
}
```

### 6.4 LiteralValue Compact Storage

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
}

// Extract with sequence/paren/unary handling
pub fn extract_literal_value(expr: &Expression) -> Option<LiteralValue> { ... }

// Convert back
pub fn literal_value_to_expression(value: &LiteralValue, ast: AstBuilder<'a>) -> Expression<'a> { ... }
```

### 6.5 Statement Creation

```rust
// Expression statement
let expr_stmt = ctx.ast.statement_expression(SPAN, expression);

// Block
let block = ctx.ast.statement_block(SPAN, stmts);

// Return
let ret = ctx.ast.statement_return(SPAN, Some(expr));

// Variable declaration
let binding = ctx.ast.binding_identifier(SPAN, ctx.ast.atom("x"));
let declarator = ctx.ast.variable_declarator(SPAN, VarKind::Var, binding, Some(init), false);
let var_decl = ctx.ast.variable_declaration(SPAN, VarKind::Var, ctx.ast.vec1(declarator), false);
```

### 6.6 Statement List Manipulation

**Pattern**: Rebuild with `take_in` for full control.

```rust
fn exit_statements(&mut self, stmts: &mut ArenaVec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a, ()>) {
    let allocator = ctx.ast.allocator;
    let mut new_stmts = ctx.ast.vec();

    for stmt in stmts.take_in(allocator) {
        if should_remove(&stmt) {
            self.modifications += 1;
            continue;
        }
        new_stmts.push(stmt);
    }

    *stmts = new_stmts;
}
```

Insertion:

- `insert(0, new_stmt)` — beginning
- Rebuild loop for position-specific

### 6.7 Advanced Patterns

#### 6.7.1 Splitting Multi-Declarations

```rust
if var_decl.declarations.len() > 1 {
    let decls = var_decl.declarations.take_in(allocator);
    for decl in decls {
        let single = ctx.ast.vec1(decl);
        new_stmts.push(ctx.ast.statement_variable_declaration(var_decl.span, var_decl.kind, single));
    }
}
```

#### 6.7.2 Block Wrapping

```rust
if !matches!(stmt, Statement::BlockStatement(_)) {
    let inner = stmt.take_in(allocator);
    *stmt = ctx.ast.statement_block(inner.span(), ctx.ast.vec1(inner));
}
```

#### 6.7.3 Unreachable Code Trimming

```rust
let mut terminated = false;
for stmt in stmts.take_in(allocator) {
    if terminated { self.modifications += 1; continue; }
    if stmt.is_terminator() { terminated = true; }
    new_stmts.push(stmt);
}
```

### 6.8 Critical Rules

**MUST**:

- Use `ctx.ast` for creation.
- Rebuild lists with `take_in`.
- Replace in `exit_*`.

**MUST NOT**:

- Direct list mutation in `enter_*`.
- Forget dummy handling after `take_in`.

---

## 7. Multi-Pass Transformation Framework

Deobfuscation is inherently **iterative**. Many patterns depend on prior simplifications (e.g., decoding strings enables constant propagation). The multi-pass framework with convergence is the proven pattern for maximum simplification.

### 7.1 Two-Pass Pattern (Collect → Apply)

Most common for single transformations requiring global knowledge.

```rust
impl Module for ConstantPropagator {
    fn transform<'a>(
        &mut self,
        allocator: &'a Allocator,
        program: &mut Program<'a>,
        scoping: Scoping,
    ) -> Result<TransformResult> {
        // Pass 1: Collect safe constants
        let mut collector = Collector::default();
        let scoping = traverse_mut(&mut collector, allocator, program, scoping, ());

        if collector.constants.is_empty() {
            return Ok(TransformResult { modifications: 0, scoping });
        }

        // Pass 2: Apply inlining (shares scoping — same SymbolIds)
        let mut inliner = Inliner {
            constants: collector.constants,
            modifications: 0,
        };
        let scoping = traverse_mut(&mut inliner, allocator, program, scoping, ());

        Ok(TransformResult { modifications: inliner.modifications, scoping })
    }
}
```

Collector checks:

- No writes to variable
- Initializer is literal/inlineable

### 7.2 Three-Pass Pattern (Collect → Apply → Clean)

For transformations that remove declarations after inlining.

```rust
// After apply pass
let symbols_to_remove: FxHashSet<SymbolId> = collector.constants.keys().copied().collect();

// REBUILD scoping (fresh reference counts)
let scoping = SemanticBuilder::build(program);

// Cleanup pass — remove unused declarations
let mut cleaner = Cleaner { symbols: symbols_to_remove, modifications: 0 };
let scoping = traverse_mut(&mut cleaner, allocator, program, scoping, ());
```

### 7.3 Convergence Loop Pattern

Full deobfuscation pipeline:

```rust
pub fn deobfuscate(&mut self, source: &str) -> Result<String> {
    let mut program = parse(source);
    let mut scoping = SemanticBuilder::build(&program);

    // Pre-formatters (once)
    run_modules(&mut pre_formatters, &mut program, &mut scoping);

    // Convergence loop
    let mut passes = 0;
    loop {
        let mut mods = 0;
        for module in &mut deobfuscators {
            let result = module.transform(allocator, &mut program, scoping)?;
            mods += result.modifications;
            scoping = result.scoping;

            if module.changes_symbols() {
                scoping = SemanticBuilder::build(&program);  // Rebuild
            }
        }

        if mods == 0 { break; }  // Converged
        passes += 1;
        if passes >= 50 { break; }  // Safety
    }

    // Post-formatters (once)
    run_modules(&mut post_formatters, &mut program, &mut scoping);

    codegen(&program)
}
```

### 7.4 Scoping Sharing Rules

| Situation                         | Action  | Reason                     |
| --------------------------------- | ------- | -------------------------- |
| Collector & Applier only          | Share   | Same SymbolIds needed      |
| Read-only passes                  | Share   | No invalidation            |
| Add/remove declarations           | Rebuild | SymbolIds may change       |
| Rename identifiers                | Rebuild | Binding lookup affected    |
| Before cleanup (unused detection) | Rebuild | Need accurate fresh counts |

### 7.5 Modification Counting

**MUST** increment `modifications` on every change:

- Enables convergence detection
- Prevents infinite loops from no-op transforms

```rust
if !expr.content_eq(&new_expr) {  // Optional guard
    *expr = new_expr;
    self.modifications += 1;
}
```

### 7.6 Critical Rules

**MUST**:

- Count modifications accurately.
- Rebuild scoping when `changes_symbols() == true`.
- Limit passes to prevent infinite loops.

**MUST NOT**:

- Share scoping across structural changes.
- Forget modification counting.

---

## 8. Module System & Execution Order

The **module system** is the backbone of the deobfuscation pipeline. It provides a clean, extensible way to compose multiple transformation passes with explicit control over execution order and scoping rebuilds.

### 8.1 Module Trait

```rust
pub trait Module: Send + Sync {
    fn name(&self) -> &'static str;

    /// Returns true if the module adds/removes declarations or renames identifiers
    fn changes_symbols(&self) -> bool {
        false
    }

    fn transform<'a>(
        &mut self,
        allocator: &'a Allocator,
        program: &mut Program<'a>,
        scoping: Scoping,
    ) -> Result<TransformResult>;
}

pub struct TransformResult {
    pub modifications: usize,
    pub scoping: Scoping,
}
```

- `changes_symbols()` → caller knows when to rebuild scoping.

### 8.2 Module Registry

```rust
pub struct ModuleRegistry {
    modules: Vec<Box<dyn Module>>,
}

impl ModuleRegistry {
    pub fn new() -> Self { Self { modules: Vec::new() } }

    pub fn register<M: Module + 'static>(&mut self, module: M) {
        self.modules.push(Box::new(module));
    }

    pub fn formatters() -> Self { /* pre/post */ }
    pub fn deobfuscators() -> Self { /* convergence loop */ }
}
```

### 8.3 Execution Order — Battle-Tested Rationale

#### Pre-Formatters (run once)

| Order | Module            | Reason                                          |
| ----- | ----------------- | ----------------------------------------------- |
| 1     | IdentifierRenamer | Normalize names early for consistent resolution |
| 2     | BraceWrapper      | Ensure consistent block structure               |
| 3     | ForVarHoister     | Hoist loop vars for accurate scoping            |
| 4     | StatementSplitter | Split compound declarations                     |
| 5     | MemberSimplifier  | Early member access simplification              |

#### Deobfuscation Loop (repeated until convergence)

| Order | Module                 | Reason                                        |
| ----- | ---------------------- | --------------------------------------------- |
| 1     | StringArrayDecoder     | **Earliest** — unlocks encoded strings        |
| 2     | ConstantPropagator     | Inline literals made available by decoding    |
| 3     | AliasInliner           | Remove temporary aliases                      |
| 4     | MemberSimplifier       | Repeated access simplification                |
| 5     | ObjectFlattener        | Inline object properties                      |
| 6     | ProxyInliner           | Inline wrapper functions                      |
| 7     | IifeOptimizer          | Unwrap immediately-invoked expressions        |
| 8     | EvalCallInliner        | Handle encoded eval calls                     |
| 9     | SequenceSimplifier     | Flatten comma expressions                     |
| 10    | DeadCodeEliminator     | Remove unused vars/decls                      |
| 11    | StaticEvaluator        | Evaluate safe expressions                     |
| 12    | ControlFlowDeflattener | **Latest** — linearize flattened control flow |

#### Post-Formatters (run once)

Same as pre + `StringArrayPostCleaner` (remove leftover array infrastructure).

### 8.4 Order Rationale Summary

- **Data decoding first** (strings, arrays) → enables propagation.
- **Propagation** (constants, aliases) → enables simplification.
- **Simplification** (members, objects, proxies, IIFEs) → repeated.
- **Evaluation & cleanup** → mid-late.
- **Control flow last** — operates on fully simplified code.

Changing this order reduces effectiveness dramatically.

### 8.5 changes_symbols() Usage

- `true` for: StatementSplitter, DeadCodeEliminator, IdentifierRenamer
- `false` for: ConstantPropagator (read-only), StaticEvaluator

Triggers scoping rebuild after pass.

### 8.6 Critical Rules

**MUST**:

- Follow this exact order.
- Implement `changes_symbols()` correctly.
- Return accurate `modifications` count.

**MUST NOT**:

- Reorder passes arbitrarily.
- Forget to rebuild scoping when needed.

---

## 9. Dynamic Evaluation Integration

Many obfuscators encode logic in runtime-evaluated strings (`eval`, encoded calls). **Dynamic evaluation** with a safe JavaScript runtime is essential for decoding such patterns while preserving semantics.

### 9.1 JsEvaluator (V8-based)

```rust
pub struct JsEvaluator {
    runtime: JsRuntime,     // deno_core or similar V8 isolate
    timeout_ms: u64,        // Default 1000ms
}
```

Methods:

- `execute(code)` — side effects only (definitions, IIFEs)
- `eval(code)` — returns JSON value
- `eval_to_string()`, `eval_to_number()`

**Safety**:

- Timeout prevents infinite loops.
- Isolated context (no DOM, limited globals).

### 9.2 Safety Checking

**CRITICAL**: Only evaluate **deterministic, side-effect-free** expressions.

```rust
/// Safe global functions that are deterministic and side-effect-free
const SAFE_GLOBAL_FUNCTIONS: &[&str] = &[
    "parseInt", "parseFloat", "isNaN", "isFinite",
    "encodeURI", "decodeURI", "encodeURIComponent", "decodeURIComponent",
    "atob", "btoa", "escape", "unescape",
    "Number", "String", "Boolean", "BigInt",
];

/// Safe global objects (their methods need individual checking)
const SAFE_GLOBAL_OBJECTS: &[&str] = &["Math", "JSON", "Object", "Array", "String", "Number"];

/// Unsafe globals that should never be evaluated
const UNSAFE_GLOBALS: &[&str] = &[
    "eval", "Function", "console", "document", "window", "globalThis",
    "fetch", "XMLHttpRequest", "WebSocket", "Worker",
    "setTimeout", "setInterval", "setImmediate",
    "require", "import", "module", "exports",
    "process", "Buffer", "__dirname", "__filename",
];

/// Safe String methods (deterministic, no side effects)
const SAFE_STRING_METHODS: &[&str] = &[
    "charAt", "charCodeAt", "codePointAt", "at",
    "concat", "includes", "endsWith", "startsWith",
    "indexOf", "lastIndexOf", "localeCompare",
    "match", "matchAll", "normalize", "padEnd", "padStart",
    "repeat", "replace", "replaceAll", "search",
    "slice", "split", "substring", "substr",
    "toLowerCase", "toUpperCase", "toLocaleLowerCase", "toLocaleUpperCase",
    "trim", "trimStart", "trimEnd", "trimLeft", "trimRight",
    "toString", "valueOf", "length",
];

/// Safe Array methods (deterministic, no mutation)
const SAFE_ARRAY_METHODS: &[&str] = &[
    "at", "concat", "entries", "every", "filter", "find", "findIndex",
    "findLast", "findLastIndex", "flat", "flatMap", "forEach",
    "includes", "indexOf", "join", "keys", "lastIndexOf",
    "map", "reduce", "reduceRight", "slice", "some",
    "toReversed", "toSorted", "toSpliced", "toString", "values", "with",
    "length",
];

/// Safe Math methods (all deterministic except random)
const SAFE_MATH_METHODS: &[&str] = &[
    "abs", "acos", "acosh", "asin", "asinh", "atan", "atan2", "atanh",
    "cbrt", "ceil", "clz32", "cos", "cosh", "exp", "expm1",
    "floor", "fround", "hypot", "imul", "log", "log10", "log1p", "log2",
    "max", "min", "pow", "round", "sign", "sin", "sinh",
    "sqrt", "tan", "tanh", "trunc",
    // Constants
    "E", "LN10", "LN2", "LOG10E", "LOG2E", "PI", "SQRT1_2", "SQRT2",
];

/// UNSAFE Math methods (non-deterministic)
const UNSAFE_MATH_METHODS: &[&str] = &["random"];

/// Check if an expression is safe to evaluate (deterministic, side-effect-free)
pub fn is_safe_expr(expr: &Expression) -> bool {
    match expr {
        // Literals are always safe
        Expression::StringLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::BigIntLiteral(_) => true,

        // Template literals without expressions
        Expression::TemplateLiteral(t) => t.expressions.is_empty(),

        // Array/Object literals if all elements are safe
        Expression::ArrayExpression(arr) => arr.elements.iter().all(|el| match el {
            ArrayExpressionElement::SpreadElement(_) => false,
            ArrayExpressionElement::Elision(_) => true,
            _ => el.as_expression().map_or(false, is_safe_expr),
        }),

        Expression::ObjectExpression(obj) => obj.properties.iter().all(|prop| match prop {
            ObjectPropertyKind::ObjectProperty(p) => {
                !p.computed && p.value.as_expression().map_or(false, is_safe_expr)
            }
            ObjectPropertyKind::SpreadProperty(_) => false,
        }),

        // Identifiers: only safe globals
        Expression::Identifier(ident) => {
            let name = ident.name.as_str();
            SAFE_GLOBAL_FUNCTIONS.contains(&name)
                || SAFE_GLOBAL_OBJECTS.contains(&name)
                || name == "undefined"
                || name == "NaN"
                || name == "Infinity"
        }

        // Unary expressions (typeof, !, -, +, ~, void)
        Expression::UnaryExpression(u) => {
            matches!(
                u.operator,
                UnaryOperator::UnaryNegation
                    | UnaryOperator::UnaryPlus
                    | UnaryOperator::LogicalNot
                    | UnaryOperator::BitwiseNot
                    | UnaryOperator::Typeof
                    | UnaryOperator::Void
            ) && is_safe_expr(&u.argument)
        }

        // Binary expressions (arithmetic, comparison, bitwise)
        Expression::BinaryExpression(b) => {
            is_safe_expr(&b.left) && is_safe_expr(&b.right)
        }

        // Logical expressions (&&, ||, ??)
        Expression::LogicalExpression(l) => {
            is_safe_expr(&l.left) && is_safe_expr(&l.right)
        }

        // Conditional expressions (ternary)
        Expression::ConditionalExpression(c) => {
            is_safe_expr(&c.test) && is_safe_expr(&c.consequent) && is_safe_expr(&c.alternate)
        }

        // Sequence expressions
        Expression::SequenceExpression(s) => s.expressions.iter().all(is_safe_expr),

        // Parenthesized expressions
        Expression::ParenthesizedExpression(p) => is_safe_expr(&p.expression),

        // Call expressions: check callee and arguments
        Expression::CallExpression(call) => is_safe_call(call),

        // Member expressions on safe objects
        Expression::StaticMemberExpression(m) => is_safe_member_access(&m.object, &m.property.name),
        Expression::ComputedMemberExpression(m) => {
            is_safe_expr(&m.object) && is_safe_expr(&m.expression)
        }

        // Everything else is unsafe
        _ => false,
    }
}

/// Check if a call expression is safe to evaluate
fn is_safe_call(call: &CallExpression) -> bool {
    // All arguments must be safe
    if !call.arguments.iter().all(|arg| match arg {
        Argument::SpreadElement(_) => false,
        _ => arg.as_expression().map_or(false, is_safe_expr),
    }) {
        return false;
    }

    match &call.callee {
        // Direct global function call: parseInt("123")
        Expression::Identifier(ident) => {
            let name = ident.name.as_str();
            SAFE_GLOBAL_FUNCTIONS.contains(&name) && !UNSAFE_GLOBALS.contains(&name)
        }

        // Method call: "abc".split(""), Math.floor(1.5)
        Expression::StaticMemberExpression(m) => {
            let method = m.property.name.as_str();

            // Check for unsafe methods first
            if UNSAFE_MATH_METHODS.contains(&method) {
                return false;
            }

            match &m.object {
                // String literal method: "abc".toUpperCase()
                Expression::StringLiteral(_) => SAFE_STRING_METHODS.contains(&method),

                // Array literal method: [1,2,3].join(",")
                Expression::ArrayExpression(arr) => {
                    SAFE_ARRAY_METHODS.contains(&method)
                        && arr.elements.iter().all(|el| {
                            el.as_expression().map_or(false, is_safe_expr)
                        })
                }

                // Math.floor, JSON.parse, etc.
                Expression::Identifier(obj_ident) => {
                    let obj_name = obj_ident.name.as_str();
                    match obj_name {
                        "Math" => SAFE_MATH_METHODS.contains(&method),
                        "JSON" => matches!(method, "parse" | "stringify"),
                        "Object" => matches!(method, "keys" | "values" | "entries" | "fromEntries"),
                        "Array" => matches!(method, "isArray" | "from" | "of"),
                        "String" => matches!(method, "fromCharCode" | "fromCodePoint"),
                        "Number" => matches!(method, "isNaN" | "isFinite" | "isInteger" | "isSafeInteger" | "parseFloat" | "parseInt"),
                        _ => false,
                    }
                }

                // Chained calls: "abc".split("").reverse().join("")
                Expression::CallExpression(inner_call) => {
                    is_safe_call(inner_call) && SAFE_ARRAY_METHODS.contains(&method)
                }

                // Parenthesized: ("abc").toUpperCase()
                Expression::ParenthesizedExpression(p) => {
                    is_safe_expr(&p.expression) && is_safe_method_for_type(&p.expression, method)
                }

                _ => false,
            }
        }

        // Computed member call: obj["method"]() - generally unsafe
        Expression::ComputedMemberExpression(_) => false,

        // Parenthesized callee
        Expression::ParenthesizedExpression(p) => {
            if let Expression::CallExpression(inner) = &p.expression {
                is_safe_call(inner)
            } else {
                false
            }
        }

        _ => false,
    }
}

/// Check if member access is safe (reading property, not calling)
fn is_safe_member_access(object: &Expression, property: &str) -> bool {
    match object {
        Expression::StringLiteral(_) => property == "length" || SAFE_STRING_METHODS.contains(&property),
        Expression::ArrayExpression(_) => property == "length" || SAFE_ARRAY_METHODS.contains(&property),
        Expression::Identifier(ident) => {
            let obj_name = ident.name.as_str();
            match obj_name {
                "Math" => SAFE_MATH_METHODS.contains(&property),
                _ => false,
            }
        }
        Expression::ParenthesizedExpression(p) => is_safe_member_access(&p.expression, property),
        _ => false,
    }
}

/// Determine safe methods based on expression type
fn is_safe_method_for_type(expr: &Expression, method: &str) -> bool {
    match expr {
        Expression::StringLiteral(_) => SAFE_STRING_METHODS.contains(&method),
        Expression::ArrayExpression(_) => SAFE_ARRAY_METHODS.contains(&method),
        Expression::CallExpression(call) => {
            // Result of safe call - check if method is safe for likely return type
            SAFE_STRING_METHODS.contains(&method) || SAFE_ARRAY_METHODS.contains(&method)
        }
        Expression::ParenthesizedExpression(p) => is_safe_method_for_type(&p.expression, method),
        _ => false,
    }
}
```

**Unsafe patterns** (always return `false`):

- Variable arguments (unknown values at compile time)
- `Math.random()`, `Date.now()` (non-deterministic)
- `console`, `document`, `window`, `fetch` (side effects)
- `eval`, `Function` (code execution)
- Assignment expressions
- `new` expressions (object creation with potential side effects)

### 9.3 AST ↔ Code Conversion

```rust
pub fn expr_to_code(expr: &Expression) -> String {
    Codegen::new().print_expression(expr).into_source_text()
}

pub fn value_to_expr<'a>(value: serde_json::Value, allocator: &'a Allocator) -> Option<Expression<'a>> {
    let ast = AstBuilder::new(allocator);
    match value {
        serde_json::Value::String(s) => Some(ast.expression_string_literal(SPAN, ast.atom(&s), None)),
        serde_json::Value::Number(n) => n.as_f64().map(|f| ast.expression_numeric_literal(SPAN, f, None, NumberBase::Decimal)),
        serde_json::Value::Bool(b) => Some(ast.expression_boolean_literal(SPAN, b)),
        serde_json::Value::Null => Some(ast.expression_null_literal(SPAN)),
        serde_json::Value::Array(arr) => { /* build array */ }
        _ => None,
    }
}
```

### 9.4 Static Evaluation Pattern

```rust
struct StaticEvalPass<'e> {
    evaluator: &'e mut JsEvaluator,
    cache: FxHashMap<String, serde_json::Value>,
    modifications: usize,
}

impl<'a, 'e> Traverse<'a, ()> for StaticEvalPass<'e> {
    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a, ()>) {
        if expr.is_literal() { return; }
        if !is_safe_expr(expr) { return; }

        let code = expr_to_code(expr);
        let value = self.cache.get(&code).cloned().or_else(|| {
            self.evaluator.eval(&code).ok()
        }).and_then(|v| self.cache.insert(code.clone(), v.clone()).is_none().then_some(v));

        if let Some(v) = value {
            if let Some(result) = value_to_expr(v, ctx.ast.allocator) {
                if !expr.content_eq(&result) {  // Avoid no-op
                    *expr = result;
                    self.modifications += 1;
                }
            }
        }
    }
}
```

### 9.5 Caching

- Key: generated source code string.
- Prevents repeated evaluation of identical expressions.

### 9.6 Critical Rules

**MUST**:

- Check `is_safe_expr` strictly.
- Cache results.
- Timeout evaluation.
- Avoid no-op replacements.

**MUST NOT**:

- Evaluate expressions with variables.
- Evaluate non-deterministic calls.

---

## 10. Control Flow & Dead Code Analysis

Obfuscators frequently flatten control flow (switch-based dispatch, infinite loops with breaks) and insert dead code. Effective analysis and simplification of these patterns is crucial for readability and further deobfuscation.

### 10.1 Truthiness & Side-Effect Analysis

```rust
impl DeadCodePass {
    fn get_truthiness(expr: &Expression) -> Option<bool> {
        match expr {
            Expression::BooleanLiteral(l) => Some(l.value),
            Expression::NumericLiteral(n) => Some(n.value != 0.0 && !n.value.is_nan()),
            Expression::StringLiteral(s) => Some(!s.value.is_empty()),
            Expression::NullLiteral(_) | Expression::Identifier(i) if i.name == "undefined" => Some(false),
            Expression::ArrayExpression(_) | Expression::ObjectExpression(_) => Some(true),
            Expression::UnaryExpression(u) if u.operator == UnaryOperator::LogicalNot => {
                Self::get_truthiness(&u.argument).map(|v| !v)
            }
            Expression::ParenthesizedExpression(p) => Self::get_truthiness(&p.expression),
            _ => None,
        }
    }

    fn is_side_effect_free(expr: &Expression) -> bool {
        match expr {
            Expression::Literal(_) | Expression::Identifier(_) => true,
            Expression::UnaryExpression(u) => Self::is_side_effect_free(&u.argument),
            Expression::BinaryExpression(b) => {
                Self::is_side_effect_free(&b.left) && Self::is_side_effect_free(&b.right)
            }
            Expression::ParenthesizedExpression(p) => Self::is_side_effect_free(&p.expression),
            Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_) => true,
            _ => false,
        }
    }
}
```

### 10.2 Conditional Simplification

```rust
fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a, ()>) {
    match expr {
        // ? : with known test
        Expression::ConditionalExpression(cond) => {
            if let Some(truthy) = Self::get_truthiness(&cond.test) {
                *expr = if truthy {
                    cond.consequent.take_in(ctx.ast.allocator)
                } else {
                    cond.alternate.take_in(ctx.ast.allocator)
                };
                self.modifications += 1;
            }
        }
        // || && short-circuit
        Expression::LogicalExpression(log) => {
            if let Some(truthy) = Self::get_truthiness(&log.left) {
                *expr = match log.operator {
                    LogicalOperator::Or => if truthy { log.left.take_in(...) } else { log.right.take_in(...) },
                    LogicalOperator::And => if truthy { log.right.take_in(...) } else { log.left.take_in(...) },
                    _ => return,
                };
                self.modifications += 1;
            }
        }
        _ => {}
    }
}
```

### 10.3 Dead Code Elimination

```rust
fn exit_statements(&mut self, stmts: &mut ArenaVec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a, ()>) {
    let allocator = ctx.ast.allocator;
    let scoping = ctx.scoping();
    let mut new_stmts = ctx.ast.vec();
    let mut terminated = false;

    for stmt in stmts.take_in(allocator) {
        if terminated {
            self.modifications += 1;
            continue;
        }

        // Remove obvious dead statements
        if matches!(stmt, Statement::EmptyStatement(_) | Statement::DebuggerStatement(_)) {
            self.modifications += 1;
            continue;
        }

        // Variable declarations — remove unused
        if let Statement::VariableDeclaration(var) = &stmt {
            let mut decls = var.declarations.take_in(allocator);
            decls.retain(|d| {
                let symbol_id = d.id.get_binding_identifier()
                    .and_then(|b| b.symbol_id.get())
                    .unwrap_or_default();
                let has_reads = count_read_references(scoping, symbol_id) > 0;
                let has_side_effects = d.init.as_ref().map_or(false, |i| !Self::is_side_effect_free(i));
                has_reads || has_side_effects
            });

            if decls.is_empty() {
                self.modifications += 1;
                continue;
            }

            // Rebuild if filtered
            if decls.len() < var.declarations.len() {
                let new_var = ctx.ast.alloc_variable_declaration(var.span, var.kind, decls, var.declare);
                new_stmts.push(Statement::VariableDeclaration(new_var));
                self.modifications += 1;
                continue;
            }
        }

        if stmt.is_terminator() { terminated = true; }
        new_stmts.push(stmt);
    }

    *stmts = new_stmts;
}
```

### 10.4 Control Flow Deflattening (High-Level)

Typical pattern:

- Large switch dispatching on state variable
- Cases contain state updates and jumps

**Approach**:

1. Identify dispatcher function
2. Map case values to blocks
3. Reconstruct sequential/linear flow
4. Replace switch with rebuilt statements

Requires CFG analysis or pattern matching.

### 10.5 Critical Rules

**MUST**:

- Use side-effect-free check before dropping expressions.
- Trim after terminators.
- Rebuild scoping before final unused removal.

---

## 11. Core Deobfuscation Recipes

These are the foundational, high-impact recipes that form the backbone of any effective deobfuscator. They are proven to handle the majority of common obfuscation patterns.

### 11.1 Constant Propagation & Inlining

**Goal**: Replace identifiers with their constant values when safe.

```rust
pub struct ConstantPropagator;

impl Module for ConstantPropagator {
    fn name(&self) -> &'static str { "ConstantPropagator" }

    fn transform<'a>(
        &mut self,
        allocator: &'a Allocator,
        program: &mut Program<'a>,
        scoping: Scoping,
    ) -> Result<TransformResult> {
        let mut collector = Collector::default();
        let scoping = traverse_mut(&mut collector, allocator, program, scoping, ());

        if collector.constants.is_empty() {
            return Ok(TransformResult { modifications: 0, scoping });
        }

        let mut inliner = Inliner {
            constants: collector.constants,
            modifications: 0,
        };
        let scoping = traverse_mut(&mut inliner, allocator, program, scoping, ());

        Ok(TransformResult { modifications: inliner.modifications, scoping })
    }
}

#[derive(Default)]
struct Collector {
    constants: FxHashMap<SymbolId, LiteralValue>,
}

impl<'a> Traverse<'a, ()> for Collector<'a> {
    fn enter_variable_declarator(&mut self, node: &mut VariableDeclarator<'a>, ctx: &mut TraverseCtx<'a, ()>) {
        let Some(init) = &node.init else { return };
        let Some(binding) = node.id.get_binding_identifier() else { return };
        let Some(symbol_id) = binding.symbol_id.get() else { return };
        let Some(value) = extract_literal_value(init) else { return };

        // SAFETY: No writes
        if ctx.scoping().get_resolved_references(symbol_id).any(|r| r.flags().is_write()) {
            return;
        }

        self.constants.insert(symbol_id, value);
    }
}

struct Inliner {
    constants: FxHashMap<SymbolId, LiteralValue>,
    modifications: usize,
}

impl<'a> Traverse<'a, ()> for Inliner<'a> {
    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a, ()>) {
        if let Expression::Identifier(ident) = expr {
            if let Some(symbol_id) = get_reference_symbol(ctx.scoping(), ident) {
                if let Some(value) = self.constants.get(&symbol_id) {
                    *expr = literal_value_to_expression(value, ctx.ast);
                    self.modifications += 1;
                }
            }
        }
    }
}
```

### 11.2 Dead Code Elimination

**Goal**: Remove unused declarations and unreachable statements.

```rust
pub struct DeadCodeEliminator;

impl Module for DeadCodeEliminator {
    fn name(&self) -> &'static str { "DeadCodeEliminator" }

    fn changes_symbols(&self) -> bool { true }  // Removes declarations

    fn transform<'a>(
        &mut self,
        allocator: &'a Allocator,
        program: &mut Program<'a>,
        scoping: Scoping,
    ) -> Result<TransformResult> {
        let mut visitor = DeadCodeVisitor::default();
        let scoping = traverse_mut(&mut visitor, allocator, program, scoping, ());
        Ok(TransformResult { modifications: visitor.modifications, scoping })
    }
}

#[derive(Default)]
struct DeadCodeVisitor {
    modifications: usize,
}

impl DeadCodeVisitor {
    /// Determine truthiness of an expression at compile time
    fn get_truthiness(expr: &Expression) -> Option<bool> {
        match expr {
            Expression::BooleanLiteral(lit) => Some(lit.value),
            Expression::NumericLiteral(lit) => Some(lit.value != 0.0 && !lit.value.is_nan()),
            Expression::StringLiteral(lit) => Some(!lit.value.is_empty()),
            Expression::NullLiteral(_) => Some(false),
            Expression::Identifier(ident) if ident.name == "undefined" => Some(false),
            Expression::Identifier(ident) if ident.name == "NaN" => Some(false),
            Expression::ArrayExpression(_) | Expression::ObjectExpression(_) => Some(true),
            Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_) => Some(true),
            Expression::UnaryExpression(u) if u.operator == UnaryOperator::LogicalNot => {
                Self::get_truthiness(&u.argument).map(|v| !v)
            }
            Expression::UnaryExpression(u) if u.operator == UnaryOperator::Void => Some(false),
            Expression::ParenthesizedExpression(p) => Self::get_truthiness(&p.expression),
            Expression::SequenceExpression(s) => s.expressions.last().and_then(Self::get_truthiness),
            _ => None,
        }
    }

    /// Check if an expression has no side effects (safe to remove)
    fn is_side_effect_free(expr: &Expression) -> bool {
        match expr {
            // Literals are always side-effect free
            Expression::NumericLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_)
            | Expression::BigIntLiteral(_)
            | Expression::RegExpLiteral(_) => true,

            // Identifiers (reading a variable has no side effect)
            Expression::Identifier(_) => true,

            // This expression
            Expression::ThisExpression(_) => true,

            // Function/class expressions (not calls)
            Expression::FunctionExpression(_)
            | Expression::ArrowFunctionExpression(_)
            | Expression::ClassExpression(_) => true,

            // Template literals without tag
            Expression::TemplateLiteral(t) => {
                t.expressions.iter().all(Self::is_side_effect_free)
            }

            // Unary expressions (except delete)
            Expression::UnaryExpression(u) => {
                u.operator != UnaryOperator::Delete && Self::is_side_effect_free(&u.argument)
            }

            // Binary expressions
            Expression::BinaryExpression(b) => {
                Self::is_side_effect_free(&b.left) && Self::is_side_effect_free(&b.right)
            }

            // Logical expressions
            Expression::LogicalExpression(l) => {
                Self::is_side_effect_free(&l.left) && Self::is_side_effect_free(&l.right)
            }

            // Conditional expressions
            Expression::ConditionalExpression(c) => {
                Self::is_side_effect_free(&c.test)
                    && Self::is_side_effect_free(&c.consequent)
                    && Self::is_side_effect_free(&c.alternate)
            }

            // Sequence expressions
            Expression::SequenceExpression(s) => {
                s.expressions.iter().all(Self::is_side_effect_free)
            }

            // Parenthesized
            Expression::ParenthesizedExpression(p) => Self::is_side_effect_free(&p.expression),

            // Array expressions (if all elements are side-effect free)
            Expression::ArrayExpression(arr) => arr.elements.iter().all(|el| {
                match el {
                    ArrayExpressionElement::SpreadElement(s) => Self::is_side_effect_free(&s.argument),
                    ArrayExpressionElement::Elision(_) => true,
                    _ => el.as_expression().map_or(true, Self::is_side_effect_free),
                }
            }),

            // Object expressions (if all properties are side-effect free)
            Expression::ObjectExpression(obj) => obj.properties.iter().all(|prop| {
                match prop {
                    ObjectPropertyKind::ObjectProperty(p) => {
                        (!p.computed || Self::is_side_effect_free(p.key.as_expression().unwrap_or(&Expression::NullLiteral(oxc::allocator::Box::new_in(NullLiteral { span: SPAN }, &oxc::allocator::Allocator::default())))))
                            && p.value.as_expression().map_or(true, Self::is_side_effect_free)
                    }
                    ObjectPropertyKind::SpreadProperty(s) => Self::is_side_effect_free(&s.argument),
                }
            }),

            // Member expressions (reading property - no side effect)
            Expression::StaticMemberExpression(m) => Self::is_side_effect_free(&m.object),
            Expression::ComputedMemberExpression(m) => {
                Self::is_side_effect_free(&m.object) && Self::is_side_effect_free(&m.expression)
            }

            // Everything else potentially has side effects
            // Calls, assignments, new, await, yield, etc.
            _ => false,
        }
    }

    /// Check if a statement terminates control flow
    fn is_terminator(stmt: &Statement) -> bool {
        matches!(
            stmt,
            Statement::ReturnStatement(_)
                | Statement::ThrowStatement(_)
                | Statement::BreakStatement(_)
                | Statement::ContinueStatement(_)
        )
    }
}

impl<'a> Traverse<'a, ()> for DeadCodeVisitor {
    fn exit_statements(
        &mut self,
        stmts: &mut ArenaVec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) {
        let allocator = ctx.ast.allocator;
        let scoping = ctx.scoping();
        let mut new_stmts = ctx.ast.vec();
        let mut terminated = false;

        for stmt in stmts.take_in(allocator) {
            // Skip unreachable code after terminator
            if terminated {
                self.modifications += 1;
                continue;
            }

            // Remove empty and debugger statements
            if matches!(stmt, Statement::EmptyStatement(_) | Statement::DebuggerStatement(_)) {
                self.modifications += 1;
                continue;
            }

            // Handle variable declarations - remove unused
            if let Statement::VariableDeclaration(var_decl) = &stmt {
                let original_count = var_decl.declarations.len();
                let mut kept_decls = ctx.ast.vec();

                for decl in var_decl.declarations.iter() {
                    let should_keep = if let Some(binding) = decl.id.get_binding_identifier() {
                        if let Some(symbol_id) = binding.symbol_id.get() {
                            // Keep if has reads OR initializer has side effects
                            let has_reads = scoping
                                .get_resolved_references(symbol_id)
                                .any(|r| r.flags().is_read());
                            let has_side_effects = decl
                                .init
                                .as_ref()
                                .map_or(false, |i| !Self::is_side_effect_free(i));
                            has_reads || has_side_effects
                        } else {
                            true // Keep if no symbol (safety)
                        }
                    } else {
                        true // Keep destructuring patterns (complex analysis)
                    };

                    if should_keep {
                        kept_decls.push(decl.clone_in(allocator));
                    }
                }

                if kept_decls.is_empty() {
                    self.modifications += original_count;
                    continue;
                }

                if kept_decls.len() < original_count {
                    self.modifications += original_count - kept_decls.len();
                    let new_var = ctx.ast.alloc_variable_declaration(
                        var_decl.span,
                        var_decl.kind,
                        kept_decls,
                        var_decl.declare,
                    );
                    new_stmts.push(Statement::VariableDeclaration(new_var));
                    continue;
                }
            }

            // Handle function declarations - remove unused
            if let Statement::FunctionDeclaration(func) = &stmt {
                if let Some(id) = &func.id {
                    if let Some(symbol_id) = id.symbol_id.get() {
                        let has_reads = scoping
                            .get_resolved_references(symbol_id)
                            .any(|r| r.flags().is_read());
                        if !has_reads {
                            self.modifications += 1;
                            continue;
                        }
                    }
                }
            }

            // Handle if statements with known conditions
            if let Statement::IfStatement(if_stmt) = &stmt {
                if let Some(truthy) = Self::get_truthiness(&if_stmt.test) {
                    if truthy {
                        // if (true) { ... } → unwrap consequent
                        new_stmts.push(if_stmt.consequent.clone_in(allocator));
                        self.modifications += 1;
                        continue;
                    } else if let Some(alt) = &if_stmt.alternate {
                        // if (false) { ... } else { ... } → unwrap alternate
                        new_stmts.push(alt.clone_in(allocator));
                        self.modifications += 1;
                        continue;
                    } else {
                        // if (false) { ... } → remove entirely
                        self.modifications += 1;
                        continue;
                    }
                }
            }

            // Check for terminator
            if Self::is_terminator(&stmt) {
                terminated = true;
            }

            new_stmts.push(stmt);
        }

        *stmts = new_stmts;
    }

    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a, ()>) {
        let allocator = ctx.ast.allocator;

        match expr {
            // Simplify conditional expressions: true ? a : b → a
            Expression::ConditionalExpression(cond) => {
                if let Some(truthy) = Self::get_truthiness(&cond.test) {
                    *expr = if truthy {
                        cond.consequent.take_in(allocator)
                    } else {
                        cond.alternate.take_in(allocator)
                    };
                    self.modifications += 1;
                }
            }

            // Simplify logical expressions: true || x → true, false && x → false
            Expression::LogicalExpression(logical) => {
                if let Some(truthy) = Self::get_truthiness(&logical.left) {
                    let new_expr = match logical.operator {
                        LogicalOperator::Or => {
                            if truthy {
                                // true || x → true (short-circuit)
                                logical.left.take_in(allocator)
                            } else {
                                // false || x → x
                                logical.right.take_in(allocator)
                            }
                        }
                        LogicalOperator::And => {
                            if truthy {
                                // true && x → x
                                logical.right.take_in(allocator)
                            } else {
                                // false && x → false (short-circuit)
                                logical.left.take_in(allocator)
                            }
                        }
                        LogicalOperator::Coalesce => {
                            // null ?? x → x, value ?? x → value
                            if matches!(
                                &logical.left,
                                Expression::NullLiteral(_)
                                    | Expression::Identifier(i) if i.name == "undefined"
                            ) {
                                logical.right.take_in(allocator)
                            } else if Self::is_side_effect_free(&logical.left) {
                                logical.left.take_in(allocator)
                            } else {
                                return;
                            }
                        }
                    };
                    *expr = new_expr;
                    self.modifications += 1;
                }
            }

            // Simplify sequence expressions: remove side-effect-free non-final expressions
            Expression::SequenceExpression(seq) if seq.expressions.len() > 1 => {
                let mut new_exprs = ctx.ast.vec();
                let last_idx = seq.expressions.len() - 1;

                for (i, e) in seq.expressions.iter().enumerate() {
                    if i == last_idx || !Self::is_side_effect_free(e) {
                        new_exprs.push(e.clone_in(allocator));
                    } else {
                        self.modifications += 1;
                    }
                }

                if new_exprs.len() == 1 {
                    *expr = new_exprs.pop().unwrap();
                } else if new_exprs.len() < seq.expressions.len() {
                    *expr = ctx.ast.expression_sequence(seq.span, new_exprs);
                }
            }

            _ => {}
        }
    }
}
```

Key features:

- Trim after terminators (`return`, `throw`, `break`, `continue`)
- Remove unused variables (zero reads)
- Preserve side-effectful initializers
- Handle multi-declarations
- Simplify conditionals with known truthiness
- Remove unused function declarations

### 11.3 Proxy Function Inlining

**Goal**: Inline simple wrapper functions (common in obfuscation).

Pattern: `function _0x1234(a,b) { return expr(a,b); }`

```rust
// Collector finds proxy functions
if func.body.statements.len() == 1 {
    if let Statement::ReturnStatement(ret) = &func.body.statements[0] {
        if let Some(return_expr) = &ret.argument {
            // Map parameters
            let param_names = func.params.items.iter().map(|p| p.name.to_string()).collect();
            self.proxies.insert(func_symbol, Proxy { param_names, body: return_expr.clone_in(...) });
        }
    }
}

// Inliner replaces calls
if let Expression::CallExpression(call) = expr {
    if let Some(proxy) = self.proxies.get(&callee_symbol) {
        if call.arguments.len() == proxy.param_names.len() {
            let mut body = proxy.body.clone_in(allocator);
            let mut substituter = ParamSubstituter { subs: &build_map(&proxy.param_names, &call.arguments), alloc: allocator };
            substituter.visit_expression(&mut body);  // VisitMut for subtree
            *expr = body;
            self.modifications += 1;
        }
    }
}
```

**Key**: Use `VisitMut` for parameter substitution in cloned body.

### 11.4 Critical Patterns Summary

| Recipe                | Pass Type | changes_symbols() | Key Safety Check      |
| --------------------- | --------- | ----------------- | --------------------- |
| Constant Propagation  | Two-pass  | false             | No writes             |
| Dead Code Elimination | One-pass  | true              | Fresh scoping         |
| Proxy Inlining        | Two-pass  | false             | No writes to function |

---

## 12. Advanced Techniques

These techniques target sophisticated obfuscation patterns seen in production tools (e.g., JavaScript Obfuscator, JScrambler). They build on core recipes and often require multiple passes or dynamic evaluation.

### 12.1 String Array Decoding

**Goal**: Decode obfuscated string access via large string array + index function.

Pattern:

```js
var _0xabc = ["hello", "world", ...];
function _0x123(x) { return _0xabc[x]; }
console.log(_0x123(0) + _0x123(1));
```

**Approach**:

1. Identify array + decoder function
2. Track index → string mapping
3. Replace calls with literal strings
4. Post-cleanup removes array

**Key**: Run **early** in convergence loop.

### 12.2 Object Property Flattening

**Goal**: Inline properties from large object literals.

Pattern:

```js
var obj = { a: 1, b: "x", c: true };
var x = obj.a + obj.b;
```

**Safety**:

- Object not mutated
- Properties not deleted/reconfigured

### 12.3 Alias Inlining

**Goal**: Remove temporary variable aliases.

Pattern:

```js
var tmp = someLongExpression();
use(tmp);
use(tmp);
```

**Safety**:

- No writes to tmp
- Expression side-effect-free or handled with temp var

### 12.4 IIFE Optimization

**Goal**: Unwrap immediately-invoked function expressions.

Pattern:

```js
(function (a, b) {
  return a + b;
})(1, 2);
```

**Approach**:

- Inline parameters
- Replace with body expression

### 12.5 Sequence Expression Simplification

**Goal**: Flatten comma expressions, remove unused parts.

```js
(a = 1, b = 2, foo(), "result")
→ "result"
```

**Safety**: Preserve side effects in order.

### 12.6 Eval Call Inlining

**Goal**: Decode and inline `eval(encodedString)`.

**Approach**:

- Extract encoded string
- Decode (base64, RC4, etc.)
- Parse decoded source
- Inline or replace

**Safety**: Only if decoded string is safe.

### 12.7 Control Flow Deflattening

**Goal**: Reconstruct linear flow from switch-dispatch VM.

**Steps**:

1. Identify state variable + dispatcher switch
2. Map case values to block sequences
3. Rebuild sequential statements
4. Replace dispatcher loop

**Key**: Run **late** — requires prior simplifications.

**Pattern Recognition**:

- Large switch with consecutive cases
- Cases update state and jump
- Infinite loop wrapper

### 12.8 Critical Implementation Notes

| Technique                 | Pass Position | changes_symbols() | Dependencies              |
| ------------------------- | ------------- | ----------------- | ------------------------- |
| String Array Decoding     | Earliest      | true (post-clean) | None                      |
| Object Flattening         | Mid           | false             | Constant propagation      |
| Eval Inlining             | Mid-late      | true              | String decoding           |
| Control Flow Deflattening | Latest        | true              | All prior simplifications |

---

## 13. Anti-Patterns & Pitfalls

These are the most common and dangerous mistakes made during OXC-based deobfuscation. Many cause **silent semantic changes**, **panics**, or **infinite loops**. Avoid them at all costs.

### 13.1 Direct Accessor Panics

```rust
// WRONG — WILL PANIC if not set
let ref_id = ident.reference_id();  // Direct accessor

// CORRECT
let ref_id = ident.reference_id.get()?;  // Safe Option
```

**Rule**: Always use `.get()` for `Cell<Option<_>>`.

### 13.2 Stale Scoping Usage

```rust
// WRONG — Reference counts wrong after removal
let scoping = traverse_mut(&mut remover, ...);  // Removes declarations
let scoping = traverse_mut(&mut checker, ...);  // Uses stale scoping!
```

**Fix**: Rebuild with `SemanticBuilder::build(&program)` after structural changes.

### 13.3 Wrong Cloning

```rust
// WRONG — Lifetime/semantic issues
let cloned = expr.clone();

// CORRECT
let cloned = expr.clone_in(ctx.ast.allocator);
```

### 13.4 Storing Raw References

```rust
// WRONG — References invalidate
Vec<&Expression<'a>>  // Becomes dangling

// CORRECT
FxHashMap<SymbolId, Expression<'a>>  // Cloned into arena
```

### 13.5 Modifying in enter\_\*

```rust
// DANGEROUS — Children may not be visited
fn enter_expression(&mut self, expr: &mut Expression<'a>, ...) {
    *expr = replacement;  // Skips child processing
}
```

**Fix**: Replace in `exit_*`.

### 13.6 Manual AST Recursion

```rust
// WRONG — High maintenance, easy misses
match expr {
    Expression::BinaryExpression(b) => {
        recurse(&mut b.left);
        recurse(&mut b.right);
    }
    // Must handle ALL variants
}
```

**Fix**: Use `VisitMut` + `walk_mut::*`.

### 13.7 Missing Parentheses Handling

```rust
// WRONG — Misses (("string"))
match expr {
    Expression::StringLiteral(_) => ...
    _ => None,
}
```

**Fix**: Recurse into `ParenthesizedExpression`.

### 13.8 Wrong Member Matching

```rust
// COMPILE ERROR
Expression::MemberExpression(_)
```

**Fix**: Use specific variants (`StaticMemberExpression`, etc.).

### 13.9 No Modification Counting

```rust
// WRONG — Convergence never triggers
*expr = new_expr;  // Forgot increment!
```

**Fix**: Always `self.modifications += 1`.

### 13.10 Infinite Convergence Loops

```rust
// WRONG — Always counts as modification
*expr = evaluate(expr);
self.modifications += 1;  // Even if same!
```

**Fix**: Guard with `content_eq` or equality check.

### 13.11 Critical Summary

| Pitfall               | Symptom             | Prevention                 |
| --------------------- | ------------------- | -------------------------- |
| Direct accessor       | Panic               | Use `.get()?`              |
| Stale scoping         | Wrong counts        | Rebuild after changes      |
| Standard clone        | Lifetime errors     | `clone_in`                 |
| Raw references        | Dangling pointers   | Clone into arena           |
| enter\_\* replacement | Missed children     | Use exit\_\*               |
| Manual recursion      | Incomplete coverage | VisitMut                   |
| No parens handling    | Missed literals     | Recurse into Parenthesized |
| No modification count | No convergence      | Always increment           |
| No-op modifications   | Infinite loop       | Check for actual change    |

---

## 14. Value Representation Patterns

Choosing the right value representation is critical for performance, memory usage, and correctness. Avoid unnecessary AST cloning when only primitive values are needed.

### 14.1 LiteralValue — Compact Primitive Storage

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LiteralValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
    Undefined,
}
```

**Use Cases**:

- Constant propagation (store by SymbolId)
- Static evaluation caching
- String array mapping
- Any scenario needing only primitive data

**Advantages**:

- No lifetimes
- Cheap cloning
- Hashable (with Eq/Hash)
- Small footprint

**Extraction & Conversion**:

```rust
pub fn extract_literal_value(expr: &Expression) -> Option<LiteralValue> {
    // Handles parens, unary negation, sequences
}

pub fn literal_value_to_expression<'a>(value: &LiteralValue, ast: AstBuilder<'a>) -> Expression<'a> {
    // Preserve negative zero, unary negation for negatives
}
```

### 14.2 CaseValue — Switch Case Matching

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CaseValue {
    Number(i64),   // Exact integers for switch dispatch
    String(String),
}
```

**Use Cases**:

- Control flow deflattening
- State machine reconstruction
- Switch-based VM analysis

**Why i64**: Exact integer matching (avoids float precision issues).

### 14.3 Expression<'a> — Full AST Preservation

**Use When**:

- Need exact structure (e.g., unary negation style)
- Complex expressions (objects, arrays)
- Later codegen fidelity required

**Cost**: Arena cloning, lifetimes.

### 14.4 Decision Matrix

| Scenario                    | Recommended Type             | Reason                                 |
| --------------------------- | ---------------------------- | -------------------------------------- |
| Constant inlining           | `LiteralValue`               | Compact, fast lookup                   |
| String array mapping        | `LiteralValue::String`       | String-only, hashable                  |
| Switch case reconstruction  | `CaseValue`                  | Exact matching, hashable               |
| Proxy function body storage | `Expression<'a>`             | Full structure needed for substitution |
| Static evaluation result    | `LiteralValue`               | Primitive output                       |
| Object property inlining    | `LiteralValue` or Expression | Depends on complexity                  |

### 14.5 Critical Rules

**MUST**:

- Use compact types for storage when possible.
- Convert back accurately (preserve sign, negation style).

**MUST NOT**:

- Store full `Expression<'a>` when only primitive needed.

---

## 15. Quick Reference

This section provides concise cheat sheets for the most common operations in OXC-based deobfuscation.

### 15.1 Essential Imports

```rust
// Core
use oxc::allocator::{Allocator, CloneIn, TakeIn, Vec as ArenaVec};
use oxc::ast::ast::*;
use oxc::parser::Parser;
use oxc::semantic::{Scoping, SemanticBuilder};
use oxc::span::{Span, SPAN};
use oxc::codegen::Codegen;

// Traversal
use oxc_traverse::{Traverse, TraverseCtx, traverse_mut};

// VisitMut for subtrees
use oxc_ast_visit::{VisitMut, walk_mut};

// Collections
use rustc_hash::{FxHashMap, FxHashSet};
```

### 15.2 Symbol Resolution Snippets

| Task                  | Code                                                                             |
| --------------------- | -------------------------------------------------------------------------------- |
| Reference → SymbolId  | `ident.reference_id.get()?.let(\|r\| scoping.get_reference(r).symbol_id())`      |
| Binding → SymbolId    | `binding.symbol_id.get()`                                                        |
| Declarator → SymbolId | `decl.id.get_binding_identifier().and_then(\|b\| b.symbol_id.get())`             |
| Function → SymbolId   | `func.id.as_ref().and_then(\|i\| i.symbol_id.get())`                             |
| Read count            | `scoping.get_resolved_references(id).filter(\|r\| r.flags().is_read()).count()`  |
| Write count           | `scoping.get_resolved_references(id).filter(\|r\| r.flags().is_write()).count()` |

### 15.3 Expression Builders

| Expression           | Code                                                                   |
| -------------------- | ---------------------------------------------------------------------- |
| String literal       | `ctx.ast.expression_string_literal(SPAN, ctx.ast.atom("value"), None)` |
| Number               | `ctx.ast.expression_numeric_literal(SPAN, 42.0, None, Decimal)`        |
| Boolean              | `ctx.ast.expression_boolean_literal(SPAN, true)`                       |
| Null                 | `ctx.ast.expression_null_literal(SPAN)`                                |
| Identifier reference | `ctx.ast.expression_identifier_reference(SPAN, ctx.ast.atom("name"))`  |
| Binary               | `ctx.ast.expression_binary(SPAN, left, Addition, right)`               |
| Unary negation       | `ctx.ast.expression_unary(SPAN, UnaryNegation, arg)`                   |

### 15.4 Statement Builders

| Statement       | Code                                                                  |
| --------------- | --------------------------------------------------------------------- |
| Expression stmt | `ctx.ast.statement_expression(SPAN, expr)`                            |
| Block           | `ctx.ast.statement_block(SPAN, stmts)`                                |
| Return          | `ctx.ast.statement_return(SPAN, Some(expr))`                          |
| Var declaration | `ctx.ast.statement_variable_declaration(SPAN, Var, vec1(declarator))` |

### 15.5 Memory Operations

| Operation         | Code                               |
| ----------------- | ---------------------------------- |
| Clone expression  | `expr.clone_in(ctx.ast.allocator)` |
| Take vector       | `vec.take_in(ctx.ast.allocator)`   |
| New vector        | `ctx.ast.vec()`                    |
| Vec with one item | `ctx.ast.vec1(item)`               |

### 15.6 Common Checks

| Check            | Pattern                                                        |
| ---------------- | -------------------------------------------------------------- |
| Literal          | `matches!(expr, StringLiteral(_) \| NumericLiteral(_) \| ...)` |
| Identifier       | `matches!(expr, Expression::Identifier(_))`                    |
| Side-effect free | Custom function (literals, identifiers, pure ops)              |
| Truthiness       | Custom function (falsey values)                                |

### 15.7 Traverse Hooks Overview

| Hook                    | Timing          | Common Use             |
| ----------------------- | --------------- | ---------------------- |
| `enter_*`               | Before children | Collect, early bailout |
| `exit_*`                | After children  | Replace, transform     |
| `enter/exit_statements` | Statement lists | Rebuild, filter        |
| `enter/exit_function`   | Function scope  | Stack push/pop         |

---

## Appendix A: File Structure

Recommended project layout based on production experience:

```
src/
├── main.rs                     # Entry point, pipeline orchestration
├── lib.rs                      # Re-exports
├── core/
│   ├── module.rs               # Module trait + TransformResult
│   ├── registry.rs             # ModuleRegistry + formatters()/deobfuscators()
│   ├── engine.rs               # Convergence loop execution
│   └── error.rs                # Custom error types
├── utils/
│   ├── ast.rs                  # AstBuilder helpers, build_scoping wrapper
│   ├── symbols.rs              # Safe resolution functions (get_reference_symbol, counts)
│   ├── literal.rs              # LiteralValue, extract_literal_value, to_expression
│   ├── span.rs                 # Span utilities
│   └── eval.rs                 # expr_to_code, value_to_expr
├── eval/
│   ├── runtime.rs              # JsEvaluator wrapper (V8/deno_core)
│   ├── safety.rs               # is_safe_expr (full implementation)
│   └── cache.rs                # Evaluation result caching
├── modules/
│   ├── formatters/
│   │   ├── brace_wrapper.rs            # changes_symbols: false
│   │   ├── statement_splitter.rs       # changes_symbols: TRUE (splits declarations)
│   │   ├── identifier_renamer.rs       # changes_symbols: TRUE (renames bindings)
│   │   ├── for_var_hoister.rs          # changes_symbols: TRUE (moves declarations)
│   │   └── member_simplifier.rs        # changes_symbols: false
│   └── transformers/
│       ├── constant_propagator.rs      # changes_symbols: false (read-only)
│       ├── dead_code_eliminator.rs     # changes_symbols: TRUE (removes declarations)
│       ├── proxy_inliner.rs            # changes_symbols: false
│       ├── string_array_decoder.rs     # changes_symbols: TRUE (post-cleanup)
│       ├── object_flattener.rs         # changes_symbols: false
│       ├── alias_inliner.rs            # changes_symbols: false
│       ├── iife_optimizer.rs           # changes_symbols: TRUE (unwraps scopes)
│       ├── eval_call_inliner.rs        # changes_symbols: TRUE (inlines code)
│       ├── sequence_simplifier.rs      # changes_symbols: false
│       ├── static_evaluator.rs         # changes_symbols: false
│       └── control_flow_deflattener.rs # changes_symbols: TRUE (restructures)
└── visitors/
    └── param_substituter.rs    # VisitMut for proxy inlining
```

### changes_symbols() Quick Reference

| Module                 | changes_symbols() | Reason                                       |
| ---------------------- | ----------------- | -------------------------------------------- |
| BraceWrapper           | `false`           | Only wraps statements in blocks              |
| StatementSplitter      | **`true`**        | Splits `var a, b` into separate declarations |
| IdentifierRenamer      | **`true`**        | Renames binding identifiers                  |
| ForVarHoister          | **`true`**        | Moves var declarations out of for-init       |
| MemberSimplifier       | `false`           | Only simplifies member access syntax         |
| ConstantPropagator     | `false`           | Replaces references, doesn't remove decls    |
| DeadCodeEliminator     | **`true`**        | Removes unused var/function declarations     |
| ProxyInliner           | `false`           | Inlines calls, doesn't remove functions      |
| StringArrayDecoder     | **`true`**        | Post-cleanup removes array + decoder         |
| ObjectFlattener        | `false`           | Inlines properties, doesn't remove objects   |
| AliasInliner           | `false`           | Inlines aliases, cleanup done separately     |
| IifeOptimizer          | **`true`**        | Unwraps function scopes                      |
| EvalCallInliner        | **`true`**        | Inlines parsed code with new declarations    |
| SequenceSimplifier     | `false`           | Only simplifies expressions                  |
| StaticEvaluator        | `false`           | Replaces expressions with literals           |
| ControlFlowDeflattener | **`true`**        | Restructures control flow, removes state var |

**Rule**: When `changes_symbols() == true`, the engine rebuilds scoping after the pass.

**Rationale**:

- `core/` — framework (traits, registry, engine)
- `utils/` — shared helpers
- `eval/` — dynamic execution
- `modules/` — separated by type for clarity
- `visitors/` — VisitMut subtree transforms
