# ECMAScript 3 (ES3) Compatibility Checklist

This document outlines all features required for full ES3 (ECMA-262 3rd Edition, December 1999) compliance. ES3 is the foundational JavaScript version and represents the core language features that all subsequent versions build upon.

**Reference**: [ECMA-262 3rd Edition](https://www.ecma-international.org/wp-content/uploads/ECMA-262_3rd_edition_december_1999.pdf)

---

## Table of Contents

1. [Lexical Grammar](#1-lexical-grammar)
2. [Types](#2-types)
3. [Type Conversion](#3-type-conversion)
4. [Expressions](#4-expressions)
5. [Statements](#5-statements)
6. [Functions](#6-functions)
7. [Built-in Objects](#7-built-in-objects)
8. [Error Handling](#8-error-handling)

---

## 1. Lexical Grammar

### 1.1 Source Text
- [ ] Unicode source text support
- [ ] Line terminators: LF (U+000A), CR (U+000D), LS (U+2028), PS (U+2029)
- [ ] White space characters: TAB, VT, FF, SP, NBSP, USP

### 1.2 Comments
- [ ] Single-line comments (`//`)
- [ ] Multi-line comments (`/* */`)

### 1.3 Tokens
- [ ] Identifiers
  - [ ] Unicode letters and digits
  - [ ] `$` and `_` allowed
  - [ ] Unicode escape sequences (`\uXXXX`)
- [ ] Reserved words:
  - [ ] `break`, `case`, `catch`, `continue`, `default`
  - [ ] `delete`, `do`, `else`, `finally`, `for`
  - [ ] `function`, `if`, `in`, `instanceof`, `new`
  - [ ] `return`, `switch`, `this`, `throw`, `try`
  - [ ] `typeof`, `var`, `void`, `while`, `with`
- [ ] Future reserved words:
  - [ ] `abstract`, `boolean`, `byte`, `char`, `class`
  - [ ] `const`, `debugger`, `double`, `enum`, `export`
  - [ ] `extends`, `final`, `float`, `goto`, `implements`
  - [ ] `import`, `int`, `interface`, `long`, `native`
  - [ ] `package`, `private`, `protected`, `public`, `short`
  - [ ] `static`, `super`, `synchronized`, `throws`, `transient`
  - [ ] `volatile`

### 1.4 Literals
- [ ] Null literal (`null`)
- [ ] Boolean literals (`true`, `false`)
- [ ] Numeric literals:
  - [ ] Decimal integers
  - [ ] Decimal with fraction
  - [ ] Exponential notation (`e`, `E`)
  - [ ] Hexadecimal integers (`0x`, `0X`)
  - [ ] Octal literals (`0` prefix)
- [ ] String literals:
  - [ ] Single-quoted strings (`'...'`)
  - [ ] Double-quoted strings (`"..."`)
  - [ ] Escape sequences: `\b`, `\t`, `\n`, `\v`, `\f`, `\r`, `\"`, `\'`, `\\`
  - [ ] Unicode escapes (`\uXXXX`)
  - [ ] Hex escapes (`\xXX`)
  - [ ] Octal escapes (`\0` through `\377`)
  - [ ] Line continuation (`\` followed by line terminator)
- [ ] Regular expression literals (`/pattern/flags`)
  - [ ] Flags: `g`, `i`, `m`

### 1.5 Automatic Semicolon Insertion (ASI)
- [ ] ASI at end of input stream
- [ ] ASI before `}`
- [ ] ASI when line terminator precedes restricted token
- [ ] Restricted productions:
  - [ ] `return [no LineTerminator] Expression`
  - [ ] `throw [no LineTerminator] Expression`
  - [ ] `break [no LineTerminator] Identifier`
  - [ ] `continue [no LineTerminator] Identifier`
  - [ ] Postfix `++` and `--`

---

## 2. Types

### 2.1 Primitive Types
- [ ] Undefined
- [ ] Null
- [ ] Boolean
- [ ] Number (IEEE 754 double-precision 64-bit)
  - [ ] `NaN` (Not-a-Number)
  - [ ] `Infinity`, `-Infinity`
  - [ ] Positive and negative zero (`+0`, `-0`)
  - [ ] Range: ±1.7976931348623157 × 10^308
  - [ ] Smallest positive: 5 × 10^-324
- [ ] String (sequence of 16-bit unsigned integers)

### 2.2 Object Type
- [ ] Property access
- [ ] Internal properties:
  - [ ] `[[Prototype]]`
  - [ ] `[[Class]]`
  - [ ] `[[Value]]` (for primitive wrapper objects)
  - [ ] `[[Get]]`
  - [ ] `[[Put]]`
  - [ ] `[[CanPut]]`
  - [ ] `[[HasProperty]]`
  - [ ] `[[Delete]]`
  - [ ] `[[DefaultValue]]`
  - [ ] `[[Construct]]` (for constructors)
  - [ ] `[[Call]]` (for functions)
  - [ ] `[[HasInstance]]` (for functions)
  - [ ] `[[Scope]]` (for functions)
  - [ ] `[[Match]]` (for RegExp)

### 2.3 Reference Type (Internal)
- [ ] Base object
- [ ] Property name
- [ ] GetValue operation
- [ ] PutValue operation

---

## 3. Type Conversion

### 3.1 Abstract Operations
- [ ] `ToPrimitive(input [, PreferredType])`
- [ ] `ToBoolean(argument)`
- [ ] `ToNumber(argument)`
- [ ] `ToInteger(argument)`
- [ ] `ToInt32(argument)` (32-bit signed integer)
- [ ] `ToUint32(argument)` (32-bit unsigned integer)
- [ ] `ToUint16(argument)` (16-bit unsigned integer)
- [ ] `ToString(argument)`
- [ ] `ToObject(argument)`

---

## 4. Expressions

### 4.1 Primary Expressions
- [ ] `this`
- [ ] Identifier reference
- [ ] Literal (null, boolean, numeric, string)
- [ ] Array initializer
  - [ ] Empty array `[]`
  - [ ] Array with elements `[1, 2, 3]`
  - [ ] Array with holes (elision) `[1, , 3]`
- [ ] Object initializer
  - [ ] Empty object `{}`
  - [ ] Property: identifier name
  - [ ] Property: string name
  - [ ] Property: numeric name
- [ ] Grouping operator `(expression)`

### 4.2 Left-Hand-Side Expressions
- [ ] Property accessors
  - [ ] Dot notation `obj.property`
  - [ ] Bracket notation `obj["property"]`
- [ ] `new` expression
  - [ ] `new Constructor`
  - [ ] `new Constructor(args)`
- [ ] Function call
  - [ ] `func()`
  - [ ] `func(arg1, arg2, ...)`

### 4.3 Postfix Expressions
- [ ] `expression++` (postfix increment)
- [ ] `expression--` (postfix decrement)

### 4.4 Unary Expressions
- [ ] `delete expression`
- [ ] `void expression`
- [ ] `typeof expression`
- [ ] `++expression` (prefix increment)
- [ ] `--expression` (prefix decrement)
- [ ] `+expression` (unary plus)
- [ ] `-expression` (unary minus)
- [ ] `~expression` (bitwise NOT)
- [ ] `!expression` (logical NOT)

### 4.5 Multiplicative Operators
- [ ] `*` (multiplication)
- [ ] `/` (division)
- [ ] `%` (remainder/modulo)

### 4.6 Additive Operators
- [ ] `+` (addition / string concatenation)
- [ ] `-` (subtraction)

### 4.7 Bitwise Shift Operators
- [ ] `<<` (left shift)
- [ ] `>>` (signed right shift)
- [ ] `>>>` (unsigned right shift)

### 4.8 Relational Operators
- [ ] `<` (less than)
- [ ] `>` (greater than)
- [ ] `<=` (less than or equal)
- [ ] `>=` (greater than or equal)
- [ ] `instanceof`
- [ ] `in`

### 4.9 Equality Operators
- [ ] `==` (abstract equality)
- [ ] `!=` (abstract inequality)
- [ ] `===` (strict equality)
- [ ] `!==` (strict inequality)

### 4.10 Binary Bitwise Operators
- [ ] `&` (bitwise AND)
- [ ] `^` (bitwise XOR)
- [ ] `|` (bitwise OR)

### 4.11 Binary Logical Operators
- [ ] `&&` (logical AND) - short-circuit evaluation
- [ ] `||` (logical OR) - short-circuit evaluation

### 4.12 Conditional Operator
- [ ] `condition ? trueExpr : falseExpr`

### 4.13 Assignment Operators
- [ ] `=` (simple assignment)
- [ ] `*=` (multiplication assignment)
- [ ] `/=` (division assignment)
- [ ] `%=` (remainder assignment)
- [ ] `+=` (addition assignment)
- [ ] `-=` (subtraction assignment)
- [ ] `<<=` (left shift assignment)
- [ ] `>>=` (signed right shift assignment)
- [ ] `>>>=` (unsigned right shift assignment)
- [ ] `&=` (bitwise AND assignment)
- [ ] `^=` (bitwise XOR assignment)
- [ ] `|=` (bitwise OR assignment)

### 4.14 Comma Operator
- [ ] `expression1, expression2`

---

## 5. Statements

### 5.1 Block Statement
- [ ] `{ StatementList }`
- [ ] Empty block `{}`

### 5.2 Variable Statement
- [ ] `var` declarations
- [ ] Multiple declarations `var a, b, c`
- [ ] Initializers `var a = 1`
- [ ] Variable hoisting

### 5.3 Empty Statement
- [ ] `;`

### 5.4 Expression Statement
- [ ] Expression followed by `;`
- [ ] Lookahead restriction (not `{` or `function`)

### 5.5 The `if` Statement
- [ ] `if (expression) statement`
- [ ] `if (expression) statement else statement`
- [ ] Dangling else resolution

### 5.6 Iteration Statements
- [ ] `do statement while (expression);`
- [ ] `while (expression) statement`
- [ ] `for (expressionOpt; expressionOpt; expressionOpt) statement`
- [ ] `for (var declarationList; expressionOpt; expressionOpt) statement`
- [ ] `for (leftHandSide in expression) statement`
- [ ] `for (var declaration in expression) statement`

### 5.7 The `continue` Statement
- [ ] `continue;`
- [ ] `continue Identifier;`

### 5.8 The `break` Statement
- [ ] `break;`
- [ ] `break Identifier;`

### 5.9 The `return` Statement
- [ ] `return;`
- [ ] `return expression;`

### 5.10 The `with` Statement
- [ ] `with (expression) statement`

### 5.11 The `switch` Statement
- [ ] `switch (expression) { CaseClauses }`
- [ ] `case expression:`
- [ ] `default:`
- [ ] Fall-through behavior

### 5.12 Labelled Statements
- [ ] `Identifier: Statement`
- [ ] Nested labels

### 5.13 The `throw` Statement
- [ ] `throw expression;`

### 5.14 The `try` Statement
- [ ] `try Block Catch`
- [ ] `try Block Finally`
- [ ] `try Block Catch Finally`
- [ ] Catch clause: `catch (Identifier) Block`

---

## 6. Functions

### 6.1 Function Definitions
- [ ] Function declaration: `function name(params) { body }`
- [ ] Function expression: `function(params) { body }`
- [ ] Named function expression: `function name(params) { body }`
- [ ] Function hoisting (declarations only)

### 6.2 Formal Parameters
- [ ] Parameter list
- [ ] Duplicate parameter names (allowed in ES3)

### 6.3 Function Properties
- [ ] `length` (number of formal parameters)
- [ ] `prototype` property
- [ ] `[[Call]]` internal method
- [ ] `[[Construct]]` internal method
- [ ] `[[HasInstance]]` internal method

### 6.4 `arguments` Object
- [ ] `arguments.length`
- [ ] Indexed access `arguments[i]`
- [ ] `arguments.callee`
- [ ] Arguments-parameters binding (live binding)

### 6.5 Variable Instantiation
- [ ] Function-level scope
- [ ] Hoisting of `var` declarations
- [ ] Hoisting of function declarations
- [ ] Declaration order precedence

---

## 7. Built-in Objects

### 7.1 Global Object

#### Properties
- [ ] `NaN`
- [ ] `Infinity`
- [ ] `undefined`

#### Functions
- [ ] `eval(x)`
- [ ] `parseInt(string, radix)`
- [ ] `parseFloat(string)`
- [ ] `isNaN(number)`
- [ ] `isFinite(number)`
- [ ] `decodeURI(encodedURI)`
- [ ] `decodeURIComponent(encodedURIComponent)`
- [ ] `encodeURI(uri)`
- [ ] `encodeURIComponent(uriComponent)`

### 7.2 Object Objects

#### Constructor
- [ ] `Object()` called as function
- [ ] `new Object()` called as constructor
- [ ] `new Object(value)` type conversion

#### Properties
- [ ] `Object.prototype`

#### Prototype Methods
- [ ] `Object.prototype.constructor`
- [ ] `Object.prototype.toString()`
- [ ] `Object.prototype.toLocaleString()`
- [ ] `Object.prototype.valueOf()`
- [ ] `Object.prototype.hasOwnProperty(V)`
- [ ] `Object.prototype.isPrototypeOf(V)`
- [ ] `Object.prototype.propertyIsEnumerable(V)`

### 7.3 Function Objects

#### Constructor
- [ ] `Function(p1, p2, ..., body)` called as function
- [ ] `new Function(p1, p2, ..., body)` called as constructor

#### Properties
- [ ] `Function.prototype`
- [ ] `Function.length` (value: 1)

#### Prototype Methods
- [ ] `Function.prototype.constructor`
- [ ] `Function.prototype.toString()`
- [ ] `Function.prototype.apply(thisArg, argArray)`
- [ ] `Function.prototype.call(thisArg [, arg1 [, arg2, ...]])`

#### Instance Properties
- [ ] `length`
- [ ] `prototype`

### 7.4 Array Objects

#### Constructor
- [ ] `Array()` called as function
- [ ] `new Array()` - empty array
- [ ] `new Array(len)` - array with length
- [ ] `new Array(element0, element1, ...)` - array with elements

#### Properties
- [ ] `Array.prototype`

#### Prototype Methods
- [ ] `Array.prototype.constructor`
- [ ] `Array.prototype.toString()`
- [ ] `Array.prototype.toLocaleString()`
- [ ] `Array.prototype.concat([item1 [, item2 [, ...]]])`
- [ ] `Array.prototype.join(separator)`
- [ ] `Array.prototype.pop()`
- [ ] `Array.prototype.push([item1 [, item2 [, ...]]])`
- [ ] `Array.prototype.reverse()`
- [ ] `Array.prototype.shift()`
- [ ] `Array.prototype.slice(start, end)`
- [ ] `Array.prototype.sort(comparefn)`
- [ ] `Array.prototype.splice(start, deleteCount [, item1 [, ...]])`
- [ ] `Array.prototype.unshift([item1 [, item2 [, ...]]])`

#### Instance Properties
- [ ] `length` (special [[Put]] behavior)

### 7.5 String Objects

#### Constructor
- [ ] `String()` called as function (type conversion)
- [ ] `new String()` called as constructor

#### Properties
- [ ] `String.prototype`
- [ ] `String.fromCharCode([char0 [, char1 [, ...]]])`

#### Prototype Methods
- [ ] `String.prototype.constructor`
- [ ] `String.prototype.toString()`
- [ ] `String.prototype.valueOf()`
- [ ] `String.prototype.charAt(pos)`
- [ ] `String.prototype.charCodeAt(pos)`
- [ ] `String.prototype.concat([string1 [, string2 [, ...]]])`
- [ ] `String.prototype.indexOf(searchString, position)`
- [ ] `String.prototype.lastIndexOf(searchString, position)`
- [ ] `String.prototype.localeCompare(that)`
- [ ] `String.prototype.match(regexp)`
- [ ] `String.prototype.replace(searchValue, replaceValue)`
- [ ] `String.prototype.search(regexp)`
- [ ] `String.prototype.slice(start, end)`
- [ ] `String.prototype.split(separator, limit)`
- [ ] `String.prototype.substring(start, end)`
- [ ] `String.prototype.toLowerCase()`
- [ ] `String.prototype.toLocaleLowerCase()`
- [ ] `String.prototype.toUpperCase()`
- [ ] `String.prototype.toLocaleUpperCase()`

#### Instance Properties
- [ ] `length`

### 7.6 Boolean Objects

#### Constructor
- [ ] `Boolean()` called as function
- [ ] `new Boolean()` called as constructor

#### Properties
- [ ] `Boolean.prototype`

#### Prototype Methods
- [ ] `Boolean.prototype.constructor`
- [ ] `Boolean.prototype.toString()`
- [ ] `Boolean.prototype.valueOf()`

### 7.7 Number Objects

#### Constructor
- [ ] `Number()` called as function
- [ ] `new Number()` called as constructor

#### Properties
- [ ] `Number.prototype`
- [ ] `Number.MAX_VALUE`
- [ ] `Number.MIN_VALUE`
- [ ] `Number.NaN`
- [ ] `Number.NEGATIVE_INFINITY`
- [ ] `Number.POSITIVE_INFINITY`

#### Prototype Methods
- [ ] `Number.prototype.constructor`
- [ ] `Number.prototype.toString([radix])`
- [ ] `Number.prototype.toLocaleString()`
- [ ] `Number.prototype.valueOf()`
- [ ] `Number.prototype.toFixed(fractionDigits)`
- [ ] `Number.prototype.toExponential(fractionDigits)`
- [ ] `Number.prototype.toPrecision(precision)`

### 7.8 Math Object

#### Properties
- [ ] `Math.E` (~2.718281828459045)
- [ ] `Math.LN10` (~2.302585092994046)
- [ ] `Math.LN2` (~0.6931471805599453)
- [ ] `Math.LOG2E` (~1.4426950408889634)
- [ ] `Math.LOG10E` (~0.4342944819032518)
- [ ] `Math.PI` (~3.141592653589793)
- [ ] `Math.SQRT1_2` (~0.7071067811865476)
- [ ] `Math.SQRT2` (~1.4142135623730951)

#### Functions
- [ ] `Math.abs(x)`
- [ ] `Math.acos(x)`
- [ ] `Math.asin(x)`
- [ ] `Math.atan(x)`
- [ ] `Math.atan2(y, x)`
- [ ] `Math.ceil(x)`
- [ ] `Math.cos(x)`
- [ ] `Math.exp(x)`
- [ ] `Math.floor(x)`
- [ ] `Math.log(x)`
- [ ] `Math.max([value1 [, value2 [, ...]]])`
- [ ] `Math.min([value1 [, value2 [, ...]]])`
- [ ] `Math.pow(x, y)`
- [ ] `Math.random()`
- [ ] `Math.round(x)`
- [ ] `Math.sin(x)`
- [ ] `Math.sqrt(x)`
- [ ] `Math.tan(x)`

### 7.9 Date Objects

#### Constructor
- [ ] `Date()` called as function (returns string)
- [ ] `new Date()` - current date/time
- [ ] `new Date(value)` - milliseconds since epoch
- [ ] `new Date(year, month [, date [, hours [, minutes [, seconds [, ms]]]]])`

#### Properties
- [ ] `Date.prototype`
- [ ] `Date.parse(string)`
- [ ] `Date.UTC(year, month [, date [, hours [, minutes [, seconds [, ms]]]]])`

#### Prototype Methods
- [ ] `Date.prototype.constructor`
- [ ] `Date.prototype.toString()`
- [ ] `Date.prototype.toDateString()`
- [ ] `Date.prototype.toTimeString()`
- [ ] `Date.prototype.toLocaleString()`
- [ ] `Date.prototype.toLocaleDateString()`
- [ ] `Date.prototype.toLocaleTimeString()`
- [ ] `Date.prototype.valueOf()`
- [ ] `Date.prototype.getTime()`
- [ ] `Date.prototype.getFullYear()`
- [ ] `Date.prototype.getUTCFullYear()`
- [ ] `Date.prototype.getMonth()`
- [ ] `Date.prototype.getUTCMonth()`
- [ ] `Date.prototype.getDate()`
- [ ] `Date.prototype.getUTCDate()`
- [ ] `Date.prototype.getDay()`
- [ ] `Date.prototype.getUTCDay()`
- [ ] `Date.prototype.getHours()`
- [ ] `Date.prototype.getUTCHours()`
- [ ] `Date.prototype.getMinutes()`
- [ ] `Date.prototype.getUTCMinutes()`
- [ ] `Date.prototype.getSeconds()`
- [ ] `Date.prototype.getUTCSeconds()`
- [ ] `Date.prototype.getMilliseconds()`
- [ ] `Date.prototype.getUTCMilliseconds()`
- [ ] `Date.prototype.getTimezoneOffset()`
- [ ] `Date.prototype.setTime(time)`
- [ ] `Date.prototype.setMilliseconds(ms)`
- [ ] `Date.prototype.setUTCMilliseconds(ms)`
- [ ] `Date.prototype.setSeconds(sec [, ms])`
- [ ] `Date.prototype.setUTCSeconds(sec [, ms])`
- [ ] `Date.prototype.setMinutes(min [, sec [, ms]])`
- [ ] `Date.prototype.setUTCMinutes(min [, sec [, ms]])`
- [ ] `Date.prototype.setHours(hour [, min [, sec [, ms]]])`
- [ ] `Date.prototype.setUTCHours(hour [, min [, sec [, ms]]])`
- [ ] `Date.prototype.setDate(date)`
- [ ] `Date.prototype.setUTCDate(date)`
- [ ] `Date.prototype.setMonth(month [, date])`
- [ ] `Date.prototype.setUTCMonth(month [, date])`
- [ ] `Date.prototype.setFullYear(year [, month [, date]])`
- [ ] `Date.prototype.setUTCFullYear(year [, month [, date]])`
- [ ] `Date.prototype.toUTCString()`

### 7.10 RegExp Objects

#### Constructor
- [ ] `RegExp(pattern, flags)` called as function
- [ ] `new RegExp(pattern, flags)` called as constructor

#### Properties
- [ ] `RegExp.prototype`

#### Prototype Methods
- [ ] `RegExp.prototype.constructor`
- [ ] `RegExp.prototype.exec(string)`
- [ ] `RegExp.prototype.test(string)`
- [ ] `RegExp.prototype.toString()`

#### Instance Properties
- [ ] `source`
- [ ] `global`
- [ ] `ignoreCase`
- [ ] `multiline`
- [ ] `lastIndex`

#### Pattern Syntax
- [ ] Disjunction `|`
- [ ] Alternative (concatenation)
- [ ] Assertions: `^`, `$`, `\b`, `\B`
- [ ] Lookahead: `(?=...)`, `(?!...)`
- [ ] Quantifiers: `*`, `+`, `?`, `{n}`, `{n,}`, `{n,m}`
- [ ] Non-greedy: `*?`, `+?`, `??`, `{n,m}?`
- [ ] Capturing groups `(...)`
- [ ] Non-capturing groups `(?:...)`
- [ ] Backreferences `\1`, `\2`, etc.
- [ ] Character classes `[...]`, `[^...]`
- [ ] Character class escapes: `\d`, `\D`, `\s`, `\S`, `\w`, `\W`
- [ ] Character escapes: `\f`, `\n`, `\r`, `\t`, `\v`
- [ ] Control escapes: `\cA` through `\cZ`
- [ ] Hex escapes: `\xHH`
- [ ] Unicode escapes: `\uHHHH`
- [ ] Dot `.` (any except line terminator)

---

## 8. Error Handling

### 8.1 Error Objects

#### Error Constructor
- [ ] `Error()` called as function
- [ ] `new Error([message])` called as constructor

#### Error Properties
- [ ] `Error.prototype`

#### Error Prototype
- [ ] `Error.prototype.constructor`
- [ ] `Error.prototype.name` ("Error")
- [ ] `Error.prototype.message` ("")
- [ ] `Error.prototype.toString()`

### 8.2 Native Error Types

Each requires constructor (function and new), prototype with name and message:

- [ ] `EvalError` - errors in eval()
- [ ] `RangeError` - numeric value out of range
- [ ] `ReferenceError` - invalid reference
- [ ] `SyntaxError` - parsing error
- [ ] `TypeError` - wrong type
- [ ] `URIError` - URI handling error

---

## Summary Statistics

| Category | Items | Priority |
|----------|-------|----------|
| Lexical Grammar | ~35 | High |
| Types & Conversion | ~20 | High |
| Expressions | ~45 | High |
| Statements | ~25 | High |
| Functions | ~15 | High |
| Built-in Objects | ~175 | High |
| Error Handling | ~15 | High |
| **Total** | **~330** | |

---

## Milestone Targets

| Milestone | Description | Target |
|-----------|-------------|--------|
| M1 | Lexer complete | Week 2 |
| M2 | Parser complete | Week 4 |
| M3 | Basic interpreter (primitives, operators) | Week 6 |
| M4 | Functions and control flow | Week 8 |
| M5 | Core built-ins (Object, Array, String) | Week 10 |
| M6 | All built-ins | Week 14 |
| M7 | Full ES3 compliance | Week 16 |

---

## References

- [ECMA-262 3rd Edition (PDF)](https://www.ecma-international.org/wp-content/uploads/ECMA-262_3rd_edition_december_1999.pdf)
- [MDN JavaScript Reference](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference)

