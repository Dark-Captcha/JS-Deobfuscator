//! Test string rotator via full pipeline.
//!
//! Run: cargo run --example test_string_rotator -- js-tests/test_simple_rotator.js

use js_deobfuscator::{Extension, JSDeobfuscator};

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "js-tests/test_string_rotator_inlined.js".to_string());
    let source = std::fs::read_to_string(&path).expect("Failed to read file");

    println!("=== String Rotator ===\n");
    println!("File: {}", path);
    println!("Size: {} bytes\n", source.len());

    let result = JSDeobfuscator::new()
        .ecma(true)
        .runtime(true)
        .extensions([Extension::StringRotator])
        .max_iterations(50)
        .deobfuscate_with_result(&source)
        .expect("Deobfuscation failed");

    println!("Iterations: {}", result.iterations);
    println!("Modifications: {}", result.modifications);
    println!("Converged: {}\n", result.converged);

    println!("--- Output ---");
    println!("{}", result.code);

    if let Some(out_path) = std::env::args().nth(2) {
        std::fs::write(&out_path, &result.code).expect("Failed to write output");
        println!("\n✅ Written to {}", out_path);
    }
}
