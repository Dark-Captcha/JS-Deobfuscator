//! Code generation utilities

use oxc::ast::ast::{CallExpression, Expression, Function, Statement};
use oxc::codegen::{Codegen, Context, Gen, GenExpr};
use oxc::syntax::precedence::Precedence;

/// Generate code from a statement (full statement with semicolon)
pub fn stmt_to_code(stmt: &Statement) -> String {
    let mut cg = Codegen::new();
    stmt.print(&mut cg, Context::empty());
    cg.into_source_text()
}

/// Generate code from an expression
pub fn expr_to_code(expr: &Expression) -> String {
    let mut cg = Codegen::new();
    cg.print_expression(expr);
    cg.into_source_text()
}

/// Generate code from a call expression
pub fn call_to_code(call: &CallExpression) -> String {
    let mut cg = Codegen::new();
    call.print_expr(&mut cg, Precedence::Lowest, Context::empty());
    cg.into_source_text()
}

/// Generate full function code (including `function` keyword, name, params, body)
pub fn function_to_code(func: &Function) -> String {
    let mut cg = Codegen::new();
    func.print(&mut cg, Context::empty());
    cg.into_source_text()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::testing::parse_program;
    use oxc::allocator::Allocator;
    use oxc::ast::ast::Statement;

    #[test]
    fn test_function_to_code() {
        let alloc = Allocator::default();
        let program = parse_program(&alloc, "function foo(a, b) { return a + b; }");
        let stmt = &program.body[0];
        if let Statement::FunctionDeclaration(func) = stmt {
            let code = function_to_code(func);
            assert!(code.contains("function"));
            assert!(code.contains("foo"));
            assert!(code.contains("a, b"));
            assert!(code.contains("return"));
        } else {
            panic!("Expected FunctionDeclaration");
        }
    }

    #[test]
    fn test_function_to_code_async() {
        let alloc = Allocator::default();
        let program = parse_program(&alloc, "async function bar() { await x; }");
        let stmt = &program.body[0];
        if let Statement::FunctionDeclaration(func) = stmt {
            let code = function_to_code(func);
            assert!(code.contains("async"));
            assert!(code.contains("function"));
            assert!(code.contains("bar"));
        } else {
            panic!("Expected FunctionDeclaration");
        }
    }

    #[test]
    fn test_expr_to_code() {
        let alloc = Allocator::default();
        let program = parse_program(&alloc, "1 + 2 * 3");
        let stmt = &program.body[0];
        if let Statement::ExpressionStatement(expr_stmt) = stmt {
            let code = expr_to_code(&expr_stmt.expression);
            assert_eq!(code, "1 + 2 * 3");
        } else {
            panic!("Expected ExpressionStatement");
        }
    }

    #[test]
    fn test_stmt_to_code() {
        let alloc = Allocator::default();
        let program = parse_program(&alloc, "let x = 5;");
        let stmt = &program.body[0];
        let code = stmt_to_code(stmt);
        assert!(code.contains("let"));
        assert!(code.contains("x"));
        assert!(code.contains("5"));
    }
}
