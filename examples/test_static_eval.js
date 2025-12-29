// Test file for StaticEvaluator module
// This file contains various obfuscation patterns that should be evaluated

// === Base64 encoding/decoding ===
var decoded1 = atob("SGVsbG8gV29ybGQ=");  // Should become "Hello World"
var encoded1 = btoa("Hello");              // Should become "SGVsbG8="

// === String.fromCharCode ===
var fromCharCode1 = String.fromCharCode(72, 101, 108, 108, 111);  // Should become "Hello"
var fromCharCode2 = String.fromCharCode(87, 111, 114, 108, 100);  // Should become "World"

// === URI encoding/decoding ===
var uriDecoded = decodeURIComponent("%48%65%6C%6C%6F");  // Should become "Hello"
var uriEncoded = encodeURIComponent("Hello World");      // Should become "Hello%20World"
var unescaped = unescape("%48%65%6C%6C%6F");            // Should become "Hello"

// === parseInt/parseFloat ===
var hexNum = parseInt("ff", 16);      // Should become 255
var octNum = parseInt("777", 8);      // Should become 511
var floatNum = parseFloat("3.14159"); // Should become 3.14159

// === String methods ===
var upper = "hello".toUpperCase();           // Should become "HELLO"
var lower = "WORLD".toLowerCase();           // Should become "world"
var trimmed = "  hello  ".trim();            // Should become "hello"
var charAt = "hello".charAt(0);              // Should become "h"
var charCode = "A".charCodeAt(0);            // Should become 65
var sliced = "hello world".slice(0, 5);      // Should become "hello"
var substr = "hello world".substring(6, 11); // Should become "world"
var replaced = "hello".replace("l", "L");    // Should become "heLlo"

// === Array methods ===
var joined = [1, 2, 3].join("-");                    // Should become "1-2-3"
var reversed = "olleh".split("").reverse().join(""); // Should become "hello"
var arrLength = [1, 2, 3, 4, 5].length;              // Should become 5

// === Math methods ===
var floored = Math.floor(3.7);   // Should become 3
var ceiled = Math.ceil(3.2);     // Should become 4
var rounded = Math.round(3.5);   // Should become 4
var absVal = Math.abs(-42);      // Should become 42
var maxVal = Math.max(1, 5, 3);  // Should become 5
var minVal = Math.min(1, 5, 3);  // Should become 1
var powVal = Math.pow(2, 8);     // Should become 256

// === Number methods ===
var numStr = (255).toString(16);  // Should become "ff"
var fixed = (3.14159).toFixed(2); // Should become "3.14"

// === String length ===
var strLen = "hello".length;  // Should become 5

// === Chained operations (common obfuscation pattern) ===
var chained1 = atob("SGVsbG8=").split("").reverse().join("");  // Should become "olleH"
var chained2 = "HELLO".toLowerCase().split("").reverse().join(""); // Should become "olleh"

// === These should NOT be evaluated (unsafe or non-deterministic) ===
var random = Math.random();           // Should remain unchanged
var now = Date.now();                 // Should remain unchanged
var variable = atob(someVar);         // Should remain unchanged (variable argument)
console.log("test");                  // Should remain unchanged (side effect)

// === Complex obfuscation pattern ===
function decode(s) {
    return atob(s);
}
// This won't be evaluated because decode() is not a known safe function
var complex = decode("SGVsbG8=");
