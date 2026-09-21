# Zeta

A bytecode-VM interpreted scripting language written from scratch in Rust,
following the architecture of Crafting Interpreters' clox, with deliberate
deviations.

```
source → lexer → parser → AST → compiler → bytecode → VM
```

Unlike clox's single-pass design, Zeta is a two-pass pipeline:
lexer → parser → AST → compiler → bytecode → VM.

## How to run

Requirements: stable Rust (`cargo`).

```bash
cargo run -- test.zeta   # run a script
cargo run                # REPL, type `exit` to quit
cargo test               # lexer / parser / compiler unit tests
cargo build              # build only
```

REPL notes:

- One `VM` lives across lines, so globals and functions persist.
- Each line gets a fresh `Chunk` / `CallFrame` compiled from scratch; only
  `globals` (and the value stack, which drains to empty between lines) carry
  over.
- Multiline input is supported: the REPL buffers until braces `{}` and parens
  `()` balance (ignoring `"..."` strings and `//` comments). Fresh input shows
  `>> `, continuations show `.. `.

```
>> fun add(a, b) {
.. return a + b;
.. }
>> print(add(1, 2));
3
```

## Language tour

```zeta
let x = 10;
print(x + 20);               // 30
print("Hello from", "Zeta"); // multi-arg, space-joined, returns nil
print("foo" + "bar");        // string concat: foobar

print(true and "yes");       // yes  (and returns left or right value)
print(false and "yes");      // false (right side skipped)
print(nil or "dflt");        // dflt
print(1 or print("NOPE"));   // 1, right side never runs

if (x > 5) {
  print("big");
} else {
  print("small");
}

let i = 0;
while (i < 3) {
  print("i:", i);
  i = i + 1;
}

fun add(a, b) { return a + b; }
print(add(3, 4));            // 7

fun fact(n) {
  if (n <= 1) { return 1; }
  return n * fact(n - 1);
}
print(fact(5));              // 120
```

## Language implementation

| Stage     | Files                                                              | What it does                                                                            |
| --------- | ------------------------------------------------------------------ | --------------------------------------------------------------------------------------- |
| Lexing    | `src/lexer.rs`, `src/token.rs`                                     | Single-char tokens, == != <= >=, numbers, strings, identifiers/keywords, // comments    |
| Parsing   | `src/parser.rs`, `src/expr.rs`, `src/stmt.rs`, `src/precedence.rs` | Pratt parser, precedence climbing → Expr / Stmt AST                                     |
| Compiling | `src/compiler.rs`, `src/chunk.rs`, `src/opcode.rs`                 | Bytecode + constant pool, locals/scope tracking, jump backpatching, per-function chunks |
| Runtime   | `src/value.rs`, `src/vm.rs`                                        | Stack VM, call frames, globals table                                                    |
| Driver    | `src/main.rs`                                                      | interpret(source, vm) wiring; file mode vs REPL                                         |

Opcodes: `Constant`, `Nil`, `True`, `False`, `Pop`, `Negate`, `Not`,
`Add`, `Subtract`, `Multiply`, `Divide`, `Equal`, `Greater`, `Less`,
`DefineGlobal`, `GetGlobal`, `SetGlobal`, `GetLocal`, `SetLocal`,
`Jump`, `JumpIfFalse`, `JumpIfTrue`, `Loop`, `Call`, `Return`.

There is intentionally no `Print` opcode — `print` is a native function
(`Value::NativeFn`) registered in `VM` globals and invoked through the normal
`Call` path.

## Design decisions

### Variable resolution & scoping

- Locals resolve entirely at compile time to fixed stack slots. Scope
  boundaries are visible in source text, so the compiler tracks active locals
  by walking linearly.
- Globals resolve by name at runtime via hashtable, since execution order
  (and, in a REPL, compile order) isn't knowable at compile time.
- Locals are `Vec<Local { name, depth }>` in declaration order. Shadowing
  resolves by searching from the end (most-recent wins). Scope exit pops
  while `local.depth > current_scope_depth`.
- `scope_depth == 0` → global; `> 0` → local.
- `let x;` desugars to `let x = nil;`; bare `return;` desugars the same way.

### Assignment

- `SetGlobal` / `SetLocal` peek rather than pop, so assignment is an
  expression (e.g. `print(x = 5)` works).
- Assigning to an undeclared global is a runtime error, not an implicit
  declaration.

### Functions

- Each function compiles into its own `Chunk` (own constants / code /
  locals) via a fresh nested `Compiler` — no constant-pool or slot
  collisions between scopes.
- Values are `Value::Fun(Rc<Chunk>)`. `Rc` keeps cloning cheap (refcount
  bump) since function values are copied constantly, e.g. pushed per call.
- Recursion: the function's own name is a phantom local at slot 0 of its own
  compiler, ahead of parameters. At runtime that slot already holds the
  function value the caller pushed, so self-reference resolves for free.
- `Chunk` carries `arity: usize`, set from the parameter count.
  `Call` checks `arg_count == arity` before pushing a frame.

### Calling convention / call frames

- A frame's `base` points at the function value's own slot, not the first
  argument — consistent with slot-0 self-reference.
- One shared `Vec<Value>` across frames; each
  `CallFrame { chunk: Rc<Chunk>, ip, base }` remembers where its locals begin.
- Top-level code runs as an ordinary `CallFrame` (`base: 0`) — one loop
  handles top-level, single calls, and recursion uniformly.
- `Return`: pop value, pop frame, `truncate(frame.base)`, push value back.

### Control flow

- Forward jumps (`if`/`else`) use backpatching: emit opcode + 2-byte
  placeholder, record the position, overwrite once the target is compiled.
- Operands are 2 bytes to avoid a 255-byte block limit.
- `JumpIfFalse` peeks, never pops — the compiler emits explicit `Pop` on both
  paths. Kept uniform because `while` re-peeks the condition each iteration.
- Backward jumps use distinct `Loop` rather than negative `Jump`, since
  operands are unsigned.

### Logical operators

- `and` / `or` are short-circuiting and return operand values, not strict
  bools: `false and x` → `false`, `nil or "hi"` → `"hi"`.
- `and` compiles to `left; JumpIfFalse end; Pop; right; end:`. Falsy `left`
  lands on `end` with `left` as the result; otherwise `left` is popped and
  `right` becomes the result.
- `or` mirrors it with `JumpIfTrue`.
- `and` binds tighter than `or`; both are left-associative.

### Values & truthiness

- Only `Equal`, `Greater`, `Less` are VM primitives. `NotEqual`,
  `GreaterEqual`, `LessEqual` derive at compile time (e.g. `>=` → `Less` +
  `Not`).
- Truthiness is deliberate: `false`, `nil`, `0`, and `""` are falsy;
  functions and native functions are always truthy.
- `Add` accepts `Number + Number` and `String + String` (concatenation).
  Mixed types are a runtime error.
- Opcodes are byte-packed `Vec<u8>`, clox-style, for the learning experience
  and future benchmarking.

### Native functions

- `print` is `Value::NativeFn(fn(&[Value]) -> Value)`, registered in globals
  before the main loop — not a dedicated opcode. It stays an ordinary
  callable value, and future natives need no new opcodes or compiler changes.

### Error handling (provisional)

- All runtime errors currently `panic!` (type mismatch, undefined variable,
  divide-by-zero, arity mismatch, calling a non-function). Deliberate
  simplification for this stage.
- Line tracking is paused — all instructions emit line `0` — to revisit with
  the error-handling design.

## Planned features

- [ ] Closures / upvalues
- [ ] `for` loops (C-style vs iterator-style undecided)
- [ ] Structs, enums, pattern matching / `match`
- [ ] Pipe operator `|>` (precedence slot reserved)
- [ ] `?` error propagation
- [ ] String escapes
- [ ] Fallible natives, stack/frame limits, chunk disassembler
