//! Deobfuscate a JavaScript file using the full pipeline.
//!
//! Run: cargo run --example deobfuscate_file -- input.js [output.js]

use js_deobfuscator::core::{Engine, EngineConfig};
use oxc::allocator::Allocator;
use oxc::codegen::Codegen;
use oxc::parser::Parser;
use oxc::span::SourceType;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <input.js> [output.js]", args[0]);
        eprintln!("\nDeobfuscates a JavaScript file using all layers:");
        eprintln!("  - Extensions: String rotator (obfuscator.io style)");
        eprintln!("  - ECMA: Constant folding, dead code removal");
        eprintln!("  - Runtime: atob, btoa, escape, unescape");
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = args.get(2);

    // Read input
    let source = match std::fs::read_to_string(input_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading {}: {}", input_path, e);
            std::process::exit(1);
        }
    };

    println!("=== Deobfuscating {} ===", input_path);
    println!("Input size: {} bytes\n", source.len());

    // Parse
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, &source, SourceType::mjs()).parse();

    if !ret.errors.is_empty() {
        eprintln!("Parse errors:");
        for err in &ret.errors {
            eprintln!("  {}", err);
        }
        std::process::exit(1);
    }

    let mut program = ret.program;

    // Run engine with all layers
    let config = EngineConfig::full().with_max_iterations(50);
    let engine = Engine::with_config(config);

    let result = match engine.run(&allocator, &mut program) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Engine error: {}", e);
            std::process::exit(1);
        }
    };

    println!("=== Results ===");
    println!("Iterations: {}", result.iterations);
    println!("Modifications: {}", result.total_modifications);
    println!("Converged: {}", result.converged);
    println!("Elapsed: {:?}", result.elapsed);

    // Generate output
    let output = Codegen::new().build(&program).code;
    println!("Output size: {} bytes", output.len());

    // Write output
    if let Some(out_path) = output_path {
        match std::fs::write(out_path, &output) {
            Ok(_) => println!("\n✅ Written to {}", out_path),
            Err(e) => eprintln!("\n❌ Error writing {}: {}", out_path, e),
        }
    } else {
        println!("\n=== Output (first 1000 chars) ===");
        println!("{}", &output[..output.len().min(1000)]);
        if output.len() > 1000 {
            println!("... ({} more chars)", output.len() - 1000);
        }
    }
}
