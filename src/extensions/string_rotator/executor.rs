//! Phase 4: Execute V8 sandbox to decode strings.

use rustc_hash::FxHashMap;

use crate::utils::sandbox::JsEvaluator;

/// Decoded results: call_code -> decoded_string
pub type DecodedMap = FxHashMap<String, String>;

/// Execute V8 sandbox to decode all calls for a system.
///
/// # Arguments
/// - `array_code`: Array function code
/// - `accessor_code`: Accessor function code  
/// - `shuffler_code`: Shuffler IIFE code
/// - `calls`: Unique call codes to evaluate
///
/// # Returns
/// Map of call_code -> decoded_string
pub fn execute_decoder(
    array_code: &str,
    accessor_code: &str,
    shuffler_code: &str,
    calls: &[String],
) -> Result<DecodedMap, String> {
    if calls.is_empty() {
        return Ok(FxHashMap::default());
    }

    let mut js = JsEvaluator::new();

    // 1. Define array function
    js.exec(array_code)?;

    // 2. Define accessor function
    js.exec(accessor_code)?;

    // 3. Execute shuffler (rotates array)
    js.exec(shuffler_code)?;

    // 4. Batch evaluate all calls
    let batch_code = build_batch_eval(calls);
    let result = js.eval(&batch_code)?;

    // 5. Parse results
    parse_results(result, calls)
}

/// Build batch evaluation script.
fn build_batch_eval(calls: &[String]) -> String {
    let mut script = String::from("(function() { var r = {};\n");
    for (i, call) in calls.iter().enumerate() {
        script.push_str(&format!(
            "try {{ r[{}] = {}; }} catch(e) {{ r[{}] = null; }}\n",
            i, call, i
        ));
    }
    script.push_str("return r; })()");
    script
}

/// Parse V8 results into DecodedMap.
fn parse_results(value: serde_json::Value, calls: &[String]) -> Result<DecodedMap, String> {
    let obj = value.as_object().ok_or("Expected object result")?;

    let mut map = FxHashMap::default();
    for (i, call) in calls.iter().enumerate() {
        let key = i.to_string();
        if let Some(s) = obj.get(&key).and_then(|v| v.as_str()) {
            map.insert(call.clone(), s.to_string());
        }
    }
    Ok(map)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_decode() {
        let array_code = r#"
            function arr() {
                var r = ["hello", "world", "test"];
                return r;
            }
        "#;

        let accessor_code = r#"
            function get(i) {
                return arr()[i];
            }
        "#;

        let shuffler_code = ""; // No rotation needed for this test

        let calls = vec![
            "get(0)".to_string(),
            "get(1)".to_string(),
            "get(2)".to_string(),
        ];

        let result = execute_decoder(array_code, accessor_code, shuffler_code, &calls).unwrap();

        assert_eq!(result.get("get(0)"), Some(&"hello".to_string()));
        assert_eq!(result.get("get(1)"), Some(&"world".to_string()));
        assert_eq!(result.get("get(2)"), Some(&"test".to_string()));
    }

    #[test]
    fn test_with_rotation() {
        // Self-reassigning pattern like real obfuscator
        let array_code = r#"
            function arr() {
                var r = ["a", "b", "c", "d", "e"];
                arr = function() { return r; };
                return r;
            }
        "#;

        let accessor_code = r#"
            function get(i) {
                return arr()[i];
            }
        "#;

        // Rotate 2 times: ["a","b","c","d","e"] -> ["c","d","e","a","b"]
        let shuffler_code = r#"
            (function(a) {
                var r = a();
                r.push(r.shift());
                r.push(r.shift());
            })(arr);
        "#;

        let calls = vec![
            "get(0)".to_string(),
            "get(1)".to_string(),
            "get(2)".to_string(),
        ];

        let result = execute_decoder(array_code, accessor_code, shuffler_code, &calls).unwrap();

        assert_eq!(result.get("get(0)"), Some(&"c".to_string()));
        assert_eq!(result.get("get(1)"), Some(&"d".to_string()));
        assert_eq!(result.get("get(2)"), Some(&"e".to_string()));
    }

    #[test]
    fn test_empty_calls() {
        let result = execute_decoder("", "", "", &[]).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_error_handling() {
        let array_code = "function arr() { return ['a']; }";
        let accessor_code = "function get(i) { return arr()[i]; }";
        let shuffler_code = "";

        // One valid, one will return undefined (out of bounds)
        let calls = vec!["get(0)".to_string(), "get(999)".to_string()];

        let result = execute_decoder(array_code, accessor_code, shuffler_code, &calls).unwrap();

        assert_eq!(result.get("get(0)"), Some(&"a".to_string()));
        // get(999) returns undefined, not a string, so not in map
        assert!(!result.contains_key("get(999)"));
    }
}
