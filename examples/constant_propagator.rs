//! Example demonstrating constant folding via the engine.
//!
//! Note: Full constant propagation (collect → inline) is planned for Phase 12.
//! This example shows the current constant folding capabilities.

use js_deobfuscator::core::{Engine, EngineConfig};
use oxc::allocator::Allocator;
use oxc::codegen::Codegen;
use oxc::parser::Parser;
use oxc::span::SourceType;

fn main() {
    let source = r#"
        // Constant expressions are folded
        var x = 1 + 2;
        var y = 3 * 4;
        var name = "hello" + " world";
        var flag = !false;
        
        console.log(x);
        console.log(name);
        if (flag) {
            console.log(y);
        }
    "#;

    println!("=== Input ===");
    println!("{}", source);

    let allocator = Allocator::default();
    let source_type = SourceType::mjs();
    let ret = Parser::new(&allocator, source, source_type).parse();

    if !ret.errors.is_empty() {
        eprintln!("Parse errors: {:?}", ret.errors);
        return;
    }

    let mut program = ret.program;

    // Run engine
    let engine = Engine::with_config(EngineConfig::default().with_max_iterations(10));
    let result = engine.run(&allocator, &mut program).unwrap();

    let output = Codegen::new().build(&program).code;

    println!("\n=== Result ===");
    println!("Modifications: {}", result.total_modifications);
    println!("\n=== Output ===");
    println!("{}", output);
}
