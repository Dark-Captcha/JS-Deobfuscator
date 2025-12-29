// Test file for ControlFlowDeflattener
// Contains various control flow flattening patterns

// ============================================
// Pattern 1: Simple while+switch state machine
// ============================================
function simpleFlattened() {
    var state = 1;
    var result = 0;
    
    while (true) {
        switch (state) {
            case 1:
                result = 10;
                state = 2;
                break;
            case 2:
                result = result + 5;
                state = 3;
                break;
            case 3:
                result = result * 2;
                state = 4;
                break;
            case 4:
                return result;
        }
    }
}

// ============================================
// Pattern 2: For loop with switch
// ============================================
function forLoopFlattened() {
    var state = 0;
    var x = 0;
    
    for (;;) {
        switch (state) {
            case 0:
                x = 1;
                state = 1;
                break;
            case 1:
                x = x + 1;
                state = 2;
                break;
            case 2:
                return x;
        }
    }
}

// ============================================
// Pattern 3: Array-based state variable
// ============================================
function arrayStateFlattened() {
    var arr = [1, 0, 0];
    var result = "";
    
    while (true) {
        switch (arr[0]) {
            case 1:
                result = "hello";
                arr[0] = 2;
                break;
            case 2:
                result = result + " world";
                arr[0] = 3;
                break;
            case 3:
                return result;
        }
    }
}

// ============================================
// Pattern 4: Conditional transitions
// ============================================
function conditionalFlattened(input) {
    var state = 1;
    var result = 0;
    
    while (true) {
        switch (state) {
            case 1:
                result = input;
                state = input > 0 ? 2 : 3;
                break;
            case 2:
                result = result * 2;
                state = 4;
                break;
            case 3:
                result = result * -1;
                state = 4;
                break;
            case 4:
                return result;
        }
    }
}

// ============================================
// Pattern 5: Computed case labels
// ============================================
function computedLabelsFlattened() {
    var state = 10;
    var result = 0;
    
    while (true) {
        switch (state) {
            case 5 + 5:  // 10
                result = 1;
                state = 20;
                break;
            case 10 * 2:  // 20
                result = result + 2;
                state = 30;
                break;
            case 15 + 15:  // 30
                return result;
        }
    }
}

// ============================================
// Pattern 6: Do-while loop
// ============================================
function doWhileFlattened() {
    var state = 1;
    var x = 0;
    
    do {
        switch (state) {
            case 1:
                x = 5;
                state = 2;
                break;
            case 2:
                x = x * 2;
                state = 3;
                break;
            case 3:
                return x;
        }
    } while (true);
}

// ============================================
// Pattern 7: String state values
// ============================================
function stringStateFlattened() {
    var state = "start";
    var result = 0;
    
    while (true) {
        switch (state) {
            case "start":
                result = 1;
                state = "middle";
                break;
            case "middle":
                result = result + 1;
                state = "end";
                break;
            case "end":
                return result;
        }
    }
}

// ============================================
// Non-flattened code (should not be modified)
// ============================================
function normalCode() {
    var x = 1;
    var y = 2;
    return x + y;
}

function normalSwitch(value) {
    switch (value) {
        case 1:
            return "one";
        case 2:
            return "two";
        default:
            return "other";
    }
}

function normalLoop() {
    var sum = 0;
    for (var i = 0; i < 10; i++) {
        sum += i;
    }
    return sum;
}

// Export for testing
console.log("simpleFlattened:", simpleFlattened());
console.log("forLoopFlattened:", forLoopFlattened());
console.log("arrayStateFlattened:", arrayStateFlattened());
console.log("conditionalFlattened(5):", conditionalFlattened(5));
console.log("conditionalFlattened(-5):", conditionalFlattened(-5));
console.log("computedLabelsFlattened:", computedLabelsFlattened());
console.log("doWhileFlattened:", doWhileFlattened());
console.log("stringStateFlattened:", stringStateFlattened());
console.log("normalCode:", normalCode());
console.log("normalSwitch(1):", normalSwitch(1));
console.log("normalLoop:", normalLoop());
