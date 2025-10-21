[![Rust](https://github.com/albeva/expr-solver/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/albeva/expr-solver/actions/workflows/rust.yml)

# expr-solver

A mathematical expression evaluator library written in Rust with support for custom functions, constants, and bytecode compilation.

## Features

- **Mathematical expressions** - Arithmetic, comparisons, and built-in functions
- **Flexible numeric types** - Choose between fast f64 or high-precision 128-bit Decimal
- **Custom symbols** - Register your own constants and functions
- **Rich error messages** - Syntax errors with source location highlighting
- **Bytecode compilation** - Compile expressions to portable binary format
- **Stack-based VM** - Efficient execution on a virtual machine
- **Pretty printing** - Decompile bytecode to syntax-highlighted expressions (optional feature)

## What's it For?

Need to evaluate math but don't want to embed an entire scripting language? You're in the right place. 🎯

**Perfect for:**
- **Game engines** - Calculate damage formulas, level requirements, or item stats without hardcoding values
- **Configuration files** - Let users write `price * 0.9` instead of forcing them to update every value manually
- **Analytics dashboards** - Enable power users to define custom metrics: `(revenue - cost) / users`
- **Form calculators** - Mortgage calculators, unit converters, or any user-facing math
- **Educational tools** - Math tutoring apps, graphing calculators, or homework helpers
- **Learning compilers** - Clean example of lexer, parser, and stack-based VM in ~2K lines of Rust

## How It Works

Classic compiler pipeline with type-safe state transitions:

```
Input → Lexer → Parser → Compiler → Program<Compiled>
                                           ↓ link
                                    Program<Linked> → Execute
```

The `Program` type uses Rust's type system to enforce correct usage at compile time. You cannot execute an unlinked program, and you cannot link a program twice.

## Usage

### As a Library

Add this to your `Cargo.toml`:

```toml
[dependencies]
expr-solver-lib = "1.2.0"
```

**Numeric Type Selection:**

The library supports two numeric backends via feature flags (mutually exclusive):

- **`f64-floats`** (default) - Standard f64 floating-point arithmetic. Faster and simpler, allows Inf and NaN results.
- **`decimal-precision`** - 128-bit Decimal for high precision. No floating-point errors, checked arithmetic with overflow detection.

To use high-precision Decimal:

```toml
[dependencies]
expr-solver-lib = { version = "1.2.0", default-features = false, features = ["decimal-precision"] }
```

To enable bytecode serialization with f64:

```toml
[dependencies]
expr-solver-lib = { version = "1.2.0", features = ["serialization"] }
```

To enable bytecode serialization with Decimal:

```toml
[dependencies]
expr-solver-lib = { version = "1.2.0", default-features = false, features = ["decimal-precision", "serialization"] }
```

**Optional Features:**

- **`printing`** (enabled by default) - Pretty printing and syntax highlighting. Disable to reduce dependencies:
  ```toml
  expr-solver-lib = { version = "1.2.0", default-features = false, features = ["f64-floats"] }
  ```

### As a Command-Line Tool

Install using cargo:

```bash
cargo install expr-solver-bin
```

The command-line tool uses **Decimal precision by default** for maximum accuracy.

### Quick Evaluation

```rust
use expr_solver::eval;

// Simple one-liner
let result = eval("2 + 3 * 4").unwrap();
assert_eq!(result.to_string(), "14");

// With built-in functions
let result = eval("sqrt(16) + sin(pi/2)").unwrap();
```

### Custom Symbols

```rust
use expr_solver::{eval_with_table, SymTable, Number, ParseNumber};

let mut table = SymTable::stdlib();
table.add_const("x", Number::parse_number("10").unwrap()).unwrap();
table.add_func("double", 1, false, |args| {
    Ok(args[0] * Number::parse_number("2").unwrap())
}).unwrap();

let result = eval_with_table("double(x)", table).unwrap();
assert_eq!(result.to_string(), "20");
```

Or with f64 (default):

```rust
use expr_solver::{eval_with_table, SymTable};

let mut table = SymTable::stdlib();
table.add_const("x", 10.0).unwrap();
table.add_func("double", 1, false, |args| Ok(args[0] * 2.0)).unwrap();

let result = eval_with_table("double(x)", table).unwrap();
assert_eq!(result, 20.0);
```

### Compile Once, Execute Many Times

```rust
use expr_solver::{Program, SymTable};

// Compile expression
let program = Program::new_from_source("x * 2 + y").unwrap();

// Execute with different values
let mut table = SymTable::new();
table.add_const("x", 10.0).unwrap();
table.add_const("y", 5.0).unwrap();

let linked = program.link(table).unwrap();
let result = linked.execute().unwrap(); // 25.0
assert_eq!(result, 25.0);
```

## Numeric Precision

The library supports two numeric backends:

### f64 (Default)
- Standard IEEE 754 double-precision floating-point
- Fast and efficient for most use cases
- Allows `Inf` and `NaN` results (e.g., `1/0` → `Inf`, `sqrt(-1)` → `NaN`)
- Minimal error checking - only prevents panics

### Decimal (Optional)
- 128-bit fixed-point arithmetic via `rust_decimal`
- Exact decimal representation (no 0.1 + 0.2 ≠ 0.3 issues)
- Checked arithmetic with overflow/underflow detection
- Domain validation (e.g., `sqrt(-1)` returns an error)
- Ideal for financial calculations or when exact decimal precision is required

**Choosing the right mode:**
- Use **f64** (default) for general-purpose math, scientific computing, or when performance is critical
- Use **Decimal** for financial applications, accounting, or when exact decimal representation is required

## Built-in Functions

| Category       | Functions                                                                 |
|----------------|---------------------------------------------------------------------------|
| **Arithmetic** | `abs`, `sign`, `floor`, `ceil`, `round`, `trunc`, `fract`, `mod`, `clamp` |
| **Trig**       | `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `atan2`                      |
| **Hyperbolic** | `sinh`, `cosh`, `tanh`                                                    |
| **Exp/Log**    | `sqrt`, `cbrt`, `pow`, `exp`, `exp2`, `log`, `log2`, `log10`, `hypot`     |
| **Variadic**   | `min`, `max`, `sum`, `avg` (1+ args)                                      |
| **Special**    | `if(cond, then, else)`                                                    |

> **Note:** In `decimal-precision` mode, some operations (inverse trig, `pow`) use internal f64 conversion due to `rust_decimal` limitations, which may introduce small precision loss.

## Built-in Constants

`pi`, `e`, `tau`, `ln2`, `ln10`, `sqrt2`

> All names are case-insensitive.

## Operators

**Arithmetic**: `+`, `-`, `*`, `/`, `^` (power), `!` (factorial), unary `-`
**Comparison**: `==`, `!=`, `<`, `<=`, `>`, `>=` (returns 1 or 0)
**Grouping**: `(` `)`

## Command Line Usage

```bash
# Evaluate an expression
expr-solver "2 + 3 * 4"
# Output: 14

# Use the -e flag
expr-solver -e "sin(pi/2)"
# Output: 1

# Define custom constants
expr-solver -D x=10 -D y=20 "x + y"
# Output: 30

# Print syntax-highlighted expression (decompiled from bytecode)
expr-solver -e "let x = 10 then x * 2" --print
# Output: let x = 10 then x * 2

# View assembly (bytecode instructions)
expr-solver -e "2 + 3 * 4" --assembly
# Output:
# ; VERSION 1.2.0
# 0000 PUSH 2
# 0001 PUSH 3
# 0002 PUSH 4
# 0003 MUL
# 0004 ADD

# Compile to binary file
expr-solver -e "2+3*4" -o expr.bin

# Execute compiled binary
expr-solver -i expr.bin
# Output: 14

# View assembly from compiled file
expr-solver -i expr.bin -a

# Recompile bytecode (e.g., version migration)
expr-solver -i old.bin -o new.bin

# List available functions and constants
expr-solver -t
```

**Note:** The command-line tool uses **128-bit Decimal precision** by default for accurate calculations:
- `0.1 + 0.2` correctly equals `0.3` (no floating-point errors)
- `1 / 3` displays 28 decimal places: `0.3333333333333333333333333333`

## Testing

Run the test suite:

```bash
# Run all tests with default f64 mode
cargo test

# Test with decimal-precision mode
cargo test -p expr-solver-lib --no-default-features --features decimal-precision

# Test binary with f64 mode
cargo test -p expr-solver-bin
```

## License

This project is licensed under the MIT License.
