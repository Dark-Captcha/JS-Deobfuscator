# String Rotator Decoder

> **Status:** Complete | **Updated:** 2025-12-29

Decodes obfuscated string arrays with rotation (obfuscator.io style).

## Quick Start

### One-liner API

```rust
use js_deobfuscator::extensions::string_rotator::StringRotator;

let output = StringRotator::deobfuscate(&source)?;
```

### With Result Details

```rust
use js_deobfuscator::extensions::string_rotator::StringRotator;
use oxc::allocator::Allocator;
use oxc::codegen::Codegen;
use oxc::parser::Parser;
use oxc::span::SourceType;

let allocator = Allocator::default();
let mut program = Parser::new(&allocator, &source, SourceType::mjs()).parse().program;

let result = StringRotator::transform(&allocator, &mut program)?;

println!("Systems detected: {}", result.systems_detected);
println!("Strings decoded: {}", result.strings_decoded);
println!("Calls inlined: {}", result.calls_inlined);
println!("Functions removed: {}", result.functions_removed);

let output = Codegen::new().build(&program).code;
```

### Full Pipeline (with Engine)

```rust
use js_deobfuscator::{Engine, EngineConfig};
use js_deobfuscator::extensions::string_rotator::StringRotator;

// Step 1: Decode string arrays
StringRotator::transform(&allocator, &mut program)?;

// Step 2: Run engine (constant folding, dead code, etc.)
let engine = Engine::with_config(EngineConfig::default().with_max_iterations(50));
engine.run(&allocator, &mut program)?;
```

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│ Phase 1: ShufflerDetector                                   │
│ - Detect IIFE with push/shift>=2 + parseInt>=2              │
│ - Extract: array_symbol_id, accessor_symbol_id, code        │
│ - Remove: IIFE statement                                    │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│ Phase 2: ComponentCollector                                 │
│ - Collect function code by symbol_id                        │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│ Phase 3: CallCollector                                      │
│ - Collect all calls to accessor with literal arguments      │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│ Phase 4: V8 Executor                                        │
│ - Execute array + accessor + shuffler in V8 sandbox         │
│ - Batch evaluate all calls                                  │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│ Phase 5: Inliner                                            │
│ - StringInliner: Replace calls with decoded strings         │
│ - ComponentRemover: Remove array/accessor functions         │
└─────────────────────────────────────────────────────────────┘
```

## Files

```
src/extensions/string_rotator/
├── mod.rs                    # High-level API (StringRotator)
├── shuffler_detector.rs      # Phase 1
├── component_collector.rs    # Phase 2
├── call_collector.rs         # Phase 3
├── executor.rs               # Phase 4
└── inliner.rs                # Phase 5
```

## Test Results

```bash
cargo run --example test_string_rotator -- js-tests/pow_funcaptcha_inlined.js
```

| File | Systems | Decoded | Inlined | Removed |
|------|---------|---------|---------|---------|
| test_simple_rotator.js | 1 | 4 | 4 | 2 |
| pow_funcaptcha_inlined.js | 1 | 5 | 5 | 2 |
| test_string_rotator_inlined.js | 2 | 3 | 3 | 2 |

## Known Limitation

`accessor_symbol_id` from Phase 1 may be a **local wrapper** inside the shuffler IIFE, not the real accessor function.

**Example:**
```js
const p1 = G;  // alias at top
(function (p, g) {
  const p0 = G,  // LOCAL wrapper - Phase 1 extracts this
    J = p();
  // ... uses p0(0x13b) etc
})(m, 0x44598);
function G(p, g) { ... }  // REAL accessor
```

**Resolution:** Function inliner module should resolve wrapper chains. For testing, create inlined versions of test files.

## Validation Rules

Phase 1 validates shuffler IIFE by:
1. `push/shift` pattern count >= 2 (try + catch blocks)
2. `parseInt` call count >= 2 (checksum calculation)
3. First argument is identifier (array function reference)
4. First `parseInt(accessor(...))` extracts accessor symbol

## Low-Level API

For advanced usage, individual phases can be used directly:

```rust
use js_deobfuscator::extensions::string_rotator::{
    ShufflerDetector, ComponentCollector, CallCollector, 
    execute_decoder, StringInliner, ComponentRemover
};
```

See `examples/test_string_rotator.rs` for full low-level usage.
