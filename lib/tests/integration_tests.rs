use expr_solver::{Compiled, Linked, Number, Program, SymTable, eval, eval_with_table};
use indoc::indoc;

#[cfg(feature = "decimal-precision")]
use rust_decimal_macros::dec;

// Helper macro to create numbers based on feature
#[cfg(feature = "decimal-precision")]
macro_rules! num {
    ($val:expr) => {
        dec!($val)
    };
}

#[cfg(feature = "f64-floats")]
macro_rules! num {
    ($val:expr) => {
        $val as f64
    };
}

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
#[cfg(feature = "f64-floats")]
fn approx_eq(a: Number, b: Number, epsilon: f64) -> bool {
    (a - b).abs() < epsilon
}

#[cfg(feature = "decimal-precision")]
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
    use expr_solver::number::consts;

    assert_eq!(eval_ok("pi"), consts::PI);
    assert_eq!(eval_ok("e"), consts::E);
    assert_eq!(eval_ok("tau"), consts::TAU);
    assert_eq!(eval_ok("ln2"), consts::LN_2);
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
    assert!(approx_eq(eval_ok("sin(pi / 2) + cos(pi)"), num!(0), num!(0.0001)));
    assert_eq!(eval_ok("max(sqrt(25), pow(2, 4), 10)"), num!(16));
    assert_eq!(eval_ok("sum(1, 2, 3, max(4, 5))"), num!(11));
    assert_eq!(eval_ok("floor(abs(-3.7)) + ceil(2.1)"), num!(6));
    assert_eq!(eval_ok("-1!"), num!(-1));
    assert_eq!(eval_ok("-3!^2"), num!(-36));
}

#[test]
fn test_custom_symbols() {
    let mut table = SymTable::stdlib();
    table.add_const("my_const", num!(123)).unwrap();
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
    table.add_const("x😀", num!(10)).unwrap();
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
    assert_eq!(eval_err("foo()"), "Link error: Missing symbol: 'foo' is required by bytecode but not in symbol table");
    assert_eq!(eval_err("bar"), "Link error: Missing symbol: 'bar' is required by bytecode but not in symbol table");
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

    // In decimal-precision mode, domain errors are caught
    // In f64-floats mode, these operations return NaN (which is acceptable)
    #[cfg(feature = "decimal-precision")]
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

    #[cfg(feature = "f64-floats")]
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
    let program = load_with_table("2 + 3 * 4", SymTable::stdlib()).expect("link failed");
    assert_eq!(program.execute().expect("execution failed"), num!(14));

    let program = load_with_table("sqrt(16) + sin(0)", SymTable::stdlib()).expect("link failed");
    assert_eq!(program.execute().expect("execution failed"), num!(4));
}

#[test]
fn test_program_symtable_mutation() {
    let program = load("x + y").expect("compilation failed");

    let mut table = SymTable::new();
    table.add_const("x", num!(10)).unwrap();
    table.add_const("y", num!(20)).unwrap();

    let mut program = program.link(table).expect("link failed");
    assert_eq!(program.execute().expect("execution failed"), num!(30));

    // Modify symbol table after linking
    program.symtable_mut().add_const("z", num!(100)).unwrap();

    // x + y should still be 30
    assert_eq!(program.execute().expect("execution failed"), num!(30));
}

#[test]
#[cfg(feature = "serialization")]
fn test_program_serialization() {
    let program = load_with_table("sqrt(pi) + 2", SymTable::stdlib()).expect("link failed");

    // Execute original
    let result1 = program.execute().expect("execution failed");

    // Serialize
    let bytes = program.to_bytecode().expect("serialization failed");

    // Deserialize and re-link
    use expr_solver::Program;
    let program2 = Program::new_from_bytecode(&bytes)
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
    assert!(result.unwrap_err().to_string().contains("Missing symbol"));
}
