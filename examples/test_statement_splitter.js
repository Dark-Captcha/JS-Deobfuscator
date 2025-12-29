// Test file for StatementSplitter module
// Tests splitting multi-var declarations AND sequence expressions

// ============================================================
// 1. VARIABLE DECLARATIONS
// ============================================================

// Multiple vars
var a, b, c;
var x = 1, y = 2, z = 3;

// Multiple lets
let m, n, o;
let p = "hello", q = "world";

// Multiple consts
const A = 1, B = 2, C = 3;

// Destructuring
var [first, second] = arr, [head, ...tail] = list;
var {name, age} = person, {x: posX} = point;

// ============================================================
// 2. SEQUENCE EXPRESSIONS
// ============================================================

// Simple sequences
a, b, c;
foo(), bar(), baz();

// Assignment sequences
a = 1, b = 2, c = 3;
x += 1, y -= 2, z *= 3;

// Method calls
obj.method1(), obj.method2();
console.log("a"), console.log("b");

// ============================================================
// 3. MIXED IN BLOCKS
// ============================================================

function testFunc() {
    var local1, local2;
    init(), setup(), run();
}

if (true) {
    var ifVar1, ifVar2;
    x++, y++;
}

// ============================================================
// 4. SHOULD NOT SPLIT
// ============================================================

// Single declarations
var single = 42;
let singleLet = "test";

// Single expressions
singleCall();

// For loop init/update (not statements)
for (var i = 0, j = 10; i < j; i++, j--) {
    console.log(i, j);
}

console.log("=== StatementSplitter Test Complete ===");
