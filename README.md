# JS-Deobfuscator

A high-performance JavaScript deobfuscator built on the [OXC](https://oxc.rs) Rust AST library.

## Features

- **3-Layer Architecture**: ECMA → Runtime → Extensions
- **V8 Sandbox**: Safe execution for string decoding
- **Convergence Loop**: Iterates until no more transformations
- **High Performance**: Built on OXC's fast parser

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
js-deobfuscator = "0.1.0"
```

## Quick Start

### Simple API (Recommended)

```rust
use js_deobfuscator::{JSDeobfuscator, Extension};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = std::fs::read_to_string("obfuscated.js")?;
    
    // Standard deobfuscation (ECMA + Runtime)
    let output = JSDeobfuscator::new()
        .deobfuscate(&source)?;
    
    // With string rotator extension
    let output = JSDeobfuscator::new()
        .ecma(true)
        .runtime(true)
        .extensions([Extension::StringRotator])
        .deobfuscate(&source)?;
    
    std::fs::write("clean.js", output)?;
    Ok(())
}
```

### One-liner Functions

```rust
use js_deobfuscator::{deobfuscate, deobfuscate_full};

// Standard (ECMA + Runtime)
let output = deobfuscate("var a = 1 + 2;")?;

// Full (all extensions enabled)
let output = deobfuscate_full(&obfuscated_source)?;
```

### With Result Details

```rust
use js_deobfuscator::{JSDeobfuscator, Extension};

let result = JSDeobfuscator::new()
    .ecma(true)
    .runtime(true)
    .extensions([Extension::StringRotator])
    .max_iterations(50)
    .deobfuscate_with_result(&source)?;

println!("Iterations: {}", result.iterations);
println!("Modifications: {}", result.modifications);
println!("Converged: {}", result.converged);
println!("{}", result.code);
```

### Low-Level Engine API

For advanced use cases with full control:

```rust
use js_deobfuscator::{Engine, EngineConfig};
use oxc::allocator::Allocator;
use oxc::codegen::Codegen;
use oxc::parser::Parser;
use oxc::span::SourceType;

let allocator = Allocator::default();
let mut program = Parser::new(&allocator, &source, SourceType::mjs()).parse().program;

let config = EngineConfig::full().with_max_iterations(50);
let engine = Engine::with_config(config);
let result = engine.run(&allocator, &mut program)?;

let output = Codegen::new().build(&program).code;
```

## Configuration

### Layers

| Layer | Description | Default |
|-------|-------------|---------|
| `ecma(true)` | Constant folding, dead code removal | ✅ Enabled |
| `runtime(true)` | atob, btoa, escape, unescape | ✅ Enabled |
| `extensions([...])` | Obfuscator-specific patterns | ❌ Disabled |

### Extensions

| Extension | Description |
|-----------|-------------|
| `Extension::StringRotator` | obfuscator.io style string arrays |
| `Extension::ControlFlow` | Control flow deflattening (WIP) |
| `Extension::Proxy` | Proxy function inlining (WIP) |

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    JS-Deobfuscator                          │
├─────────────────────────────────────────────────────────────┤
│  Layer 1: ECMA (ECMAScript standard)                        │
│  ├── Operators: +, -, *, /, %, **, &, |, ^, ~, <<, >>       │
│  ├── Unary: !, +, -, ~, typeof, void                        │
│  ├── Comparison: ===, !==, <, >, <=, >=                     │
│  ├── Logical: &&, ||, ??                                    │
│  ├── Ternary: cond ? a : b                                  │
│  ├── String: +, [], .length, .charAt, .slice, etc.          │
│  ├── Array: .length, .join, .indexOf, etc.                  │
│  └── Number: parseInt, parseFloat, isNaN, isFinite          │
├─────────────────────────────────────────────────────────────┤
│  Layer 2: Runtime (Browser/Node APIs)                       │
│  ├── Encoding: atob, btoa                                   │
│  └── Deprecated: escape, unescape                           │
├─────────────────────────────────────────────────────────────┤
│  Layer 3: Extensions (Obfuscator patterns)                  │
│  └── String Rotator: obfuscator.io style string arrays      │
└─────────────────────────────────────────────────────────────┘
```

## Supported Transformations

### Layer 1: ECMA

| Category | Transformations |
|----------|----------------|
| Arithmetic | `1 + 2` → `3`, `10 % 3` → `1`, `2 ** 4` → `16` |
| Bitwise | `5 & 3` → `1`, `5 \| 3` → `7`, `~0` → `-1` |
| String | `"a" + "b"` → `"ab"`, `"abc"[1]` → `"b"` |
| Logical | `true && x` → `x`, `false \|\| x` → `x` |
| Ternary | `true ? a : b` → `a` |
| Typeof | `typeof "x"` → `"string"` |
| Comparison | `1 === 1` → `true` |

### Layer 2: Runtime

| Function | Example |
|----------|---------|
| `atob` | `atob("SGVsbG8=")` → `"Hello"` |
| `btoa` | `btoa("Hello")` → `"SGVsbG8="` |
| `escape` | `escape("hello world")` → `"hello%20world"` |
| `unescape` | `unescape("hello%20world")` → `"hello world"` |

### Layer 3: Extensions

| Pattern | Description |
|---------|-------------|
| String Rotator | Decodes obfuscator.io style string arrays with shuffler IIFE |

## Examples

```bash
# Run examples
cargo run --example test_string_rotator -- input.js
cargo run --example full_pipeline
cargo run --example simple_deob
```

## License

Apache-2.0 - see [LICENSE](LICENSE)
