[![Rust](https://github.com/albeva/expr-solver/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/albeva/expr-solver/actions/workflows/rust.yml)

# expr-solver

A mathematical expression evaluator library written in Rust with support for custom functions, constants, and bytecode compilation.

## Features

- **Mathematical expressions** - Arithmetic, comparisons, and built-in functions
- **128-bit decimal precision** - No floating-point errors using `rust_decimal`
- **Custom symbols** - Register your own constants and functions
- **Rich error messages** - Syntax errors with source location highlighting
- **Bytecode compilation** - Compile expressions to portable binary format
- **Stack-based VM** - Efficient execution on a virtual machine

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
expr-solver-lib = "1.1.1"
```

### As a binary

Add this to your `Cargo.toml`:

```toml
[dependencies]
expr-solver-bin = "1.1.1"
```

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
use expr_solver::{eval_with_table, SymTable};
use rust_decimal_macros::dec;

let mut table = SymTable::stdlib();
table.add_const("x", dec!(10)).unwrap();
table.add_func("double", 1, false, |args| Ok(args[0] * dec!(2))).unwrap();

let result = eval_with_table("double(x)", table).unwrap();
assert_eq!(result, dec!(20));
```

### Compile Once, Execute Many Times

```rust
use expr_solver::{load, SymTable};
use rust_decimal_macros::dec;

// Compile expression
let program = load("x * 2 + y").unwrap();

// Execute with different values
let mut table = SymTable::new();
table.add_const("x", dec!(10)).unwrap();
table.add_const("y", dec!(5)).unwrap();

let linked = program.link(table).unwrap();
let result = linked.execute().unwrap(); // 25
```

## Precision

Uses **128-bit `Decimal`** arithmetic for exact decimal calculations without floating-point errors.

## Built-in Functions

| Category       | Functions                                                                 |
|----------------|---------------------------------------------------------------------------|
| **Arithmetic** | `abs`, `sign`, `floor`, `ceil`, `round`, `trunc`, `fract`, `mod`, `clamp` |
| **Trig**       | `sin`, `cos`, `tan`, `asin`*, `acos`*, `atan`*, `atan2`*                  |
| **Hyperbolic** | `sinh`*, `cosh`*, `tanh`*                                                 |
| **Exp/Log**    | `sqrt`, `cbrt`*, `pow`, `exp`, `exp2`*, `log`, `log2`*, `log10`, `hypot`* |
| **Variadic**   | `min`, `max`, `sum`, `avg` (1+ args)                                      |
| **Special**    | `if(cond, then, else)`                                                    |

\* *Uses f64 internally, may have minor precision differences*

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

# Use the -e flag
expr-solver -e "sin(pi/2)"

# Define custom constants
expr-solver -D x=10 -D y=20 "x + y"

# Compile to binary
expr-solver -e "2+3*4" -o expr.bin

# Execute compiled binary
expr-solver -i expr.bin

# View assembly from expression or file
expr-solver -e "2+3" -a
expr-solver -i expr.bin -a

# Recompile bytecode (e.g., version migration)
expr-solver -i old.bin -o new.bin

# List available functions and constants
expr-solver -t
```

## Testing

Run the test suite:

```bash
# Run all tests
cargo test
```

## License

This project is licensed under the MIT License.
