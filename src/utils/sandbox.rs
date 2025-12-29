//! V8 JavaScript Sandbox

use deno_core::{JsRuntime, RuntimeOptions, serde_v8, v8};

/// V8 JavaScript evaluator
pub struct JsEvaluator {
    runtime: JsRuntime,
}

impl JsEvaluator {
    pub fn new() -> Self {
        Self {
            runtime: JsRuntime::new(RuntimeOptions::default()),
        }
    }

    /// Execute code for side effects
    pub fn exec(&mut self, code: &str) -> Result<(), String> {
        self.runtime
            .execute_script("<exec>", code.to_string())
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Evaluate code and return JSON result
    pub fn eval(&mut self, code: &str) -> Result<serde_json::Value, String> {
        let result = self
            .runtime
            .execute_script("<eval>", code.to_string())
            .map_err(|e| e.to_string())?;

        deno_core::scope!(scope, &mut self.runtime);
        let local = v8::Local::new(scope, result);
        serde_v8::from_v8(scope, local).map_err(|e| e.to_string())
    }
}

impl Default for JsEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exec_eval() {
        let mut js = JsEvaluator::new();
        js.exec("var x = 1 + 2").unwrap();
        assert_eq!(js.eval("x").unwrap(), serde_json::json!(3));
    }

    #[test]
    fn test_json_stringify() {
        let mut js = JsEvaluator::new();
        js.exec("var obj = {a: 1, b: 'hello'}").unwrap();
        let result = js.eval("JSON.stringify(obj)").unwrap();
        assert_eq!(result, serde_json::json!("{\"a\":1,\"b\":\"hello\"}"));
    }
}
