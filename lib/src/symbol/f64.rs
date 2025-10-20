//! F64-specific symbol table implementation.
//!
//! This module provides standard f64 floating-point arithmetic with relaxed error handling.
//! NaN and Inf values are allowed as results. Only domain errors that would cause panics
//! are caught and returned as errors.

use super::{SymTable, Symbol};

#[cfg(feature = "f64-floats")]
impl SymTable {
    /// Creates a symbol table with the standard library for f64 precision.
    ///
    /// This implementation uses standard f64 floating-point arithmetic.
    /// NaN and Inf results are allowed and not treated as errors.
    ///
    /// ## Fixed arity functions
    /// - `sin(x)` - Sine
    /// - `cos(x)` - Cosine
    /// - `tan(x)` - Tangent
    /// - `asin(x)` - Arcsine (returns NaN for |x| > 1)
    /// - `acos(x)` - Arccosine (returns NaN for |x| > 1)
    /// - `atan(x)` - Arctangent
    /// - `atan2(y, x)` - Two-argument arctangent
    /// - `sinh(x)` - Hyperbolic sine
    /// - `cosh(x)` - Hyperbolic cosine
    /// - `tanh(x)` - Hyperbolic tangent
    /// - `sqrt(x)` - Square root (returns NaN for x < 0)
    /// - `cbrt(x)` - Cube root
    /// - `pow(x, y)` - x raised to power y
    /// - `log(x)` - Natural logarithm (returns NaN for x <= 0)
    /// - `log2(x)` - Base-2 logarithm (returns NaN for x <= 0)
    /// - `log10(x)` - Base-10 logarithm (returns NaN for x <= 0)
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
        Self {
            symbols: vec![
                // Constants
                Symbol::Const {
                    name: "pi".into(),
                    value: std::f64::consts::PI,
                    description: Some("π (3.14159...)".into()),
                },
                Symbol::Const {
                    name: "e".into(),
                    value: std::f64::consts::E,
                    description: Some("Euler's number (2.71828...)".into()),
                },
                Symbol::Const {
                    name: "tau".into(),
                    value: std::f64::consts::TAU,
                    description: Some("2π (6.28318...)".into()),
                },
                Symbol::Const {
                    name: "ln2".into(),
                    value: std::f64::consts::LN_2,
                    description: Some("Natural logarithm of 2".into()),
                },
                Symbol::Const {
                    name: "ln10".into(),
                    value: std::f64::consts::LN_10,
                    description: Some("Natural logarithm of 10".into()),
                },
                Symbol::Const {
                    name: "sqrt2".into(),
                    value: std::f64::consts::SQRT_2,
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
                    callback: |args| Ok(args[0].tan()),
                    description: Some("Tangent".into()),
                },
                Symbol::Func {
                    name: "asin".into(),
                    args: 1,
                    variadic: false,
                    callback: |args| Ok(args[0].asin()),
                    description: Some("Arcsine".into()),
                },
                Symbol::Func {
                    name: "acos".into(),
                    args: 1,
                    variadic: false,
                    callback: |args| Ok(args[0].acos()),
                    description: Some("Arccosine".into()),
                },
                Symbol::Func {
                    name: "atan".into(),
                    args: 1,
                    variadic: false,
                    callback: |args| Ok(args[0].atan()),
                    description: Some("Arctangent".into()),
                },
                Symbol::Func {
                    name: "atan2".into(),
                    args: 2,
                    variadic: false,
                    callback: |args| Ok(args[0].atan2(args[1])),
                    description: Some("Two-argument arctangent".into()),
                },
                Symbol::Func {
                    name: "sinh".into(),
                    args: 1,
                    variadic: false,
                    callback: |args| Ok(args[0].sinh()),
                    description: Some("Hyperbolic sine".into()),
                },
                Symbol::Func {
                    name: "cosh".into(),
                    args: 1,
                    variadic: false,
                    callback: |args| Ok(args[0].cosh()),
                    description: Some("Hyperbolic cosine".into()),
                },
                Symbol::Func {
                    name: "tanh".into(),
                    args: 1,
                    variadic: false,
                    callback: |args| Ok(args[0].tanh()),
                    description: Some("Hyperbolic tangent".into()),
                },
                // Power and root functions
                Symbol::Func {
                    name: "sqrt".into(),
                    args: 1,
                    variadic: false,
                    callback: |args| Ok(args[0].sqrt()),
                    description: Some("Square root".into()),
                },
                Symbol::Func {
                    name: "cbrt".into(),
                    args: 1,
                    variadic: false,
                    callback: |args| Ok(args[0].cbrt()),
                    description: Some("Cube root".into()),
                },
                Symbol::Func {
                    name: "pow".into(),
                    args: 2,
                    variadic: false,
                    callback: |args| Ok(args[0].powf(args[1])),
                    description: Some("x raised to power y".into()),
                },
                // Logarithmic and exponential functions
                Symbol::Func {
                    name: "log".into(),
                    args: 1,
                    variadic: false,
                    callback: |args| Ok(args[0].ln()),
                    description: Some("Natural logarithm".into()),
                },
                Symbol::Func {
                    name: "log2".into(),
                    args: 1,
                    variadic: false,
                    callback: |args| Ok(args[0].log2()),
                    description: Some("Base-2 logarithm".into()),
                },
                Symbol::Func {
                    name: "log10".into(),
                    args: 1,
                    variadic: false,
                    callback: |args| Ok(args[0].log10()),
                    description: Some("Base-10 logarithm".into()),
                },
                Symbol::Func {
                    name: "exp".into(),
                    args: 1,
                    variadic: false,
                    callback: |args| Ok(args[0].exp()),
                    description: Some("e raised to power x".into()),
                },
                Symbol::Func {
                    name: "exp2".into(),
                    args: 1,
                    variadic: false,
                    callback: |args| Ok(args[0].exp2()),
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
                    callback: |args| {
                        let x = args[0];
                        if x > 0.0 {
                            Ok(1.0)
                        } else if x < 0.0 {
                            Ok(-1.0)
                        } else {
                            Ok(0.0)
                        }
                    },
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
                    callback: |args| Ok(args[0].hypot(args[1])),
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
                    callback: |args| Ok(args.iter().fold(f64::INFINITY, |acc, &x| acc.min(x))),
                    description: Some("Minimum value".into()),
                },
                Symbol::Func {
                    name: "max".into(),
                    args: 1,
                    variadic: true,
                    callback: |args| Ok(args.iter().fold(f64::NEG_INFINITY, |acc, &x| acc.max(x))),
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
                        let sum: f64 = args.iter().sum();
                        let count = args.len() as f64;
                        Ok(sum / count)
                    },
                    description: Some("Average of values".into()),
                },
            ],
        }
    }
}
