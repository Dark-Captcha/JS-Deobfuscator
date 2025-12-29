// ============================================================================
// COMPREHENSIVE IDENTIFIER RENAMER TEST
// This file tests ALL JavaScript scope scenarios for the identifier renamer
// ============================================================================

// ============================================================================
// 1. BASIC VARIABLE DECLARATIONS
// ============================================================================

// 1.1 var declarations (function-scoped)
var globalVar = 1;
var anotherGlobal = globalVar + 1;

// 1.2 let declarations (block-scoped)
let globalLet = 2;

// 1.3 const declarations (block-scoped)
const globalConst = 3;

// ============================================================================
// 2. FUNCTION DECLARATIONS AND EXPRESSIONS
// ============================================================================

// 2.1 Function declaration
function namedFunction(param1, param2) {
    var localVar = param1 + param2;
    return localVar;
}

// 2.2 Function expression
var funcExpr = function anonymousFunc(x) {
    return x * 2;
};

// 2.3 Named function expression (name only visible inside)
var outer = function inner(n) {
    if (n <= 1) return 1;
    return n * inner(n - 1); // inner is only visible here
};

// 2.4 Arrow function
var arrowFunc = (a, b) => {
    var sum = a + b;
    return sum;
};

// 2.5 Arrow function with single param (no parens)
var singleParamArrow = x => x * 2;

// 2.6 Arrow function with implicit return
var implicitReturn = (a, b) => a + b;

// ============================================================================
// 3. SHADOWING - CRITICAL FOR DEOBFUSCATION
// ============================================================================

// 3.1 Basic shadowing
var shadow = "outer";
function testShadow() {
    var shadow = "inner"; // shadows outer
    console.log(shadow);  // should reference inner
}
console.log(shadow); // should reference outer

// 3.2 Nested shadowing - CRITICAL: same name at different scope levels
var level = 0;
function level1() {
    var level = 1;
    function level2() {
        var level = 2;
        function level3() {
            var level = 3;
            console.log(level); // 3
        }
        console.log(level); // 2
        level3();
    }
    console.log(level); // 1
    level2();
}
console.log(level); // 0

// 3.2.1 Function inside function with SAME NAME - CRITICAL TEST
function outer() {
    var x = 1;
    function inner() {
        var x = 2; // shadows outer x
        console.log(x); // should be 2
    }
    inner();
    console.log(x); // should be 1
}

// 3.2.2 Multiple nested functions with same variable name
function a() {
    var name = "a";
    function b() {
        var name = "b";
        function c() {
            var name = "c";
            function d() {
                var name = "d";
                return name; // "d"
            }
            return name + d(); // "c" + "d"
        }
        return name + c(); // "b" + "cd"
    }
    return name + b(); // "a" + "bcd"
}

// 3.2.3 Same function name at different levels
function foo() {
    console.log("outer foo");
    function foo() { // shadows outer foo inside this scope
        console.log("inner foo");
    }
    foo(); // calls inner foo
}

// 3.2.4 Recursive function with shadowed parameter
function recurse(n) {
    if (n <= 0) return 0;
    function recurse(n) { // shadows outer recurse
        return n * 2;
    }
    return recurse(n) + n; // calls inner recurse
}

// 3.3 Parameter shadowing
var param = "global";
function shadowParam(param) {
    console.log(param); // parameter, not global
}

// 3.4 Let/const block shadowing
var blockShadow = "outer";
{
    let blockShadow = "inner"; // shadows in this block only
    console.log(blockShadow);
}
console.log(blockShadow); // outer

// ============================================================================
// 4. BLOCK SCOPES
// ============================================================================

// 4.1 If statement blocks
if (true) {
    let ifScoped = 1;
    const ifConst = 2;
    var ifVar = 3; // hoisted to function/global scope
}
// ifScoped and ifConst not accessible here
// ifVar is accessible

// 4.2 Nested blocks
{
    let outer = 1;
    {
        let inner = 2;
        let outer = 3; // shadows
        console.log(outer, inner);
    }
    console.log(outer); // 1
}

// 4.3 Switch statement
var switchVar = 1;
switch (switchVar) {
    case 1:
        let caseScoped = "case1";
        break;
    case 2:
        let caseScoped2 = "case2"; // different binding
        break;
}

// ============================================================================
// 5. LOOP SCOPES
// ============================================================================

// 5.1 For loop with let (each iteration gets new binding)
for (let i = 0; i < 3; i++) {
    let loopLocal = i * 2;
    console.log(i, loopLocal);
}

// 5.2 For loop with var (single binding, hoisted)
for (var j = 0; j < 3; j++) {
    var loopVar = j;
}
console.log(j, loopVar); // accessible outside

// 5.3 For-in loop
var obj = { a: 1, b: 2 };
for (let key in obj) {
    console.log(key, obj[key]);
}

// 5.4 For-of loop
var arr = [1, 2, 3];
for (let value of arr) {
    console.log(value);
}

// 5.5 For-of with destructuring
var pairs = [[1, 2], [3, 4]];
for (let [first, second] of pairs) {
    console.log(first, second);
}

// 5.6 While loop
let whileCounter = 0;
while (whileCounter < 3) {
    let whileLocal = whileCounter;
    whileCounter++;
}

// 5.7 Do-while loop
let doCounter = 0;
do {
    let doLocal = doCounter;
    doCounter++;
} while (doCounter < 3);

// ============================================================================
// 6. TRY-CATCH-FINALLY
// ============================================================================

// 6.1 Catch parameter scope
try {
    throw new Error("test");
} catch (error) {
    console.log(error.message);
    let catchLocal = "in catch";
}

// 6.2 Catch with shadowing
var error = "global error";
try {
    throw new Error("thrown");
} catch (error) { // shadows global error
    console.log(error.message);
}
console.log(error); // "global error"

// 6.3 Finally block
try {
    let tryLocal = 1;
} catch (e) {
    let catchLocal = 2;
} finally {
    let finallyLocal = 3;
}

// ============================================================================
// 7. CLASSES
// ============================================================================

// 7.1 Class declaration
class MyClass {
    constructor(value) {
        this.value = value;
    }
    
    getValue() {
        return this.value;
    }
    
    static staticMethod(x) {
        return x * 2;
    }
}

// 7.2 Class expression
var ClassExpr = class InnerClassName {
    constructor() {
        // InnerClassName only visible inside
        console.log(InnerClassName.name);
    }
};

// 7.3 Class with private fields (if supported)
class PrivateClass {
    #privateField = 1;
    
    getPrivate() {
        return this.#privateField;
    }
}

// 7.4 Class inheritance
class ChildClass extends MyClass {
    constructor(value, extra) {
        super(value);
        this.extra = extra;
    }
}

// 7.5 Static block
class StaticBlockClass {
    static {
        let staticBlockLocal = 1;
        this.initialized = true;
    }
}

// ============================================================================
// 8. DESTRUCTURING
// ============================================================================

// 8.1 Array destructuring
var [first, second, ...rest] = [1, 2, 3, 4, 5];

// 8.2 Object destructuring
var { name, age, address: addr } = { name: "John", age: 30, address: "123 St" };

// 8.3 Nested destructuring
var { user: { id, profile: { bio } } } = { user: { id: 1, profile: { bio: "test" } } };

// 8.4 Destructuring with defaults
var { missing = "default" } = {};
var [firstWithDefault = 0] = [];

// 8.5 Destructuring in function parameters
function destructParams({ x, y }, [a, b]) {
    return x + y + a + b;
}

// 8.6 Destructuring in arrow functions
var arrowDestruct = ({ prop }) => prop * 2;

// ============================================================================
// 9. CLOSURES
// ============================================================================

// 9.1 Basic closure
function createCounter() {
    var count = 0;
    return function() {
        count++;
        return count;
    };
}

// 9.2 Closure capturing multiple variables
function createAdder(x) {
    var multiplier = 2;
    return function(y) {
        return (x + y) * multiplier;
    };
}

// 9.3 IIFE (Immediately Invoked Function Expression)
var iifeResult = (function(x) {
    var local = x * 2;
    return local;
})(5);

// 9.4 IIFE with arrow function
var arrowIife = ((x) => {
    let local = x + 1;
    return local;
})(10);

// ============================================================================
// 10. ASSIGNMENT TARGETS
// ============================================================================

// 10.1 Simple assignment
var assignTarget;
assignTarget = 1;

// 10.2 Compound assignment
var compound = 1;
compound += 2;
compound *= 3;

// 10.3 Destructuring assignment
var destA, destB;
[destA, destB] = [1, 2];
({ destA, destB } = { destA: 3, destB: 4 });

// 10.4 Update expressions
var updateVar = 0;
updateVar++;
++updateVar;
updateVar--;
--updateVar;

// ============================================================================
// 11. COMPLEX NESTED SCOPES (OBFUSCATION PATTERNS)
// ============================================================================

// 11.1 Deeply nested functions (common in obfuscated code)
var _0x1234 = function(_0x5678) {
    var _0x9abc = function(_0xdef0) {
        var _0x1111 = function(_0x2222) {
            return _0x2222 + _0xdef0 + _0x5678;
        };
        return _0x1111;
    };
    return _0x9abc;
};

// 11.2 Multiple same-named variables in different scopes
(function() {
    var a = 1;
    (function() {
        var a = 2;
        (function() {
            var a = 3;
            console.log(a); // 3
        })();
        console.log(a); // 2
    })();
    console.log(a); // 1
})();

// 11.3 Hoisting edge cases
function hoistTest() {
    console.log(hoisted); // undefined (var hoisting)
    var hoisted = 1;
    console.log(hoisted); // 1
    
    // let/const temporal dead zone
    // console.log(notHoisted); // would throw ReferenceError
    let notHoisted = 2;
}

// 11.4 Function hoisting
function funcHoistTest() {
    hoistedFunc(); // works due to function hoisting
    
    function hoistedFunc() {
        console.log("hoisted");
    }
}

// ============================================================================
// 12. MODULES (ES6)
// ============================================================================

// 12.1 Import bindings (if this were a module)
// import { imported } from './module';
// import defaultImport from './module';
// import * as namespace from './module';

// 12.2 Export bindings
// export var exportedVar = 1;
// export function exportedFunc() {}
// export { localVar as exportedAlias };

// ============================================================================
// 13. GENERATORS AND ASYNC
// ============================================================================

// 13.1 Generator function
function* generatorFunc(start) {
    let current = start;
    while (true) {
        yield current++;
    }
}

// 13.2 Async function
async function asyncFunc(url) {
    let response = await fetch(url);
    let data = await response.json();
    return data;
}

// 13.3 Async arrow function
var asyncArrow = async (x) => {
    let result = await Promise.resolve(x);
    return result;
};

// 13.4 For-await-of
async function asyncIteration(asyncIterable) {
    for await (let item of asyncIterable) {
        console.log(item);
    }
}

// ============================================================================
// 14. LABELS (rare but valid)
// ============================================================================

// 14.1 Labeled statement
outerLoop: for (let i = 0; i < 3; i++) {
    innerLoop: for (let j = 0; j < 3; j++) {
        if (i === 1 && j === 1) {
            break outerLoop;
        }
    }
}

// ============================================================================
// 15. WITH STATEMENT (deprecated but valid)
// ============================================================================

// 15.1 With statement (creates dynamic scope)
var withObj = { x: 1, y: 2 };
with (withObj) {
    // x and y refer to withObj.x and withObj.y
    console.log(x + y);
}

// ============================================================================
// 16. EVAL (dynamic scope modification)
// ============================================================================

// 16.1 Direct eval
function evalTest() {
    var localVar = 1;
    eval("var evalCreated = 2");
    // evalCreated might exist here depending on strict mode
}

// ============================================================================
// 17. GLOBAL REFERENCES (should NOT be renamed)
// ============================================================================

// These should NOT be renamed as they're global/builtin
console.log("test");
Math.random();
Object.keys({});
Array.isArray([]);
JSON.stringify({});
window.location; // browser global
document.body;   // browser global

// ============================================================================
// 18. PROPERTY ACCESS (should NOT rename property names)
// ============================================================================

var propObj = {
    propertyName: 1,
    method: function() {
        return this.propertyName;
    }
};

// These property accesses should NOT be renamed
propObj.propertyName;
propObj["propertyName"];
propObj.method();

// ============================================================================
// 19. COMPUTED PROPERTY NAMES
// ============================================================================

var computedKey = "dynamic";
var computedObj = {
    [computedKey]: 1,
    ["literal"]: 2,
    [1 + 2]: 3
};

// ============================================================================
// 20. SHORTHAND PROPERTIES
// ============================================================================

var shorthand = 1;
var shorthandObj = {
    shorthand,  // shorthand for shorthand: shorthand
    method() { return this.shorthand; }
};

// ============================================================================
// END OF TEST FILE
// ============================================================================

// Summary of what should be renamed:
// - All local variable declarations (var, let, const)
// - All function names (declarations and expressions)
// - All function parameters
// - All class names
// - All catch parameters
// - All loop variables
// - All destructuring bindings

// Summary of what should NOT be renamed:
// - Global/builtin references (console, Math, Object, etc.)
// - Property names in object literals
// - Property access (obj.prop, obj["prop"])
// - Labels
// - Import/export specifiers (external names)
