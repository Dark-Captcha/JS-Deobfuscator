// Comprehensive test file for BraceWrapper module
// Tests ALL JavaScript statement types that have bodies

// ============================================================
// 1. IF STATEMENTS
// ============================================================

// Simple if without braces
if (true) console.log("if");

// If-else without braces
if (true) console.log("if");
else console.log("else");

// If-else-if chain without braces
if (a) console.log("a");
else if (b) console.log("b");
else if (c) console.log("c");
else console.log("else");

// Nested if without braces
if (outer) if (inner) console.log("nested");

// If with expression statement
if (x) x++;

// If with return (in function context)
function testIfReturn() {
    if (x) return 1;
    else return 2;
}

// If with throw
function testIfThrow() {
    if (error) throw new Error("error");
}

// If with break/continue (in loop context)
for (var i = 0; i < 10; i++) {
    if (i === 5) break;
    if (i === 3) continue;
}

// ============================================================
// 2. FOR STATEMENTS
// ============================================================

// Simple for without braces
for (var i = 0; i < 10; i++) console.log(i);

// For with empty body
for (var j = 0; j < 10; j++);

// Nested for without braces
for (var a = 0; a < 5; a++) for (var b = 0; b < 5; b++) console.log(a, b);

// For with let/const
for (let k = 0; k < 10; k++) console.log(k);

// ============================================================
// 3. FOR-IN STATEMENTS
// ============================================================

// Simple for-in without braces
for (var key in obj) console.log(key);

// For-in with let
for (let prop in object) console.log(prop);

// Nested for-in
for (var k1 in obj1) for (var k2 in obj2) console.log(k1, k2);

// ============================================================
// 4. FOR-OF STATEMENTS
// ============================================================

// Simple for-of without braces
for (var item of array) console.log(item);

// For-of with let
for (let element of collection) console.log(element);

// For-of with const
for (const value of iterable) console.log(value);

// For-of with destructuring
for (var [a, b] of pairs) console.log(a, b);
for (var {x, y} of points) console.log(x, y);

// ============================================================
// 5. WHILE STATEMENTS
// ============================================================

// Simple while without braces
while (condition) doSomething();

// While with decrement
while (count--) process();

// Nested while
while (outer) while (inner) work();

// While with complex condition
while (a && b || c) iterate();

// ============================================================
// 6. DO-WHILE STATEMENTS
// ============================================================

// Simple do-while without braces
do something(); while (condition);

// Do-while with assignment
do x = getNext(); while (x !== null);

// Nested do-while
do do inner(); while (innerCond); while (outerCond);

// ============================================================
// 7. WITH STATEMENTS (deprecated but valid JS)
// ============================================================

// Simple with without braces
with (obj) console.log(property);

// With accessing multiple properties
with (Math) console.log(PI);

// Nested with
with (outer) with (inner) console.log(value);

// ============================================================
// 8. LABELED STATEMENTS
// ============================================================

// Label with expression
label1: console.log("labeled");

// Label with for loop (should NOT wrap the for)
outerLoop: for (var i = 0; i < 10; i++) console.log(i);

// Label with while (should NOT wrap the while)
whileLabel: while (true) break whileLabel;

// Label with if (should wrap)
ifLabel: if (condition) doSomething();

// Nested labels
outer: inner: console.log("nested labels");

// Label with break target
search: for (var i = 0; i < 10; i++) for (var j = 0; j < 10; j++) if (found) break search;

// ============================================================
// 9. SWITCH CASES
// ============================================================

// Switch with single statement cases
switch (value) {
    case 1: console.log("one");
    case 2: console.log("two"); break;
    case 3: console.log("three"); break;
    default: console.log("default");
}

// Switch with multiple statements per case
switch (type) {
    case "a":
        var x = 1;
        process(x);
        break;
    case "b":
        var y = 2;
        var z = 3;
        process(y, z);
        break;
    default:
        console.log("unknown");
        throw new Error("invalid type");
}

// Switch with fall-through
switch (grade) {
    case "A":
    case "B":
    case "C":
        console.log("pass");
        break;
    case "D":
    case "F":
        console.log("fail");
        break;
}

// Empty switch cases
switch (x) {
    case 1:
    case 2:
    case 3:
        break;
    default:
}

// ============================================================
// 10. MIXED/COMPLEX CASES
// ============================================================

// If inside for without braces
for (var i = 0; i < 10; i++) if (i % 2 === 0) console.log(i);

// For inside if without braces
if (shouldLoop) for (var i = 0; i < 10; i++) console.log(i);

// While inside for-of
for (var item of items) while (item.hasNext()) item.next();

// Labeled for-in with if
searchLoop: for (var key in obj) if (obj[key] === target) break searchLoop;

// Complex nesting
if (a)
    for (var i = 0; i < 10; i++)
        if (b)
            while (c)
                if (d)
                    console.log("deep");

// Arrow function bodies (should NOT be wrapped - they're expressions)
const fn1 = () => console.log("arrow");
const fn2 = x => x * 2;
const fn3 = (a, b) => a + b;

// Already has braces (should NOT be modified)
if (x) { console.log("already braced"); }
for (var i = 0; i < 10; i++) { console.log(i); }
while (true) { break; }

console.log("=== BraceWrapper Test Complete ===");
