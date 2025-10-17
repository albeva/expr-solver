use expr_solver::{Eval, SymTable};
use indoc::indoc;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal_macros::dec;

// Helper function to evaluate an expression and expect an Ok result.
fn eval_ok(expr: &str) -> Decimal {
    let mut eval = Eval::new(expr);
    eval.run().expect("Evaluation should be successful")
}

// Helper function to evaluate an expression and expect an Err result.
fn eval_err(expr: &str) -> String {
    colored::control::set_override(false);
    let mut eval = Eval::new(expr);
    eval.run().expect_err("Evaluation should fail")
}

// Helper function to evaluate an expression with a custom symbol table and expect an Ok result.
fn eval_with_custom_table_ok(expr: &str, table: SymTable) -> Decimal {
    let mut eval = Eval::with_table(expr, table);
    eval.run().expect("Evaluation should be successful")
}

#[test]
fn test_valid_arithmetic() {
    assert_eq!(eval_ok("1 + 2"), dec!(3));
    assert_eq!(eval_ok("10 - 5"), dec!(5));
    assert_eq!(eval_ok("2 * 3"), dec!(6));
    assert_eq!(eval_ok("10 / 2"), dec!(5));
    assert_eq!(eval_ok("2 ^ 3"), dec!(8));
    assert_eq!(eval_ok("-5"), dec!(-5));
    assert_eq!(eval_ok("1 + -5"), dec!(-4));
    assert_eq!(eval_ok("5!"), dec!(120));
    assert_eq!(eval_ok("0!"), dec!(1));
    assert_eq!(eval_ok("10 / 3").round_dp(20), dec!(3.33333333333333333333));
}

#[test]
fn test_operator_precedence() {
    assert_eq!(eval_ok("1 + 2 * 3"), dec!(7));
    assert_eq!(eval_ok("(1 + 2) * 3"), dec!(9));
    assert_eq!(eval_ok("10 / 2 * 5"), dec!(25));
    assert_eq!(eval_ok("2 ^ 3 ^ 2"), dec!(512)); // Right-associative
    assert_eq!(eval_ok("(2 ^ 3) ^ 2"), dec!(64));
    assert_eq!(eval_ok("-2 ^ 2"), dec!(-4));
    assert_eq!(eval_ok("3 + 4 * 2 / (1 - 5) ^ 2"), dec!(3.5));
}

#[test]
fn test_comparisons() {
    assert_eq!(eval_ok("1 > 0"), dec!(1));
    assert_eq!(eval_ok("1 < 0"), dec!(0));
    assert_eq!(eval_ok("1 == 1"), dec!(1));
    assert_eq!(eval_ok("1 != 1"), dec!(0));
    assert_eq!(eval_ok("1 >= 1"), dec!(1));
    assert_eq!(eval_ok("1 <= 1"), dec!(1));
    assert_eq!(eval_ok("5 > 10"), dec!(0));
    assert_eq!(eval_ok("1 + 1 == 2"), dec!(1));
    assert_eq!(eval_ok("1 + 1 != 3"), dec!(1));
}

#[test]
fn test_constants() {
    assert_eq!(eval_ok("pi"), Decimal::PI);
    assert_eq!(eval_ok("e"), Decimal::E);
    assert_eq!(eval_ok("tau"), Decimal::TWO_PI);
    assert_eq!(
        eval_ok("ln2"),
        Decimal::from_f64(std::f64::consts::LN_2).unwrap()
    );
}

#[test]
fn test_functions() {
    assert_eq!(eval_ok("sqrt(16)"), dec!(4));
    assert_eq!(eval_ok("abs(-5)"), dec!(5));
    assert_eq!(eval_ok("max(1, 10, 3, -5)"), dec!(10));
    assert_eq!(eval_ok("min(1, 10, 3, -5)"), dec!(-5));
    assert_eq!(eval_ok("sum(1, 2, 3, 4)"), dec!(10));
    assert_eq!(eval_ok("avg(1, 2, 3, 4)"), dec!(2.5));
    assert_eq!(eval_ok("pow(2, 3)"), dec!(8));
    assert_eq!(eval_ok("round(3.5)"), dec!(4));
    assert_eq!(eval_ok("floor(3.9)"), dec!(3));
    assert_eq!(eval_ok("ceil(3.1)"), dec!(4));
    assert_eq!(eval_ok("max(4, 5)"), dec!(5));
    assert_eq!(eval_ok("sum(1, 2, 3)"), dec!(6));
    assert_eq!(eval_ok("sin(pi)"), dec!(0)); // sin(pi) is very close to 0
    assert_eq!(eval_ok("cos(0)"), dec!(1));
    assert_eq!(eval_ok("tan(0)"), dec!(0));
    assert_eq!(eval_ok("log10(100)"), dec!(2));
    assert_eq!(eval_ok("clamp(5, 1, 10)"), dec!(5));
    assert_eq!(eval_ok("clamp(0, 1, 10)"), dec!(1));
    assert_eq!(eval_ok("clamp(12, 1, 10)"), dec!(10));
}

#[test]
fn test_complex_expressions() {
    assert_eq!(eval_ok("sin(pi / 2) + cos(pi)"), dec!(0));
    assert_eq!(eval_ok("max(sqrt(25), pow(2, 4), 10)"), dec!(16));
    assert_eq!(eval_ok("sum(1, 2, 3, max(4, 5))"), dec!(11));
    assert_eq!(eval_ok("avg(10, 20, 30) * 2"), dec!(40));
    assert_eq!(eval_ok("floor(abs(-3.7)) + ceil(2.1)"), dec!(6));
    assert_eq!(eval_ok("pow(-1, 0.5)"), dec!(-1));
    assert_eq!(eval_ok("-1!"), dec!(-1));
    assert_eq!(eval_ok("-3!^2"), dec!(-36));
}

#[test]
fn test_custom_symbols() {
    let mut table = SymTable::stdlib();
    table.add_const("my_const", dec!(123)).unwrap();
    table
        .add_func("add_one", 1, false, |args| Ok(args[0] + dec!(1)))
        .unwrap();

    assert_eq!(
        eval_with_custom_table_ok("my_const + 10", table.clone()),
        dec!(133)
    );
    assert_eq!(
        eval_with_custom_table_ok("add_one(5)", table.clone()),
        dec!(6)
    );
    assert_eq!(
        eval_with_custom_table_ok("add_one(my_const)", table),
        dec!(124)
    );
}

#[test]
#[rustfmt::skip]
fn test_syntax_errors() {
    assert_eq!(eval_err("1 + * 2"), indoc! {r#"
        Unexpected token '*', expected 'an expression'
        1 | 1 + * 2
          |     ^"#
    });
    assert_eq!(eval_err("(1 + 2"), indoc! {r#"
        Unexpected token 'EOF', expected ')'
        1 | (1 + 2
          |       ^"#
    });
    assert_eq!(eval_err("1 2"), indoc! {r#"
        Unexpected token '2', expected 'EOF'
        1 | 1 2
          |   ^"#
    });
    assert_eq!(eval_err("()"), indoc! {r#"
        Unexpected token ')', expected 'an expression'
        1 | ()
          |  ^"#
    });
    assert_eq!(eval_err("sin("), indoc! {r#"
        Unexpected token 'EOF', expected 'an expression'
        1 | sin(
          |     ^"#
    });
    assert_eq!(eval_err("1 + "), indoc! {r#"
        Unexpected token 'EOF', expected 'an expression'
        1 | 1 +
          |    ^"#
    });
    assert_eq!(eval_err("* 2"), indoc! {r#"
        Unexpected token '*', expected 'an expression'
        1 | * 2
          | ^"#
    });
    assert_eq!(eval_err("1 (2 + 3)"), indoc! {r#"
        Unexpected token '(', expected 'EOF'
        1 | 1 (2 + 3)
          |   ^"#
    });
    assert_eq!(eval_err("sin 1"), indoc! {r#"
        Unexpected token '1', expected 'EOF'
        1 | sin 1
          |     ^"#
    });
}

#[test]
#[rustfmt::skip]
fn test_semantic_errors() {
    assert_eq!(eval_err("foo()"), indoc! {r#"
        Undefined symbol 'foo'
        1 | foo()
          | ^~~"#
    });
    assert_eq!(eval_err("🙈🍅🎉🌴🎶()"), indoc! {r#"
        Undefined symbol '🙈🍅🎉🌴🎶'
        1 | 🙈🍅🎉🌴🎶()
          | ^~~~~~~~~~"#
    });
    assert_eq!(eval_err("bar"), indoc! {r#"
        Undefined symbol 'bar'
        1 | bar
          | ^~~"#
    });
    assert_eq!(eval_err("sin(1, 2)"), indoc! {r#"
        Function 'sin' expects exactly 1 arguments but got 2
        1 | sin(1, 2)
          | ^~~~~~~~~"#
    });
    assert_eq!(eval_err("max()"), indoc! {r#"
        Function 'max' expects at least 1 arguments but got 0
        1 | max()
          | ^~~~~"#
    });
    assert_eq!(eval_err("pi()"), indoc! {r#"
        Symbol 'pi' is not a function
        1 | pi()
          | ^~"#
    });
    assert_eq!(eval_err("1 + sin"), indoc! {r#"
        Symbol 'sin' is not a constant
        1 | 1 + sin
          |     ^~~"#
    });
    assert_eq!(eval_err("avg()"), indoc! {r#"
        Function 'avg' expects at least 1 arguments but got 0
        1 | avg()
          | ^~~~~"#
    });
    assert_eq!(eval_err("clamp(1, 2)"), indoc! {r#"
        Function 'clamp' expects exactly 3 arguments but got 2
        1 | clamp(1, 2)
          | ^~~~~~~~~~~"#
    });
}

#[test]
fn test_runtime_errors() {
    assert_eq!(eval_err("1 / 0"), "Division by zero");
    assert_eq!(
        eval_err("1.5!"),
        "Invalid factorial: 1.5 (must be a non-negative integer)"
    );
    assert_eq!(
        eval_err("log(-1)"),
        "Function error: Domain error in function 'log': invalid input -1"
    );
    assert_eq!(
        eval_err("sqrt(-4)"),
        "Function error: Square root of negative number: -4"
    );
    assert_eq!(eval_err("1 / (2 - 2)"), "Division by zero");
}

#[test]
fn test_emoji_identifiers() {
    let mut table = SymTable::stdlib();
    table.add_const("x😀", dec!(10)).unwrap();
    table
        .add_func("add🚀", 2, false, |args| Ok(args[0] + args[1]))
        .unwrap();

    assert_eq!(
        eval_with_custom_table_ok("x😀 + 5", table.clone()),
        dec!(15)
    );
    assert_eq!(eval_with_custom_table_ok("add🚀(x😀, 2)", table), dec!(12));
}

#[test]
fn test_if_function() {
    // True condition (non-zero)
    assert_eq!(eval_ok("if(1, 10, 20)"), dec!(10));
    // False condition (zero)
    assert_eq!(eval_ok("if(0, 10, 20)"), dec!(20));
    // True condition (positive decimal)
    assert_eq!(eval_ok("if(0.5, 10, 20)"), dec!(10));
    // True condition (negative decimal)
    assert_eq!(eval_ok("if(-1, 10, 20)"), dec!(10));
    // Nested if
    assert_eq!(eval_ok("if(1, if(0, 100, 200), 300)"), dec!(200));
    // If with expressions as arguments
    assert_eq!(eval_ok("if(1 > 0, 5 * 2, 10 / 2)"), dec!(10));
    // If with comparison as condition
    assert_eq!(eval_ok("if(5 == 5, 1, 0)"), dec!(1));
}

#[test]
#[rustfmt::skip]
fn test_if_function_semantic_errors() {
    assert_eq!(eval_err("if(1, 2)"), indoc! {r#"
        Function 'if' expects exactly 3 arguments but got 2
        1 | if(1, 2)
          | ^~~~~~~~"#
    });
    assert_eq!(eval_err("if(1, 2, 3, 4)"), indoc! {r#"
        Function 'if' expects exactly 3 arguments but got 4
        1 | if(1, 2, 3, 4)
          | ^~~~~~~~~~~~~~"#
    });
}
