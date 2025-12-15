# ES3 Full Compatibility Implementation Plan

**Goal**: Achieve 100% ES3 (ECMA-262 3rd Edition) compliance in the Spacey JavaScript engine.

**Current Status**: ~85% complete (280/330 items)

**Estimated Effort**: 3-4 weeks of focused development

---

## Executive Summary

The Spacey engine has strong ES3 foundations with all core language features working. The remaining work focuses on:

1. **Critical Bug Fix**: Error handling in complex expressions (blocking full test suite)
2. **Test Suite Fix**: Deprecated `Math_abs` syntax in test file
3. **Partial Implementations**: ~36 features that work but need edge case fixes
4. **Missing Features**: ~14 features not yet started

---

## Phase 1: Critical Blockers (Week 1)

These issues block the full ES3 compliance test suite from passing.

### 1.1 Fix Error Object Construction in Complex Expressions

**Priority**: P0 - Blocking
**Issue**: `TypeError("Error: TypeError: Expected numbers")` in test suite
**Root Cause**: Error objects fail when constructed in certain expression contexts

**Files to modify**:
- `crates/spacey-spidermonkey/src/builtins/error.rs`
- `crates/spacey-spidermonkey/src/vm/interpreter.rs`

**Tasks**:
- [ ] Debug the exact failure point in error handling section
- [ ] Fix Error constructor to handle all invocation contexts
- [ ] Ensure `new Error()`, `new TypeError()`, etc. work in all expression positions
- [ ] Add error property access tests (`e.message`, `e.name`)

### 1.2 Fix Deprecated Test Syntax

**Priority**: P0 - Blocking
**Issue**: `test_es3_math` uses `Math_abs` instead of `Math.abs`

**Files to modify**:
- `crates/spacey-spidermonkey/tests/es3_compliance_test.rs`

**Tasks**:
- [ ] Update line 209: `Math_abs(-5)` → `Math.abs(-5)`
- [ ] Update line 210: `Math_floor(3.7)` → `Math.floor(3.7)`
- [ ] Update line 211: `Math_ceil(3.2)` → `Math.ceil(3.2)`
- [ ] Update line 212: `Math_pow(2, 3)` → `Math.pow(2, 3)`

---

## Phase 2: Built-in Object Completion (Week 2)

### 2.1 Function Constructor from String

**Priority**: P1 - Required for ES3
**Current**: Partially implemented

**Files to modify**:
- `crates/spacey-spidermonkey/src/builtins/function.rs`

**Tasks**:
- [ ] Implement `Function('x', 'return x * 2')` constructor
- [ ] Implement `new Function('a', 'b', 'return a + b')` with multiple params
- [ ] Parse and compile the function body string at runtime
- [ ] Handle syntax errors in function body

### 2.2 String Regex Methods

**Priority**: P1 - Required for ES3
**Current**: Basic implementation, edge cases failing

**Files to modify**:
- `crates/spacey-spidermonkey/src/builtins/string.rs`
- `crates/spacey-spidermonkey/src/vm/interpreter.rs`

**Tasks**:
- [ ] `String.prototype.match(regexp)`
  - [ ] Return array of matches
  - [ ] Handle global flag correctly
  - [ ] Return null when no match
  - [ ] Set capturing groups in result array
- [ ] `String.prototype.replace(searchValue, replaceValue)`
  - [ ] Handle string replacement patterns (`$1`, `$&`, etc.)
  - [ ] Handle function replacement
  - [ ] Handle global flag
- [ ] `String.prototype.search(regexp)`
  - [ ] Return index of first match
  - [ ] Return -1 on no match

### 2.3 Date Static Methods

**Priority**: P1 - Required for ES3
**Current**: Partially implemented

**Files to modify**:
- `crates/spacey-spidermonkey/src/builtins/date.rs`

**Tasks**:
- [ ] `Date.parse(string)` - Parse date strings
  - [ ] ISO 8601 format
  - [ ] RFC 2822 format
  - [ ] Return NaN on invalid input
- [ ] `Date.UTC(year, month, ...)` - Return UTC timestamp
  - [ ] Handle all optional parameters
  - [ ] Return milliseconds since epoch

### 2.4 Number Methods

**Priority**: P1 - Required for ES3

**Files to modify**:
- `crates/spacey-spidermonkey/src/builtins/number.rs`

**Tasks**:
- [ ] `Number.prototype.toExponential(fractionDigits)`
- [ ] `Number.prototype.toPrecision(precision)`
- [ ] Handle edge cases (NaN, Infinity)

---

## Phase 3: Object Prototype Methods (Week 2-3)

### 3.1 Object Prototype

**Priority**: P2 - Important for completeness

**Files to modify**:
- `crates/spacey-spidermonkey/src/builtins/object.rs`

**Tasks**:
- [ ] `Object.prototype.isPrototypeOf(V)` - Check prototype chain
- [ ] `Object.prototype.propertyIsEnumerable(V)` - Check enumerability
- [ ] `Object.prototype.toLocaleString()` - Locale-aware stringification

### 3.2 Array Prototype

**Priority**: P2 - Important for completeness

**Files to modify**:
- `crates/spacey-spidermonkey/src/builtins/array.rs`
- `crates/spacey-spidermonkey/src/vm/interpreter.rs`

**Tasks**:
- [ ] `Array.prototype.splice()` - Complete implementation
  - [ ] Handle negative indices
  - [ ] Handle insertion of multiple elements
  - [ ] Return deleted elements array
- [ ] `Array.prototype.toLocaleString()`

---

## Phase 4: RegExp Enhancements (Week 3)

### 4.1 RegExp Pattern Features

**Priority**: P2 - Important for completeness

**Files to modify**:
- `crates/spacey-spidermonkey/src/builtins/regexp.rs`
- `crates/spacey-spidermonkey/src/runtime/regexp.rs`

**Tasks**:
- [ ] Positive lookahead `(?=...)`
- [ ] Negative lookahead `(?!...)`
- [ ] Control escapes `\cA` through `\cZ`
- [ ] Verify backreference behavior

---

## Phase 5: Lexical & Type System Polish (Week 3-4)

### 5.1 Lexer Enhancements

**Priority**: P3 - Nice to have

**Files to modify**:
- `crates/spacey-spidermonkey/src/lexer/mod.rs`
- `crates/spacey-spidermonkey/src/lexer/string.rs`

**Tasks**:
- [ ] Unicode escape sequences in identifiers (`\uXXXX`)
- [ ] Octal escape sequences in strings (`\0` - `\377`)

### 5.2 Arguments Object

**Priority**: P2 - Important for ES3

**Files to modify**:
- `crates/spacey-spidermonkey/src/vm/interpreter.rs`

**Tasks**:
- [ ] Implement live binding between arguments and parameters
  - [ ] When `arguments[0]` changes, first parameter changes
  - [ ] When first parameter changes, `arguments[0]` changes

### 5.3 Internal Properties

**Priority**: P3 - Nice to have

**Tasks**:
- [ ] `[[Class]]` internal property
- [ ] `[[Value]]` for primitive wrappers
- [ ] `[[CanPut]]` proper implementation
- [ ] `[[DefaultValue]]` hint handling
- [ ] `[[HasInstance]]` for custom constructors

---

## Phase 6: Locale Methods (Week 4)

### 6.1 Locale-Aware Methods

**Priority**: P3 - Nice to have (currently stubs)

**Files to modify**:
- `crates/spacey-spidermonkey/src/builtins/string.rs`
- `crates/spacey-spidermonkey/src/builtins/number.rs`
- `crates/spacey-spidermonkey/src/builtins/date.rs`
- `crates/spacey-spidermonkey/src/builtins/array.rs`

**Tasks**:
- [ ] `String.prototype.toLocaleLowerCase()`
- [ ] `String.prototype.toLocaleUpperCase()`
- [ ] `String.prototype.localeCompare()`
- [ ] `Number.prototype.toLocaleString()`
- [ ] `Date.prototype.toLocaleString()`
- [ ] `Date.prototype.toLocaleDateString()`
- [ ] `Date.prototype.toLocaleTimeString()`
- [ ] `Array.prototype.toLocaleString()`

---

## Verification Strategy

### Test Suites

1. **Unit Tests** (`cargo test -p spacey-spidermonkey`)
   - Individual feature tests
   - Edge case coverage

2. **ES3 Compliance Suite** (`tests/es3_compliance.js`)
   - Full integration test
   - Currently ~95% passing

3. **Ignored Tests** (`cargo test -- --ignored`)
   - `test_es3_compliance_full` - Target: PASS

### Success Criteria

| Metric | Current | Target |
|--------|---------|--------|
| Unit tests passing | 22/23 | 23/23 |
| Full compliance suite | ~95% | 100% |
| TODO items complete | 280/330 | 330/330 |

---

## Immediate Action Items

### This Week

1. **Fix Error Handling Bug**
   ```bash
   cd crates/spacey-spidermonkey
   cargo test test_es3_compliance_full -- --ignored --nocapture
   ```
   Debug the TypeError in error handling section.

2. **Fix Test File**
   ```rust
   // In es3_compliance_test.rs line 209-212
   assert_eq!(engine.eval("Math.abs(-5);").unwrap().to_string(), "5");
   assert_eq!(engine.eval("Math.floor(3.7);").unwrap().to_string(), "3");
   assert_eq!(engine.eval("Math.ceil(3.2);").unwrap().to_string(), "4");
   assert_eq!(engine.eval("Math.pow(2, 3);").unwrap().to_string(), "8");
   ```

3. **Run Full Test Suite**
   ```bash
   cargo test -p spacey-spidermonkey
   cargo test -p spacey-spidermonkey test_es3_compliance_full -- --ignored
   ```

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Error handling deeply broken | Low | High | Isolated issue, fix one location |
| Regex lookahead complex | Medium | Medium | Use Rust regex crate features |
| Locale methods need ICU | High | Low | Accept stub implementations for ES3 |
| Arguments live binding tricky | Medium | Medium | Careful VM modification |

---

## Dependencies

### External Crates Already Used
- `regex` - RegExp support
- `chrono` - Date handling
- No new dependencies needed for ES3 compliance

---

## Appendix: Full Test Output Analysis

### Passing Sections (from last test run)
- ✅ Section 1: Primitive Values and Types
- ✅ Section 2: Operators
- ✅ Section 3: Statements
- ✅ Section 4: Functions
- ✅ Section 5: Objects
- ✅ Section 6: Arrays
- ✅ Section 7: String Methods
- ✅ Section 8: Number Methods
- ✅ Section 9: Math Object
- ✅ Section 10: Date Object
- ✅ Section 11: Global Functions

### Failing Section
- ❌ Section 12: Error Handling - `TypeError("Error: TypeError: Expected numbers")`

The failure occurs somewhere in the Error Handling section, after all other sections pass. This indicates the error construction or throwing mechanism fails in a specific context.

---

## Conclusion

ES3 compliance is within reach. The primary blocker is a bug in error handling, and most remaining work is polish/edge cases. With focused effort, full compliance can be achieved in 3-4 weeks.

