//! Test shuffler detector with real JS files.
//!
//! Run: cargo run --example test_shuffler_detector

use js_deobfuscator::extensions::string_rotator::ShufflerDetector;
use oxc::allocator::Allocator;
use oxc::parser::Parser;
use oxc::semantic::SemanticBuilder;
use oxc::span::SourceType;
use oxc_traverse::traverse_mut;

fn main() {
    let source = std::fs::read_to_string("js-tests/test_string_rotator.js")
        .expect("Failed to read test file");

    println!("=== Shuffler Detector Test ===\n");
    println!("Source: {} bytes\n", source.len());

    let allocator = Allocator::default();
    let source_type = SourceType::mjs();
    let parser = Parser::new(&allocator, &source, source_type);
    let parsed = parser.parse();

    if !parsed.errors.is_empty() {
        for err in &parsed.errors {
            eprintln!("Parse error: {}", err);
        }
        return;
    }

    let mut program = parsed.program;
    let semantic = SemanticBuilder::new().build(&program).semantic;
    let scoping = semantic.into_scoping();

    let mut detector = ShufflerDetector::default();
    traverse_mut(&mut detector, &allocator, &mut program, scoping, ());

    if detector.results.is_empty() {
        println!("❌ No shuffler IIFE detected");
        return;
    }

    println!("✅ Found {} system(s)\n", detector.results.len());

    for (id, info) in &detector.results {
        println!("--- System {} ---", id);
        println!("Array symbol:    {:?}", info.array_symbol_id);
        println!("Accessor symbol: {:?}", info.accessor_symbol_id);
        println!(
            "Code:            {}...\n",
            &info.code[..info.code.len().min(100)]
        );
    }
}
