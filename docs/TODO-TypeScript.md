# TypeScript Native Support TODO

This document tracks the implementation of **native** TypeScript support in the Spacey JavaScript engine.

**Target**: Execute TypeScript directly without transpilation (like Deno/Bun)

**Last Updated**: December 2025

**Current Status**: ✅ **Implemented** (Core Features Complete)

---

## Overview

Unlike traditional TypeScript tooling that transpiles `.ts` to `.js`, Spacey **natively parses and executes TypeScript** by extending the lexer and parser to understand TypeScript syntax and strip types at parse time.

### Why Native Execution?

| Approach | Pros | Cons |
|----------|------|------|
| **Transpile (SWC)** | Battle-tested, full TS support | Extra step, string generation overhead |
| **Native Parse** | Faster, simpler pipeline, better errors | More work to implement |

```
Traditional:  .ts → SWC → .js string → Parse JS → AST → Execute
Native:       .ts → Parse TS (strip types) → AST → Execute
```

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    TypeScript Source                         │
│  const x: number = 42;                                       │
│  function add(a: number, b: number): number { return a + b } │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Extended Lexer                            │
│  - Recognize type keywords (type, interface, as, etc.)      │
│  - Handle angle brackets for generics                        │
│  - Support TypeScript-specific tokens                        │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Extended Parser                           │
│  - Parse type annotations and SKIP them                      │
│  - Parse interfaces/type aliases and SKIP them               │
│  - Parse generics and SKIP them                              │
│  - Output same AST as JavaScript (types erased)              │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Existing Pipeline                         │
│  Compiler → Bytecode → VM (unchanged)                        │
└─────────────────────────────────────────────────────────────┘
```

---

## Implementation Checklist

### Phase 1: Lexer Extensions (spacey-spidermonkey) ✅

**File: `crates/spacey-spidermonkey/src/lexer/`**

New tokens recognized:
- [x] `type` keyword
- [x] `interface` keyword
- [x] `namespace` keyword
- [x] `declare` keyword
- [x] `readonly` keyword
- [x] `abstract` keyword
- [x] `implements` keyword
- [x] `private`, `protected`, `public` modifiers
- [x] `as` keyword (type assertion)
- [x] `is` keyword (type predicate)
- [x] `keyof` keyword
- [x] `infer` keyword
- [x] `never`, `unknown`, `any` type keywords
- [x] `asserts` keyword
- [x] `override` keyword
- [x] `satisfies` keyword
- [x] `out` keyword
- [x] `accessor` keyword
- [x] Angle brackets `<>` for generics
- [x] `@` decorator syntax

### Phase 2: Parser Extensions - Type Annotations ✅

**File: `crates/spacey-spidermonkey/src/parser/parser.rs`**

Parse and skip type annotations:
- [x] Variable type annotations: `let x: number`
- [x] Function parameter types: `function f(x: number)`
- [x] Function return types: `function f(): number`
- [x] Arrow function types: `(x: number): number => x`
- [x] Optional parameters: `function f(x?: number)`
- [x] Rest parameter types: `function f(...args: number[])`
- [x] Type assertions: `x as number`
- [x] Non-null assertions: `x!`
- [x] Definite assignment: `x!: number`

### Phase 3: Parser Extensions - Type Declarations ✅

Parse and skip entire type declarations:
- [x] Type aliases: `type Foo = string`
- [x] Interfaces: `interface Foo { ... }`
- [x] Generic type parameters: `function f<T>(x: T): T`
- [x] Generic constraints: `<T extends Foo>`
- [x] Generic defaults: `<T = string>`
- [x] Mapped types: `{ [K in keyof T]: ... }`
- [x] Conditional types: `T extends U ? X : Y`
- [x] Index signatures: `{ [key: string]: number }`
- [x] Union types: `string | number`
- [x] Intersection types: `A & B`
- [x] Tuple types: `[string, number]`
- [x] Function types: `(x: number) => number`
- [x] Constructor types: `new (x: number) => Foo`

### Phase 4: Parser Extensions - Classes ✅

- [x] Property modifiers: `private x: number`
- [x] Abstract classes: `abstract class Foo`
- [x] Implements clause: `class Foo implements Bar`
- [x] Override keyword: `override method()`
- [x] Declare modifier: `declare class Foo`
- [x] Decorators: `@decorator class Foo`

### Phase 5: Parser Extensions - Modules ✅

- [x] Declare module: `declare module 'foo' { ... }`
- [x] Namespace declarations: `namespace Foo { ... }`

### Phase 6: Parser Extensions - Enums ✅

- [x] Numeric enums: `enum Color { Red, Green, Blue }` → compiled to JS object
- [x] String enums: `enum Color { Red = "RED" }` → compiled to JS object (no reverse mapping)
- [x] Enums with explicit values: `enum Status { Active = 1, Inactive = 0 }`

### Phase 7: Node Runtime Integration ✅

**File: `crates/spacey-node/src/`**

- [x] Recognize `.ts`, `.tsx`, `.mts`, `.cts` extensions
- [x] Enable TypeScript parsing mode for these files
- [x] `is_typescript_file()` helper function
- [x] `is_jsx_file()` helper function
- [x] `eval_typescript()` method on `NodeRuntime`

### Phase 8: CLI Integration ✅

- [x] Run `.ts` files directly: `spacey-node app.ts`
- [x] `--typescript` / `-T` flag to force TS parsing
- [x] `--eval-ts` for evaluating TypeScript from command line

---

## Usage

### Running TypeScript Files

```bash
# TypeScript files are automatically detected and executed
spacey-node server.ts

# Force TypeScript mode for any file
spacey-node --typescript script.js

# Evaluate TypeScript inline
spacey-node --eval-ts "const x: number = 42; console.log(x)"
```

### Programmatic Usage

```rust
use spacey_spidermonkey::Engine;

let mut engine = Engine::new();

// Evaluate TypeScript directly
let result = engine.eval_typescript("const x: number = 42; x;")?;

// Or use auto-detection with files
let result = engine.eval_file_auto(Path::new("script.ts"))?;
```

```rust
use spacey_node::NodeRuntime;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut runtime = NodeRuntime::new(vec![]);
    
    // TypeScript files are automatically handled
    runtime.run_file(Path::new("server.ts")).await?;
    
    Ok(())
}
```

---

## TypeScript Syntax Supported

### Core Features (Fully Implemented)

```typescript
// Type annotations
let x: number = 42;
const arr: string[] = [];
const obj: { name: string } = { name: "foo" };

// Function types
function add(a: number, b: number): number { return a + b; }
const fn: (x: number) => number = (x) => x * 2;

// Optional parameters
function greet(name?: string): string { return "Hello, " + (name || "World"); }

// Interfaces (parsed and skipped)
interface User { name: string; age: number; }

// Type aliases (parsed and skipped)
type ID = string | number;

// Generics
function identity<T>(x: T): T { return x; }
function map<T, U>(arr: T[], fn: (x: T) => U): U[] { /* ... */ }

// Type assertions
const x = value as string;

// Non-null assertions
const w = obj!.prop;

// Union and intersection types
type StringOrNumber = string | number;
type Combined = A & B;

// Classes with types
class Foo {
    private x: number;
    constructor(x: number) { this.x = x; }
}

// Abstract classes
abstract class Base {
    abstract method(): void;
}

// Enums (converted to objects at runtime)
enum Color { Red, Green, Blue }
enum Direction { Up = "UP", Down = "DOWN" }
```

### Skipped Type Declarations

The following TypeScript-only constructs are parsed and completely skipped
(they produce no runtime code):

```typescript
// Type aliases
type MyType = string | number;

// Interfaces
interface User { name: string; age: number; }

// Declare statements
declare var globalVar: string;
declare function externalFn(): void;

// Namespaces (skipped, no runtime code)
namespace Utils { /* ... */ }
```

---

## Future Enhancements

These features are not yet implemented but could be added:

- [ ] Parameter properties: `constructor(private x: number)`
- [ ] Type-only imports: `import type { Foo } from './foo'`
- [ ] Type-only exports: `export type { Foo }`
- [ ] tsconfig.json support for path mappings
- [ ] JSX/TSX support for React-like syntax
- [ ] Const enums (inlined at compile time)
- [ ] Source map support for debugging

---

## Testing

All TypeScript parsing features are covered by unit tests:

```bash
# Run TypeScript-specific tests
cargo test -p spacey-spidermonkey test_parse_typescript
```

Tests cover:
- Variable type annotations
- Function parameter and return types
- Arrow functions with types
- Optional parameters
- Type assertions (`as` and `!`)
- Type aliases (skipped)
- Interfaces (skipped)
- Declare statements (skipped)
- Namespaces (skipped)
- Generic functions
- Union types
- Intersection types
- Complex nested types
- Numeric enums
- String enums
- Enums with explicit values

---

## References

- [TypeScript Spec](https://github.com/microsoft/TypeScript/blob/main/doc/spec-ARCHIVED.md)
- [TypeScript AST Viewer](https://ts-ast-viewer.com/)
- [Deno's TypeScript handling](https://deno.land/manual/typescript)
- [Bun's TypeScript support](https://bun.sh/docs/runtime/typescript)
