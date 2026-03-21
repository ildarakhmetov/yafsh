# CLAUDE.md — AI Assistant Guide for yafsh

## Project Overview

**yafsh** (Yet Another Forth SHell) is a Forth-inspired, stack-based interactive shell written in Rust. It uses Reverse Polish Notation (RPN) and a dictionary of "words" (functions) to provide a fully-featured REPL with syntax highlighting, history, tab completion, and full shell integration.

- **Language**: Rust (Edition 2021)
- **Version**: 0.5.0
- **License**: MIT
- **Primary dependency**: `rustyline` v17

---

## Repository Structure

```
yafsh/
├── src/
│   ├── main.rs           # CLI entry point, REPL loop (interactive + pipe mode)
│   ├── lib.rs            # Library root — re-exports all modules
│   ├── types.rs          # Core types: Value, Word, State, LoopInfo, ControlFlow
│   ├── eval.rs           # Core evaluation engine (token dispatch, control flow)
│   ├── tokenizer.rs      # Lexer: quote-aware, position-tracking
│   ├── config.rs         # Config paths: ~/.yafshrc, ~/.yafsh_history, VERSION
│   ├── highlight.rs      # ANSI syntax highlighting via rustyline
│   ├── multiline.rs      # Incomplete input detection (unclosed quotes, keywords)
│   ├── loops.rs          # Loop engines: begin/until, begin/while, do/loop, do/+loop
│   └── builtins/
│       ├── mod.rs         # Registers all builtins into State
│       ├── stack.rs       # dup, swap, drop, clear, over, rot
│       ├── io.rs          # ., type, .s, >output, >string, >file, >>file
│       ├── computation.rs # Arithmetic, comparisons, boolean, string ops, loop indices
│       ├── system.rs      # exec, ?, cd, getenv, setenv, env-*, pushd, popd
│       └── introspection.rs # words, help, see, trace, $stack, $cwd, $gitbranch, etc.
├── tests/
│   └── eval_integration.rs  # 155 integration tests
├── Cargo.toml
├── Cargo.lock
├── LICENSE
└── README.md
```

---

## Core Architecture

### Type System (`src/types.rs`)

Three value types on the stack:

```rust
pub enum Value {
    Str(String),     // User input or string literals
    Int(i64),        // Integers
    Output(String),  // Shell command output (auto-pipes to next shell command)
}
```

The `Output` type is special — when passed to a shell command, it becomes stdin automatically.

### State (`src/types.rs`)

All mutable interpreter state lives in `State`:

```rust
pub struct State {
    pub stack: Vec<Value>,
    pub dict: HashMap<String, Word>,
    pub defining: Option<String>,        // Active `: name` definition
    pub def_body: Vec<String>,           // Tokens being collected for definition
    pub last_exit_code: i32,
    pub control_flow: ControlFlow,       // if/then/else tracking
    pub dir_stack: Vec<String>,          // pushd/popd
    pub loop_stack: Vec<LoopInfo>,       // Active loops (for i/j indices)
    pub collecting_loop: Option<...>,    // Loop body being collected
    pub collecting_each: Option<...>,    // each body being collected
    pub trace: u8,                       // 0=off, 1=tokens, 2=+stack, 3=+dict
    pub trace_step: usize,
    // prompt-related fields
}
```

### Word Dictionary (`src/types.rs`)

```rust
pub enum Word {
    Builtin(BuiltinFn, Option<&'static str>),  // Native function + docstring
    Defined(Vec<String>),                       // User-defined list of tokens
    ShellCmd(String),                           // Cached PATH-resolved command
}
```

### Evaluation Engine (`src/eval.rs`)

Token dispatch order:
1. Check `state.dict` — builtins and user-defined words
2. Check PATH for shell executables → cached as `ShellCmd`
3. Push as `Str` (fallback)

### Loop Engine (`src/loops.rs`)

Four loop types, all supporting nesting with `i` (inner index) and `j` (outer index):
- `begin ... until` — repeat until top of stack is true
- `begin ... while ... repeat` — repeat while condition holds
- `N M do ... loop` — counted loop from N to M
- `N M do ... STEP +loop` — counted loop with custom step

---

## Development Workflows

### Build

```bash
cargo build            # Debug build → target/debug/yafsh
cargo build --release  # Optimized build → target/release/yafsh
```

### Run

```bash
cargo run              # Run interactive REPL
cargo run -- script.fsh  # Run a script file (piped to stdin)
echo "2 3 +" | cargo run  # Pipe mode
```

### Test

```bash
cargo test             # Run all 155 integration tests
cargo test <pattern>   # Run matching tests (e.g., cargo test loop)
```

All tests are in `tests/eval_integration.rs`. Use the test helpers:

```rust
fn new_state() -> State        // Fresh state with all builtins registered
fn eval(line: &str) -> Vec<Value>        // Evaluate one line, return stack
fn eval_lines(lines: &[&str]) -> State   // Evaluate multiple lines, return state
```

### Version bumping

Update `VERSION` in **both** `Cargo.toml` and `src/config.rs`.

---

## Key Conventions

### Adding a Builtin

1. Choose or create the appropriate module under `src/builtins/`
2. Write a function matching signature `fn name(state: &mut State) -> Result<(), String>`
3. Register it in `src/builtins/mod.rs` via `register()`:
   ```rust
   state.dict.insert("word-name".to_string(), Word::Builtin(name, Some("doc string")));
   ```
4. Add integration tests in `tests/eval_integration.rs`

### Error Handling

- All builtin and eval functions return `Result<(), String>`
- Error strings are human-readable (displayed directly to the user)
- Always check stack depth before popping: return `Err("stack underflow".to_string())` if insufficient items

### Stack Operations Pattern

```rust
// Pop one value
let val = state.stack.pop().ok_or("stack underflow")?;

// Pop two values (note order: second is pushed first in RPN)
let b = state.stack.pop().ok_or("stack underflow")?;
let a = state.stack.pop().ok_or("stack underflow")?;
// Now a op b (e.g., a + b)
```

### Naming Conventions

- Rust modules/functions/variables: `snake_case`
- Types and enums: `PascalCase`
- Constants: `UPPER_CASE`
- Shell words in dictionary: lowercase with hyphens (e.g., `"env-append"`, `"?prefix"`)

### Adding Tests

Tests follow this pattern:

```rust
#[test]
fn test_my_feature() {
    let result = eval("1 2 my-word");
    assert_eq!(result, vec![Value::Int(3)]);
}
```

For stateful tests (definitions, env vars, directory):

```rust
#[test]
fn test_stateful() {
    let state = eval_lines(&[
        ": double dup + ;",
        "5 double",
    ]);
    assert_eq!(state.stack, vec![Value::Int(10)]);
}
```

---

## Module Responsibilities

| Module | Responsibility |
|---|---|
| `main.rs` | REPL loop, rustyline setup, RC file loading, prompt evaluation |
| `types.rs` | All type definitions — single source of truth for data shapes |
| `eval.rs` | Token dispatch, word definition, if/then/else, shell execution, glob expansion |
| `tokenizer.rs` | Quote-aware lexing with byte-offset positions for highlighting |
| `highlight.rs` | ANSI color highlighting via rustyline `Highlighter` trait |
| `multiline.rs` | Validates input completeness — detects unbalanced keywords/quotes |
| `loops.rs` | All loop execution logic with nested index support |
| `config.rs` | File paths (`~/.yafshrc`, `~/.yafsh_history`) and VERSION constant |
| `builtins/` | All built-in word implementations, organized by category |

---

## Syntax Highlighting Colors

| Color | Token type |
|---|---|
| Yellow | Quoted strings |
| Magenta | Keywords (`:`, `;`, `if`, `then`, `else`, `begin`, `do`, `loop`, etc.) |
| Cyan | Numbers |
| Green | Dictionary words (builtins and user-defined) |

---

## Shell Integration Details

- Shell commands are looked up via PATH and cached in the dictionary as `ShellCmd`
- `Output` values on the stack are automatically piped as stdin to the next shell command
- Exit codes are stored in `state.last_exit_code` and accessible via `$exitcode`
- Glob patterns (`*`, `?`) are expanded before passing arguments to shell commands

---

## User Configuration

- **RC file**: `~/.yafshrc` — evaluated on startup, define words and set prompt here
- **History**: `~/.yafsh_history` — persisted between sessions
- **Custom prompt**: Define the word `$prompt` to customize the shell prompt

Example `~/.yafshrc`:
```forth
: $prompt $gitbranch " [" swap concat "] " concat $cwd concat ;
```

---

## No CI/CD

There is currently no CI/CD configuration. Tests must be run manually with `cargo test` before committing.
