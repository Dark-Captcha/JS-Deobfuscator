//! Debug shuffler detection
//!
//! Run: cargo run --example debug_shuffler -- inputs/captcha.js

use js_deobfuscator::extensions::string_rotator::ShufflerDetector;
use oxc::allocator::Allocator;
use oxc::parser::Parser;
use oxc::semantic::SemanticBuilder;
use oxc::span::SourceType;
use oxc_traverse::traverse_mut;

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "inputs/captcha.js".to_string());
    let source = std::fs::read_to_string(&path).expect("Failed to read file");

    println!("=== Shuffler Detection Debug ===\n");
    println!("File: {}", path);
    println!("Size: {} bytes\n", source.len());

    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, &source, SourceType::mjs()).parse();

    if !ret.errors.is_empty() {
        for err in &ret.errors {
            eprintln!("Parse error: {}", err);
        }
        return;
    }

    let mut program = ret.program;
    let scoping = SemanticBuilder::new()
        .build(&program)
        .semantic
        .into_scoping();

    let mut detector = ShufflerDetector::default();
    traverse_mut(&mut detector, &allocator, &mut program, scoping, ());

    println!("Systems detected: {}\n", detector.results.len());

    for (id, info) in &detector.results {
        println!("--- System {} ---", id);
        println!("  Array symbol:    {:?}", info.array_symbol_id);
        println!("  Accessor symbol: {:?}", info.accessor_symbol_id);
        println!(
            "  Code preview:    {}...",
            &info.code[..info.code.len().min(60)]
        );
    }
}
