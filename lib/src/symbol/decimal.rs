//! Decimal-specific symbol table implementation.
//!
//! This module provides high-precision 128-bit Decimal arithmetic with checked operations.

use super::{FuncError, Symbol};
use crate::number::Number;
use crate::symtable::SymTable;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use std::panic;

// Mathematical constants for symbol table
const LN_2: Number = dec!(0.6931471805599453094172321);
const LN_10: Number = dec!(2.3025850929940456840179915);
const SQRT_2: Number = dec!(1.4142135623730950488016887);

/// Helper function for single-argument f64 calculations
/// Used for inverse trig functions that don't have native Decimal implementations
fn f64_calc_1<F>(args: &[Number], func: F) -> Result<Number, FuncError>
where
    F: Fn(f64) -> f64,
{
    let arg = args[0].to_f64().ok_or(FuncError::ToF64Conversion)?;
    let result = func(arg);
    Decimal::from_f64(result).ok_or(FuncError::FromF64Conversion)
}

/// Helper function for two-argument f64 calculations
fn f64_calc_2<F>(args: &[Number], func: F) -> Result<Number, FuncError>
where
    F: Fn(f64, f64) -> f64,
{
    let arg1 = args[0].to_f64().ok_or(FuncError::ToF64Conversion)?;
    let arg2 = args[1].to_f64().ok_or(FuncError::ToF64Conversion)?;
    let result = func(arg1, arg2);
    Decimal::from_f64(result).ok_or(FuncError::FromF64Conversion)
}

/// Cube root using Newton-Raphson iteration
fn cbrt_decimal(x: Decimal) -> Decimal {
    if x == Decimal::ZERO {
        return Decimal::ZERO;
    }

    let sign = if x < Decimal::ZERO {
        Decimal::NEGATIVE_ONE
    } else {
        Decimal::ONE
    };
    let abs_x = x.abs();

    // Initial guess using x^(1/3) ≈ x / 3 for small values, or a fraction of x for large
    let mut guess = if abs_x < Decimal::ONE {
        abs_x
    } else {
        abs_x / Decimal::from(3)
    };

    let three = Decimal::from(3);
    let precision = Decimal::new(1, 28); // 10^-28

    // Newton-Raphson: x_n+1 = (2*x_n + a/x_n²) / 3
    for _ in 0..50 {
        let guess_squared = guess * guess;
        let next = (guess + guess + abs_x / guess_squared) / three;

        if (next - guess).abs() < precision {
            return next * sign;
        }
        guess = next;
    }

    guess * sign
}

#[cfg(feature = "decimal-precision")]
impl SymTable {
    /// Creates a symbol table with the standard library for Decimal precision.
    ///
    /// Most functions use native 128-bit `Decimal` arithmetic for high precision.
    /// Some operations use f64 conversion internally due to `rust_decimal` limitations:
    /// - Inverse trig functions: `asin`, `acos`, `atan`, `atan2`
    /// - Power function: `pow` (for non-integer exponents)
    ///
    /// ## Fixed arity functions
    /// - `sin(x)` - Sine
    /// - `cos(x)` - Cosine
    /// - `tan(x)` - Tangent
    /// - `asin(x)` - Arcsine
    /// - `acos(x)` - Arccosine
    /// - `atan(x)` - Arctangent
    /// - `atan2(y, x)` - Two-argument arctangent
    /// - `sinh(x)` - Hyperbolic sine
    /// - `cosh(x)` - Hyperbolic cosine
    /// - `tanh(x)` - Hyperbolic tangent
    /// - `sqrt(x)` - Square root
    /// - `cbrt(x)` - Cube root
    /// - `pow(x, y)` - x raised to power y
    /// - `log(x)` - Natural logarithm
    /// - `log2(x)` - Base-2 logarithm
    /// - `log10(x)` - Base-10 logarithm
    /// - `exp(x)` - e raised to power x
    /// - `exp2(x)` - 2 raised to power x
    /// - `abs(x)` - Absolute value
    /// - `sign(x)` - Sign function (-1, 0, or 1)
    /// - `floor(x)` - Floor function
    /// - `ceil(x)` - Ceiling function
    /// - `round(x)` - Round to nearest integer
    /// - `trunc(x)` - Truncate to integer
    /// - `fract(x)` - Fractional part
    /// - `mod(x, y)` - Remainder of x/y
    /// - `hypot(x, y)` - Euclidean distance sqrt(x²+y²)
    /// - `clamp(x, min, max)` - Constrain value between bounds
    ///
    /// ## Variadic functions
    /// - `min(x, ...)` - Minimum value
    /// - `max(x, ...)` - Maximum value
    /// - `sum(x, ...)` - Sum of values
    /// - `avg(x, ...)` - Average of values
    pub fn stdlib() -> Self {
        Self::from_symbols(vec![
            // Constants
            Symbol::Const {
                name: "pi".into(),
                value: Decimal::PI,
                description: Some("π (3.14159...)".into()),
            },
            Symbol::Const {
                name: "e".into(),
                value: Decimal::E,
                description: Some("Euler's number (2.71828...)".into()),
            },
            Symbol::Const {
                name: "tau".into(),
                value: Decimal::TWO_PI,
                description: Some("2π (6.28318...)".into()),
            },
            Symbol::Const {
                name: "ln2".into(),
                value: LN_2,
                description: Some("Natural logarithm of 2".into()),
            },
            Symbol::Const {
                name: "ln10".into(),
                value: LN_10,
                description: Some("Natural logarithm of 10".into()),
            },
            Symbol::Const {
                name: "sqrt2".into(),
                value: SQRT_2,
                description: Some("Square root of 2".into()),
            },
            // Trigonometric functions
            Symbol::Func {
                name: "sin".into(),
                args: 1,
                variadic: false,
                callback: |args| Ok(args[0].sin()),
                description: Some("Sine".into()),
            },
            Symbol::Func {
                name: "cos".into(),
                args: 1,
                variadic: false,
                callback: |args| Ok(args[0].cos()),
                description: Some("Cosine".into()),
            },
            Symbol::Func {
                name: "tan".into(),
                args: 1,
                variadic: false,
                callback: |args| {
                    let input = args[0];
                    match panic::catch_unwind(panic::AssertUnwindSafe(|| input.tan())) {
                        Ok(result) => Ok(result),
                        Err(_) => Err(FuncError::DomainError {
                            function: "tan".to_string(),
                            input,
                        }),
                    }
                },
                description: Some("Tangent".into()),
            },
            Symbol::Func {
                name: "asin".into(),
                args: 1,
                variadic: false,
                callback: |args| f64_calc_1(args, |x| x.asin()),
                description: Some("Arcsine".into()),
            },
            Symbol::Func {
                name: "acos".into(),
                args: 1,
                variadic: false,
                callback: |args| f64_calc_1(args, |x| x.acos()),
                description: Some("Arccosine".into()),
            },
            Symbol::Func {
                name: "atan".into(),
                args: 1,
                variadic: false,
                callback: |args| f64_calc_1(args, |x| x.atan()),
                description: Some("Arctangent".into()),
            },
            Symbol::Func {
                name: "atan2".into(),
                args: 2,
                variadic: false,
                callback: |args| f64_calc_2(args, |y, x| y.atan2(x)),
                description: Some("Two-argument arctangent".into()),
            },
            Symbol::Func {
                name: "sinh".into(),
                args: 1,
                variadic: false,
                callback: |args| {
                    let x = args[0];
                    // sinh(x) = (e^x - e^-x) / 2
                    match (
                        panic::catch_unwind(panic::AssertUnwindSafe(|| x.exp())),
                        panic::catch_unwind(panic::AssertUnwindSafe(|| (-x).exp())),
                    ) {
                        (Ok(exp_x), Ok(exp_neg_x)) => Ok((exp_x - exp_neg_x) / Decimal::TWO),
                        _ => Err(FuncError::MathError {
                            message: "Exponential overflow or underflow in sinh".to_string(),
                        }),
                    }
                },
                description: Some("Hyperbolic sine".into()),
            },
            Symbol::Func {
                name: "cosh".into(),
                args: 1,
                variadic: false,
                callback: |args| {
                    let x = args[0];
                    // cosh(x) = (e^x + e^-x) / 2
                    match (
                        panic::catch_unwind(panic::AssertUnwindSafe(|| x.exp())),
                        panic::catch_unwind(panic::AssertUnwindSafe(|| (-x).exp())),
                    ) {
                        (Ok(exp_x), Ok(exp_neg_x)) => Ok((exp_x + exp_neg_x) / Decimal::TWO),
                        _ => Err(FuncError::MathError {
                            message: "Exponential overflow or underflow in cosh".to_string(),
                        }),
                    }
                },
                description: Some("Hyperbolic cosine".into()),
            },
            Symbol::Func {
                name: "tanh".into(),
                args: 1,
                variadic: false,
                callback: |args| {
                    let x = args[0];
                    // tanh(x) = sinh(x) / cosh(x) = (e^x - e^-x) / (e^x + e^-x)
                    match (
                        panic::catch_unwind(panic::AssertUnwindSafe(|| x.exp())),
                        panic::catch_unwind(panic::AssertUnwindSafe(|| (-x).exp())),
                    ) {
                        (Ok(exp_x), Ok(exp_neg_x)) => {
                            let sinh_x = exp_x - exp_neg_x;
                            let cosh_x = exp_x + exp_neg_x;
                            Ok(sinh_x / cosh_x)
                        }
                        _ => Err(FuncError::MathError {
                            message: "Exponential overflow or underflow in tanh".to_string(),
                        }),
                    }
                },
                description: Some("Hyperbolic tangent".into()),
            },
            // Power and root functions
            Symbol::Func {
                name: "sqrt".into(),
                args: 1,
                variadic: false,
                callback: |args| {
                    args[0]
                        .sqrt()
                        .ok_or_else(|| FuncError::NegativeSqrt { value: args[0] })
                },
                description: Some("Square root".into()),
            },
            Symbol::Func {
                name: "cbrt".into(),
                args: 1,
                variadic: false,
                callback: |args| Ok(cbrt_decimal(args[0])),
                description: Some("Cube root".into()),
            },
            Symbol::Func {
                name: "pow".into(),
                args: 2,
                variadic: false,
                callback: |args| {
                    use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
                    let base = args[0];
                    let exponent = args[1];

                    // Convert to f64, compute power, convert back
                    let base_f64 = base.to_f64().ok_or(FuncError::ToF64Conversion)?;
                    let exp_f64 = exponent.to_f64().ok_or(FuncError::ToF64Conversion)?;
                    let result_f64 = base_f64.powf(exp_f64);
                    Decimal::from_f64(result_f64).ok_or(FuncError::FromF64Conversion)
                },
                description: Some("x raised to power y".into()),
            },
            // Logarithmic and exponential functions
            Symbol::Func {
                name: "log".into(),
                args: 1,
                variadic: false,
                callback: |args| {
                    if args[0] <= Decimal::ZERO {
                        Err(FuncError::DomainError {
                            function: "log".to_string(),
                            input: args[0],
                        })
                    } else {
                        Ok(args[0].ln())
                    }
                },
                description: Some("Natural logarithm".into()),
            },
            Symbol::Func {
                name: "log2".into(),
                args: 1,
                variadic: false,
                callback: |args| {
                    if args[0] <= Decimal::ZERO {
                        Err(FuncError::DomainError {
                            function: "log2".to_string(),
                            input: args[0],
                        })
                    } else {
                        // log2(x) = ln(x) / ln(2)
                        Ok(args[0].ln() / Decimal::TWO.ln())
                    }
                },
                description: Some("Base-2 logarithm".into()),
            },
            Symbol::Func {
                name: "log10".into(),
                args: 1,
                variadic: false,
                callback: |args| {
                    if args[0] <= Decimal::ZERO {
                        Err(FuncError::DomainError {
                            function: "log10".to_string(),
                            input: args[0],
                        })
                    } else {
                        // log10(x) = ln(x) / ln(10)
                        Ok(args[0].ln() / LN_10)
                    }
                },
                description: Some("Base-10 logarithm".into()),
            },
            Symbol::Func {
                name: "exp".into(),
                args: 1,
                variadic: false,
                callback: |args| {
                    let input = args[0];
                    match panic::catch_unwind(panic::AssertUnwindSafe(|| input.exp())) {
                        Ok(result) => Ok(result),
                        Err(_) => Err(FuncError::MathError {
                            message: "Exponential overflow or underflow".to_string(),
                        }),
                    }
                },
                description: Some("e raised to power x".into()),
            },
            Symbol::Func {
                name: "exp2".into(),
                args: 1,
                variadic: false,
                callback: |args| {
                    let input = args[0];
                    // exp2(x) = exp(x * ln(2))
                    let exponent = input * Decimal::TWO.ln();
                    match panic::catch_unwind(panic::AssertUnwindSafe(|| exponent.exp())) {
                        Ok(result) => Ok(result),
                        Err(_) => Err(FuncError::MathError {
                            message: "Exponential overflow or underflow".to_string(),
                        }),
                    }
                },
                description: Some("2 raised to power x".into()),
            },
            // Basic math functions
            Symbol::Func {
                name: "abs".into(),
                args: 1,
                variadic: false,
                callback: |args| Ok(args[0].abs()),
                description: Some("Absolute value".into()),
            },
            Symbol::Func {
                name: "sign".into(),
                args: 1,
                variadic: false,
                callback: |args| Ok(args[0].signum()),
                description: Some("Sign function (-1, 0, or 1)".into()),
            },
            Symbol::Func {
                name: "floor".into(),
                args: 1,
                variadic: false,
                callback: |args| Ok(args[0].floor()),
                description: Some("Floor function".into()),
            },
            Symbol::Func {
                name: "ceil".into(),
                args: 1,
                variadic: false,
                callback: |args| Ok(args[0].ceil()),
                description: Some("Ceiling function".into()),
            },
            Symbol::Func {
                name: "round".into(),
                args: 1,
                variadic: false,
                callback: |args| Ok(args[0].round()),
                description: Some("Round to nearest integer".into()),
            },
            Symbol::Func {
                name: "trunc".into(),
                args: 1,
                variadic: false,
                callback: |args| Ok(args[0].trunc()),
                description: Some("Truncate to integer".into()),
            },
            Symbol::Func {
                name: "fract".into(),
                args: 1,
                variadic: false,
                callback: |args| Ok(args[0].fract()),
                description: Some("Fractional part".into()),
            },
            Symbol::Func {
                name: "mod".into(),
                args: 2,
                variadic: false,
                callback: |args| Ok(args[0] % args[1]),
                description: Some("Remainder of x/y".into()),
            },
            Symbol::Func {
                name: "hypot".into(),
                args: 2,
                variadic: false,
                callback: |args| {
                    let x = args[0];
                    let y = args[1];
                    // hypot(x, y) = sqrt(x² + y²)
                    let sum_of_squares = x * x + y * y;
                    sum_of_squares.sqrt().ok_or_else(|| FuncError::MathError {
                        message: "hypot: sqrt failed (should not happen for sum of squares)"
                            .to_string(),
                    })
                },
                description: Some("Euclidean distance sqrt(x²+y²)".into()),
            },
            Symbol::Func {
                name: "clamp".into(),
                args: 3,
                variadic: false,
                callback: |args| Ok(args[0].clamp(args[1].min(args[2]), args[2].max(args[1]))),
                description: Some("Constrain value between bounds".into()),
            },
            // Variadic functions
            Symbol::Func {
                name: "min".into(),
                args: 1,
                variadic: true,
                callback: |args| {
                    Ok(*args.iter().min().ok_or_else(|| FuncError::MathError {
                        message: "min() requires at least one argument".to_string(),
                    })?)
                },
                description: Some("Minimum value".into()),
            },
            Symbol::Func {
                name: "max".into(),
                args: 1,
                variadic: true,
                callback: |args| {
                    Ok(*args.iter().max().ok_or_else(|| FuncError::MathError {
                        message: "max() requires at least one argument".to_string(),
                    })?)
                },
                description: Some("Maximum value".into()),
            },
            Symbol::Func {
                name: "sum".into(),
                args: 1,
                variadic: true,
                callback: |args| Ok(args.iter().sum()),
                description: Some("Sum of values".into()),
            },
            Symbol::Func {
                name: "avg".into(),
                args: 1,
                variadic: true,
                callback: |args| {
                    let sum: Decimal = args.iter().sum();
                    let count = Decimal::from(args.len());
                    Ok(sum / count)
                },
                description: Some("Average of values".into()),
            },
        ])
    }
}
