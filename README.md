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
expr-solver-lib = "0.1.0"
```

### Basic Example

```rust
use expr_solver::{Eval};

fn main() {
    let mut eval = Eval::new("2+3*4");
    match eval.eval() {
        Ok(result) => println!("Result: {}", result),
        Err(e) => eprintln!("Error: {}", e),
    }
    
}
```

This will evaluate the hardcoded expression `2 + 3 * 4` and print the result.

## Testing

Run the test suite:

```bash
# Run all tests
cargo test
```

## License

This project is licensed under the MIT License.
