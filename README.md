# Expression Solver Workspace

A Rust workspace containing a mathematical expression solver library and command-line interface.

## Project Structure

This workspace is organized into two main components:

- **`expr-solver-lib/`** - The core expression solver library
- **`expr-solver-bin/`** - Command-line interface that uses the library

## Quick Start

### Building the Project

```bash
# Build everything
cargo build

# Build just the library
cargo build -p expr-solver-lib

# Build just the binary
cargo build -p expr-solver-bin
```

### Running the Binary

```bash
# Run the command-line tool
cargo run -p expr-solver-bin

# Or after building
./target/debug/expr-solver
```

### Testing

```bash
# Run all tests
cargo test

# Run only library tests
cargo test -p expr-solver-lib

# Run only integration tests
cargo test -p expr-solver-lib --test integration_tests

# Run doc tests
cargo test -p expr-solver-lib --doc
```

## Components

### expr-solver-lib

The main library providing mathematical expression parsing and evaluation capabilities.

**Features:**
- Mathematical operators: `+`, `-`, `*`, `/`, `^`, unary `-`, `!` (factorial)
- Built-in constants: `pi`, `e`, `tau`, `ln2`, `ln10`, `sqrt2`, `infinity`
- Trigonometric functions: `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `atan2`, `sinh`, `cosh`, `tanh`
- Math functions: `sqrt`, `cbrt`, `pow`, `log`, `log2`, `log10`, `exp`, `exp2`, `abs`, `sign`
- Rounding functions: `floor`, `ceil`, `round`, `trunc`, `fract`, `mod`, `clamp`
- Variadic functions: `min`, `max`, `sum`, `avg`
- **Safe arithmetic** with overflow detection and error handling
- Excellent error messages with source location information
- Compilation to bytecode with serialization support
- Stack-based virtual machine for execution

**Usage:**
```rust
use expr_solver::Eval;

let mut eval = Eval::new("sin(pi / 2) + cos(0)").unwrap();
let result = eval.run().unwrap();
assert_eq!(result, 2.0);
```

See the [library README](lib/README.md) for detailed documentation.

## Safe Math Features

The expression evaluator includes comprehensive safe arithmetic operations that detect and handle mathematical errors gracefully:

### Error Detection
- **Division by zero**: `10 / 0` → `Safe math error: Division by zero`
- **Factorial overflow**: `200!` → `Safe math error: Arithmetic overflow`
- **Invalid factorial**: `(-5)!` → `Safe math error: Invalid operation: Factorial of negative number: -5`
- **Domain errors**: `(-2) ^ 0.5` → `Safe math error: Invalid operation: Cannot raise negative number to fractional power`
- **Arithmetic overflow**: `2 ^ 1024` → `Safe math error: Arithmetic overflow`

### Benefits
✓ No panics or crashes on mathematical errors  
✓ Descriptive error messages for debugging  
✓ NaN/Infinity result detection  
✓ Domain validation for mathematical functions  
✓ Overflow prevention for large calculations

### Example
```rust
use expr_solver::Eval;

// This would cause overflow in regular f64 arithmetic
let mut eval = Eval::new("200!").unwrap();
match eval.run() {
    Ok(result) => println!("Result: {}", result),
    Err(e) => println!("Error: {}", e), // "Safe math error: Arithmetic overflow"
}
```

Run the safe math demo:
```bash
cargo run --example safe_math_demo -p expr-solver-lib
```

### expr-solver-bin

A simple command-line interface that demonstrates the library usage.

**Features:**
- Evaluates a hardcoded mathematical expression
- Shows error handling with nice formatting
- Minimal example of library integration

## Development

### Workspace Commands

```bash
# Check all code
cargo check

# Format all code
cargo fmt

# Lint all code
cargo clippy

# Update dependencies
cargo update
```

### Adding Dependencies

Dependencies can be shared across the workspace by adding them to the `[workspace.dependencies]` section in the root `Cargo.toml`, then referencing them with `{ workspace = true }` in individual crates.

## Architecture

The expression solver follows a traditional compiler pipeline:

1. **Lexer** - Tokenizes input text
2. **Parser** - Builds Abstract Syntax Tree (AST)
3. **Semantic Analyzer** - Resolves symbols and validates semantics
4. **IR Builder** - Lowers AST to stack-based bytecode
5. **Virtual Machine** - Executes the bytecode

## License

This project is licensed under the MIT License.