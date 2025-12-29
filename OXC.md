# OXC.md — The Definitive OXC Reference for JavaScript Deobfuscation

> **Version:** 1.0.0 | **Date:** 2025-12-27 | **Source:** OXC research (December 2025)

Comprehensive reference for OXC crates in Rust for JavaScript/TypeScript parsing, analysis, and transformation.

---

## Table of Contents

| #   | Section Title                                   | Tier     |
| --- | ----------------------------------------------- | -------- |
| 1   | Introduction and Explicit Assumptions           | Tier 1   |
| 2   | Span, Atom, and CompactStr                      | Tier 1   |
| 3   | SourceType and Language Configuration           | Tier 1   |
| 4   | Memory Model and Allocator                      | Tier 1   |
| 5   | AST Structure and Design Principles             | Tier 2   |
| 6   | Parser Architecture                             | Tier 2   |
| 7   | Traversal System (oxc_traverse)                 | Tier 2   |
| 8   | Semantic Analysis (Symbols, Scopes, References) | Tier 2   |
| 9   | Control Flow Graph (oxc_cfg)                    | Tier 3   |
| 10  | ECMAScript Specification Helpers                | Tier 3   |
| 11  | Syntax Types (Operators, Flags, Precedence)     | Tier 3   |
| 12  | Code Generation (oxc_codegen)                   | Tier 4   |
| 13  | Built-in Transformers (oxc_transformer)         | Tier 4   |
| 14  | AST Visiting vs Traversing                      | Tier 4   |
| 15  | Common Patterns and Anti-Patterns               | Tier 5   |
| 16  | Quick Reference Tables                          | Tier 5   |
| A   | Appendix A: Generated Code Insights             | Appendix |
| B   | Appendix B: Performance Considerations          | Appendix |

## Section Summary

| #   | Section Title                          | Summary of Coverage                                                                                                                                                  | Primary Sources                                    | Depth  |
| --- | -------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------- | ------ |
| 1   | Introduction and Explicit Assumptions  | Purpose; version context (2025-12-26); explicit assumptions (arena allocation, no Drop, SymbolId/ReferenceId preservation, Without-type safety); deobfuscation goals | New synthesis + scattered notes from all research  | High   |
| 2   | Span, Atom, and CompactStr             | Span structure/methods (merge, expand, source_text); SPAN constant; Atom creation/interning; CompactStr inline optimization (16-byte); ContentEq semantics           | OXC Span Research                                  | Full   |
| 3   | SourceType and Language Configuration  | Language/ModuleKind/LanguageVariant enums; from_path detection; builder methods; strict mode implications; valid extensions                                          | OXC Span + Parser Research                         | Full   |
| 4   | Memory Model and Allocator             | Bump-based arena; no Drop enforcement; Allocator recycling; Box/Vec/StringBuilder/HashMap in arena; CloneIn vs TakeIn; Address stability; AllocatorPool              | OXC Allocator Research                             | Full   |
| 5   | AST Structure and Design Principles    | Enum inheritance (inherit*variants!); boxed variants; 4 identifier types; literal types; AstBuilder generation; match*\* macros; GetSpan/GetSpanMut                  | OXC AST Research                                   | Full   |
| 6   | Parser Architecture                    | Lexer → ParserImpl flow; Token 128-bit layout; Context flags; checkpoint/rewind; error recovery; ParseOptions; module record                                         | OXC Parser Research                                | Full   |
| 7   | Traversal System (oxc_traverse)        | Without-type safety scheme; Ancestor enum; TraverseCtx namespacing; ancestry stack (retag_stack, PopToken, NonEmptyStack); walk.rs raw pointers; scope tracking      | OXC Traverse Research                              | Full   |
| 8   | Semantic Analysis                      | Scoping unified structure (SoA layout); SymbolFlags/ReferenceFlags/ScopeFlags; reference read/write tracking; root unresolved references; scope tree operations      | OXC Semantic Research                              | Full   |
| 9   | Control Flow Graph (oxc_cfg)           | BasicBlock/Instruction/EdgeType; reachability/infinite loop detection; backedge vs unreachable; DOT visualization; filtered traversal                                | OXC CFG Research                                   | Full   |
| 10  | ECMAScript Specification Helpers       | ToBoolean/ToNumber/ToJsString; ConstantEvaluation (side-free variants); MayHaveSideEffects (pure functions list); string methods; ValueType detection                | OXC ECMAScript Research                            | Full   |
| 11  | Syntax Types                           | All operator enums (Binary, Unary, Logical, Assignment, Update); Precedence; Scope/Symbol/ReferenceFlags bitflags; identifier validation; reserved keywords          | OXC Syntax Research                                | Full   |
| 12  | Code Generation                        | Codegen struct/options; Gen vs GenExpr; Context flags; quote selection; comment handling; soft/hard spacing; sourcemaps                                              | OXC Codegen Research                               | Full   |
| 13  | Built-in Transformers                  | Single-pass design; TransformCtx stores (VarDeclarations, StatementInjector); duplicate_expression; stack patterns; reference flag management; common anti-patterns  | OXC Transformer Research                           | Full   |
| 14  | AST Visiting vs Traversing             | Visit vs VisitMut vs Traverse comparison; when to use each; symbol/ancestor access differences                                                                       | OXC AST Visit Research                             | Full   |
| 15  | Common Patterns and Anti-Patterns      | Safe temp var creation; expression duplication; reference splitting; deferred insertion; stack-based nested state; direct statement modification avoidance           | Synthesized from Transformer + Traverse + Semantic | Full   |
| 16  | Quick Reference Tables                 | Operator tables; flag meanings; precedence; common helpers; symbol/reference resolution snippets                                                                     | All research                                       | Full   |
| A   | Appendix A: Generated Code Insights    | How ancestor.rs, walk.rs are generated; offset constants; raw pointer patterns                                                                                       | Traverse notes                                     | Medium |
| B   | Appendix B: Performance Considerations | Allocator recycling; retag_stack optimization; NonEmptyStack; cache-friendly SoA layout                                                                              | Allocator + Traverse + Semantic                    | Medium |

---

## 1. Introduction and Explicit Assumptions

### 1.1 Purpose and Scope

This document constitutes the **canonical, formal knowledge framework** for understanding and utilizing the OXC (Oxford Compiler Collection) crates in the context of high-accuracy JavaScript and TypeScript parsing, analysis, and transformation—specifically targeted at **deobfuscation** tasks.

It consolidates exhaustive research conducted on December 26–27, 2025, across all major OXC crates, preserving every critical detail, safety invariant, performance consideration, and implementation pattern necessary for building robust, correct, and efficient deobfuscation tools in Rust.

The document is structured hierarchically:

- **Tier 1**: Foundational concepts that everything else depends on
- **Tier 2**: Core systems (AST, parsing, traversal, semantics)
- **Tier 3**: Advanced analysis tools
- **Tier 4**: Transformation and code generation
- **Tier 5**: Practical patterns and reference

All sections are traceable: low-level details reference higher-level concepts, and cross-references are explicit.

### 1.2 Version Context

This reference reflects the state of OXC as researched on **December 26, 2025**. Key crate versions and characteristics at that time:

- All crates use **arena allocation** via `oxc_allocator` (bumpalo-based)
- AST nodes are **boxed enum variants** with `inherit_variants!` macro
- Traversal uses the **"Without" type safety scheme** (generated)
- Semantic analysis produces a unified `Scoping` structure
- No `Drop` impls allowed in arena-allocated types (compile-time enforced)
- Generated code in `*/generated/` directories (do not edit manually)

Future changes to OXC may invalidate specific implementation details, but the **architectural principles** (arena safety, Without types, SoA layouts, etc.) are expected to remain stable.

### 1.3 Explicit Assumptions

The following assumptions are **fundamental** and must be understood before proceeding. Violating them will lead to undefined behavior, memory safety violations, or incorrect transformations.

#### Assumption 1: Arena Allocation and Ownership

- All AST nodes are allocated in an **`oxc_allocator::Allocator`** arena.
- AST nodes are **never** `Drop`ped individually—memory is freed only when the entire `Allocator` is dropped or reset.
- No types with `Drop` impls may be placed in the arena (enforced at compile time).
- **Consequence**: Custom data stored alongside AST nodes must not contain `Drop` types unless manually managed.

#### Assumption 2: CloneIn and TakeIn Preserve Semantic IDs

- Both `clone_in(&allocator)` and `take_in(&allocator)` **preserve** `SymbolId`, `ReferenceId`, and `ScopeId` values.
- This is critical for maintaining semantic correctness across transformations.
- Standard `.clone()` on AST nodes is **forbidden**—it does not allocate in the arena and breaks lifetimes.

#### Assumption 3: The "Without" Type Safety Scheme

- The traversal system guarantees safe simultaneous access to current node (`&mut`) and ancestors (`&`) via generated "Without" types.
- It is **impossible** (at compile time) to access the child field from which you descended in a parent node.
- This eliminates aliasing violations without runtime overhead or reference counting.

#### Assumption 4: Scoping is Unified and Shared

- `Scoping` is a single structure containing symbol table, reference table, and scope tree.
- It uses **Structure of Arrays (SoA)** layout for cache efficiency.
- SymbolId/ReferenceId/ScopeId are `NonMaxU32` indices—valid IDs are never `u32::MAX`.
- `Scoping` is `Send + Sync` but mutation requires exclusive access.

#### Assumption 5: Generated Code is Authoritative

- Files in `*/generated/` (e.g., `ancestor.rs`, `walk.rs`, `ast_builder.rs`) are auto-generated.
- Manual edits will be overwritten.
- Understanding their patterns (raw pointers, offset constants, retag_stack) is essential for advanced usage.

#### Assumption 6: Parser Produces Structurally Valid AST

- Even with syntax errors, the AST is structurally valid (no dangling pointers).
- `panicked == true` → AST is empty.
- Full validation requires semantic analysis with syntax error checking enabled.

#### Assumption 7: Deobfuscation Requires Semantic Awareness

- Simple string manipulation or regex-based tools are insufficient.
- Correct deobfuscation demands:
  - Symbol resolution (to inline constants safely)
  - Reference tracking (read/write flags)
  - Side-effect analysis
  - Control flow understanding
- OXC provides all necessary infrastructure for this.

### 1.4 Goals for Deobfuscation Tools

Tools built on this reference should aim for:

- **Correctness**: Preserve program semantics exactly
- **Safety**: No memory unsafety, no invalid AST states
- **Efficiency**: Leverage OXC's zero-cost abstractions and cache-friendly layouts
- **Convergence**: Multi-pass transformations until fixed point
- **Extensibility**: Modular passes with clear data dependencies

### 1.5 Document Conventions

- **Cross-references**: Section numbers (e.g., → §4) link to related content
- **Critical rules**: Highlighted as **MUST**, **MUST NOT**, **SHOULD**
- **Code examples**: Minimal, focused on invariants
- **Diagrams**: Mermaid where helpful

---

## 2. Span, Atom, and CompactStr

These three types form the **foundational layer** for source code representation in OXC. They are used ubiquitously across parsing, AST construction, semantic analysis, and code generation.

→ **Dependencies**: None (Tier 1)  
→ **Used by**: All other crates (§5–§16)

### 2.1 Span

A `Span` represents a byte range in the original source text. It is the primary mechanism for source location tracking, diagnostics, and source text extraction.

#### 2.1.1 Structure

```rust
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Span {
    pub start: u32,  // Zero-based inclusive start offset (bytes)
    pub end:   u32,  // Zero-based exclusive end offset (bytes)
}
```

- Maximum file size: 4 GiB (u32 limit)
- Size: 8 bytes (64-bit aligned)

#### 2.1.2 Constants

```rust
pub const SPAN: Span = Span::new(0, 0);
```

- **MUST** use `SPAN` for all generated AST nodes (not present in source).
- `span.is_unspanned()` returns `true` iff `span == SPAN`.

#### 2.1.3 Creation

```rust
Span::new(start, end)                    // General case
Span::sized(start, size)                  // Convenience (end = start + size)
Span::empty(at)                           // Zero-length span at position
SPAN                                      // Generated nodes
```

#### 2.1.4 Key Methods

| Method                                             | Purpose                                   | Critical Notes                        |
| -------------------------------------------------- | ----------------------------------------- | ------------------------------------- |
| `size()`                                           | `end - start`                             |                                       |
| `is_empty()`                                       | `start == end`                            |                                       |
| `is_unspanned()`                                   | `self == SPAN`                            | Identifies generated nodes            |
| `contains_inclusive(other)`                        | Containment test                          |                                       |
| `merge(other)`                                     | Union of two spans                        | Returns smallest span containing both |
| `expand(n)` / `expand_left(n)` / `expand_right(n)` | Grow span symmetrically or asymmetrically |                                       |
| `shrink(n)` / `shrink_left(n)` / `shrink_right(n)` | Reduce span                               |                                       |
| `move_left(n)` / `move_right(n)`                   | Shift span without changing size          |                                       |
| `source_text(&self, source: &str) -> &str`         | Extract substring                         | Equivalent to `&source[self]`         |
| Indexing: `&source[span]`                          | Direct extraction                         | Panics on invalid range               |

#### 2.1.5 Traits

```rust
pub trait GetSpan { fn span(&self) -> Span; }
pub trait GetSpanMut { fn span_mut(&mut self) -> &mut Span; }
```

- Implemented for all AST nodes.
- Use `GetSpan` for immutable access, `GetSpanMut` only when modifying location (rare).

#### 2.1.6 Diagnostic Usage

```rust
span.label("message")           // LabeledSpan for error reporting
span.primary_label("primary")   // Primary error location
span.primary()                  // Shorthand for primary
```

### 2.2 Atom<'a>

`Atom<'a>` is an **arena-allocated interned string**. It provides O(1) equality and hashing while borrowing from the allocator's lifetime.

#### 2.2.1 Purpose

- Primary string type for identifiers, property names, and literal values in AST.
- Guarantees uniqueness: identical content → identical `Atom`.
- Avoids repeated string allocation and comparison costs.

#### 2.2.2 Creation

```rust
Atom::from("string")                              // Borrows existing &str
Atom::from_in("string", &allocator)               // Allocates in arena
Atom::from_strs_array_in(["a", "b"], &allocator)  // Concatenation in arena
Atom::from_cow_in(&cow, &allocator)
format_atom!(&allocator, "prefix_{}", var)        // fmt-like macro
Atom::new_const("literal")                        // Compile-time static
Atom::empty()                                     // Empty atom
```

#### 2.2.3 Key Properties

- Implements `Deref<Target = str>`
- Conversions: `&str`, `String`, `CompactStr`, `Cow<'a, str>`
- **Lifetime**: Tied to the allocator (`'a`)

#### 2.2.4 Usage in AST

- `IdentifierReference.name`, `BindingIdentifier.name`, `StringLiteral.value`: all `Atom<'a>`
- **MUST** create via allocator when constructing new identifiers/literals.

### 2.3 CompactStr

`CompactStr` is an **owned, lifetimeless string** with small-string optimization.

#### 2.3.1 Structure

- Inline storage for strings ≤ 16 bytes (including null terminator)
- Heap allocation for longer strings
- Size: 24 bytes on 64-bit (similar to `String`)

#### 2.3.2 Creation

```rust
CompactStr::new("string")                         // Inline if ≤16 bytes
CompactStr::from(string)                          // Consumes String
format_compact_str!("prefix_{}", var)             // fmt-like macro
const SHORT: CompactStr = CompactStr::new_const("short"); // ≤16 bytes
```

#### 2.3.3 Key Properties

- No lifetime parameter → can be stored independently
- Implements `Deref<Target = str>`, `From<String>`, `Into<String>`
- Used for owned names when lifetime decoupling is needed

#### 2.3.4 Conversion from Atom

```rust
atom.into_compact_str()     // Preferred
atom.to_compact_str()       // Clones if necessary
```

### 2.4 ContentEq Trait

```rust
pub trait ContentEq {
    fn content_eq(&self, other: &Self) -> bool;
}
```

- Compares AST nodes **ignoring spans and metadata**.
- Critical for detecting semantically identical subtrees.

#### 2.4.1 Special Cases

- `f64`: Bitwise comparison (`to_bits()`)
- `NaN.content_eq(NaN) == true` (unlike `PartialEq`)
- `0.0.content_eq(-0.0) == false` (unlike `PartialEq`)
- Implemented for all arena types (`Box<'a, T>`, `Vec<'a, T>` where `T: ContentEq`)

#### 2.4.2 Usage

```rust
expr1.content_eq(&expr2)  // True if structurally identical (ignoring spans)
```

### 2.5 Critical Rules

**MUST**:

- Use `SPAN` for all generated nodes.
- Create identifiers/literals via allocator (`Atom::from_in`, `ctx.ast.*`).
- Use `ContentEq` for semantic comparison, never `PartialEq` on AST nodes with different spans.

**MUST NOT**:

- Use `String` directly in AST (use `Atom` or `CompactStr`).
- Compare AST nodes with `==` expecting span ignorance.

---

## 3. SourceType and Language Configuration

`SourceType` is the central configuration type that determines how OXC parses and interprets a source file. It encapsulates three orthogonal dimensions: **language**, **module kind**, and **language variant**.

→ **Dependencies**: §2 (Span for source text extraction)  
→ **Used by**: Parser (§6), Semantic Analysis (§8), Transformers (§13)

### 3.1 Structure

```rust
pub struct SourceType {
    language: Language,
    module_kind: ModuleKind,
    variant: LanguageVariant,
}
```

### 3.2 Language Enum

```rust
pub enum Language {
    JavaScript = 0,             // .js, .mjs, .cjs, .jsx
    TypeScript = 1,             // .ts, .mts, .cts, .tsx
    TypeScriptDefinition = 2,   // .d.ts, .d.mts, .d.cts
}
```

- **MUST** use `TypeScriptDefinition` only for declaration files.
- Influences type-related syntax acceptance and ambient context (§11 ScopeFlags).

### 3.3 ModuleKind Enum

```rust
pub enum ModuleKind {
    Script = 0,         // CommonJS or vanilla script (no import/export at top level)
    Module = 1,         // ESM (import/export allowed, strict mode always)
    Unambiguous = 2,    // Input only — parser detects from syntax
}
```

- **Modules are always strict mode** (`ScopeFlags::StrictMode`).
- `Unambiguous` is **only for parser input** — output is always `Script` or `Module`.

### 3.4 LanguageVariant Enum

```rust
pub enum LanguageVariant {
    Standard = 0,       // No JSX
    Jsx = 1,            // JSX or TSX enabled
}
```

### 3.5 Valid File Extensions

```rust
pub const VALID_EXTENSIONS: &[&str] = &[
    "js", "mjs", "cjs", "jsx",
    "ts", "mts", "cts", "tsx",
    "d.ts", "d.mts", "d.cts",
];
```

- Detection is case-insensitive.

### 3.6 Creation and Detection

#### 3.6.1 Constructors

```rust
SourceType::cjs()       // JavaScript + Script
SourceType::mjs()       // JavaScript + Module
SourceType::jsx()       // JavaScript + Module + Jsx
SourceType::ts()        // TypeScript + Module
SourceType::tsx()       // TypeScript + Module + Jsx
SourceType::d_ts()      // TypeScriptDefinition + Script
SourceType::unambiguous() // Detect module kind from syntax
```

#### 3.6.2 From Path

```rust
SourceType::from_path("file.tsx")?   // Returns Ok(SourceType::tsx())
```

- Falls back to `.js` if unknown extension.
- Errors on completely invalid extensions.

#### 3.6.3 Builder Methods

```rust
source_type
    .with_typescript(true)           // Change language
    .with_module(true)               // Force Module
    .with_jsx(true)                  // Enable JSX
    .with_script(true)               // Force Script
```

### 3.7 Query Methods

| Method                       | Returns | Notes                     |
| ---------------------------- | ------- | ------------------------- |
| `is_javascript()`            | bool    |                           |
| `is_typescript()`            | bool    | Excludes definition files |
| `is_typescript_definition()` | bool    |                           |
| `is_script()`                | bool    |                           |
| `is_module()`                | bool    |                           |
| `is_jsx()`                   | bool    |                           |
| `is_strict()`                | bool    | Always true for modules   |

### 3.8 Parser Behavior Implications

| Configuration        | Effect on Parsing                                               |
| -------------------- | --------------------------------------------------------------- |
| TypeScript enabled   | Allows `interface`, `type`, `declare`, etc.                     |
| TypeScriptDefinition | Ambient context (`ScopeFlags::Ambient`)                         |
| JSX enabled          | Allows `<Component />` syntax                                   |
| Module kind          | Top-level `import`/`export` allowed only in Module              |
| Unambiguous input    | Parser sets final ModuleKind based on presence of import/export |

### 3.9 Critical Rules

**MUST**:

- Use `SourceType::from_path(path)` for real files.
- Use `SourceType::unambiguous()` when module kind is unknown (e.g., REPL).
- Preserve `SourceType` through pipeline for correct codegen and semantics.

**MUST NOT**:

- Assume JavaScript for `.ts` files.
- Enable JSX without checking file extension or explicit flag.

**SHOULD**:

- Store original `SourceType` in transformation context for diagnostics.

### 3.10 Example Usage

```rust
let source_type = SourceType::from_path("component.tsx")?; // tsx()
assert!(source_type.is_typescript());
assert!(source_type.is_module());
assert!(source_type.is_jsx());

// Manual override
let custom = SourceType::mjs().with_jsx(true); // ESM JavaScript with JSX
```

---

## 4. Memory Model and Allocator

The `oxc_allocator` crate implements OXC's **arena-based memory model**. This is the **most foundational system** in OXC—**everything else depends on it**.

→ **Dependencies**: None (Tier 1)  
→ **Used by**: AST (§5), Parser (§6), Traversal (§7), Semantic (§8), Codegen (§12)

### 4.1 Core Principles

OXC uses a **bump allocator** (`bumpalo::Bump` wrapper) for all AST nodes and related data.

| Standard Rust Allocation           | OXC Arena Allocation                                 |
| ---------------------------------- | ---------------------------------------------------- |
| Individual allocation/deallocation | Bulk allocation, bulk free on drop/reset             |
| `Drop` runs per object             | **No `Drop` runs** (compile-time forbidden in arena) |
| Many system calls, fragmentation   | Few system calls, contiguous memory                  |
| Reference counting or GC possible  | Pure ownership via lifetimes                         |

**Critical Consequences**:

- **MUST NOT** place types with `Drop` impls in the arena.
- Memory is only freed when `Allocator` is dropped or `reset()`.
- All AST nodes live exactly as long as the `Allocator`.

### 4.2 Allocator Structure

```rust
#[derive(Default)]
pub struct Allocator {
    bump: Bump,
}
```

- Wraps `bumpalo::Bump` with exponential chunk growth (1st chunk small, subsequent ~2× previous).

#### 4.2.1 Creation and Lifecycle

```rust
Allocator::default()                     // Lazy, allocates on first use
Allocator::with_capacity(1 << 20)        // Pre-allocate 1 MiB
allocator.reset()                        // Discard all data, reuse memory
drop(allocator)                          // Free all memory
```

**Performance Rule**:

- **MUST** recycle allocators across files:

```rust
let mut allocator = Allocator::new();
for file in files {
    process(&allocator, file);
    allocator.reset();  // Critical for performance
}
```

- Creating new `Allocator` per file is expensive.
- Never resetting causes unbounded growth.

#### 4.2.2 Query Methods

```rust
allocator.capacity()      // Total owned memory
allocator.used_bytes()    // Currently allocated
```

### 4.3 Arena Types

All arena-allocated equivalents:

| Type                    | Purpose                       | Key Methods                                             |
| ----------------------- | ----------------------------- | ------------------------------------------------------- |
| `Box<'alloc, T>`        | Single value                  | `Box::new_in(value, &allocator)`                        |
| `Vec<'alloc, T>`        | Growable array                | `Vec::new_in()`, `with_capacity_in()`, `from_iter_in()` |
| `HashMap<'alloc, K, V>` | Hash map (FxHasher)           | `HashMap::new_in()`, `with_capacity_in()`               |
| `HashSet<'alloc, T>`    | Hash set                      | Similar to HashMap                                      |
| `StringBuilder<'alloc>` | Efficient string construction | `push_str()`, `into_str()`                              |

**Thread Safety**:

- `Vec`, `Box`, etc. are `Sync` if contents are `Sync`.
- **NOT `Send`** — multiple collections could allocate concurrently into same arena.

### 4.4 CloneIn and TakeIn Traits

**Critical**: Both preserve `SymbolId`, `ReferenceId`, `ScopeId`.

#### 4.4.1 CloneIn

```rust
pub trait CloneIn<'new_alloc> {
    type Cloned;
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned;
}
```

- Creates deep copy in target allocator.
- Used when storing expressions for later use.

#### 4.4.2 TakeIn and Dummy

```rust
pub trait TakeIn<'a>: Dummy<'a> {
    fn take_in<A: AllocatorAccessor<'a>>(&mut self, allocator: A) -> Self;
}

pub trait Dummy<'a> {
    fn dummy(allocator: &'a Allocator) -> Self;
}
```

- Moves value out, leaves dummy placeholder.
- Dummy values:
  - `Expression` → `NullLiteral`
  - `Statement` → `EmptyStatement`
  - `Vec<T>` → empty vec
- **MUST** replace or remove dummy after `take_in`.

#### 4.4.3 Decision Matrix

| Scenario                      | Use              | Reason                       |
| ----------------------------- | ---------------- | ---------------------------- |
| Need original + copy          | `clone_in`       | Original remains intact      |
| Moving to new location        | `take_in`        | More efficient, leaves dummy |
| Rebuilding vector (filtering) | `take_in` on vec | Full control                 |
| In-place replacement          | Direct `*`       | Simple                       |

### 4.5 FromIn / IntoIn Traits

```rust
pub trait FromIn<'a, T> { fn from_in(value: T, allocator: &'a Allocator) -> Self; }
pub trait IntoIn<'a, T> { fn into_in(self, allocator: &'a Allocator) -> T; }
```

Examples:

- `String` → `&'a str`
- `T` → `Box<'a, T>`

### 4.6 Address System

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Address(usize);
```

- Stable for `Box<T>` (never moves)
- Unstable for `Vec<T>` items (may move on push)

Traits:

- `GetAddress` — stable address
- `UnstableAddress` — may change

### 4.7 AllocatorPool

Thread-safe pool for reusing allocators:

```rust
let pool = AllocatorPool::new(num_threads);
let guard = pool.get();  // Returns AllocatorGuard
// Use &*guard
drop(guard);  // Returns to pool, reset automatically
```

### 4.8 Critical Rules

**MUST**:

- Recycle allocators with `reset()`.
- Use `clone_in` / `take_in` for AST nodes.
- Use `SPAN` and allocator when creating nodes.

**MUST NOT**:

- Use standard `.clone()` on AST nodes.
- Place `Drop` types in arena.
- Assume `Vec<T>` item addresses are stable after mutation.

**Performance Notes**:

- Recycling → warm cache, fewer syscalls
- `retag_stack` (§7) relies on arena stability

---

## 5. AST Structure and Design Principles

The `oxc_ast` crate defines the Abstract Syntax Tree (AST) nodes used throughout OXC. The design is highly optimized for performance, memory efficiency, and transformation safety.

→ **Dependencies**: §2 (Span, Atom), §4 (Allocator)  
→ **Used by**: Parser (§6), Traversal (§7), Semantic (§8), Codegen (§12), Transformers (§13)

### 5.1 Core Design Principles

1. **Arena Allocation**  
   All AST nodes are allocated in the arena (`Box<'a, T>`, `Vec<'a, T>`). No individual `Drop`.

2. **Boxed Enum Variants**  
   Large enum variants are boxed to keep enum size small and enable pointer stability.

3. **Enum Inheritance via Macro**  
   `inherit_variants!` macro flattens nested enums (e.g., MemberExpression variants appear directly in Expression).

4. **Explicit Discriminants**  
   `#[repr(C, u8)]` on enums for zero-cost conversions and generated code.

5. **Semantic IDs in Cell**  
   `SymbolId`, `ReferenceId`, `ScopeId` stored in `Cell<Option<...>>` — populated post-parsing.

6. **Type-Safe Identifiers**  
   Four distinct identifier types (see §5.5).

7. **Auto-Generated Code**  
   `AstBuilder`, matching macros, and trait impls are generated.

### 5.2 Expression Enum (~40 variants)

```rust
#[repr(C, u8)]
pub enum Expression<'a> {
    BooleanLiteral(Box<'a, BooleanLiteral>) = 0,
    NullLiteral(Box<'a, NullLiteral>) = 1,
    NumericLiteral(Box<'a, NumericLiteral<'a>>) = 2,
    BigIntLiteral(Box<'a, BigIntLiteral<'a>>) = 3,
    RegExpLiteral(Box<'a, RegExpLiteral<'a>>) = 4,
    StringLiteral(Box<'a, StringLiteral<'a>>) = 5,
    TemplateLiteral(Box<'a, TemplateLiteral<'a>>) = 6,

    Identifier(Box<'a, IdentifierReference<'a>>) = 10,

    ThisExpression(Box<'a, ThisExpression>) = 15,
    Super(Box<'a, Super>) = 16,
    MetaProperty(Box<'a, MetaProperty<'a>>) = 17,

    ArrayExpression(Box<'a, ArrayExpression<'a>>) = 20,
    ObjectExpression(Box<'a, ObjectExpression<'a>>) = 21,

    ArrowFunctionExpression(Box<'a, ArrowFunctionExpression<'a>>) = 25,
    FunctionExpression(Box<'a, Function<'a>>) = 26,
    ClassExpression(Box<'a, Class<'a>>) = 27,

    UnaryExpression(Box<'a, UnaryExpression<'a>>) = 30,
    UpdateExpression(Box<'a, UpdateExpression<'a>>) = 31,
    BinaryExpression(Box<'a, BinaryExpression<'a>>) = 32,
    LogicalExpression(Box<'a, LogicalExpression<'a>>) = 33,
    ConditionalExpression(Box<'a, ConditionalExpression<'a>>) = 34,
    AssignmentExpression(Box<'a, AssignmentExpression<'a>>) = 35,
    SequenceExpression(Box<'a, SequenceExpression<'a>>) = 36,

    CallExpression(Box<'a, CallExpression<'a>>) = 40,
    NewExpression(Box<'a, NewExpression<'a>>) = 41,
    ImportExpression(Box<'a, ImportExpression<'a>>) = 42,

    // Member expressions (inherited)
    ComputedMemberExpression(Box<'a, ComputedMemberExpression<'a>>) = 48,
    StaticMemberExpression(Box<'a, StaticMemberExpression<'a>>) = 49,
    PrivateFieldExpression(Box<'a, PrivateFieldExpression<'a>>) = 50,

    AwaitExpression(Box<'a, AwaitExpression<'a>>) = 55,
    YieldExpression(Box<'a, YieldExpression<'a>>) = 56,
    ChainExpression(Box<'a, ChainExpression<'a>>) = 57,
    ParenthesizedExpression(Box<'a, ParenthesizedExpression<'a>>) = 58,
    TaggedTemplateExpression(Box<'a, TaggedTemplateExpression<'a>>) = 59,

    // JSX and TypeScript variants...
}
```

- **MUST** match using generated macros (§5.8) for exhaustiveness.

### 5.3 Statement Enum (~30 variants)

```rust
#[repr(C, u8)]
pub enum Statement<'a> {
    BlockStatement(Box<'a, BlockStatement<'a>>) = 0,
    BreakStatement(Box<'a, BreakStatement<'a>>) = 1,
    ContinueStatement(Box<'a, ContinueStatement<'a>>) = 2,
    DebuggerStatement(Box<'a, DebuggerStatement>) = 3,
    EmptyStatement(Box<'a, EmptyStatement>) = 4,

    ExpressionStatement(Box<'a, ExpressionStatement<'a>>) = 10,
    ReturnStatement(Box<'a, ReturnStatement<'a>>) = 11,
    ThrowStatement(Box<'a, ThrowStatement<'a>>) = 12,

    IfStatement(Box<'a, IfStatement<'a>>) = 15,
    SwitchStatement(Box<'a, SwitchStatement<'a>>) = 16,
    LabeledStatement(Box<'a, LabeledStatement<'a>>) = 17,
    TryStatement(Box<'a, TryStatement<'a>>) = 18,
    WithStatement(Box<'a, WithStatement<'a>>) = 19,

    // Loops
    DoWhileStatement(Box<'a, DoWhileStatement<'a>>) = 25,
    WhileStatement(Box<'a, WhileStatement<'a>>) = 26,
    ForStatement(Box<'a, ForStatement<'a>>) = 27,
    ForInStatement(Box<'a, ForInStatement<'a>>) = 28,
    ForOfStatement(Box<'a, ForOfStatement<'a>>) = 29,

    // Declarations (inherited)
    VariableDeclaration(Box<'a, VariableDeclaration<'a>>) = 35,
    FunctionDeclaration(Box<'a, Function<'a>>) = 36,
    ClassDeclaration(Box<'a, Class<'a>>) = 37,

    // Module declarations (inherited)
    ImportDeclaration(Box<'a, ImportDeclaration<'a>>) = 40,
    ExportNamedDeclaration(Box<'a, ExportNamedDeclaration<'a>>) = 41,
    ExportDefaultDeclaration(Box<'a, ExportDefaultDeclaration<'a>>) = 42,
    ExportAllDeclaration(Box<'a, ExportAllDeclaration<'a>>) = 43,

    // TypeScript declarations...
}
```

### 5.4 Key Struct Types

#### 5.4.1 Member Expressions (3 variants)

- `ComputedMemberExpression` — `obj[expr]`
- `StaticMemberExpression` — `obj.prop`
- `PrivateFieldExpression` — `obj.#private`

#### 5.4.2 VariableDeclaration

```rust
pub struct VariableDeclaration<'a> {
    pub span: Span,
    pub kind: VariableDeclarationKind, // Var, Let, Const, Using
    pub declarations: Vec<'a, VariableDeclarator<'a>>,
    pub declare: bool, // TypeScript 'declare'
}
```

#### 5.4.3 Function

```rust
pub struct Function<'a> {
    pub span: Span,
    pub id: Option<BindingIdentifier<'a>>,
    pub generator: bool,
    pub r#async: bool,
    pub declare: bool,
    pub params: Box<'a, FormalParameters<'a>>,
    pub body: Option<Box<'a, FunctionBody<'a>>>,
    pub scope_id: Cell<Option<ScopeId>>,
    pub pure: bool, // @__NO_SIDE_EFFECTS__
}
```

### 5.5 Type-Safe Identifiers (Critical)

Four distinct types — **MUST NOT** mix:

| Type                      | Purpose                     | Symbol/Ref Field                          |
| ------------------------- | --------------------------- | ----------------------------------------- |
| `IdentifierName<'a>`      | Property names (not scoped) | None                                      |
| `IdentifierReference<'a>` | Variable reads/writes       | `reference_id: Cell<Option<ReferenceId>>` |
| `BindingIdentifier<'a>`   | Variable declarations       | `symbol_id: Cell<Option<SymbolId>>`       |
| `LabelIdentifier<'a>`     | Loop/block labels           | None                                      |

### 5.6 Literal Types

- `StringLiteral<'a>` — `value: Atom<'a>`, `lone_surrogates: bool`
- `NumericLiteral<'a>` — `value: f64`, `raw: Option<Atom<'a>>`, `base: NumberBase`
- `BigIntLiteral<'a>` — `base: BigintBase`

### 5.7 Enum Inheritance

`inherit_variants!` macro example:

```rust
inherit_variants! {
    pub enum Expression<'a> {
        // ... direct variants
        @inherit MemberExpression<'a>  // Adds 48–50
        @inherit JSXExpression<'a>
        @inherit TypeScriptExpression<'a>
    }
}
```

Result: `Expression::ComputedMemberExpression` exists directly.

Conversion methods:

- `.to_*()`, `.as_*()`, `.is_*()`

### 5.8 AstBuilder (Generated)

```rust
pub struct AstBuilder<'a> { allocator: &'a Allocator }
```

- Methods: `expression_string_literal()`, `alloc_expression_*()`, `vec()`, `vec1()`, etc.
- **MUST** use `ctx.ast` in traversal for new nodes.

### 5.9 Matching Macros (Generated)

```rust
match expr {
    match_expression!(Expression) => { /* all variants */ }
    match_member_expression!(Expression) => { /* 3 member variants */ }
    match_declaration!(Statement) => { /* declaration statements */ }
}
```

### 5.10 Critical Rules

**MUST**:

- Use `ctx.ast` to create nodes.
- Use matching macros for exhaustive matching.
- Use correct identifier type (Reference vs Binding).

**MUST NOT**:

- Match `Expression::MemberExpression` (doesn't exist).
- Use standard allocation for AST nodes.

---

## 6. Parser Architecture

The `oxc_parser` crate implements a **recursive descent parser** for JavaScript, TypeScript, JSX, and related extensions. It produces a structurally valid AST even in the presence of syntax errors.

→ **Dependencies**: §2 (Span, Atom), §3 (SourceType), §4 (Allocator), §5 (AST)  
→ **Used by**: Semantic (§8), Transformers (§13)

### 6.1 High-Level Flow

```
Source Text → Lexer → Tokens → ParserImpl → AST (Program<'a>)
                        ↑
                  Context Flags
                        ↑
                  ParseOptions
```

- **Lexer** tokenizes with trivia (comments, whitespace).
- **ParserImpl** consumes tokens recursively.
- **AstBuilder** allocates nodes in arena.

### 6.2 Public API

```rust
pub struct Parser<'a> {
    allocator: &'a Allocator,
    source: &'a str,
    source_type: SourceType,
    options: ParseOptions,
}
```

#### 6.2.1 Parsing

```rust
let ret = Parser::new(&allocator, source, source_type)
    .with_options(options)
    .parse();  // Returns ParserReturn<'a>
```

#### 6.2.2 Expression Only

```rust
let expr = Parser::new(...).parse_expression()?;  // Ok(Expression<'a>) or Err
```

### 6.3 ParserReturn

```rust
pub struct ParserReturn<'a> {
    pub program: Program<'a>,
    pub module_record: ModuleRecord<'a>,
    pub errors: Vec<OxcDiagnostic>,
    pub irregular_whitespaces: Box<[Span]>,
    pub panicked: bool,
    pub is_flow_language: bool,
}
```

- **AST always structurally valid**.
- `errors.non_empty()` → possible semantic issues.
- `panicked == true` → AST empty (unrecoverable).

### 6.4 ParseOptions

```rust
pub struct ParseOptions {
    pub parse_regular_expression: bool,     // Feature flag
    pub allow_return_outside_function: bool,
    pub preserve_parens: bool,              // Emit ParenthesizedExpression
    pub allow_v8_intrinsics: bool,          // %DebugPrint()
}
```

### 6.5 Lexer

```rust
pub struct Lexer<'a> {
    source: Source<'a>,
    token: Token,
    trivia_builder: TriviaBuilder,
    // ...
}
```

#### 6.5.1 Token (128-bit compact)

```rust
pub struct Token(u128);  // Bits: start/end/kind/flags
```

Methods: `kind()`, `span()`, `is_on_new_line()`, `escaped()`, etc.

#### 6.5.2 Kind Enum (~150 variants)

Categories: Keywords, Punctuators, Literals, Contextual, TypeScript, JSX.

### 6.6 ParserImpl Internals

Recursive descent with:

- **Context flags** (§11 ScopeFlags mirrored as parse context)
- **Checkpoint/rewind** for backtracking
- **Automatic Semicolon Insertion (ASI)**

#### 6.6.1 Navigation

```rust
self.cur_kind() / self.cur_token()
self.at(Kind::LParen)
self.bump_any() / self.eat(Kind::Comma) / self.expect(Kind::RBrace)
```

#### 6.6.2 List Parsing

```rust
self.parse_delimited_list(close, separator, opening_span, parser_fn)
self.parse_normal_list(open, close, parser_fn)
```

#### 6.6.3 Context Modification

```rust
self.context_add(flags, |p| { ... })
self.context_remove(flags, |p| { ... })
self.context(new_flags, removed_flags, |p| { ... })
```

### 6.7 Error Handling

- **Recoverable**: Add to `errors`, continue
- **Fatal**: Set `fatal_error`, stop
- Common diagnostics: unexpected_token, expect_closing, ASI

**AST remains valid** even with errors.

### 6.8 Module Record

Collected during parsing:

- Import/Export entries
- Local export entries

Used by semantic analysis for module resolution.

### 6.9 Comments and Trivia

- Stored on `Program.comments`
- Kind: Line or Block
- Attached via position for codegen (§12)

### 6.10 Critical Rules

**MUST**:

- Use `SourceType::from_path()` for files.
- Check `ret.errors` and `ret.panicked`.
- Use `preserve_parens: true` if transformations need exact structure.

**MUST NOT**:

- Assume error-free input → always check `errors`.

**Performance Notes**:

- Single pass tokenization + parsing.
- Trivia builder avoids allocation until needed.

---

## 7. Traversal System (oxc_traverse)

The `oxc_traverse` crate provides OXC's **core AST traversal mechanism**. It is the primary interface for all analysis and transformation passes, enabling safe simultaneous access to the current node (`&mut`) and ancestors (`&`) without violating Rust's aliasing rules.

→ **Dependencies**: §4 (Allocator), §5 (AST), §8 (Semantic for Scoping)  
→ **Used by**: Semantic (§8), Transformers (§13), Codegen (§12)

### 7.1 The Aliasing Problem and OXC's Solution

Standard Rust borrowing prevents holding `&mut` to a child and `&` to its parent simultaneously. Traditional solutions (cloning, Rc<RefCell>) incur overhead.

OXC's solution: **generated "Without" types** that statically prevent access to the child field from which descent occurred.

### 7.2 Traverse Trait

```rust
pub trait Traverse<'a, State> {
    fn enter_node(&mut self, kind: AstKind<'a>) {}
    fn leave_node(&mut self, kind: AstKind<'a>) {}
    fn enter_scope(&mut self, flags: ScopeFlags, scope_id: &Cell<Option<ScopeId>>) {}
    fn leave_scope(&mut self) {}

    // Per-node hooks (enter_*/exit_*)
    fn enter_expression(&mut self, node: &mut Expression<'a>, ctx: &mut TraverseCtx<'a, State>) {}
    fn exit_expression(&mut self, node: &mut Expression<'a>, ctx: &mut TraverseCtx<'a, State>) {}
    // ... hundreds more
}
```

- **enter\_\*** called top-down (before children)
- **exit\_\*** called bottom-up (after children)
- Ideal: collect in `enter_*`, transform in `exit_*`

### 7.3 TraverseCtx

```rust
pub struct TraverseCtx<'a, State> {
    pub state: State,
    pub ancestry: TraverseAncestry<'a>,
    pub scoping: TraverseScoping<'a>,
    pub ast: AstBuilder<'a>,
}
```

- `ast`: node factory (§5.8)
- `scoping`: symbol/scope operations (§8)
- `ancestry`: parent stack

Namespaced access (`ctx.ancestry.parent()`) allows simultaneous borrowing.

### 7.4 TraverseAncestry — The Safety Stack

```rust
pub struct TraverseAncestry<'a> {
    stack: NonEmptyStack<Ancestor<'a, 'static>>,
}
```

- **Never empty** (bottom is `Ancestor::None`)
- Private constructor — cannot create externally

#### 7.4.1 Ancestor Enum (Generated)

```rust
#[repr(C, u16)]
pub enum Ancestor<'a, 't> {
    None = 0,
    BinaryExpressionLeft(BinaryExpressionWithoutLeft<'a, 't>) = ...,
    BinaryExpressionRight(BinaryExpressionWithoutRight<'a, 't>) = ...,
    // ~300 variants
}
```

- Variant encodes **parent type** and **child direction**.
- `'t` bound to method call — cannot escape.

#### 7.4.2 Without Types (Generated)

```rust
pub struct BinaryExpressionWithoutLeft<'a, 't> {
    ptr: *const BinaryExpression<'a>,
    _marker: PhantomData<&'t ()>,
}

impl BinaryExpressionWithoutLeft<'a, 't> {
    pub fn operator(&self) -> &'t BinaryOperator { ... }
    pub fn right(&self) -> &'t Expression<'a> { ... }
    // NO left() method!
}
```

- Raw pointer + PhantomData for lifetime.
- Missing methods prevent access to descent path.

#### 7.4.3 Stack Operations

```rust
ctx.parent() -> Ancestor<'a, 't>                 // Most recent
ctx.ancestor(level)                              // 0 = parent
ctx.ancestors() -> impl Iterator                 // Parent → root
ctx.ancestors_depth() -> usize
```

### 7.5 Generated walk.rs

**Raw pointers** in walk functions avoid reference creation:

```rust
unsafe fn walk_binary_expression(...) {
    traverser.enter_binary_expression(&mut *node, ctx);

    let pop_token = ctx.push_stack(Ancestor::BinaryExpressionLeft(...));
    walk_expression(traverser, &mut (*node).left as *mut _, ctx);

    ctx.retag_stack(AncestorType::BinaryExpressionRight);  // Fast sibling switch
    walk_expression(traverser, &mut (*node).right as *mut _, ctx);

    ctx.pop_stack(pop_token);
    traverser.exit_binary_expression(&mut *node, ctx);
}
```

### 7.6 TraverseScoping

Integrated scope tracking:

- `current_scope_id()`, `current_hoist_scope_id()`, `current_block_scope_id()`
- `create_child_scope()`, `insert_scope_below_*()`
- UID generation: `generate_uid_in_current_hoist_scope()`

### 7.7 Performance Optimizations

1. **retag_stack**  
   Change discriminant only between siblings (same pointer).

2. **PopToken**  
   Zero-sized token ensures push/pop balance at compile time.

3. **NonEmptyStack**  
   Allows unchecked operations.

4. **Offset Constants**  
   Pre-computed field offsets for direct access.

### 7.8 Critical Rules

**MUST**:

- Use `ctx.ast` for new nodes.
- Use `ctx.parent()` / `ctx.ancestors()` for ancestor access.
- Implement `exit_*` for transformations (children processed first).

**MUST NOT**:

- Hold ancestors across visitor calls.
- Manually manage stack.

**Usage Pattern**:

```rust
match ctx.parent() {
    Ancestor::CallExpressionCallee(call) => { /* callee-specific */ }
    Ancestor::BinaryExpressionLeft(bin) => { bin.right() /* sibling */ }
}
```

---

## 8. Semantic Analysis (Symbols, Scopes, References)

The `oxc_semantic` crate performs **symbol resolution**, **scope analysis**, and **reference tracking** after parsing. It produces a unified `Scoping` structure that is the **primary data source for safe transformations**.

→ **Dependencies**: §5 (AST), §7 (Traversal)  
→ **Used by**: Transformers (§13), CFG (§9), ECMAScript helpers (§10)

### 8.1 Architecture Overview

```
Parser → AST → SemanticBuilder → Scoping
                    ↑
          traverse_mut (Binder passes)
```

- Multiple binder passes populate symbols/references.
- Final `Scoping` is immutable (`Send + Sync`).

### 8.2 Core ID Types (`oxc_syntax`)

All are `NonMaxU32` (never `u32::MAX`):

- `SymbolId` — declaration
- `ReferenceId` — usage
- `ScopeId` — lexical scope
- `NodeId` — AST node

### 8.3 Scoping Structure

Unified **Structure of Arrays (SoA)** layout for cache efficiency:

```rust
pub struct Scoping {
    // Symbol table
    symbol_spans: IndexVec<SymbolId, Span>,
    symbol_flags: IndexVec<SymbolId, SymbolFlags>,
    symbol_scope_ids: IndexVec<SymbolId, ScopeId>,
    symbol_declarations: IndexVec<SymbolId, NodeId>,

    // References
    references: IndexVec<ReferenceId, Reference>,

    // Scope tree
    scope_parent_ids: IndexVec<ScopeId, Option<ScopeId>>,
    scope_node_ids: IndexVec<ScopeId, NodeId>,
    scope_flags: IndexVec<ScopeId, ScopeFlags>,

    // Arena-allocated (in ScopingCell)
    symbol_names: ArenaVec<'a, Atom<'a>>,
    resolved_references: ArenaVec<'a, ArenaVec<'a, ReferenceId>>,
    bindings: IndexVec<ScopeId, HashMap<&'a str, SymbolId>>,
    // ...
}
```

- **SoA** → fast iteration over single fields.
- Arena for variable-sized data (names, reference lists).

### 8.4 SymbolFlags

```rust
bitflags! { pub struct SymbolFlags: u32 { ... } }
```

Key categories:

- Variables: `FunctionScopedVariable` (var/param), `BlockScopedVariable` (let/const), `ConstVariable`
- Declarations: `Function`, `Class`, `Import`
- TypeScript: `TypeAlias`, `Interface`, `Enum`
- Composite: `Variable`, `Value`, `Type`

Methods: `is_const_variable()`, `is_function()`, `is_import()`, etc.

### 8.5 Reference System

```rust
pub struct Reference {
    node_id: NodeId,
    symbol_id: Option<SymbolId>,  // None = unresolved (global)
    flags: ReferenceFlags,
}
```

#### 8.5.1 ReferenceFlags

```rust
bitflags! { pub struct ReferenceFlags: u8 { Read, Write, Type, ValueAsType } }
```

Critical queries:

- `is_read()`, `is_write()`, `is_read_only()`, `is_read_write()`

#### 8.5.2 Resolution

- Bound → `symbol_id.is_some()`
- Unbound → globals (`root_unresolved_references()`)

### 8.6 Scope Tree

- Root: program/module scope
- Children: functions, blocks, catch clauses, etc.

Key flags (`ScopeFlags`):

- `Var` — where `var` hoists to
- `Function`, `Arrow`, `StrictMode`

### 8.7 Key Operations (via TraverseCtx.scoping)

#### 8.7.1 Symbol Queries

```rust
scoping.symbol_name(id)
scoping.symbol_flags(id)
scoping.symbol_is_mutated(id)     // Any write reference?
scoping.symbol_is_unused(id)      // No read references?
scoping.get_resolved_references(id) -> impl Iterator<Reference>
```

#### 8.7.2 Reference Queries

```rust
scoping.get_reference(id) -> &Reference
scoping.has_binding(ref_id) -> bool
scoping.root_unresolved_references() -> HashMap<&str, Vec<ReferenceId>>
```

#### 8.7.3 Scope Queries

```rust
scoping.find_binding(start_scope, "name") -> Option<SymbolId>
scoping.scope_ancestors(scope_id) -> impl Iterator<ScopeId>
scoping.scope_has_binding(scope_id, "name")
```

#### 8.7.4 Mutation (scoping_mut)

- Create/delete symbols/references/scopes
- Rename, move bindings
- **MUST** maintain consistency (update references when moving)

### 8.8 Critical Rules for Transformers

**MUST**:

- Use `symbol_is_mutated()` before inlining (no writes → safe).
- Update references when duplicating identifiers.
- Rebuild scoping after structural changes affecting declarations.

**MUST NOT**:

- Assume resolved references for globals.
- Access stale scoping after mutations without rebuild.

**Safety Pattern**:

```rust
if scoping.symbol_flags(id).is_const_variable() ||
   !scoping.get_resolved_references(id).any(|r| r.is_write())
{
    // Safe to inline
}
```

---

## 9. Control Flow Graph (oxc_cfg)

The `oxc_cfg` crate constructs and analyzes a **Control Flow Graph (CFG)** from the AST. It is essential for reachability analysis, dead code detection, loop identification, and advanced deobfuscation techniques.

→ **Dependencies**: §5 (AST), §7 (Traversal), §8 (Semantic)  
→ **Used by**: Advanced deobfuscation passes (infinite loop detection, unreachable code elimination)

### 9.1 Overview

Built on `petgraph::DiGraph` with custom node/edge types.

```
AST → ControlFlowGraphBuilder → ControlFlowGraph
```

- **Basic blocks**: Sequences of instructions with single entry/exit
- **Edges**: Control flow transitions (normal, jump, backedge, error, etc.)

### 9.2 Core Types

#### 9.2.1 IDs

```rust
pub type BlockNodeId = petgraph::stable_graph::NodeIndex;
pub struct BasicBlockId(NonMaxU32);
pub type Graph = DiGraph<BasicBlockId, EdgeType>;
```

#### 9.2.2 ControlFlowGraph

```rust
pub struct ControlFlowGraph {
    pub graph: Graph,
    pub basic_blocks: IndexVec<BasicBlockId, BasicBlock>,
}
```

### 9.3 BasicBlock and Instructions

```rust
pub struct BasicBlock {
    instructions: Vec<Instruction>,
    unreachable: bool,
}
```

#### 9.3.1 InstructionKind

```rust
pub enum InstructionKind {
    Unreachable,
    Statement,
    ImplicitReturn,
    Return(ReturnInstructionKind),
    Break(LabeledInstruction),
    Continue(LabeledInstruction),
    Throw,
    Condition,
    Iteration(IterationInstructionKind),
}
```

### 9.4 EdgeType

```rust
pub enum EdgeType {
    Jump,           // Conditional true branch
    Normal,         // Fall-through
    Backedge,       // Loop backward edge
    NewFunction,    // Nested function entry
    Finalize,       // finally block entry
    Error(ErrorEdgeKind),
    Unreachable,    // Dead path
    Join,           // After finalizer
}
```

- `Backedge` identifies loop headers.
- `Unreachable` marks dead code.

### 9.5 ControlFlowGraphBuilder

Manages construction with stacks for break/continue/finally/error contexts.

Key methods:

- `new_basic_block_*()`
- `add_edge(from, to, EdgeType)`
- `push_statement(node_id)`
- `append_condition_to(block)`
- `attach_error_harness()`

### 9.6 Reachability and Loop Analysis

#### 9.6.1 Reachability

```rust
cfg.is_reachable(from, to) -> bool
cfg.is_reachable_filtered(from, to, filter) -> bool
```

- Default filters out `NewFunction` and `Unreachable`.

#### 9.6.2 Dead Code

```rust
cfg.basic_block(node).is_unreachable()
```

#### 9.6.3 Loop Detection

```rust
cfg.is_cyclic(node) -> bool  // Any cycle
cfg.is_infinite_loop_start(node, eval_condition) -> Option<(start, end)>
```

- Requires condition evaluator callback.

#### 9.6.4 Traversal Helpers

- `set_depth_first_search` with `Control::Continue/Prune/Break`
- `neighbors_filtered_by_edge_weight`

### 9.7 DOT Visualization

```rust
let dot = cfg.display_dot();  // String for Graphviz
```

- Colors: green entry, dotted unreachable, red error

### 9.8 Critical Rules for Deobfuscation

**MUST**:

- Use `is_unreachable()` for dead code elimination.
- Use `is_infinite_loop_start()` with constant evaluator (§10) for infinite loop flattening.
- Filter `NewFunction` edges when analyzing top-level flow.

**Usage Patterns**:

1. **Dead Code**:

```rust
cfg.graph.node_indices()
    .filter(|&n| cfg.basic_block(n).is_unreachable())
```

2. **Single Successor** (linear flow):

```rust
let successors = cfg.graph
    .edges_directed(node, Outgoing)
    .filter(|e| !matches!(e.weight(), Unreachable | NewFunction))
    .count() == 1;
```

3. **Loop Headers**:

```rust
incoming_edges.any(|e| matches!(e.weight(), Backedge))
```

---

## 10. ECMAScript Specification Helpers (oxc_ecmascript)

The `oxc_ecmascript` crate implements **ECMAScript abstract operations** in Rust. It provides the foundation for **static constant evaluation**, **side-effect analysis**, **type coercion**, and **value type detection**—critical for high-accuracy deobfuscation.

→ **Dependencies**: §5 (AST), §8 (Semantic)  
→ **Used by**: Constant folding, dead code elimination, control flow simplification

### 10.1 Core Traits

All traits operate on `Expression<'a>` and require a `GlobalContext<'a>` for global reference information.

#### 10.1.1 Type Coercion

| Trait                  | Method                         | Returns                | Key Conversions                                        |
| ---------------------- | ------------------------------ | ---------------------- | ------------------------------------------------------ |
| `ToBoolean`            | `to_boolean()`                 | `Option<bool>`         | falsey: `false`, `0`, `""`, `null`, `undefined`, `NaN` |
| `ToNumber`             | `to_number()`                  | `Option<f64>`          | `true`→1, `""`→0, `[]`→0, `[5]`→5, `[1,2]`→NaN         |
| `ToJsString`           | `to_js_string()`               | `Option<Cow<'a, str>>` | `null`→"null", arrays joined with ","                  |
| `ToInt32` / `ToUint32` | `to_int_32()` / `to_uint_32()` | `i32` / `u32`          | Bitwise ops, `fromCharCode`                            |

#### 10.1.2 Constant Evaluation

```rust
pub trait ConstantEvaluation<'a> {
    fn evaluate_value(&self, ctx: &impl ConstantEvaluationCtx<'a>) -> Option<ConstantValue<'a>>;
    fn get_side_free_number_value(&self, ...) -> Option<f64>;
    // ... boolean, string, bigint variants
}
```

`ConstantValue<'a>` enum:

- `Number(f64)`, `String(Cow<'a, str>)`, `Boolean(bool)`, `Null`, `Undefined`, `BigInt`

**Side-effect-free variants** return `None` if expression has side effects.

#### 10.1.3 Side Effect Analysis

```rust
pub trait MayHaveSideEffects<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool;
}
```

- **No side effects**: literals, `this`, pure calls
- **Potential side effects**: assignments, calls, property access (configurable)

Known pure functions (always safe):

- Math.\*, String.fromCharCode, Array.isArray, etc.
- Constructors without `new` in some cases

#### 10.1.4 Value Type Detection

```rust
pub enum ValueType { Undefined, Null, Number, BigInt, String, Boolean, Object, Undetermined }

pub trait DetermineValueType<'a> {
    fn value_type(&self, ctx: &impl GlobalContext<'a>) -> ValueType;
}
```

### 10.2 Supported Evaluations

| Category | Examples                                              |
| -------- | ----------------------------------------------------- | --- | ----------------------------- |
| Literals | All direct literals                                   |
| Unary    | `typeof`, `void`, `!`, `+`, `-`, `~`                  |
| Binary   | Arithmetic, bitwise, comparison, `+` concatenation    |
| Logical  | `&&`, `                                               |     | `, `??` (short-circuit aware) |
| Member   | `.length` on strings/arrays                           |
| Calls    | Math.\*, parseInt, String methods, known pure globals |

### 10.3 GlobalContext Trait

```rust
pub trait GlobalContext<'a> {
    fn is_global_reference(&self, reference: &IdentifierReference<'a>) -> bool;
    fn get_constant_value_for_reference_id(&self, id: ReferenceId) -> Option<ConstantValue<'a>>;
}
```

Default: `WithoutGlobalReferenceInformation` (nothing global).

### 10.4 Critical Rules for Deobfuscation

**MUST**:

- Use `get_side_free_*` variants before folding (ensures no side effects).
- Provide accurate `GlobalContext` for globals like `Math`, `parseInt`.
- Combine with reference tracking (§8) for safe inlining.

**Usage Patterns**:

1. **Constant Folding**:

```rust
if expr.may_have_side_effects(ctx) { return None; }
expr.evaluate_value(ctx)
```

2. **Control Flow Simplification**:

```rust
expr.to_boolean(ctx) == Some(true)  // Always truthy → remove branch
```

3. **Dead Expression Removal**:

```rust
!expr.may_have_side_effects(ctx)  // Safe to drop
```

---

## 11. Syntax Types (Operators, Flags, Precedence)

The `oxc_syntax` crate defines **core syntactic elements** shared across OXC: operators, scope/symbol/reference flags, precedence, and identifier utilities. These are foundational for correct parsing, transformation, and code generation.

→ **Dependencies**: None (used throughout)  
→ **Used by**: Parser (§6), Codegen (§12), Transformers (§13), ECMAScript helpers (§10)

### 11.1 Operators

All operators are exhaustive enums with source string and precedence methods.

#### 11.1.1 BinaryOperator

```rust
pub enum BinaryOperator {
    Equality, Inequality, StrictEquality, StrictInequality,
    LessThan, LessEqualThan, GreaterThan, GreaterEqualThan,
    Addition, Subtraction, Multiplication, Division, Remainder, Exponential,
    ShiftLeft, ShiftRight, ShiftRightZeroFill,
    BitwiseOR, BitwiseXOR, BitwiseAnd,
    In, Instanceof,
}
```

Key methods:

- `is_equality()`, `is_compare()`, `is_arithmetic()`, `is_bitwise()`, `is_relational()`
- `compare_inverse_operator()`, `equality_inverse_operator()`
- `to_assignment_operator()`
- `as_str()`, `precedence()`

#### 11.1.2 UnaryOperator

```rust
pub enum UnaryOperator {
    UnaryPlus, UnaryNegation, LogicalNot, BitwiseNot,
    Typeof, Void, Delete,
}
```

- `is_keyword()` for `typeof`, `void`, `delete`

#### 11.1.3 LogicalOperator

```rust
pub enum LogicalOperator { Or, And, Coalesce }
```

- `to_assignment_operator()` for `||=`, `&&=`, `??=`

#### 11.1.4 AssignmentOperator

Includes compound: `+=`, `*=` , `||=`, etc.

#### 11.1.5 UpdateOperator

```rust
pub enum UpdateOperator { Increment, Decrement }
```

### 11.2 Precedence

```rust
#[repr(u8)]
pub enum Precedence {
    Lowest = 0, Comma = 1, ... , Member = 22,
}
```

- Right-associative: `**`, `=`, `?:`
- Used by codegen (§12) for parenthesization.

Trait: `GetPrecedence` on expressions/operators.

### 11.3 ScopeFlags

```rust
bitflags! { pub struct ScopeFlags: u16 { ... } }
```

Key:

- `StrictMode`, `Top`, `Function`, `Arrow`
- `Var` = `Top | Function | ClassStaticBlock | TsModuleBlock` (var hoisting targets)

Methods: `is_var()`, `is_function()`, `is_strict_mode()`

### 11.4 SymbolFlags

```rust
bitflags! { pub struct SymbolFlags: u32 { ... } }
```

Key categories:

- Variables: `FunctionScopedVariable`, `BlockScopedVariable`, `ConstVariable`
- Declarations: `Class`, `Function`, `Import`
- TypeScript: `TypeAlias`, `Interface`, `Enum`
- Composite: `Variable`, `Value`, `Type`

Methods: `is_const_variable()`, `is_function()`, `is_value()`, `is_type()`

### 11.5 ReferenceFlags

```rust
bitflags! { pub struct ReferenceFlags: u8 { Read, Write, Type, ValueAsType } }
```

Critical:

- `is_read()`, `is_write()`, `is_read_only()`, `is_read_write()`

### 11.6 Identifier Utilities

```rust
is_identifier_name(&str) -> bool
is_identifier_start(char) -> bool
is_identifier_part(char) -> bool
is_reserved_keyword(&str) -> bool
is_global_object(&str) -> bool  // Infinity, NaN, undefined, globalThis
```

- Handles Unicode and ZWNJ/ZWJ.

### 11.7 Other Types

- `NumberBase`, `BigintBase`
- `ESTarget` (ES2015–ESNext)
- `NodeFlags` (JSDoc, HasYield, etc.)

### 11.8 Critical Rules for Deobfuscation

**MUST**:

- Use `ReferenceFlags` to detect writes before inlining.
- Use `SymbolFlags` to distinguish `const` vs `let` vs `var`.
- Use `ScopeFlags::is_var()` for hoisting boundaries.
- Use `Precedence` for correct parenthesization in codegen.

**Usage Patterns**:

1. **Safe Inlining**:

```rust
!references.iter().any(|r| r.flags().is_write())
```

2. **Inverse Comparison**:

```rust
op.compare_inverse_operator()  // < → >
```

---

## 12. Code Generation (oxc_codegen)

The `oxc_codegen` crate converts an AST back to JavaScript/TypeScript source code. It supports pretty-printing, minification, source maps, and comment preservation.

→ **Dependencies**: §5 (AST), §7 (Traversal)  
→ **Used by**: Final output of transformers (§13)

### 12.1 Core Architecture

```rust
pub struct Codegen<'a> {
    options: CodegenOptions,
    scoping: Option<Scoping>,      // For identifier mangling
    code: CodeBuffer,
    comments: CommentsMap,
    sourcemap_builder: Option<SourcemapBuilder<'a>>,
    // State: indent, quote, needs_semicolon, etc.
}
```

### 12.2 CodegenOptions

```rust
pub struct CodegenOptions {
    pub minify: bool,
    pub single_quote: bool,
    pub comments: CommentOptions,
    pub source_map_path: Option<PathBuf>,
    pub indent_char: IndentChar,   // Space or Tab
    pub indent_width: usize,
    pub initial_indent: u32,
}
```

#### 12.2.1 CommentOptions

- Preserve: normal, jsdoc, annotation (`@__PURE__`), legal
- Legal handling: Inline, Eof, Linked, External

Presets:

- `CodegenOptions::default()` — pretty with comments
- `CodegenOptions::minify()` — no whitespace/comments

### 12.3 Gen and GenExpr Traits

- **Gen**: Statements/declarations — `gen(&self, p: &mut Codegen, ctx: Context)`
- **GenExpr**: Expressions — `gen_expr(&self, p: &mut Codegen, precedence: Precedence, ctx: Context)`

**Precedence** drives parenthesization via `p.wrap()`.

### 12.4 Context Flags

```rust
bitflags! { pub struct Context: u8 { FORBID_IN, FORBID_CALL, TYPESCRIPT } }
```

- `FORBID_IN`: for-loop init
- `FORBID_CALL`: `new` callee
- `TYPESCRIPT`: enable TS syntax

### 12.5 String Handling

- **Quote selection** (minify): chooses shortest (`'`, `"`, `` ` ``)
- **Escapes**: `\0`, control chars, lone surrogates (`\u{FFFD}XXXX`), `<\/script`
- **Backticks**: no `$` escape needed except before `{`

### 12.6 Spacing and Formatting

- **Soft** space/newline: omitted in minify
- **Hard** space/newline: always printed
- **Semicolon handling**: `needs_semicolon` flag, ASI aware

### 12.7 Comment Handling

- `CommentsMap`: position → Vec<Comment>
- Methods: `print_leading_comments(span.start)`, `has_comment(pos)`
- Legal comments: `/*!`, `//!`, `/* @license */`

### 12.8 Source Maps

Enabled via `source_map_path`. Mappings added with:

- `add_source_mapping(span)`
- `add_source_mapping_for_name(span, name)`

### 12.9 Identifier Mangling

Requires `with_scoping(Some(scoping))` — uses semantic names.

### 12.10 Critical Rules

**MUST**:

- Use `Codegen::new().build(&program)` for final output.
- Preserve `__PURE__` annotations for side-effect analysis.

**Performance Notes**:

- Single pass over AST.
- Quote selection only in minify.

**Usage Patterns**:

1. **Minified**:

```rust
Codegen::new()
    .with_options(CodegenOptions::minify())
    .build(&program)
    .code
```

2. **With Mangling**:

```rust
Codegen::new()
    .with_scoping(Some(scoping))
    .build(&program)
    .code
```

---

## 13. Built-in Transformers (oxc_transformer)

The `oxc_transformer` crate implements OXC's **built-in AST transformations** (ES2015–ES2026, TypeScript, JSX, etc.). It uses a **single-pass, multi-transformer** architecture with shared state and deferred operations.

→ **Dependencies**: §7 (Traversal), §8 (Semantic), §12 (Codegen)  
→ **Used by**: Deobfuscation pipelines (as reference patterns)

### 13.1 Architecture Overview

- **Single traversal pass** over the AST.
- All transforms run in ordered phases (TypeScript → JSX → ES2026 → … → Common).
- Each transform implements `Traverse<'a, TransformState<'a>>`.

```mermaid
graph TD
    A[AST + Scoping] --> B[TransformerImpl]
    B --> C[TypeScript Transform]
    B --> D[JSX Transform]
    B --> E[ES2026..ES2015 Transforms]
    B --> F[Common Transform (last)]
    F --> G[Modified AST]
```

### 13.2 TransformCtx — Shared State

```rust
pub struct TransformCtx<'a> {
    // File info
    filename: String,
    source_type: SourceType,
    source_text: &'a str,

    // Stores for deferred operations
    var_declarations: VarDeclarationsStore<'a>,
    statement_injector: StatementInjectorStore<'a>,
    top_level_statements: TopLevelStatementsStore<'a>,
    helper_loader: HelperLoaderStore<'a>,
    module_imports: ModuleImportsStore<'a>,
}
```

- **Deferred insertion**: No direct statement modification during traversal.
- `Common` transform applies stores in `exit_statements`.

### 13.3 TransformState

Currently minimal marker type (PhantomData).

### 13.4 Key Shared Utilities

#### 13.4.1 VarDeclarationsStore

- Creates temp vars (`_temp`, `_temp2`).
- Auto-inserts `var`/`let` at block top.
- Methods: `create_uid_var()`, `create_uid_var_with_init()`, `create_uid_var_based_on_node()`

#### 13.4.2 StatementInjectorStore

- Insert before/after specific statements (by address).
- `move_insertions(old, new)` when replacing nodes.

#### 13.4.3 duplicate_expression

```rust
ctx.duplicate_expression(expr, mutated_symbol_needs_temp_var, ctx)
    -> (first_use, subsequent_use)
```

- Handles side effects: `foo()` → `(_t = foo(), _t)`
- Identifier cases: bound → reuse, unbound → temp assignment

#### 13.4.4 AST Builder Helpers

- `create_member_callee()`, `create_bind_call()`
- `wrap_expression_in_arrow_function_iife()`
- `wrap_statements_in_arrow_function_iife()`

### 13.5 Transform Patterns

#### 13.5.1 Simple: Logical Assignment

- `a ||= b` → `a || (a = b)`
- Uses `duplicate_expression` for member targets.

#### 13.5.2 Complex: Nullish Coalescing

- `a ?? b` → `a !== null && a !== void 0 ? a : b`
- Temp var only if side effects or mutated symbol.

#### 13.5.3 Stateful: Arrow Function Converter

- Stacks for `this`, `arguments`, super methods.
- Inserts declarations at function top on exit.

#### 13.5.4 Recursive: Optional Chaining

- Depth-first chain transformation.
- Converts `.?` to temp var checks.

### 13.6 Critical Rules

**MUST**:

- Use stores for insertions.
- Use `duplicate_expression` for potentially effectful expressions.
- Update reference flags when splitting read/write.
- Adjust scopes when moving bindings.

**MUST NOT**:

- Insert statements directly in `enter/exit`.
- Forget `move_insertions` when replacing nodes.

---

## 14. AST Visiting vs Traversing

OXC provides **two distinct traversal systems**: `oxc_ast_visit` (`Visit`/`VisitMut`) and `oxc_traverse` (`Traverse`). Choosing the correct one is critical for deobfuscation accuracy.

→ **Dependencies**: §5 (AST), §7 (Traversal)  
→ **Used by**: All analysis/transformation passes

### 14.1 Overview Comparison

| Feature                  | Visit (immutable)        | VisitMut (mutable)      | Traverse (oxc_traverse)                       |
| ------------------------ | ------------------------ | ----------------------- | --------------------------------------------- |
| Mutability               | Read-only (`&T`)         | Mutable (`&mut T`)      | Mutable (`&mut T`)                            |
| Context                  | None                     | None                    | `TraverseCtx` (state, ancestry, scoping, ast) |
| Ancestor Access          | No                       | No                      | Yes (§7.4)                                    |
| Symbol/Scoping Access    | No                       | No                      | Yes (§8)                                      |
| Scope Tracking           | Basic enter/leave hooks  | Basic enter/leave hooks | Full `TraverseScoping`                        |
| Safety for Parent Access | N/A                      | N/A                     | Compile-time "Without" types                  |
| Performance              | Fastest                  | Fast                    | Slightly slower (stack management)            |
| Generated Code           | walk.rs                  | walk_mut.rs             | walk.rs + ancestor.rs                         |
| Use Case                 | Simple analysis, linting | Simple mutations        | Semantic-aware transformations                |

### 14.2 Visit / VisitMut Traits

```rust
pub trait Visit<'a> {
    fn visit_expression(&mut self, it: &Expression<'a>) {
        walk::walk_expression(self, it);
    }
}

pub trait VisitMut<'a> {
    fn visit_expression(&mut self, it: &mut Expression<'a>) {
        walk_mut::walk_expression(self, it);
    }
}
```

- **Recursion**: Handled by generated `walk_*` functions.
- **Hooks**: `visit_*` methods, `enter_node`/`leave_node`.
- **No context** — cannot access parent, symbols, or create nodes.

Usage:

```rust
let mut visitor = MyVisitor;
visitor.visit_program(&mut program);  // VisitMut
```

### 14.3 Traverse Trait (§7)

- Full `TraverseCtx` with ancestry stack, scoping, AST builder.
- `enter_*`/`exit_*` per node type.
- Safe parent/sibling access via "Without" types.

### 14.4 Critical Decision Guide

**Use Traverse when**:

- Need symbol resolution (§8)
- Need reference read/write tracking
- Need ancestor context (parent/sibling data)
- Creating/modifying symbols or scopes
- High-accuracy deobfuscation (constant inlining, dead code, control flow)

**Use VisitMut only when**:

- Simple syntactic mutations (no semantic info needed)
- Span updates, comment stripping
- Performance-critical read/write without context

**Use Visit only when**:

- Read-only analysis (counting, collecting)
- Linting rules without mutation

**For Deobfuscation**: **ALWAYS use Traverse**.

- Visit/VisitMut lack symbol resolution and ancestor access.
- Cannot safely inline variables without write detection.
- Cannot handle nested scopes correctly.

### 14.5 Migration Considerations

If starting from a `VisitMut` pass:

- Replace with `Traverse` impl.
- Use `ctx.scoping()` for symbol queries.
- Use `ctx.parent()` for context.
- Use `ctx.ast` for new nodes.

---

## 15. Common Patterns and Anti-Patterns

This section distills **best-practice patterns** and **critical anti-patterns** observed across OXC's own transformers (§13) and derived from traversal (§7), semantic (§8), and allocator (§4) invariants. These are essential for writing correct, safe, and efficient deobfuscation passes.

→ **Dependencies**: §7 (Traversal), §8 (Semantic), §13 (Transformers)  
→ **Used by**: All custom transformation passes

### 15.1 Core Principles

1. **Never mutate during traversal without safety guarantees** (§7 Without types).
2. **Deferred operations** over direct mutation.
3. **Side-effect awareness** for duplication.
4. **Reference flag hygiene** when splitting operations.
5. **Scope consistency** when moving bindings.

### 15.2 Recommended Patterns

#### 15.2.1 Deferred Statement Insertion

**Problem**: Direct insertion in `enter/exit` can invalidate iterators or break traversal order.

**Pattern**: Use shared stores (`VarDeclarationsStore`, `StatementInjectorStore`).

```rust
// Create temp var — declaration auto-inserted at block top
let binding = ctx.var_declarations.create_uid_var("temp", ctx);

// Insert after specific statement
ctx.statement_injector.insert_after(&target_stmt, new_stmt);

// When replacing a statement
ctx.statement_injector.move_insertions(&old_stmt, &new_stmt);
```

#### 15.2.2 Safe Expression Duplication

**Problem**: Naive cloning executes side effects multiple times.

**Pattern**: Use `TransformCtx::duplicate_expression`.

```rust
let (first_use, subsequent_use) = ctx.duplicate_expression(
    expr.clone_in(&allocator),
    true,
    ctx
);
// first_use may be assignment (_t = expr())
// subsequent_use is safe reference (_t)
```

Variants: `duplicate_expression_twice()`, `duplicate_expression_multiple()`.

#### 15.2.3 Reference Flag Management

**Problem**: Splitting read+write loses write information.

**Pattern**: Update flags explicitly.

```rust
// Original: read_write identifier
let ref_id = ident.reference_id();
let reference = ctx.scoping_mut().get_reference_mut(ref_id);
reference.flags_mut().remove(ReferenceFlags::Write);  // Now read-only

// Create new write reference if needed
let write_ident = ctx.create_ident_reference(span, name, symbol_id, ReferenceFlags::Write);
```

#### 15.2.4 Stack-Based Nested State

**Problem**: Arrow functions, classes create nested scopes with shared state (`this`, `arguments`, super).

**Pattern**: Push/pop stacks in `enter/exit_function`.

```rust
struct MyTransform<'a> {
    this_stack: Vec<Option<BoundIdentifier<'a>>>,
    // ...
}

fn enter_function(&mut self, func: &mut Function<'a>, ctx: &mut TraverseCtx<'a>) {
    self.this_stack.push(None);
}

fn exit_function(&mut self, func: &mut Function<'a>, ctx: &mut TraverseCtx<'a>) {
    let this_var = self.this_stack.pop();
    // Insert declarations using collected vars
}
```

#### 15.2.5 Temp Variable Creation

Options:

- `ctx.generate_uid_in_current_hoist_scope("prefix")` — manual scope
- `ctx.var_declarations.create_uid_var("prefix", ctx)` — auto-inserted
- `ctx.var_declarations.create_uid_var_based_on_node(&node, ctx)` — name from node

#### 15.2.6 Conditional Expression Construction

Common for nullish/optional transforms:

```rust
fn create_nullish_check(left: Expression<'a>, ctx: &TraverseCtx<'a>) -> Expression<'a> {
    let null_check = ctx.ast.expression_binary(
        SPAN, left.clone_in(ctx.ast.allocator),
        BinaryOperator::StrictInequality,
        ctx.ast.expression_null_literal(SPAN)
    );
    let undef_check = ctx.ast.expression_binary(
        SPAN, left,
        BinaryOperator::StrictInequality,
        ctx.ast.void_0(SPAN)
    );
    ctx.ast.expression_logical(SPAN, null_check, LogicalOperator::And, undef_check)
}
```

### 15.3 Anti-Patterns to Avoid

#### 15.3.1 Direct Statement List Mutation

```rust
// WRONG — can break traversal
fn exit_statements(&mut self, stmts: &mut ArenaVec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a>) {
    stmts.insert(0, new_stmt);  // Invalidates addresses, breaks injector
}
```

#### 15.3.2 Naive Expression Cloning

```rust
// WRONG — duplicates side effects
let copy = expr.clone_in(ctx.ast.allocator);
use_expr(&copy); use_expr(&copy);  // foo() called twice!
```

#### 15.3.3 Forgetting Reference Flag Updates

```rust
// WRONG — loses write information
*expr = read_expr;  // Original was read_write → now appears unread
```

#### 15.3.4 Manual Stack Management Without PopToken

```rust
// WRONG — unbalanced push/pop risks stack corruption
ctx.push_stack(ancestor);
// Forgot pop → leak
```

#### 15.3.5 Using VisitMut for Semantic Transforms

```rust
// WRONG — no symbol resolution
impl VisitMut<'a> for MyDeobfuscator { ... }  // Cannot safely inline
```

### 15.4 Summary Rules

**MUST**:

- Use deferred stores.
- Use `duplicate_expression` for effectful expressions.
- Update reference flags.
- Balance stack push/pop.

**MUST NOT**:

- Mutate statement lists directly.
- Clone expressions with side effects.
- Use Visit/VisitMut for semantic-aware transforms.

---

## 16. Quick Reference Tables

This section provides **concise lookup tables** for frequently used OXC syntax elements, flags, operators, and helpers.

### 16.1 Operator Tables

#### 16.1.1 BinaryOperator

| Operator   | String       | Precedence     | Category                 | Inverse |
| ---------- | ------------ | -------------- | ------------------------ | ------- |
| ==         | "=="         | Equals         | Equality                 | !=      |
| !=         | "!="         | Equals         | Equality                 | ==      |
| ===        | "==="        | Equals         | Equality                 | !==     |
| !==        | "!=="        | Equals         | Equality                 | ===     |
| <          | "<"          | Compare        | Compare                  | >       |
| <=         | "<="         | Compare        | Compare                  | >=      |
| >          | ">"          | Compare        | Compare                  | <       |
| >=         | ">="         | Compare        | Compare                  | <=      |
| +          | "+"          | Add            | Arithmetic               | -       |
| -          | "-"          | Add            | Arithmetic               | +       |
| \*         | "\*"         | Multiply       | Arithmetic               | /       |
| /          | "/"          | Multiply       | Arithmetic               | \*      |
| %          | "%"          | Multiply       | Arithmetic               |         |
| \*\*       | "\*\*"       | Exponentiation | Arithmetic (right-assoc) |         |
| <<         | "<<"         | Shift          | Bitshift                 |         |
| >>         | ">>"         | Shift          | Bitshift                 |         |
| >>>        | ">>>"        | Shift          | Bitshift                 |         |
| \|         | "\|"         | BitwiseOr      | Bitwise                  |         |
| ^          | "^"          | BitwiseXor     | Bitwise                  |         |
| &          | "&"          | BitwiseAnd     | Bitwise                  |         |
| in         | "in"         | Relational     | Relational (keyword)     |         |
| instanceof | "instanceof" | Relational     | Relational (keyword)     |         |

#### 16.1.2 UnaryOperator

| Operator | String   | Keyword? |
| -------- | -------- | -------- |
| +        | "+"      | No       |
| -        | "-"      | No       |
| !        | "!"      | No       |
| ~        | "~"      | No       |
| typeof   | "typeof" | Yes      |
| void     | "void"   | Yes      |
| delete   | "delete" | Yes      |

#### 16.1.3 LogicalOperator

| Operator | String | Assignment Variant |
| -------- | ------ | ------------------ |
| \|\|     | "\|\|" | \|\|=              |
| &&       | "&&"   | &&=                |
| ??       | "??"   | ??=                |

### 16.2 Flag Tables

#### 16.2.1 ScopeFlags (Key Composites)

| Flag             | Meaning           | Composite `Var`? |
| ---------------- | ----------------- | ---------------- |
| Top              | Program/module    | Yes              |
| Function         | Function body     | Yes              |
| Arrow            | Arrow function    | No               |
| ClassStaticBlock | class static {}   | Yes              |
| TsModuleBlock    | declare namespace | Yes              |
| StrictMode       | Always in modules | —                |

#### 16.2.2 SymbolFlags (Key Categories)

| Category      | Flags                                                                                    | `is_*()` Method                        |
| ------------- | ---------------------------------------------------------------------------------------- | -------------------------------------- |
| Variable      | FunctionScopedVariable, BlockScopedVariable, ConstVariable                               | is_variable(), is_const_variable()     |
| Declaration   | Function, Class, Import                                                                  | is_function(), is_class(), is_import() |
| TypeScript    | TypeAlias, Interface, Enum                                                               | is_type_alias(), is_interface()        |
| Value vs Type | Value = Variable\|Class\|Function\|Enum<br>Type = Interface\|TypeAlias\|Enum (non-value) | is_value(), is_type()                  |

#### 16.2.3 ReferenceFlags

| Flag  | Meaning              | Query Methods               |
| ----- | -------------------- | --------------------------- |
| Read  | Value read           | is_read(), is_read_only()   |
| Write | Value written        | is_write(), is_read_write() |
| Type  | Used in type context | is_type()                   |

### 16.3 Precedence Levels

| Level | Precedence     | Examples                         |
| ----- | -------------- | -------------------------------- |
| 22    | Member         | . [] #                           |
| 21    | Call           | () ?.()                          |
| 20    | New            | new (with args)                  |
| 18    | Prefix         | ++ -- ! ~ + - typeof void delete |
| 17    | Exponentiation | \*\* (right-assoc)               |
| 16    | Multiply       | \* / %                           |
| 15    | Add            | + -                              |
| 4     | Assign         | = += etc. (right-assoc)          |
| 5     | Conditional    | ?: (right-assoc)                 |
| 6     | Nullish        | ??                               |
| 7     | LogicalOr      | \|\|                             |
| 8     | LogicalAnd     | &&                               |

### 16.4 Common Helpers

| Helper                              | Location               | Purpose                      |
| ----------------------------------- | ---------------------- | ---------------------------- |
| `ctx.ast.*`                         | TraverseCtx            | Node creation                |
| `ctx.scoping().symbol_is_mutated()` | TraverseScoping        | Write detection              |
| `ctx.duplicate_expression()`        | TransformCtx           | Side-effect-safe duplication |
| `ctx.var_declarations.create_uid_*` | VarDeclarationsStore   | Temp var with auto-insert    |
| `ctx.statement_injector.insert_*`   | StatementInjectorStore | Deferred insertion           |
| `SPAN`                              | oxc_span               | Generated nodes              |
| `format_atom!(&allocator, ...)`     | oxc_span               | Atom formatting              |

### 16.5 Symbol Resolution Snippets

```rust
// Safe inline check
let safe = scoping.symbol_flags(id).is_const_variable() ||
    !scoping.get_resolved_references(id).any(|r| r.is_write());

// Global detection
scoping.root_unresolved_references().contains_key("console")
```

---

## Appendix A: Generated Code Insights

OXC relies heavily on **auto-generated code** for safety, performance, and maintainability. These files are in `*/generated/` directories and **MUST NOT** be edited manually.

→ **Related Sections**: §7 (Traversal), §5 (AST)

### A.1 Generation Process

Generated by `scripts/build.mjs` (Node.js script) from AST definitions.

Key outputs:

- `ancestor.rs` — Ancestor enum + Without types
- `walk.rs` — Traversal functions
- `ast_builder.rs` — Node factory methods
- Matching macros

### A.2 ancestor.rs

#### A.2.1 Ancestor Enum

```rust
#[repr(C, u16)]
pub enum Ancestor<'a, 't> { ... }  // ~300 variants
```

- Discriminant order matches `AncestorType` for `retag_stack`.
- Each variant holds a Without type.

#### A.2.2 Without Types

Example:

```rust
pub struct BinaryExpressionWithoutLeft<'a, 't> {
    ptr: *const BinaryExpression<'a>,
    _marker: PhantomData<&'t ()>,
}
```

- Raw pointer to parent.
- Methods for all fields **except** the descent child.
- Generated for every multi-child node.

### A.3 walk.rs

Raw pointer usage pattern:

```rust
unsafe fn walk_binary_expression(...) {
    // enter hook
    // push_stack with WithoutLeft
    walk_expression(..., &mut (*node).left as *mut _, ...);
    ctx.retag_stack(AncestorType::BinaryExpressionRight);
    walk_expression(..., &mut (*node).right as *mut _, ...);
    // pop_stack
    // exit hook
}
```

- **No references** created until hooks (safe).
- `retag_stack` for sibling transitions.

### A.4 Offset Constants

Pre-computed for direct field access:

```rust
pub const OFFSET_BINARY_EXPRESSION_LEFT: usize = offset_of!(BinaryExpression, left);
```

Used in Without impls.

### A.5 ast_builder.rs

Generated factory methods:

```rust
pub fn expression_binary(
    self,
    span: Span,
    left: Expression<'a>,
    operator: BinaryOperator,
    right: Expression<'a>,
) -> Expression<'a>
```

- All variants covered.

### A.6 Critical Notes

- **Never edit** generated files.
- Changes require AST definition updates + regeneration.
- Understanding patterns enables advanced debugging.

---

## Appendix B: Performance Considerations

OXC is designed for **high-throughput** JavaScript processing. The following optimizations are critical for maintaining performance in large-scale deobfuscation pipelines.

→ **Related Sections**: §4 (Allocator), §7 (Traversal), §8 (Semantic)

### B.1 Allocator Recycling

**Critical Rule**: **MUST** recycle `Allocator` instances.

```rust
let mut allocator = Allocator::new();
for file in files {
    let program = parse(&allocator, file);
    transform(&allocator, program);
    allocator.reset();  // Reuse chunks, warm cache
}
```

- New `Allocator` per file → many syscalls, cold cache.
- No reset → unbounded growth.
- After warmup, capacity stabilizes to workload size.

### B.2 retag_stack Optimization (§7)

Sibling transitions use discriminant rewrite:

- No pop + push → fewer stack operations.
- Enabled by `#[repr(C, u16)]` on `Ancestor`.
- Zero-cost for common cases (binary expressions, call arguments).

### B.3 NonEmptyStack and PopToken

- Stack always contains `Ancestor::None` at bottom → unchecked ops.
- `PopToken` ensures balance at compile time.

### B.4 Structure of Arrays (SoA) in Scoping

- Cache-friendly iteration over single fields (flags, spans).
- Avoids padding in struct-per-symbol layout.

### B.5 Raw Pointers in walk.rs

- No reference creation until hooks → no borrowing conflicts.
- Direct field access via offset constants.

### B.6 Arena Properties

- Contiguous memory → better locality.
- No individual `Drop` → fast teardown.

### B.7 Thread Safety Notes

- `Scoping` is `Send + Sync`.
- Mutation requires exclusive access.
- Read-only passes can parallelize safely.

### B.8 Practical Guidelines

1. **Process files sequentially** with one recycled allocator.
2. **Avoid unnecessary scoping rebuilds** (share when no structural changes).
3. **Minimize stack depth** in custom transforms (avoid deep recursion).
4. **Profile with large obfuscated inputs** (real-world worst case).
