//! Integration tests for v2 implementation

use expr_solver::v2::{Program, Source};
use expr_solver::SymTable;
use rust_decimal_macros::dec;

#[test]
fn test_v2_basic_arithmetic() {
    let source = Source::new("2 + 3 * 4");
    let program = Program::new_from_source(source)
        .parse()
        .expect("parse failed")
        .compile()
        .link(SymTable::stdlib())
        .expect("link failed");

    let result = program.execute().expect("execution failed");
    assert_eq!(result, dec!(14));
}

#[test]
fn test_v2_with_constants() {
    let source = Source::new("pi * 2");
    let program = Program::new_from_source(source)
        .parse()
        .expect("parse failed")
        .compile()
        .link(SymTable::stdlib())
        .expect("link failed");

    let result = program.execute().expect("execution failed");
    // pi * 2 ≈ 6.28...
    assert!(result > dec!(6.28) && result < dec!(6.29));
}

#[test]
fn test_v2_with_functions() {
    let source = Source::new("sqrt(16) + sin(0)");
    let program = Program::new_from_source(source)
        .parse()
        .expect("parse failed")
        .compile()
        .link(SymTable::stdlib())
        .expect("link failed");

    let result = program.execute().expect("execution failed");
    assert_eq!(result, dec!(4)); // sqrt(16) + sin(0) = 4 + 0 = 4
}

#[test]
fn test_v2_symtable_mutation() {
    let source = Source::new("x + y");
    let program = Program::new_from_source(source)
        .parse()
        .expect("parse failed")
        .compile();

    // Create symbol table with x and y
    let mut table = SymTable::new();
    table.add_const("x", dec!(10)).unwrap();
    table.add_const("y", dec!(20)).unwrap();

    let mut program = program.link(table).expect("link failed");

    // First execution
    let result = program.execute().expect("execution failed");
    assert_eq!(result, dec!(30));

    // Modify symbol table
    program.symtable_mut().add_const("z", dec!(100)).unwrap();

    // Execute again (x + y should still be 30)
    let result = program.execute().expect("execution failed");
    assert_eq!(result, dec!(30));
}

#[test]
fn test_v2_serialization() {
    let source = Source::new("sqrt(pi) + 2");
    let program = Program::new_from_source(source)
        .parse()
        .expect("parse failed")
        .compile()
        .link(SymTable::stdlib())
        .expect("link failed");

    // Execute original
    let result1 = program.execute().expect("execution failed");

    // Serialize
    let bytes = program.serialize().expect("serialization failed");

    // Deserialize
    let program2 = Program::new_from_file("test.bin".to_string())
        .deserialize(&bytes)
        .expect("deserialization failed")
        .link(SymTable::stdlib())
        .expect("link failed");

    // Execute deserialized
    let result2 = program2.execute().expect("execution failed");

    assert_eq!(result1, result2);
}

#[test]
fn test_v2_get_assembly() {
    let source = Source::new("2 + 3");
    let program = Program::new_from_source(source)
        .parse()
        .expect("parse failed")
        .compile()
        .link(SymTable::stdlib())
        .expect("link failed");

    let assembly = program.get_assembly();
    assert!(assembly.contains("PUSH"));
    assert!(assembly.contains("ADD"));
}

#[test]
fn test_v2_emit_symbols() {
    let source = Source::new("sin(pi) + sqrt(e)");
    let program = Program::new_from_source(source)
        .parse()
        .expect("parse failed")
        .compile()
        .link(SymTable::stdlib())
        .expect("link failed");

    let symbols = program.emit_symbols();
    assert!(symbols.contains(&"sin".to_string()));
    assert!(symbols.contains(&"sqrt".to_string()));
    assert!(symbols.contains(&"pi".to_string()));
    assert!(symbols.contains(&"e".to_string()));
}

#[test]
fn test_v2_link_validation() {
    let source = Source::new("x + y");
    let program = Program::new_from_source(source)
        .parse()
        .expect("parse failed")
        .compile();

    // Try to link with empty symbol table (should fail)
    let empty_table = SymTable::new();
    let result = program.link(empty_table);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Missing symbol"));
}
