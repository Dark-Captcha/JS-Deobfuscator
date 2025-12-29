//! Debug why inlined file isn't working
//!
//! Run: cargo run --example debug_inlined

use js_deobfuscator::extensions::string_rotator::{
    CallCollector, ComponentCollector, ShufflerDetector, StringInliner, execute_decoder,
};
use oxc::allocator::Allocator;
use oxc::codegen::Codegen;
use oxc::parser::Parser;
use oxc::semantic::SemanticBuilder;
use oxc::span::SourceType;
use oxc_traverse::traverse_mut;

fn main() {
    let source = std::fs::read_to_string("js-tests/test_string_rotator_inlined.js")
        .expect("Failed to read test file");

    println!("=== Debug Inlined File ===\n");
    println!("Source: {} bytes\n", source.len());

    let allocator = Allocator::default();
    let mut program = Parser::new(&allocator, &source, SourceType::mjs())
        .parse()
        .program;

    // Phase 1: Detect shufflers
    println!("--- Phase 1: ShufflerDetector ---");
    let scoping = SemanticBuilder::new()
        .build(&program)
        .semantic
        .into_scoping();
    let mut detector = ShufflerDetector::default();
    traverse_mut(&mut detector, &allocator, &mut program, scoping, ());

    if detector.results.is_empty() {
        println!("❌ No shufflers detected\n");
        return;
    }

    println!("✅ Found {} system(s)\n", detector.results.len());

    for (id, info) in &detector.results {
        println!("--- System {} ---", id);
        println!("Array:    {:?}", info.array_symbol_id);
        println!("Accessor: {:?}", info.accessor_symbol_id);
        println!("Code:     {}...\n", &info.code[..info.code.len().min(80)]);
    }

    // Collect all target symbols
    let mut all_targets = Vec::new();
    for info in detector.results.values() {
        all_targets.push(info.array_symbol_id);
        all_targets.push(info.accessor_symbol_id);
    }

    // Phase 2: Collect components
    println!("--- Phase 2: ComponentCollector ---");
    let scoping = SemanticBuilder::new()
        .build(&program)
        .semantic
        .into_scoping();
    let mut collector = ComponentCollector::new(all_targets.clone());
    traverse_mut(&mut collector, &allocator, &mut program, scoping, ());

    println!("Collected {} function(s):", collector.results.len());
    for (sym, func_info) in &collector.results {
        println!(
            "  {:?}: {}...",
            sym,
            &func_info.code[..func_info.code.len().min(50)]
        );
    }
    println!();

    // Phase 3-5: For each system
    for (id, info) in &detector.results {
        println!("--- System {} Processing ---", id);

        // Phase 3: Collect calls
        let scoping = SemanticBuilder::new()
            .build(&program)
            .semantic
            .into_scoping();
        let mut call_collector = CallCollector::new(info.accessor_symbol_id);
        traverse_mut(&mut call_collector, &allocator, &mut program, scoping, ());

        println!("Calls collected: {}", call_collector.calls.len());
        for call in call_collector.calls.iter().take(5) {
            println!("  {}", call);
        }
        if call_collector.calls.len() > 5 {
            println!("  ... and {} more", call_collector.calls.len() - 5);
        }

        let calls: Vec<String> = call_collector.calls.into_iter().collect();

        // Phase 4: Execute decoder
        let array_code = collector
            .results
            .get(&info.array_symbol_id)
            .map(|f| f.code.as_str())
            .unwrap_or("");
        let accessor_code = collector
            .results
            .get(&info.accessor_symbol_id)
            .map(|f| f.code.as_str())
            .unwrap_or("");

        println!("\nArray code found: {}", !array_code.is_empty());
        println!("Accessor code found: {}", !accessor_code.is_empty());

        match execute_decoder(array_code, accessor_code, &info.code, &calls) {
            Ok(decoded) => {
                println!("✅ Decoded {} strings", decoded.len());
                for (call, value) in decoded.iter().take(3) {
                    println!("  {} → {:?}", call, value);
                }

                // Phase 5a: Inline strings
                let scoping = SemanticBuilder::new()
                    .build(&program)
                    .semantic
                    .into_scoping();
                let mut inliner = StringInliner::new(info.accessor_symbol_id, decoded);
                traverse_mut(&mut inliner, &allocator, &mut program, scoping, ());
                println!("✅ Inlined {} calls", inliner.replaced_count);
            }
            Err(e) => {
                println!("❌ Error: {}", e);
            }
        }
        println!();
    }

    // Output first 500 chars
    let output = Codegen::new().build(&program).code;
    println!("--- Output (first 500 chars) ---");
    println!("{}", &output[..output.len().min(500)]);
}
