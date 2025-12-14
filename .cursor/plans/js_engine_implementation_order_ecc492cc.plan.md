---
name: JS Engine Implementation Order
overview: "A phased implementation plan for completing the Spacey JavaScript engine, building on the existing scaffolding. The order follows dependencies: lexer feeds parser, parser feeds compiler, compiler feeds VM, with runtime/builtins woven in as needed."
todos:
  - id: wire-pipeline
    content: Wire up eval() to use lexer -> parser -> compiler -> VM pipeline
    status: completed
  - id: lexer-comments
    content: Add comment handling to lexer (// and /* */)
    status: completed
  - id: parser-control
    content: Complete parser control flow (switch, do-while, for-in/of, try-catch)
    status: completed
  - id: parser-functions
    content: Add arrow functions and function expressions to parser
    status: completed
  - id: runtime-callframe
    content: Add call frames and proper scope chain to runtime
    status: completed
  - id: compiler-vars
    content: Implement variable declaration and scope tracking in compiler
    status: completed
  - id: compiler-control
    content: Add control flow compilation with jump patching
    status: completed
  - id: vm-vars
    content: Implement variable load/store opcodes in VM
    status: completed
  - id: vm-calls
    content: Add function call/return with call stack to VM
    status: completed
  - id: builtin-console
    content: Implement console.log as first builtin
    status: completed
  - id: builtin-object
    content: Implement Object constructor and prototype
    status: completed
  - id: gc-basic
    content: Implement basic mark-and-sweep garbage collector
    status: completed
---

# JavaScript Engine Implementation Order

## Current State Assessment

Your codebase already has solid foundations:

| Component | Completeness | Status |

|-----------|--------------|--------|

| Lexer | ~85% | Most tokens, missing comments |

| AST | ~75% | Core nodes defined |

| Parser | ~60% | Basic statements/expressions |

| Runtime | ~40% | Value enum, basic Object/Environment |

| Compiler | ~25% | Literals and binary ops only |

| VM | ~25% | Basic arithmetic only |

| Builtins | 0% | Stub only |

| GC | 0% | Stub only |

## Recommended Implementation Order

```mermaid
flowchart TD
    subgraph phase1 [Phase 1: Foundation]
        L[Lexer Completion]
        P[Parser Completion]
        L --> P
    end

    subgraph phase2 [Phase 2: Execution Core]
        R[Runtime Enhancement]
        C[Compiler Expansion]
        V[VM Implementation]
        R --> C
        C --> V
    end

    subgraph phase3 [Phase 3: Language Features]
        B[Basic Builtins]
        F[Functions and Closures]
        B --> F
    end

    subgraph phase4 [Phase 4: Memory and Advanced]
        G[Garbage Collector]
        A[Advanced Features]
        G --> A
    end

    phase1 --> phase2
    phase2 --> phase3
    phase3 --> phase4
```

---

## Phase 1: Complete Lexer and Parser

### 1.1 Lexer Completion

Location: [`crates/spacey-spidermonkey/src/lexer/scanner.rs`](crates/spacey-spidermonkey/src/lexer/scanner.rs)

Missing features to add:

- Single-line comments (`//`)
- Multi-line comments (`/* */`)
- Unicode escape sequences in strings (`\u{xxxx}`)
- Regex literal scanning (context-sensitive)
- Line terminator tracking for ASI
- Better error recovery with source locations

### 1.2 Parser Completion  

Location: [`crates/spacey-spidermonkey/src/parser/parser.rs`](crates/spacey-spidermonkey/src/parser/parser.rs)

Missing features to add:

- Switch statements
- Do-while loops
- For-in/for-of loops
- Try/catch/finally (structure exists in AST)
- Break/continue with labels
- Arrow functions
- Class declarations
- Spread/rest operators
- Destructuring patterns
- Template literals with expressions
- Automatic Semicolon Insertion (ASI)

---

## Phase 2: Runtime and Execution

### 2.1 Runtime Enhancement

Location: [`crates/spacey-spidermonkey/src/runtime/`](crates/spacey-spidermonkey/src/runtime/)

Key additions:

- **Call frames** for function execution context
- **Scope chain** linking environments properly
- **`this` binding** resolution
- **Object heap** (prepare for GC integration)
- **Function objects** as first-class values

### 2.2 Compiler Expansion

Location: [`crates/spacey-spidermonkey/src/compiler/codegen.rs`](crates/spacey-spidermonkey/src/compiler/codegen.rs)

Add compilation for:

- Variable declarations with scope tracking
- Control flow (if/while/for) with jump patching
- Function declarations and expressions
- Closure capture analysis
- Object and array literals
- Property access and assignment
- Call expressions

### 2.3 VM Completion

Location: [`crates/spacey-spidermonkey/src/vm/interpreter.rs`](crates/spacey-spidermonkey/src/vm/interpreter.rs)

Implement remaining opcodes:

- Variable load/store (local, global, upvalue)
- Jump instructions with proper offsets
- Function call/return with call stack
- Object/array creation and property access
- Exception handling (try/catch)

---

## Phase 3: Builtins and Functions

### 3.1 Basic Builtins

Location: [`crates/spacey-spidermonkey/src/builtins/`](crates/spacey-spidermonkey/src/builtins/)

Start with essential builtins:

1. `console.log` (critical for testing/debugging)
2. `Object` constructor and prototype methods
3. `Array` constructor and prototype methods
4. `String` prototype methods
5. `Number`, `Boolean`, `Math`
6. `Error` types

### 3.2 Functions and Closures

- Proper closure implementation with upvalue capture
- `Function.prototype.call/apply/bind`
- Arrow function `this` binding
- Rest parameters and spread arguments

---

## Phase 4: GC and Advanced Features

### 4.1 Garbage Collector

Location: [`crates/spacey-spidermonkey/src/gc/`](crates/spacey-spidermonkey/src/gc/)

Implementation strategy:

1. Start with simple mark-and-sweep
2. Define `GcObject` wrapper type for heap objects
3. Track roots (stack, globals)
4. Add write barriers later for generational GC

### 4.2 Advanced Features (Later)

- Promises and async/await
- Iterators and generators
- Proxies and Reflect
- ES Modules
- JIT compilation (much later)

---

## Suggested First Steps

Wire up the existing pipeline in [`lib.rs`](crates/spacey-spidermonkey/src/lib.rs):

```rust
pub fn eval(&mut self, source: &str) -> Result<Value, Error> {
    // 1. Lex
    let mut parser = Parser::new(source);
    
    // 2. Parse
    let ast = parser.parse_program()?;
    
    // 3. Compile
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast)?;
    
    // 4. Execute
    let mut vm = VM::new();
    vm.execute(&bytecode)
}
```

This will immediately let you test the full pipeline with expressions like `1 + 2 * 3`.

---

## Development Strategy

1. **Test-driven**: Add tests before/during each feature
2. **Incremental**: Get simple cases working, then add complexity
3. **Wire early**: Connect components even if incomplete
4. **REPL feedback**: Use your REPL to manually test as you go