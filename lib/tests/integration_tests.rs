use expr_solver::{Compiled, Linked, Number, Program, SymTable, eval, eval_with_table, num};
use indoc::indoc;

// Helper function to evaluate an expression and expect an Ok result.
fn eval_ok(expr: &str) -> Number {
    eval(expr).expect("Evaluation should be successful")
}

// Helper function to evaluate an expression and expect an Err result.
fn eval_err(expr: &str) -> String {
    colored::control::set_override(false);
    eval(expr).expect_err("Evaluation should fail")
}

// Helper function to evaluate an expression with a custom symbol table and expect an Ok result.
fn eval_with_custom_table_ok(expr: &str, table: SymTable) -> Number {
    eval_with_table(expr, table).expect("Evaluation should be successful")
}

// Helper function for approximate equality (for f64 mode)
#[cfg(not(feature = "decimal"))]
fn approx_eq(a: Number, b: Number, epsilon: f64) -> bool {
    (a - b).abs() < epsilon
}

#[cfg(feature = "decimal")]
fn approx_eq(a: Number, b: Number, epsilon: Number) -> bool {
    (a - b).abs() < epsilon
}

#[test]
fn test_arithmetic_and_precedence() {
    // Basic arithmetic
    assert_eq!(eval_ok("1 + 2"), num!(3));
    assert_eq!(eval_ok("10 - 5"), num!(5));
    assert_eq!(eval_ok("2 * 3"), num!(6));
    assert_eq!(eval_ok("10 / 2"), num!(5));
    assert_eq!(eval_ok("2 ^ 3"), num!(8));
    assert_eq!(eval_ok("-5"), num!(-5));
    assert_eq!(eval_ok("1 + -5"), num!(-4));
    assert_eq!(eval_ok("5!"), num!(120));
    assert_eq!(eval_ok("0!"), num!(1));

    // Operator precedence
    assert_eq!(eval_ok("1 + 2 * 3"), num!(7));
    assert_eq!(eval_ok("(1 + 2) * 3"), num!(9));
    assert_eq!(eval_ok("2 ^ 3 ^ 2"), num!(512)); // Right-associative
    assert_eq!(eval_ok("-2 ^ 2"), num!(-4));
    assert_eq!(eval_ok("3 + 4 * 2 / (1 - 5) ^ 2"), num!(3.5));
}

#[test]
fn test_comparisons() {
    assert_eq!(eval_ok("1 > 0"), num!(1));
    assert_eq!(eval_ok("1 < 0"), num!(0));
    assert_eq!(eval_ok("1 == 1"), num!(1));
    assert_eq!(eval_ok("1 != 1"), num!(0));
    assert_eq!(eval_ok("1 >= 1"), num!(1));
    assert_eq!(eval_ok("1 <= 1"), num!(1));
    assert_eq!(eval_ok("1 + 1 == 2"), num!(1));
}

#[test]
fn test_constants() {
    // Test that constants evaluate to expected string representations
    // Using string comparison for type-neutrality
    let pi_result = eval_ok("pi").to_string();
    assert!(pi_result.starts_with("3.14"));

    let e_result = eval_ok("e").to_string();
    assert!(e_result.starts_with("2.71"));

    let tau_result = eval_ok("tau").to_string();
    assert!(tau_result.starts_with("6.28"));

    let ln2_result = eval_ok("ln2").to_string();
    assert!(ln2_result.starts_with("0.69"));
}

#[test]
fn test_functions() {
    // Basic functions
    assert_eq!(eval_ok("sqrt(16)"), num!(4));
    assert_eq!(eval_ok("abs(-5)"), num!(5));
    assert_eq!(eval_ok("pow(2, 3)"), num!(8));
    assert_eq!(eval_ok("round(3.5)"), num!(4));
    assert_eq!(eval_ok("floor(3.9)"), num!(3));
    assert_eq!(eval_ok("ceil(3.1)"), num!(4));

    // Variadic functions
    assert_eq!(eval_ok("max(1, 10, 3, -5)"), num!(10));
    assert_eq!(eval_ok("min(1, 10, 3, -5)"), num!(-5));
    assert_eq!(eval_ok("sum(1, 2, 3, 4)"), num!(10));
    assert_eq!(eval_ok("avg(1, 2, 3, 4)"), num!(2.5));

    // Trigonometric
    assert!(approx_eq(eval_ok("sin(pi)"), num!(0), num!(0.0001)));
    assert_eq!(eval_ok("cos(0)"), num!(1));
    assert!(approx_eq(eval_ok("tan(0)"), num!(0), num!(0.0001)));

    // Other
    assert!(approx_eq(eval_ok("log10(100)"), num!(2), num!(0.0001)));
    assert_eq!(eval_ok("clamp(5, 1, 10)"), num!(5));
    assert_eq!(eval_ok("clamp(0, 1, 10)"), num!(1));
    assert_eq!(eval_ok("clamp(12, 1, 10)"), num!(10));
}

#[test]
fn test_decimal_native_functions() {
    // Logarithmic and exponential
    let log2_1024 = eval_ok("log2(1024)");
    assert!(approx_eq(log2_1024, num!(10), num!(0.00001)));
    let exp2_10 = eval_ok("exp2(10)");
    assert!(approx_eq(exp2_10, num!(1024), num!(0.001)));

    // Hyperbolic functions
    let sinh_1 = eval_ok("sinh(1)");
    assert!(approx_eq(sinh_1, num!(1.175201193), num!(0.0001)));
    let cosh_1 = eval_ok("cosh(1)");
    assert!(approx_eq(cosh_1, num!(1.543080634), num!(0.0001)));
    let tanh_1 = eval_ok("tanh(1)");
    assert!(approx_eq(tanh_1, num!(0.761594156), num!(0.0001)));
    assert!(eval_ok("tanh(10)") > num!(0.99));

    // Cube root
    assert_eq!(eval_ok("cbrt(27)"), num!(3));
    assert_eq!(eval_ok("cbrt(-8)"), num!(-2));
    let cbrt_10 = eval_ok("cbrt(10)");
    assert!(approx_eq(cbrt_10, num!(2.154434690), num!(0.0001)));

    // Hypot (Pythagorean theorem)
    assert_eq!(eval_ok("hypot(3, 4)"), num!(5));
    assert_eq!(eval_ok("hypot(5, 12)"), num!(13));
}

#[test]
fn test_complex_expressions() {
    assert!(approx_eq(
        eval_ok("sin(pi / 2) + cos(pi)"),
        num!(0),
        num!(0.0001)
    ));
    assert_eq!(eval_ok("max(sqrt(25), pow(2, 4), 10)"), num!(16));
    assert_eq!(eval_ok("sum(1, 2, 3, max(4, 5))"), num!(11));
    assert_eq!(eval_ok("floor(abs(-3.7)) + ceil(2.1)"), num!(6));
    assert_eq!(eval_ok("-1!"), num!(-1));
    assert_eq!(eval_ok("-3!^2"), num!(-36));
}

#[test]
fn test_custom_symbols() {
    let mut table = SymTable::stdlib();
    table.add_const("my_const", num!(123), false).unwrap();
    table
        .add_func("add_one", 1, false, |args| Ok(args[0] + num!(1)))
        .unwrap();

    assert_eq!(
        eval_with_custom_table_ok("my_const + 10", table.clone()),
        num!(133)
    );
    assert_eq!(
        eval_with_custom_table_ok("add_one(my_const)", table),
        num!(124)
    );
}

#[test]
fn test_emoji_identifiers() {
    let mut table = SymTable::stdlib();
    table.add_const("x😀", num!(10), false).unwrap();
    table
        .add_func("add🚀", 2, false, |args| Ok(args[0] + args[1]))
        .unwrap();

    assert_eq!(
        eval_with_custom_table_ok("x😀 + 5", table.clone()),
        num!(15)
    );
    assert_eq!(eval_with_custom_table_ok("add🚀(x😀, 2)", table), num!(12));
}

#[test]
#[rustfmt::skip]
fn test_syntax_errors() {
    assert_eq!(eval_err("1 + * 2"), indoc! {r#"
        Unexpected token: unexpected token '*', expected an expression
        1 | 1 + * 2
          |     ^"#
    });
    assert_eq!(eval_err("(1 + 2"), indoc! {r#"
        Unexpected token: unexpected token 'EOF', expected ')'
        1 | (1 + 2
          |       ^"#
    });
    assert_eq!(eval_err("1 2"), indoc! {r#"
        Unexpected token: unexpected token '2', expected 'EOF'
        1 | 1 2
          |   ^"#
    });
    assert_eq!(eval_err("()"), indoc! {r#"
        Unexpected token: unexpected token ')', expected an expression
        1 | ()
          |  ^"#
    });
    assert_eq!(eval_err("sin("), indoc! {r#"
        Unexpected token: unexpected token 'EOF', expected an expression
        1 | sin(
          |     ^"#
    });
    assert_eq!(eval_err("1 + "), indoc! {r#"
        Unexpected token: unexpected token 'EOF', expected an expression
        1 | 1 +
          |    ^"#
    });
}

#[test]
#[rustfmt::skip]
fn test_semantic_errors() {
    // V2 defers validation to link time
    assert_eq!(eval_err("foo()"), "Symbol 'foo' not found");
    assert_eq!(eval_err("bar"), "Symbol 'bar' not found");
    assert_eq!(eval_err("sin(1, 2)"), "Link error: Type mismatch for symbol 'sin': expected exactly 1 arguments, found 2 arguments provided");
    assert_eq!(eval_err("max()"), "Link error: Type mismatch for symbol 'max': expected at least 1 arguments, found 0 arguments provided");
    assert_eq!(eval_err("pi()"), "Link error: Type mismatch for symbol 'pi': expected function, found constant");
    assert_eq!(eval_err("1 + sin"), "Link error: Type mismatch for symbol 'sin': expected constant, found function");
}

#[test]
fn test_runtime_errors() {
    assert_eq!(eval_err("1 / 0"), "Division by zero");
    assert_eq!(
        eval_err("1.5!"),
        "Invalid factorial: 1.5 (must be a non-negative integer)"
    );

    // In decimal mode, domain errors are caught
    // In decimal mode, these operations return NaN (which is acceptable)
    #[cfg(feature = "decimal")]
    {
        assert_eq!(
            eval_err("log(-1)"),
            "Function error: Domain error in function 'log': invalid input -1"
        );
        assert_eq!(
            eval_err("sqrt(-4)"),
            "Function error: Square root of negative number: -4"
        );
    }

    #[cfg(not(feature = "decimal"))]
    {
        // These return NaN in f64 mode
        let result = eval_ok("log(-1)");
        assert!(result.is_nan());
        let result = eval_ok("sqrt(-4)");
        assert!(result.is_nan());
    }
}

#[test]
fn test_if_expressions() {
    // Basic true/false
    assert_eq!(eval_ok("if(1, 10, 20)"), num!(10));
    assert_eq!(eval_ok("if(0, 10, 20)"), num!(20));
    assert_eq!(eval_ok("if(0.5, 10, 20)"), num!(10)); // Non-zero decimal

    // With comparisons
    assert_eq!(eval_ok("if(5 > 3, 100, 200)"), num!(100));
    assert_eq!(eval_ok("if(5 == 5, 1, 0)"), num!(1));
    assert_eq!(eval_ok("if(5 != 3, 1, 0)"), num!(1));

    // With arithmetic
    assert_eq!(eval_ok("if(5 - 5, 1, 0)"), num!(0));
    assert_eq!(eval_ok("if(1, 2 + 3, 4 * 5)"), num!(5));
    assert_eq!(eval_ok("if(0, 2 + 3, 4 * 5) + 10"), num!(30));

    // With functions
    assert_eq!(eval_ok("if(abs(-5), 1, 0)"), num!(1));
    assert_eq!(eval_ok("if(1, abs(-10), abs(-20))"), num!(10));
    assert_eq!(eval_ok("if(max(1, 2) > 0, 42, 0)"), num!(42));

    // Case insensitive
    assert_eq!(eval_ok("IF(1, 10, 20)"), num!(10));
}

#[test]
fn test_if_nested() {
    // Nested in branches
    assert_eq!(eval_ok("if(1, if(1, 10, 20), 30)"), num!(10));
    assert_eq!(eval_ok("if(0, 10, if(1, 20, 30))"), num!(20));

    // Nested in condition
    assert_eq!(eval_ok("if(if(1, 1, 0), 100, 200)"), num!(100));

    // Multiple levels
    assert_eq!(eval_ok("if(1, if(1, if(1, 1, 2), 3), 4)"), num!(1));
    assert_eq!(eval_ok("if(1, if(1, if(0, 1, 2), 3), 4)"), num!(2));
    assert_eq!(eval_ok("if(0, if(1, if(1, 1, 2), 3), 4)"), num!(4));
}

#[test]
fn test_if_short_circuit() {
    // Critical: only the taken branch executes
    assert_eq!(eval_ok("if(1, 42, 1/0)"), num!(42));
    assert_eq!(eval_ok("if(0, 1/0, 42)"), num!(42));
}

#[test]
fn test_if_error_cases() {
    let err = eval_err("if(1, 2)");
    assert!(err.contains("expected ')'") || err.contains("expected ','"));

    let err = eval_err("if 1, 2, 3");
    assert!(err.contains("expected '('"));
}

// ====================
// Program API Tests
// ====================

fn load_with_table(
    expr: &'static str,
    table: SymTable,
) -> Result<Program<'static, Linked>, String> {
    let program = Program::new_from_source(expr).map_err(|err| err.to_string())?;
    program.link(table).map_err(|err| err.to_string())
}

fn load(expr: &'static str) -> Result<Program<'static, Compiled>, String> {
    Program::new_from_source(expr).map_err(|err| err.to_string())
}

#[test]
fn test_program_compile_link_execute() {
    let mut program = load_with_table("2 + 3 * 4", SymTable::stdlib()).expect("link failed");
    assert_eq!(program.execute().expect("execution failed"), num!(14));

    let mut program =
        load_with_table("sqrt(16) + sin(0)", SymTable::stdlib()).expect("link failed");
    assert_eq!(program.execute().expect("execution failed"), num!(4));
}

#[test]
fn test_program_symtable_mutation() {
    let program = load("x + y").expect("compilation failed");

    let mut table = SymTable::new();
    table.add_const("x", num!(10), false).unwrap();
    table.add_const("y", num!(20), false).unwrap();

    let mut program = program.link(table).expect("link failed");
    assert_eq!(program.execute().expect("execution failed"), num!(30));

    // Modify symbol table after linking
    program
        .symtable_mut()
        .add_const("z", num!(100), false)
        .unwrap();

    // x + y should still be 30
    assert_eq!(program.execute().expect("execution failed"), num!(30));
}

#[test]
#[cfg(feature = "serialization")]
fn test_program_serialization() {
    let mut program = load_with_table("sqrt(pi) + 2", SymTable::stdlib()).expect("link failed");

    // Execute original
    let result1 = program.execute().expect("execution failed");

    // Serialize
    let bytes = program.to_bytecode().expect("serialization failed");

    // Deserialize and re-link
    use expr_solver::Program;
    let mut program2 = Program::new_from_bytecode(&bytes)
        .expect("deserialization failed")
        .link(SymTable::stdlib())
        .expect("link failed");

    // Execute deserialized
    let result2 = program2.execute().expect("execution failed");

    assert_eq!(result1, result2);
}

#[test]
fn test_program_assembly() {
    let program = load_with_table("2 + 3", SymTable::stdlib()).expect("link failed");
    let assembly = program.get_assembly();
    assert!(assembly.contains("PUSH"));
    assert!(assembly.contains("ADD"));
}

#[test]
fn test_program_link_validation() {
    let program = load("x + y").expect("compilation failed");

    // Try to link with empty symbol table (should fail)
    let empty_table = SymTable::new();
    let result = program.link(empty_table);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

// ============================================================================
// Let Statement Tests
// ============================================================================

#[test]
fn test_let_simple() {
    // Single declaration
    assert_eq!(eval_ok("let x = 10 then x"), num!(10));
    assert_eq!(eval_ok("let x = 5 * 2 then x + 1"), num!(11));
}

#[test]
fn test_let_multiple_declarations() {
    // Multiple declarations
    assert_eq!(eval_ok("let x = 2, y = 3 then x + y"), num!(5));
    assert_eq!(eval_ok("let x = 1, y = 2, z = 3 then x + y + z"), num!(6));
}

#[test]
fn test_let_func_declaration() {
    assert_eq!(eval_ok("let fn(x) = x * x then fn(2)"), num!(4));
    assert_eq!(eval_ok("let add(a, b) = a + b then add(2, 3)"), num!(5));
    assert_eq!(eval_ok("let y = 1, fn(x) = x + y then fn(2)"), num!(3));
    assert_eq!(eval_ok("let foo(x) = x * x, bar(x) = foo(x+1) then bar(2)"), num!(9));
    assert_eq!(eval_ok("let fact(n) = if(n > 0, n * fact(n - 1), 1) then fact(5)"), num!(120));
}

#[test]
fn test_let_reference_previous() {
    // Reference previously declared let variables
    assert_eq!(eval_ok("let x = 1, y = x + 1 then y"), num!(2));
    assert_eq!(eval_ok("let x = 2, y = x * 3, z = y + 1 then z"), num!(7));
}

#[test]
fn test_let_with_globals() {
    // Use global constants and functions
    assert_eq!(eval_ok("let x = pi then x"), eval_ok("pi"));
    assert_eq!(eval_ok("let x = sin(pi / 2) then x"), num!(1));
    assert_eq!(
        eval_ok("let r = 5, area = pi * r ^ 2 then area"),
        eval_ok("pi * 25")
    );
}

#[test]
fn test_let_complex_expressions() {
    // Complex expressions in declarations
    assert_eq!(eval_ok("let x = if(1 < 2, 10, 20) then x"), num!(10));
    assert_eq!(eval_ok("let x = 2, y = x ^ 3 then y * 2"), num!(16));
    assert_eq!(eval_ok("let x = sqrt(16), y = x + 4 then y"), num!(8));
}

#[test]
fn test_let_error_shadowing_global() {
    // Should error when trying to shadow global constants
    let err = eval_err("let pi = 3 then pi");
    assert_eq!(err, "Duplicate symbol definition: 'pi'");

    let err = eval_err("let e = 2 then e");
    assert_eq!(err, "Duplicate symbol definition: 'e'");
}

#[test]
fn test_let_error_duplicate_names() {
    // Should error when same name declared twice in one let
    let err = eval_err("let x = 1, x = 2 then x");
    assert_eq!(err, "Duplicate symbol definition: 'x'");

    let err = eval_err("let x = 1, y = 2, x = 3 then x + y");
    assert_eq!(err, "Duplicate symbol definition: 'x'");
}

#[test]
fn test_let_error_forward_reference() {
    // Should error when referencing a variable before it's declared
    let err = eval_err("let x = y, y = 1 then x");
    assert_eq!(err, "Duplicate symbol definition: 'y'");
}

#[test]
fn test_let_error_self_reference() {
    // Should error when variable references itself in its own definition
    let err = eval_err("let x = x + 1 then x");
    assert_eq!(err, "Symbol 'x' not found");
}

#[test]
fn test_let_with_custom_table() {
    // Using let with a custom symbol table
    let mut table = SymTable::stdlib();
    table.add_const("custom", num!(42), false).unwrap();

    let result = eval_with_custom_table_ok("let x = custom * 2 then x", table);
    assert_eq!(result, num!(84));
}

#[test]
fn test_let_case_insensitive_keywords() {
    // Keywords should be case-insensitive
    assert_eq!(eval_ok("LET x = 10 THEN x"), num!(10));
    assert_eq!(eval_ok("Let x = 5 Then x * 2"), num!(10));
    assert_eq!(eval_ok("leT x = 3, y = 7 tHeN x + y"), num!(10));
}

// ============================================================================
// Serialization Tests with LET and IF
// ============================================================================

#[test]
#[cfg(feature = "serialization")]
fn test_let_if_serialization_roundtrip() {
    use expr_solver::Program;

    // Complex expression with LET and IF in both declaration and body
    // let a = if(5 > 3, 10, 20),
    //     b = if(a > 15, a * 2, a + 5)
    // then if(b < 20, b * 3, b - 10)
    let source =
        "let a = if(5 > 3, 10, 20), b = if(a > 15, a * 2, a + 5) then if(b < 20, b * 3, b - 10)";

    // Compile and execute original
    let program = Program::new_from_source(source).expect("Failed to compile");
    let table = SymTable::stdlib();
    let mut linked = program.link(table.clone()).expect("Failed to link");
    let result1 = linked.execute().expect("Failed to execute");

    // Serialize to bytecode
    let bytecode = linked.to_bytecode().expect("Failed to serialize");

    // Deserialize from bytecode
    let loaded_program = Program::new_from_bytecode(&bytecode).expect("Failed to deserialize");
    let mut relinked = loaded_program.link(table).expect("Failed to relink");
    let result2 = relinked.execute().expect("Failed to execute reloaded");

    // Results should be identical
    assert_eq!(
        result1, result2,
        "Serialization roundtrip produced different result"
    );

    // Verify the actual computation is correct:
    // a = if(5 > 3, 10, 20) = 10
    // b = if(10 > 15, 20, 15) = 15
    // result = if(15 < 20, 45, 5) = 45
    assert_eq!(result1, num!(45));
}

// ============================================================================
// Print tests - syntax highlighting and assembly printing
// ============================================================================

#[test]
fn test_print_expression_with_source() {
    use expr_solver::{Print, Program};

    // Test that Print can display source with highlighting
    let source = "let x = 10 then x * 2";
    let program = Program::new_from_source(source).expect("Failed to compile");
    let printer = Print::new(&program);

    // Just verify it doesn't panic - actual colors won't be visible in test output
    let output = printer.to_string();
    assert!(output.len() > 0, "Printed output should not be empty");
    // The output should contain the keywords and expressions
    assert!(output.contains("let"));
    assert!(output.contains("then"));
}

#[test]
fn test_print_assembly_for_linked() {
    use expr_solver::{Print, Program, SymTable};

    let source = "2 + 3 * 4";
    let program = Program::new_from_source(source).expect("Failed to compile");
    let linked = program.link(SymTable::stdlib()).expect("Failed to link");
    let printer = Print::new(&linked);

    // Print assembly
    let assembly = printer.assembly();
    assert!(assembly.len() > 0, "Assembly should not be empty");

    // Check for expected instructions
    assert!(assembly.contains("PUSH"));
    assert!(assembly.contains("MUL"));
    assert!(assembly.contains("ADD"));
}

#[test]
fn test_print_get_assembly_convenience() {
    use expr_solver::{Program, SymTable};

    let source = "sin(pi)";
    let program = Program::new_from_source(source).expect("Failed to compile");
    let linked = program.link(SymTable::stdlib()).expect("Failed to link");

    // Test the convenience method
    let assembly = linked.get_assembly();
    assert!(assembly.len() > 0, "Assembly should not be empty");
    assert!(assembly.contains("LOAD"));
    assert!(assembly.contains("CALL"));
}
