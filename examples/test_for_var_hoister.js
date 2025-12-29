// Comprehensive test file for ForVarHoister module
// Tests hoisting var declarations from for-loop initializers

// ============================================================
// 1. SIMPLE FOR LOOPS WITH VAR
// ============================================================

// Basic for loop with var
for (var i = 0; i < 10; i++) {
    console.log(i);
}

// For loop with multiple var declarations
for (var j = 0, k = 10; j < k; j++, k--) {
    console.log(j, k);
}

// For loop with complex initializer
for (var x = getStart(); x < getEnd(); x++) {
    process(x);
}

// ============================================================
// 2. FOR LOOPS WITH LET/CONST (should NOT be hoisted)
// ============================================================

// For loop with let (block-scoped, don't hoist)
for (let m = 0; m < 10; m++) {
    console.log(m);
}

// For loop with const in for-of (don't hoist)
for (const item of items) {
    console.log(item);
}

// For loop with let in for-in (don't hoist)
for (let key in obj) {
    console.log(key);
}

// ============================================================
// 3. NESTED FOR LOOPS
// ============================================================

// Nested for loops with var
for (var outer = 0; outer < 5; outer++) {
    for (var inner = 0; inner < 5; inner++) {
        console.log(outer, inner);
    }
}

// Mixed var and let in nested loops
for (var a = 0; a < 10; a++) {
    for (let b = 0; b < 10; b++) {
        console.log(a, b);
    }
}

// ============================================================
// 4. FOR LOOPS WITHOUT INIT (should NOT be modified)
// ============================================================

// For loop with no init
var preInit = 0;
for (; preInit < 10; preInit++) {
    console.log(preInit);
}

// For loop with expression init (not declaration)
var exprInit;
for (exprInit = 0; exprInit < 10; exprInit++) {
    console.log(exprInit);
}

// ============================================================
// 5. FOR LOOPS IN DIFFERENT CONTEXTS
// ============================================================

// In function body
function testFunc() {
    for (var funcVar = 0; funcVar < 10; funcVar++) {
        console.log(funcVar);
    }
}

// In block
{
    for (var blockVar = 0; blockVar < 10; blockVar++) {
        console.log(blockVar);
    }
}

// In if body
if (true) {
    for (var ifVar = 0; ifVar < 10; ifVar++) {
        console.log(ifVar);
    }
}

// In while body
while (condition) {
    for (var whileVar = 0; whileVar < 10; whileVar++) {
        console.log(whileVar);
    }
    break;
}

// ============================================================
// 6. LABELED FOR LOOPS
// ============================================================

// Labeled for with var
outerLabel: for (var labeled = 0; labeled < 10; labeled++) {
    console.log(labeled);
}

// ============================================================
// 7. FOR-IN AND FOR-OF (should NOT be modified - different syntax)
// ============================================================

// For-in with var (different structure, not hoisted by this module)
for (var key in object) {
    console.log(key);
}

// For-of with var (different structure, not hoisted by this module)
for (var value of array) {
    console.log(value);
}

// ============================================================
// 8. EDGE CASES
// ============================================================

// Empty for loop body
for (var empty = 0; empty < 10; empty++);

// For loop with complex condition
for (var complex = start(); complex < end() && isValid(complex); complex = next(complex)) {
    process(complex);
}

// For loop with no condition or update
for (var infinite = 0;;) {
    if (infinite++ > 100) break;
}

// Multiple for loops in sequence
for (var seq1 = 0; seq1 < 5; seq1++) console.log(seq1);
for (var seq2 = 0; seq2 < 5; seq2++) console.log(seq2);
for (var seq3 = 0; seq3 < 5; seq3++) console.log(seq3);

// ============================================================
// 9. READ-ONLY VARIABLES (SHOULD BE HOISTED)
// ============================================================

// Read-only length variable - SHOULD be hoisted
for (var len = arr.length; idx < len; idx++) {
    console.log(arr[idx]);
}

// Read-only max variable - SHOULD be hoisted
for (var max = getMax(); counter < max; counter++) {
    process(counter);
}

// Multiple read-only vars - SHOULD be hoisted
for (var start = 0, end = arr.length; start < end; start++) {
    // Note: start is written, so this should NOT be hoisted
    console.log(start);
}

// Pure read-only (no writes anywhere) - SHOULD be hoisted
for (var limit = 100; someOtherVar < limit;) {
    someOtherVar++;
}

// Read-only with function call init - SHOULD be hoisted
for (var config = getConfig(); isValid(config);) {
    doSomething();
    break;
}

// ============================================================
// 10. WRITTEN VARIABLES (should NOT be hoisted)
// ============================================================

// Classic i++ pattern - should NOT be hoisted
for (var idx = 0; idx < 10; idx++) {
    console.log(idx);
}

// Assignment in body - should NOT be hoisted
for (var val = 0; val < 10;) {
    val = val + 1;
}

// Compound assignment - should NOT be hoisted
for (var num = 0; num < 100;) {
    num += 10;
}

// Increment in body - should NOT be hoisted
for (var cnt = 0; cnt < 10;) {
    cnt++;
}

console.log("=== ForVarHoister Test Complete ===");
