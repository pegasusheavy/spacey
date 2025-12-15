# TypeScript Native Support TODO

This document tracks the implementation of **native** TypeScript support in the Spacey JavaScript engine.

**Target**: Execute TypeScript directly without transpilation (like Deno/Bun)

**Last Updated**: December 2025

**Current Status**: Planning

---

## Overview

Unlike traditional TypeScript tooling that transpiles `.ts` to `.js`, Spacey will **natively parse and execute TypeScript** by extending the lexer and parser to understand TypeScript syntax and strip types at parse time.

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

### Phase 1: Lexer Extensions (spacey-spidermonkey)

**File: `crates/spacey-spidermonkey/src/lexer/`**

New tokens to recognize:
- [ ] `type` keyword
- [ ] `interface` keyword
- [ ] `namespace` keyword
- [ ] `declare` keyword
- [ ] `readonly` keyword
- [ ] `abstract` keyword
- [ ] `implements` keyword
- [ ] `private`, `protected`, `public` modifiers
- [ ] `as` keyword (type assertion)
- [ ] `is` keyword (type predicate)
- [ ] `keyof` keyword
- [ ] `infer` keyword
- [ ] `never`, `unknown`, `any` type keywords
- [ ] `asserts` keyword
- [ ] `override` keyword
- [ ] Angle brackets `<>` for generics (context-aware)
- [ ] `?:` optional property syntax
- [ ] `!:` definite assignment assertion
- [ ] `@` decorator syntax

### Phase 2: Parser Extensions - Type Annotations

**File: `crates/spacey-spidermonkey/src/parser/`**

Parse and skip type annotations:
- [ ] Variable type annotations: `let x: number`
- [ ] Function parameter types: `function f(x: number)`
- [ ] Function return types: `function f(): number`
- [ ] Arrow function types: `(x: number): number => x`
- [ ] Optional parameters: `function f(x?: number)`
- [ ] Rest parameter types: `function f(...args: number[])`
- [ ] Type assertions: `x as number`, `<number>x`
- [ ] Non-null assertions: `x!`
- [ ] Definite assignment: `x!: number`

### Phase 3: Parser Extensions - Type Declarations

Parse and skip entire type declarations:
- [ ] Type aliases: `type Foo = string`
- [ ] Interfaces: `interface Foo { ... }`
- [ ] Generic type parameters: `function f<T>(x: T): T`
- [ ] Generic constraints: `<T extends Foo>`
- [ ] Generic defaults: `<T = string>`
- [ ] Mapped types: `{ [K in keyof T]: ... }`
- [ ] Conditional types: `T extends U ? X : Y`
- [ ] Index signatures: `{ [key: string]: number }`
- [ ] Call signatures: `{ (x: number): void }`
- [ ] Construct signatures: `{ new (x: number): Foo }`

### Phase 4: Parser Extensions - Classes

- [ ] Property modifiers: `private x: number`
- [ ] Parameter properties: `constructor(private x: number)`
- [ ] Abstract classes: `abstract class Foo`
- [ ] Abstract methods: `abstract method(): void`
- [ ] Implements clause: `class Foo implements Bar`
- [ ] Override keyword: `override method()`
- [ ] Declare modifier: `declare class Foo`

### Phase 5: Parser Extensions - Modules

- [ ] Type-only imports: `import type { Foo } from './foo'`
- [ ] Type-only exports: `export type { Foo }`
- [ ] Import type: `import { type Foo } from './foo'`
- [ ] Declare module: `declare module 'foo' { ... }`
- [ ] Namespace declarations: `namespace Foo { ... }`
- [ ] Module augmentation

### Phase 6: Parser Extensions - Enums

- [ ] Numeric enums: `enum Color { Red, Green, Blue }`
- [ ] String enums: `enum Color { Red = "RED" }`
- [ ] Const enums: `const enum Color { ... }`
- [ ] Declare enums: `declare enum Color { ... }`
- [ ] Computed enum members

### Phase 7: Decorators (Optional/Experimental)

- [ ] Class decorators: `@decorator class Foo`
- [ ] Method decorators: `@decorator method()`
- [ ] Property decorators: `@decorator prop: number`
- [ ] Parameter decorators: `method(@decorator x: number)`

### Phase 8: Node Runtime Integration

**File: `crates/spacey-node/src/`**

- [ ] Recognize `.ts`, `.tsx`, `.mts`, `.cts` extensions
- [ ] Enable TypeScript parsing mode for these files
- [ ] Handle tsconfig.json for project settings
- [ ] Support path mappings from tsconfig
- [ ] JSX support for `.tsx` files

### Phase 9: CLI Integration

- [ ] Run `.ts` files directly: `spacey-node app.ts`
- [ ] `--no-typescript` flag to disable TS parsing
- [ ] `--tsconfig <path>` for custom config

---

## TypeScript Syntax to Support

### Must Have (Core)
```typescript
// Type annotations
let x: number = 42;
const arr: string[] = [];
const obj: { name: string } = { name: "foo" };

// Function types
function add(a: number, b: number): number { return a + b; }
const fn: (x: number) => number = (x) => x * 2;

// Interfaces (parsed and skipped)
interface User { name: string; age: number; }

// Type aliases (parsed and skipped)
type ID = string | number;

// Generics
function identity<T>(x: T): T { return x; }

// Type assertions
const x = value as string;
const y = <string>value;

// Optional/non-null
const z = obj?.prop;
const w = obj!.prop;

// Classes with types
class Foo {
    private x: number;
    constructor(x: number) { this.x = x; }
}
```

### Nice to Have
```typescript
// Enums (converted to objects)
enum Color { Red, Green, Blue }

// Namespaces (converted to objects)
namespace Utils { export function helper() {} }

// Decorators (experimental)
@decorator
class Foo {}
```

---

## Implementation Notes

### Key Insight: Type Erasure

TypeScript types exist only at parse time. The key is to:
1. **Parse** the type syntax correctly
2. **Skip/ignore** it when building the AST
3. **Output** the same AST as if parsing JavaScript

### Example Transform

Input (TypeScript):
```typescript
function greet(name: string): string {
    return "Hello, " + name;
}
```

AST Output (same as JavaScript):
```
FunctionDeclaration {
    name: "greet",
    params: [Identifier("name")],  // No type info
    body: BlockStatement { ... }    // No return type
}
```

### Enum Handling

Enums need special handling - they generate runtime code:
```typescript
enum Color { Red, Green, Blue }
```
Becomes:
```javascript
var Color;
(function (Color) {
    Color[Color["Red"] = 0] = "Red";
    Color[Color["Green"] = 1] = "Green";
    Color[Color["Blue"] = 2] = "Blue";
})(Color || (Color = {}));
```

---

## Testing Strategy

1. **Unit tests**: Each new token/parse rule
2. **Integration tests**: Full TypeScript files
3. **Compatibility tests**: Run TypeScript test suite
4. **Error tests**: Proper error messages for invalid TS

---

## References

- [TypeScript Spec](https://github.com/microsoft/TypeScript/blob/main/doc/spec-ARCHIVED.md)
- [TypeScript AST Viewer](https://ts-ast-viewer.com/)
- [Deno's TypeScript handling](https://deno.land/manual/typescript)
- [Bun's TypeScript support](https://bun.sh/docs/runtime/typescript)
