[![Rust](https://github.com/albeva/expr-solver/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/albeva/expr-solver/actions/workflows/rust.yml)

# expr-solver

A simple mathematical expression solver library written in Rust.

## Features

- **Evaluate simple math expressions**
- **Register custom constants and functions**
- **Proper error handling** with source location information
- **Compilation to bytecode** with serialization support
- **Stack-based virtual machine** for efficient execution

## Usage

### As a Library

Add this to your `Cargo.toml`:

```toml
[dependencies]
expr-solver-lib = "1.0.2"
```

### As a binary

Add this to your `Cargo.toml`:

```toml
[dependencies]
expr-solver-bin = "1.0.2"
```

### Basic Example

```rust
use expr_solver::Eval;

fn main() {
    // Quick one-liner evaluation
    match Eval::evaluate("2+3*4") {
        Ok(result) => println!("Result: {}", result),
        Err(e) => eprintln!("Error: {}", e),
    }

    // Or create an evaluator instance for more control
    let mut eval = Eval::new("sqrt(16) + pi");
    match eval.run() {
        Ok(result) => println!("Result: {}", result),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

This will evaluate mathematical expressions and print the results.

## Testing

Run the test suite:

```bash
# Run all tests
cargo test
```

## License

This project is licensed under the MIT License.
