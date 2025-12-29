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

### Basic Deobfuscation

```rust
use js_deobfuscator::{Engine, EngineConfig};
use oxc::allocator::Allocator;
use oxc::codegen::Codegen;
use oxc::parser::Parser;
use oxc::span::SourceType;

fn main() {
    let source = r#"
        var a = 1 + 2;
        var b = "hello" + " world";
        var c = true ? "yes" : "no";
    "#;

    let allocator = Allocator::default();
    let mut program = Parser::new(&allocator, source, SourceType::mjs()).parse().program;

    let engine = Engine::with_config(EngineConfig::default().with_max_iterations(50));
    engine.run(&allocator, &mut program).unwrap();

    let output = Codegen::new().build(&program).code;
    println!("{}", output);
    // var a = 3;
    // var b = "hello world";
    // var c = "yes";
}
```

### String Rotator (obfuscator.io style)

One-liner API:

```rust
use js_deobfuscator::extensions::string_rotator::StringRotator;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = std::fs::read_to_string("obfuscated.js")?;
    let output = StringRotator::deobfuscate(&source)?;
    std::fs::write("clean.js", output)?;
    Ok(())
}
```

With result details:

```rust
use js_deobfuscator::extensions::string_rotator::StringRotator;
use oxc::allocator::Allocator;
use oxc::codegen::Codegen;
use oxc::parser::Parser;
use oxc::span::SourceType;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = std::fs::read_to_string("obfuscated.js")?;
    
    let allocator = Allocator::default();
    let mut program = Parser::new(&allocator, &source, SourceType::mjs()).parse().program;

    let result = StringRotator::transform(&allocator, &mut program)?;
    
    println!("Systems detected: {}", result.systems_detected);
    println!("Strings decoded: {}", result.strings_decoded);
    println!("Calls inlined: {}", result.calls_inlined);
    println!("Functions removed: {}", result.functions_removed);

    let output = Codegen::new().build(&program).code;
    std::fs::write("clean.js", output)?;
    Ok(())
}
```

### Full Pipeline (Engine with all layers)

The engine runs all layers in a convergence loop:
1. **Extensions** (string rotator) - decodes obfuscated strings first
2. **ECMA** (constant folding, dead code) - simplifies expressions
3. **Runtime** (atob, btoa) - evaluates runtime APIs

```rust
use js_deobfuscator::{Engine, EngineConfig, LayerConfig, ExtensionsConfig};
use oxc::allocator::Allocator;
use oxc::codegen::Codegen;
use oxc::parser::Parser;
use oxc::span::SourceType;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = std::fs::read_to_string("obfuscated.js")?;
    
    let allocator = Allocator::default();
    let mut program = Parser::new(&allocator, &source, SourceType::mjs()).parse().program;

    // Enable all layers including extensions
    let config = EngineConfig::full();  // or EngineConfig::aggressive() for more iterations
    let engine = Engine::with_config(config);
    
    let result = engine.run(&allocator, &mut program)?;
    
    println!("Iterations: {}", result.iterations);
    println!("Modifications: {}", result.total_modifications);
    println!("Converged: {}", result.converged);

    let output = Codegen::new().build(&program).code;
    std::fs::write("clean.js", output)?;
    Ok(())
}
```

### Custom Configuration

```rust
use js_deobfuscator::{Engine, EngineConfig, LayerConfig, ExtensionsConfig};

// Enable only string rotator, disable other extensions
let config = EngineConfig::default()
    .with_max_iterations(50)
    .with_layers(LayerConfig {
        ecma: true,
        runtime: true,
        extensions: true,
        extensions_config: ExtensionsConfig {
            string_array: true,  // Enable string rotator
            control_flow: false,
            proxy: false,
        },
        ..Default::default()
    });

let engine = Engine::with_config(config);
```

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

## Examples

```bash
# Basic deobfuscation
cargo run --example simple_deob

# Full pipeline demo (all layers)
cargo run --example full_pipeline

# Deobfuscate a file
cargo run --example deobfuscate_file -- input.js output.js

# String rotator only (low-level)
cargo run --example test_string_rotator -- input.js output.js
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

## CLI (Coming Soon)

```bash
js-deobfuscator input.js -o output.js
js-deobfuscator input.js --string-rotator -o output.js
```

## License

Apache-2.0 - see [LICENSE](LICENSE)
