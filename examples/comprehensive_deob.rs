//! Comprehensive deobfuscation example demonstrating all passes.

use js_deobfuscator::core::{Engine, EngineConfig};
use oxc::allocator::Allocator;
use oxc::codegen::Codegen;
use oxc::parser::Parser;
use oxc::span::SourceType;

fn main() {
    // Engine now uses all passes via static dispatch (no registry needed)
    let config = EngineConfig::default().with_max_iterations(50);
    let engine = Engine::with_config(config);

    let source = r#"
        // Numeric operations
        var a = 1 + 2 * 3;
        var b = 10 % 3;
        var c = 2 ** 4;
        var d = 5 & 3;
        var e = 5 | 3;
        var f = 5 ^ 3;
        var g = ~0;
        var h = 1 << 4;
        var i = 16 >> 2;

        // Boolean operations
        var j = !false;
        var k = true && false;
        var l = false || true;

        // String operations
        var m = "hello" + " " + "world";
        var n = "abc"[1];
        var o = "test".length;

        // Comparisons
        var p = 1 === 1;
        var q = 1 !== 2;
        var r = 1 < 2;
        var s = 2 > 1;

        // Typeof
        var t = typeof "string";
        var u = typeof 42;
        var v = typeof true;

        // Conditionals
        var w = true ? "yes" : "no";
        var x = false && expensive();
        var y = true || expensive();
        var z = null ?? "default";

        // Control flow
        if (true) {
            console.log("always");
        } else {
            console.log("never");
        }

        if (false) {
            console.log("never");
        }

        while (false) {
            console.log("never");
        }

        // Dead code
        debugger;
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
    let start = std::time::Instant::now();
    let result = engine.run(&allocator, &mut program).unwrap();
    let elapsed = start.elapsed();

    let output = Codegen::new().build(&program).code;

    println!("\n=== Result ===");
    println!("Iterations: {}", result.iterations);
    println!("Total modifications: {}", result.total_modifications);
    println!("Converged: {}", result.converged);
    println!("Elapsed: {:?}", elapsed);
    println!("\n=== Output ===");
    println!("{}", output);
}
