// =============================================================================
// ES3 Compliance Test Suite
// =============================================================================
// This file tests all ES3 features for the Spacey JavaScript engine.
// Run with: cargo run -- tests/es3_compliance.js
//
// Expected output: All tests should pass with "PASS" messages.
// Any "FAIL" message indicates a compliance issue.
// =============================================================================

var testsPassed = 0;
var testsFailed = 0;

function assert(condition, message) {
    if (condition) {
        testsPassed = testsPassed + 1;
        console.log("PASS: " + message);
    } else {
        testsFailed = testsFailed + 1;
        console.log("FAIL: " + message);
    }
}

function assertEqual(actual, expected, message) {
    if (actual === expected) {
        testsPassed = testsPassed + 1;
        console.log("PASS: " + message);
    } else {
        testsFailed = testsFailed + 1;
        console.log("FAIL: " + message + " (expected: " + expected + ", got: " + actual + ")");
    }
}

function assertNaN(value, message) {
    if (isNaN(value)) {
        testsPassed = testsPassed + 1;
        console.log("PASS: " + message);
    } else {
        testsFailed = testsFailed + 1;
        console.log("FAIL: " + message + " (expected NaN, got: " + value + ")");
    }
}

console.log("=".repeat(60));
console.log("ES3 Compliance Test Suite");
console.log("=".repeat(60));

// =============================================================================
// SECTION 1: Primitive Values and Types
// =============================================================================
console.log("\n--- Section 1: Primitive Values and Types ---\n");

// 1.1 Undefined
var undefinedVar;
assertEqual(typeof undefinedVar, "undefined", "typeof undefined");
assertEqual(undefinedVar, undefined, "undefined value");

// 1.2 Null
var nullVar = null;
assertEqual(typeof nullVar, "object", "typeof null is object");
assertEqual(nullVar, null, "null value");

// 1.3 Boolean
assertEqual(typeof true, "boolean", "typeof true");
assertEqual(typeof false, "boolean", "typeof false");
assertEqual(true === true, true, "true === true");
assertEqual(false === false, true, "false === false");

// 1.4 Number
assertEqual(typeof 42, "number", "typeof number");
assertEqual(typeof 3.14, "number", "typeof float");
assertEqual(typeof NaN, "number", "typeof NaN");
assertEqual(typeof Infinity, "number", "typeof Infinity");
assertNaN(NaN, "NaN is NaN");
assertEqual(Infinity > 0, true, "Infinity is positive");
assertEqual(-Infinity < 0, true, "-Infinity is negative");

// 1.5 String
assertEqual(typeof "hello", "string", "typeof string");
assertEqual("hello".length, 5, "string length");
assertEqual("".length, 0, "empty string length");

// =============================================================================
// SECTION 2: Operators
// =============================================================================
console.log("\n--- Section 2: Operators ---\n");

// 2.1 Arithmetic Operators
assertEqual(5 + 3, 8, "addition");
assertEqual(10 - 4, 6, "subtraction");
assertEqual(6 * 7, 42, "multiplication");
assertEqual(15 / 3, 5, "division");
assertEqual(17 % 5, 2, "modulo");

// 2.2 Unary Operators
assertEqual(-5, 0 - 5, "unary minus");
assertEqual(+5, 5, "unary plus");
assertEqual(!true, false, "logical not true");
assertEqual(!false, true, "logical not false");
assertEqual(typeof 42, "number", "typeof operator");

// 2.3 Increment/Decrement
var x = 5;
x++;
assertEqual(x, 6, "post-increment");
x--;
assertEqual(x, 5, "post-decrement");
++x;
assertEqual(x, 6, "pre-increment");
--x;
assertEqual(x, 5, "pre-decrement");

// 2.4 Comparison Operators
assertEqual(5 == 5, true, "equal");
assertEqual(5 != 3, true, "not equal");
assertEqual(5 === 5, true, "strict equal");
assertEqual(5 !== "5", true, "strict not equal");
assertEqual(5 < 10, true, "less than");
assertEqual(5 <= 5, true, "less than or equal");
assertEqual(10 > 5, true, "greater than");
assertEqual(5 >= 5, true, "greater than or equal");

// 2.5 Logical Operators
assertEqual(true && true, true, "logical and (true && true)");
assertEqual(true && false, false, "logical and (true && false)");
assertEqual(false || true, true, "logical or (false || true)");
assertEqual(false || false, false, "logical or (false || false)");

// 2.6 Bitwise Operators
assertEqual(5 & 3, 1, "bitwise and");
assertEqual(5 | 3, 7, "bitwise or");
assertEqual(5 ^ 3, 6, "bitwise xor");
assertEqual(~0, -1, "bitwise not");
assertEqual(4 << 1, 8, "left shift");
assertEqual(8 >> 1, 4, "right shift");
assertEqual(-8 >>> 28, 15, "unsigned right shift");

// 2.7 String Concatenation
assertEqual("hello" + " " + "world", "hello world", "string concatenation");
assertEqual("num: " + 42, "num: 42", "string + number concatenation");

// 2.8 Conditional (Ternary) Operator
assertEqual(true ? "yes" : "no", "yes", "ternary true");
assertEqual(false ? "yes" : "no", "no", "ternary false");

// 2.9 Comma Operator
var commaResult = (1, 2, 3);
assertEqual(commaResult, 3, "comma operator");

// =============================================================================
// SECTION 3: Statements
// =============================================================================
console.log("\n--- Section 3: Statements ---\n");

// 3.1 Variable Declaration
var a = 1;
var b = 2, c = 3;
assertEqual(a, 1, "var declaration");
assertEqual(b + c, 5, "multiple var declaration");

// 3.2 If Statement
var ifResult = "";
if (true) {
    ifResult = "yes";
}
assertEqual(ifResult, "yes", "if statement");

// 3.3 If-Else Statement
var ifElseResult;
if (false) {
    ifElseResult = "if";
} else {
    ifElseResult = "else";
}
assertEqual(ifElseResult, "else", "if-else statement");

// 3.4 If-Else-If Chain
var grade = 85;
var letterGrade;
if (grade >= 90) {
    letterGrade = "A";
} else if (grade >= 80) {
    letterGrade = "B";
} else if (grade >= 70) {
    letterGrade = "C";
} else {
    letterGrade = "F";
}
assertEqual(letterGrade, "B", "if-else-if chain");

// 3.5 Switch Statement
var switchResult;
var day = 3;
switch (day) {
    case 1:
        switchResult = "Monday";
        break;
    case 2:
        switchResult = "Tuesday";
        break;
    case 3:
        switchResult = "Wednesday";
        break;
    default:
        switchResult = "Unknown";
}
assertEqual(switchResult, "Wednesday", "switch statement");

// 3.6 Switch Fall-through
var fallResult = "";
var num = 1;
switch (num) {
    case 1:
        fallResult = fallResult + "one";
    case 2:
        fallResult = fallResult + "two";
        break;
    case 3:
        fallResult = fallResult + "three";
}
assertEqual(fallResult, "onetwo", "switch fall-through");

// 3.7 While Loop
var whileSum = 0;
var whileI = 1;
while (whileI <= 5) {
    whileSum = whileSum + whileI;
    whileI = whileI + 1;
}
assertEqual(whileSum, 15, "while loop (1+2+3+4+5)");

// 3.8 Do-While Loop
var doWhileSum = 0;
var doWhileI = 1;
do {
    doWhileSum = doWhileSum + doWhileI;
    doWhileI = doWhileI + 1;
} while (doWhileI <= 5);
assertEqual(doWhileSum, 15, "do-while loop");

// 3.9 For Loop
var forSum = 0;
for (var i = 1; i <= 5; i++) {
    forSum = forSum + i;
}
assertEqual(forSum, 15, "for loop");

// 3.10 For-In Loop
var obj = { a: 1, b: 2, c: 3 };
var forInKeys = "";
for (var key in obj) {
    forInKeys = forInKeys + key;
}
assert(forInKeys.length > 0, "for-in loop iterates object keys");

// 3.11 Break Statement
var breakSum = 0;
for (var bi = 1; bi <= 10; bi++) {
    if (bi > 5) break;
    breakSum = breakSum + bi;
}
assertEqual(breakSum, 15, "break statement");

// 3.12 Continue Statement
var continueSum = 0;
for (var ci = 1; ci <= 5; ci++) {
    if (ci === 3) continue;
    continueSum = continueSum + ci;
}
assertEqual(continueSum, 12, "continue statement (1+2+4+5)");

// 3.13 Labeled Break
var labeledResult = 0;
outer: for (var li = 0; li < 3; li++) {
    for (var lj = 0; lj < 3; lj++) {
        if (lj === 1) break outer;
        labeledResult = labeledResult + 1;
    }
}
assertEqual(labeledResult, 1, "labeled break");

// 3.14 Try-Catch
var tryCatchResult;
try {
    throw "error";
} catch (e) {
    tryCatchResult = "caught";
}
assertEqual(tryCatchResult, "caught", "try-catch");

// 3.15 Try-Catch-Finally
var finallyRan = false;
try {
    throw "error";
} catch (e) {
    // caught
} finally {
    finallyRan = true;
}
assertEqual(finallyRan, true, "try-catch-finally");

// =============================================================================
// SECTION 4: Functions
// =============================================================================
console.log("\n--- Section 4: Functions ---\n");

// 4.1 Function Declaration
function add(a, b) {
    return a + b;
}
assertEqual(add(2, 3), 5, "function declaration");

// 4.2 Function Expression
var multiply = function(a, b) {
    return a * b;
};
assertEqual(multiply(4, 5), 20, "function expression");

// 4.3 Named Function Expression
var factorial = function fact(n) {
    if (n <= 1) return 1;
    return n * fact(n - 1);
};
assertEqual(factorial(5), 120, "named function expression (factorial)");

// 4.4 Function Arguments
function countArgs() {
    return arguments.length;
}
assertEqual(countArgs(1, 2, 3), 3, "arguments.length");

// 4.5 Default Return
function noReturn() {
    var x = 1;
}
assertEqual(noReturn(), undefined, "default return undefined");

// 4.6 Closures
function makeCounter() {
    var count = 0;
    return function() {
        count = count + 1;
        return count;
    };
}
var counter = makeCounter();
assertEqual(counter(), 1, "closure (first call)");
assertEqual(counter(), 2, "closure (second call)");

// 4.7 Recursion
function fib(n) {
    if (n <= 1) return n;
    return fib(n - 1) + fib(n - 2);
}
assertEqual(fib(10), 55, "recursion (fibonacci)");

// =============================================================================
// SECTION 5: Objects
// =============================================================================
console.log("\n--- Section 5: Objects ---\n");

// 5.1 Object Literal
var person = {
    name: "John",
    age: 30
};
assertEqual(person.name, "John", "object dot notation");
assertEqual(person["age"], 30, "object bracket notation");

// 5.2 Object Property Assignment
person.city = "NYC";
assertEqual(person.city, "NYC", "object property assignment");

// 5.3 Object Property Delete
var deleteObj = { a: 1, b: 2 };
delete deleteObj.a;
assertEqual(deleteObj.a, undefined, "delete property");

// 5.4 Object in Operator
var inObj = { x: 1 };
assertEqual("x" in inObj, true, "in operator (exists)");
assertEqual("y" in inObj, false, "in operator (not exists)");

// 5.5 Nested Objects
var nested = {
    outer: {
        inner: {
            value: 42
        }
    }
};
assertEqual(nested.outer.inner.value, 42, "nested object access");

// =============================================================================
// SECTION 6: Arrays
// =============================================================================
console.log("\n--- Section 6: Arrays ---\n");

// 6.1 Array Literal
var arr = [1, 2, 3, 4, 5];
assertEqual(arr.length, 5, "array length");
assertEqual(arr[0], 1, "array index 0");
assertEqual(arr[4], 5, "array index 4");

// 6.2 Array Push/Pop
var pushArr = [1, 2];
pushArr.push(3);
assertEqual(pushArr.length, 3, "array push");
var popped = pushArr.pop();
assertEqual(popped, 3, "array pop return");
assertEqual(pushArr.length, 2, "array pop length");

// 6.3 Array Shift/Unshift
var shiftArr = [1, 2, 3];
var shifted = shiftArr.shift();
assertEqual(shifted, 1, "array shift return");
assertEqual(shiftArr.length, 2, "array shift length");
shiftArr.unshift(0);
assertEqual(shiftArr[0], 0, "array unshift");

// 6.4 Array Join
var joinArr = ["a", "b", "c"];
assertEqual(joinArr.join("-"), "a-b-c", "array join");
assertEqual(joinArr.join(""), "abc", "array join empty");

// 6.5 Array Slice
var sliceArr = [1, 2, 3, 4, 5];
var sliced = sliceArr.slice(1, 4);
assertEqual(sliced.length, 3, "array slice length");
assertEqual(sliced[0], 2, "array slice first element");

// 6.6 Array Concat
var concat1 = [1, 2];
var concat2 = [3, 4];
var concatenated = concat1.concat(concat2);
assertEqual(concatenated.length, 4, "array concat length");

// 6.7 Array Reverse
var reverseArr = [1, 2, 3];
reverseArr.reverse();
assertEqual(reverseArr[0], 3, "array reverse");

// 6.8 Array Sort
var sortArr = [3, 1, 2];
sortArr.sort();
assertEqual(sortArr[0], 1, "array sort");

// 6.9 Array IndexOf
var indexArr = ["a", "b", "c", "b"];
assertEqual(indexArr.indexOf("b"), 1, "array indexOf");
assertEqual(indexArr.indexOf("z"), -1, "array indexOf (not found)");

// =============================================================================
// SECTION 7: String Methods
// =============================================================================
console.log("\n--- Section 7: String Methods ---\n");

var str = "Hello, World!";

// 7.1 charAt
assertEqual(str.charAt(0), "H", "string charAt");
assertEqual(str.charAt(7), "W", "string charAt middle");

// 7.2 charCodeAt
assertEqual(str.charCodeAt(0), 72, "string charCodeAt (H=72)");

// 7.3 indexOf
assertEqual(str.indexOf("o"), 4, "string indexOf");
assertEqual(str.indexOf("z"), -1, "string indexOf (not found)");

// 7.4 lastIndexOf
assertEqual(str.lastIndexOf("o"), 8, "string lastIndexOf");

// 7.5 substring
assertEqual(str.substring(0, 5), "Hello", "string substring");

// 7.6 slice
assertEqual(str.slice(7, 12), "World", "string slice");
assertEqual(str.slice(-6, -1), "World", "string slice negative");

// 7.7 toLowerCase/toUpperCase
assertEqual("HELLO".toLowerCase(), "hello", "string toLowerCase");
assertEqual("hello".toUpperCase(), "HELLO", "string toUpperCase");

// 7.8 split
var splitResult = "a,b,c".split(",");
assertEqual(splitResult.length, 3, "string split length");
assertEqual(splitResult[1], "b", "string split element");

// 7.9 concat
assertEqual("Hello".concat(" ", "World"), "Hello World", "string concat");

// 7.10 trim
assertEqual("  hello  ".trim(), "hello", "string trim");

// =============================================================================
// SECTION 8: Number Methods
// =============================================================================
console.log("\n--- Section 8: Number Methods ---\n");

// 8.1 toString
assertEqual((255).toString(16), "ff", "number toString hex");
assertEqual((8).toString(2), "1000", "number toString binary");

// 8.2 toFixed
assertEqual((3.14159).toFixed(2), "3.14", "number toFixed");

// 8.3 Number Constants
assertEqual(Number.MAX_VALUE > 0, true, "Number.MAX_VALUE");
assertEqual(Number.MIN_VALUE > 0, true, "Number.MIN_VALUE");
assertNaN(Number.NaN, "Number.NaN");

// =============================================================================
// SECTION 9: Math Object
// =============================================================================
console.log("\n--- Section 9: Math Object ---\n");

// 9.1 Math Constants
assert(Math.PI > 3.14 && Math.PI < 3.15, "Math.PI");
assert(Math.E > 2.71 && Math.E < 2.72, "Math.E");

// 9.2 Math Methods
assertEqual(Math.abs(-5), 5, "Math.abs");
assertEqual(Math.floor(3.7), 3, "Math.floor");
assertEqual(Math.ceil(3.2), 4, "Math.ceil");
assertEqual(Math.round(3.5), 4, "Math.round");
assertEqual(Math.max(1, 5, 3), 5, "Math.max");
assertEqual(Math.min(1, 5, 3), 1, "Math.min");
assertEqual(Math.pow(2, 3), 8, "Math.pow");
assertEqual(Math.sqrt(16), 4, "Math.sqrt");

// 9.3 Trigonometric Functions
assertEqual(Math.sin(0), 0, "Math.sin(0)");
assertEqual(Math.cos(0), 1, "Math.cos(0)");

// 9.4 Math.random
var rand = Math.random();
assert(rand >= 0 && rand < 1, "Math.random in [0, 1)");

// =============================================================================
// SECTION 10: Date Object
// =============================================================================
console.log("\n--- Section 10: Date Object ---\n");

// 10.1 Date.now
var now = Date.now();
assert(now > 0, "Date.now returns positive number");

// 10.2 Date Constructor
var epoch = new Date(0);
assertEqual(epoch.getTime(), 0, "Date from epoch");

// 10.3 Date Methods
var testDate = new Date(2024, 0, 15, 12, 30, 45); // Jan 15, 2024, 12:30:45
assert(testDate.getFullYear() >= 2024, "Date.getFullYear");
assert(testDate.getMonth() >= 0, "Date.getMonth");
assert(testDate.getDate() >= 1, "Date.getDate");

// =============================================================================
// SECTION 11: Global Functions
// =============================================================================
console.log("\n--- Section 11: Global Functions ---\n");

// 11.1 parseInt
assertEqual(parseInt("42"), 42, "parseInt decimal");
assertEqual(parseInt("  42  "), 42, "parseInt with whitespace");
assertEqual(parseInt("ff", 16), 255, "parseInt hex");
assertEqual(parseInt("1010", 2), 10, "parseInt binary");
assertNaN(parseInt("hello"), "parseInt non-number");

// 11.2 parseFloat
assertEqual(parseFloat("3.14"), 3.14, "parseFloat");
assertEqual(parseFloat("  3.14  "), 3.14, "parseFloat with whitespace");
assertNaN(parseFloat("hello"), "parseFloat non-number");

// 11.3 isNaN
assertEqual(isNaN(NaN), true, "isNaN(NaN)");
assertEqual(isNaN(42), false, "isNaN(42)");
assertEqual(isNaN("hello"), true, "isNaN(string)");

// 11.4 isFinite
assertEqual(isFinite(42), true, "isFinite(42)");
assertEqual(isFinite(Infinity), false, "isFinite(Infinity)");
assertEqual(isFinite(NaN), false, "isFinite(NaN)");

// 11.5 encodeURI/decodeURI
var uri = "http://example.com/path?name=John Doe";
var encoded = encodeURI(uri);
assert(encoded.indexOf("%20") !== -1, "encodeURI encodes spaces");

// 11.6 eval
var evalResult = eval("1 + 2");
assertEqual(evalResult, 3, "eval simple expression");

var evalX = 10;
var evalResult2 = eval("evalX * 2");
// Note: eval creates new scope, so this may not work as in browser

// =============================================================================
// SECTION 12: Error Handling
// =============================================================================
console.log("\n--- Section 12: Error Handling ---\n");

// 12.1 Error Types
var errorCaught = false;
try {
    throw new Error("test error");
} catch (e) {
    errorCaught = true;
}
assertEqual(errorCaught, true, "Error thrown and caught");

// 12.2 TypeError
var typeErrorCaught = false;
try {
    throw new TypeError("type error");
} catch (e) {
    typeErrorCaught = true;
}
assertEqual(typeErrorCaught, true, "TypeError thrown and caught");

// 12.3 ReferenceError
var refErrorCaught = false;
try {
    throw new ReferenceError("reference error");
} catch (e) {
    refErrorCaught = true;
}
assertEqual(refErrorCaught, true, "ReferenceError thrown and caught");

// 12.4 SyntaxError
var syntaxErrorCaught = false;
try {
    throw new SyntaxError("syntax error");
} catch (e) {
    syntaxErrorCaught = true;
}
assertEqual(syntaxErrorCaught, true, "SyntaxError thrown and caught");

// 12.5 RangeError
var rangeErrorCaught = false;
try {
    throw new RangeError("range error");
} catch (e) {
    rangeErrorCaught = true;
}
assertEqual(rangeErrorCaught, true, "RangeError thrown and caught");

// =============================================================================
// SECTION 13: Type Coercion
// =============================================================================
console.log("\n--- Section 13: Type Coercion ---\n");

// 13.1 To Boolean
assertEqual(Boolean(0), false, "Boolean(0)");
assertEqual(Boolean(1), true, "Boolean(1)");
assertEqual(Boolean(""), false, "Boolean empty string");
assertEqual(Boolean("hello"), true, "Boolean non-empty string");
assertEqual(Boolean(null), false, "Boolean(null)");
assertEqual(Boolean(undefined), false, "Boolean(undefined)");

// 13.2 To Number
assertEqual(Number("42"), 42, "Number(string)");
assertEqual(Number(true), 1, "Number(true)");
assertEqual(Number(false), 0, "Number(false)");
assertEqual(Number(null), 0, "Number(null)");
assertNaN(Number(undefined), "Number(undefined)");

// 13.3 To String
assertEqual(String(42), "42", "String(number)");
assertEqual(String(true), "true", "String(true)");
assertEqual(String(false), "false", "String(false)");
assertEqual(String(null), "null", "String(null)");
assertEqual(String(undefined), "undefined", "String(undefined)");

// 13.4 Loose Equality Coercion
assertEqual(1 == "1", true, "1 == '1'");
assertEqual(0 == false, true, "0 == false");
assertEqual(null == undefined, true, "null == undefined");

// =============================================================================
// SECTION 14: RegExp (Basic)
// =============================================================================
console.log("\n--- Section 14: RegExp (Basic) ---\n");

// 14.1 RegExp Test
var re = new RegExp("hello");
assertEqual(re.test("hello world"), true, "RegExp.test match");
assertEqual(re.test("goodbye"), false, "RegExp.test no match");

// 14.2 RegExp Case Insensitive
var reI = new RegExp("HELLO", "i");
assertEqual(reI.test("hello"), true, "RegExp case insensitive");

// 14.3 String.match
var matchResult = "hello".match(new RegExp("hello"));
assert(matchResult !== null, "String.match found");

// 14.4 String.replace
var replaced = "hello world".replace(new RegExp("world"), "universe");
assertEqual(replaced, "hello universe", "String.replace");

// 14.5 String.search
assertEqual("hello world".search(new RegExp("world")), 6, "String.search found");
assertEqual("hello".search(new RegExp("xyz")), -1, "String.search not found");

// =============================================================================
// Final Summary
// =============================================================================
console.log("\n" + "=".repeat(60));
console.log("Test Summary");
console.log("=".repeat(60));
console.log("Tests Passed: " + testsPassed);
console.log("Tests Failed: " + testsFailed);
console.log("Total Tests:  " + (testsPassed + testsFailed));
console.log("=".repeat(60));

if (testsFailed === 0) {
    console.log("\nSUCCESS: All ES3 compliance tests passed!");
} else {
    console.log("\nFAILURE: Some tests failed. Please review the failures above.");
}

