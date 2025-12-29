//! Simple deobfuscation example.
//!
//! Run with: cargo run --example simple_deob

use js_deobfuscator::core::{Engine, EngineConfig};
use oxc::allocator::Allocator;
use oxc::codegen::Codegen;
use oxc::parser::Parser;
use oxc::span::SourceType;

fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Source code with numeric expressions
    let source = r#"
        var a = 1 + 2;
        var b = 10 - 3;
        var c = 4 * 5;
        var d = 20 / 4;
        console.log(a, b, c, d);
    "#;

    println!("=== Input ===");
    println!("{}", source);

    // Create engine with all passes (static dispatch)
    let engine = Engine::with_config(EngineConfig::default().with_max_iterations(10));

    // Parse
    let allocator = Allocator::default();
    let source_type = SourceType::mjs();
    let ret = Parser::new(&allocator, source, source_type).parse();

    if !ret.errors.is_empty() {
        eprintln!("Parse errors:");
        for err in &ret.errors {
            eprintln!("  {}", err);
        }
        return;
    }

    let mut program = ret.program;

    // Run deobfuscation
    let result = engine.run(&allocator, &mut program);

    match result {
        Ok(result) => {
            println!("\n=== Result ===");
            println!("Iterations: {}", result.iterations);
            println!("Total modifications: {}", result.total_modifications);
            println!("Converged: {}", result.converged);
            println!("Elapsed: {:?}", result.elapsed);

            // Generate output
            let output = Codegen::new().build(&program).code;
            println!("\n=== Output ===");
            println!("{}", output);
        }
        Err(err) => {
            eprintln!("Engine error: {}", err);
        }
    }
}
