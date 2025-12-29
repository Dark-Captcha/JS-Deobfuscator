//! Generate baseline output (parse + codegen, no transforms)

use oxc::allocator::Allocator;
use oxc::codegen::Codegen;
use oxc::parser::Parser;
use oxc::span::SourceType;
use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <input> <output>", args[0]);
        std::process::exit(1);
    }

    let source = fs::read_to_string(&args[1]).unwrap();
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, &source, SourceType::mjs()).parse();
    let code = Codegen::new().build(&ret.program).code;
    fs::write(&args[2], &code).unwrap();
    println!(
        "Written {} bytes to {} (from {} bytes)",
        code.len(),
        args[2],
        source.len()
    );
}
