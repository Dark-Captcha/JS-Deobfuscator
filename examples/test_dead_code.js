// Test file for DeadCodeEliminator module
// Tests ALL dead code elimination cases

// ============================================================
// 1. CONSTANT IF CONDITIONS
// ============================================================

if (true) { console.log("kept"); } else { console.log("removed"); }
if (false) { console.log("removed"); } else { console.log("kept"); }
if (false) { console.log("removed"); }
if (1) { console.log("truthy number"); }
if (0) { console.log("falsy number"); }
if ("") { console.log("falsy string"); }
if ("hello") { console.log("truthy string"); }
if (null) { console.log("null is falsy"); }
if (undefined) { console.log("undefined is falsy"); }

// ============================================================
// 2. CONSTANT WHILE/DO-WHILE/FOR
// ============================================================

while (false) { console.log("never runs"); }
do { console.log("runs once"); } while (false);
for (; false; ) { console.log("never runs"); }

// ============================================================
// 3. TERNARY EXPRESSIONS
// ============================================================

var a = true ? "yes" : "no";
var b = false ? "yes" : "no";
var c = 1 ? "truthy" : "falsy";
var d = 0 ? "truthy" : "falsy";

// ============================================================
// 4. LOGICAL SHORT-CIRCUIT
// ============================================================

var e = true || expensive();
var f = false || fallback();
var g = true && getValue();
var h = false && getValue();
var i = null ?? "default";
var j = undefined ?? "default";

// ============================================================
// 5. UNREACHABLE CODE
// ============================================================

function testReturn() {
    return 1;
    console.log("unreachable");
    var x = 2;
}

function testThrow() {
    throw new Error("error");
    console.log("unreachable");
}

function testBreak() {
    for (var i = 0; i < 10; i++) {
        break;
        console.log("unreachable");
    }
}

// ============================================================
// 6. UNUSED VARIABLES
// ============================================================

var unused1 = 1;
var unused2 = "hello";
var used = 42;
console.log(used);

// ============================================================
// 7. UNUSED FUNCTIONS
// ============================================================

function unusedFunc() { return 1; }
function usedFunc() { return 2; }
console.log(usedFunc());

// ============================================================
// 8. EMPTY/DEBUG STATEMENTS
// ============================================================

;
;;
debugger;

// ============================================================
// 9. NESTED CASES
// ============================================================

if (true) {
    if (false) {
        console.log("inner removed");
    }
    console.log("outer kept");
}

var nested = true ? (false ? "a" : "b") : "c";

// ============================================================
// 10. NEGATION (including multiple !)
// ============================================================

if (!false) { console.log("!false is true"); }
if (!true) { console.log("!true is false"); }
var neg = !true ? "yes" : "no";

// Multiple negations
var n1 = !!true;           // true
var n2 = !!!true;          // false
var n3 = !!!!true;         // true
var n4 = !!!!!true;        // false
var n5 = !!!!!!true;       // true
var n6 = !!false;          // false
var n7 = !!!false;         // true
if (!!true) { console.log("double neg true"); }
if (!!!true) { console.log("triple neg - removed"); }
if (!!!!true) { console.log("quad neg true"); }
if (!!!!!true) { console.log("quint neg - removed"); }

console.log("=== DeadCodeEliminator Test Complete ===");
