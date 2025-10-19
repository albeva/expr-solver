# V2 Architecture Migration Guide

## Overview

The v2 implementation introduces a **type-state pattern** for `Program` with clear state transitions and improved architecture. The key improvement is that `Program` now owns its `SymTable` after linking, allowing for modification and better encapsulation.

## Key Improvements

✅ **Type-safe state transitions** - Impossible to execute unlinked programs
✅ **Program owns SymTable** - Can modify constants/functions after linking
✅ **Better serialization** - Includes symbol metadata for validation
✅ **Index remapping** - Bytecode works with any compatible SymTable
✅ **Cleaner API** - Clear flow: parse → compile → link → execute

## Architecture Comparison

### V1 (Original)
```rust
// V1: Multiple components, external symbol table
let source = Source::new("sin(pi / 2)");
let mut parser = Parser::new(source);
let mut ast = parser.parse()?;
Sema::new(&table).visit(&mut ast)?;
let program = IrBuilder::new().build(&ast)?;
let result = Vm::default().run(&program, &table)?;
```

### V2 (New)
```rust
// V2: Unified Program with type states
let source = Source::new("sin(pi / 2)".to_string());
let program = Program::new_from_source(source)
    .parse()?           // → Program<Parsed>
    .compile()?         // → Program<Compiled>
    .link(table)?;      // → Program<Linked>

let result = program.execute()?;
```

## Program States

### 1. `Program<Initial>`
Created from source code or file path.

```rust
// From source
let program = Program::new_from_source(Source::new("2 + 3"));

// From file
let program = Program::new_from_file("program.bin".to_string());
```

### 2. `Program<Parsed>`
After parsing source to AST.

```rust
let parsed = program.parse()?;
// Contains: Source + AST with owned strings
```

### 3. `Program<Compiled>`
After compilation to bytecode with symbol metadata.

```rust
let compiled = parsed.compile()?;
// Contains: Bytecode + SymbolMetadata[]
// Indices in bytecode reference metadata positions
```

### 4. `Program<Linked>`
After linking with a SymTable - ready to execute.

```rust
let linked = compiled.link(SymTable::stdlib())?;
// Contains: Bytecode + SymTable (owned)
// Indices in bytecode now reference SymTable positions
```

## Execution Paths

### Path 1: From Source
```
Source
  → parse()
  → Program<Parsed>
  → compile()
  → Program<Compiled>
  → link(table)
  → Program<Linked>
  → execute()
```

### Path 2: From Binary File
```
File path
  → deserialize(bytes)
  → Program<Compiled>
  → link(table)
  → Program<Linked>
  → execute()
```

## Key Features

### 1. Mutable SymTable

```rust
let source = Source::new("x + y".to_string());
let mut program = Program::new_from_source(source)
    .parse()?
    .compile()?;

// Create custom symbol table
let mut table = SymTable::new();
table.add_const("x", dec!(10))?;
table.add_const("y", dec!(20))?;

let mut program = program.link(table)?;

// Modify symbols after linking!
program.symtable_mut().add_const("z", dec!(100))?;
```

### 2. Serialization with Validation

```rust
// Compile and link
let program = Program::new_from_source(source)
    .parse()?
    .compile()?
    .link(SymTable::stdlib())?;

// Serialize (includes symbol metadata)
let bytes = program.serialize()?;
std::fs::write("program.bin", bytes)?;

// Later: deserialize and link with compatible table
let bytes = std::fs::read("program.bin")?;
let program = Program::new_from_file("program.bin".to_string())
    .deserialize(&bytes)?
    .link(SymTable::stdlib())?;  // Validates symbols match!

program.execute()?;
```

### 3. Index Remapping

The v2 implementation uses a clever **two-phase indexing** system:

#### Phase 1: Compilation (Metadata Indices)
```
bytecode: [LOAD 0, PUSH 2, DIV, CALL 1 1]
metadata: [
  0: { name: "pi", kind: Const },
  1: { name: "sin", kind: Func{arity: 1} }
]
```

#### Phase 2: Linking (SymTable Indices)
```rust
// User's SymTable may have different ordering:
// 0: "e", 1: "tau", 2: "pi", ..., 15: "sin"

// link() remaps indices:
bytecode: [LOAD 2, PUSH 2, DIV, CALL 15 1]
//              ^                     ^^
//         pi now at 2           sin now at 15
```

This allows:
- Different SymTable implementations
- Adding new symbols without breaking existing binaries
- Reordering symbols freely

### 4. Utility Methods

```rust
// Get assembly representation
let asm = program.get_assembly();
println!("{}", asm);

// List symbols used in program
let symbols = program.emit_symbols();
for sym in symbols {
    println!("Uses: {}", sym);
}

// Access symbol table
let table = program.symtable();
let e_value = table.get("e");
```

## Error Handling

### LinkError

Occurs when bytecode requirements don't match SymTable:

```rust
// Bytecode needs "x" constant, but table provides "x" function
LinkError::TypeMismatch {
    name: "x",
    expected: "constant",
    found: "function"
}

// Bytecode needs symbol not in table
LinkError::MissingSymbol { name: "foo" }
```

### CompileError

Occurs during bytecode generation:

```rust
// Semantic errors (undefined symbols, wrong arity, etc.)
CompileError::SemanticError(...)

// Code generation failures
CompileError::CodeGenError("...")
```

## Migration Checklist

If migrating existing code from v1 to v2:

- [ ] Replace separate Parser/Sema/IrBuilder calls with unified Program API
- [ ] Update to use type-state transitions (parse → compile → link)
- [ ] Store Program<Linked> instead of separate Program + SymTable
- [ ] Use `program.execute()` instead of `vm.run(&program, &table)`
- [ ] Update serialization to use `program.serialize()` / `deserialize()`
- [ ] Use `program.symtable_mut()` for modifying symbols
- [ ] Handle new error types (LinkError, CompileError, ProgramError)

## Examples

See `lib/tests/v2_integration_test.rs` for comprehensive examples including:
- Basic arithmetic
- Functions and constants
- SymTable mutation
- Serialization/deserialization
- Assembly generation
- Symbol extraction
- Link validation

## Performance Notes

- **Parsing**: Slightly slower due to string allocation (owned strings in AST)
- **Compilation**: Adds symbol discovery pass, but overall similar performance
- **Linking**: New index remapping step (O(n) where n = symbols used)
- **Execution**: **Identical to v1** - same VM, same bytecode format

## Future Enhancements

Potential improvements for future versions:
- [ ] Optimize parser to avoid string allocations where possible
- [ ] Add bytecode optimization passes
- [ ] Support for multiple symbol table namespaces
- [ ] Incremental compilation/linking
- [ ] Debug information in compiled programs

## Questions?

The v2 implementation is fully backward compatible at the VM level - v1 and v2 generate the same bytecode format and use the same VM. The main differences are in the API design and ownership model.
