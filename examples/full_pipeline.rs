//! Full deobfuscation pipeline example.
//!
//! Demonstrates the engine with all layers:
//! 1. Extensions (string rotator) - decodes obfuscated strings
//! 2. ECMA (operators, string methods) - constant folding
//! 3. Runtime (atob, btoa) - runtime API evaluation
//!
//! Run: cargo run --example full_pipeline

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

    // Example with multiple obfuscation patterns
    let source = r#"
        // Expressions to evaluate (ECMA layer)
        var sum = 1 + 2;
        var product = 2 * 3 * 4;
        var greeting = "hello" + " " + "world";
        
        // Conditionals to simplify
        var result = true ? "yes" : "no";
        var short = false && expensive();
        
        // Runtime APIs (Runtime layer)
        var decoded = atob("SGVsbG8gV29ybGQ=");
        var encoded = btoa("test");
        
        // Control flow to simplify
        if (true) {
            console.log(sum);
        } else {
            console.log("never");
        }
        
        // Dead code
        debugger;
        while (false) {
            console.log("unreachable");
        }
    "#;

    println!("=== Input ===");
    println!("{}", source);

    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, SourceType::mjs()).parse();

    if !ret.errors.is_empty() {
        eprintln!("Parse errors: {:?}", ret.errors);
        return;
    }

    let mut program = ret.program;

    // Run engine with all layers
    println!("\n=== Running Engine (all layers) ===");
    let config = EngineConfig::full().with_max_iterations(50);
    let engine = Engine::with_config(config);

    let result = engine.run(&allocator, &mut program).unwrap();

    println!("Iterations: {}", result.iterations);
    println!("Modifications: {}", result.total_modifications);
    println!("Converged: {}", result.converged);
    println!("Elapsed: {:?}", result.elapsed);

    // Generate output
    let output = Codegen::new().build(&program).code;

    println!("\n=== Final Output ===");
    println!("{}", output);
}
