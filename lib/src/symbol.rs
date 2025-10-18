use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use std::borrow::Cow;
use std::panic;
use thiserror::Error;

/// Errors that can occur during function evaluation.
#[derive(Error, Debug, Clone)]
pub enum FuncError {
    #[error("Conversion error: failed to convert Decimal to f64")]
    DecimalToF64Conversion,
    #[error("Conversion error: failed to convert f64 result back to Decimal")]
    F64ToDecimalConversion,
    #[error("Square root of negative number: {value}")]
    NegativeSqrt { value: Decimal },
    #[error("Domain error in function '{function}': invalid input {input}")]
    DomainError { function: String, input: Decimal },
    #[error("Math error: {message}")]
    MathError { message: String },
}

/// Helper function for single-argument f64 calculations
fn f64_calc_1<F>(args: &[Decimal], func: F) -> Result<Decimal, FuncError>
where
    F: Fn(f64) -> f64,
{
    let arg = args[0].to_f64().ok_or(FuncError::DecimalToF64Conversion)?;
    let result = func(arg);
    Decimal::from_f64(result).ok_or(FuncError::F64ToDecimalConversion)
}

/// Helper function for two-argument f64 calculations
fn f64_calc_2<F>(args: &[Decimal], func: F) -> Result<Decimal, FuncError>
where
    F: Fn(f64, f64) -> f64,
{
    let arg1 = args[0].to_f64().ok_or(FuncError::DecimalToF64Conversion)?;
    let arg2 = args[1].to_f64().ok_or(FuncError::DecimalToF64Conversion)?;
    let result = func(arg1, arg2);
    Decimal::from_f64(result).ok_or(FuncError::F64ToDecimalConversion)
}

/// Errors that can occur during symbol table operations.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum SymbolError {
    /// A symbol with this name already exists in the table.
    #[error("Duplicate symbol definition: '{0}'")]
    DuplicateSymbol(String),
}

/// A symbol representing either a constant or function.
///
/// Symbols are stored in a [`SymTable`] and referenced during evaluation.
#[derive(Debug, Clone)]
pub enum Symbol {
    /// Named constant (e.g., `pi`).
    Const {
        name: Cow<'static, str>,
        value: Decimal,
        description: Option<Cow<'static, str>>,
    },
    /// Function with specified arity and callback.
    Func {
        name: Cow<'static, str>,
        /// Minimum number of arguments
        args: usize,
        /// Whether the function accepts additional arguments
        variadic: bool,
        callback: fn(&[Decimal]) -> Result<Decimal, FuncError>,
        description: Option<Cow<'static, str>>,
    },
}

impl Symbol {
    /// Returns the name of the symbol.
    pub fn name(&self) -> &str {
        match self {
            Symbol::Const { name, .. } => name,
            Symbol::Func { name, .. } => name,
        }
    }

    /// Returns the description of the symbol, if available.
    pub fn description(&self) -> Option<&str> {
        match self {
            Symbol::Const { description, .. } => description.as_deref(),
            Symbol::Func { description, .. } => description.as_deref(),
        }
    }
}

/// Symbol table containing constants and functions.
///
/// The table stores mathematical constants like `pi` and functions like `sin`.
/// Symbol lookups are case-insensitive.
///
/// # Examples
///
/// ```
/// use expr_solver::SymTable;
/// use rust_decimal_macros::dec;
///
/// let mut table = SymTable::stdlib();
/// table.add_const("x", dec!(42)).unwrap();
/// ```
#[derive(Debug, Default, Clone)]
pub struct SymTable {
    symbols: Vec<Symbol>,
}

impl SymTable {
    /// Creates an empty symbol table.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a symbol table with the standard library.
    ///
    /// ## Constants
    /// - `pi` - π (3.14159...)
    /// - `e` - Euler's number (2.71828...)
    /// - `tau` - 2π (6.28318...)
    /// - `ln2` - Natural logarithm of 2
    /// - `ln10` - Natural logarithm of 10
    /// - `sqrt2` - Square root of 2
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
        Self {
            symbols: vec![
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
                    value: Decimal::TWO.ln(),
                    description: Some("Natural logarithm of 2".into()),
                },
                Symbol::Const {
                    name: "ln10".into(),
                    value: Decimal::TEN.log10(),
                    description: Some("Natural logarithm of 10".into()),
                },
                Symbol::Const {
                    name: "sqrt2".into(),
                    value: Decimal::TWO.sqrt().unwrap(),
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
                    callback: |args| f64_calc_1(args, |x| x.sinh()),
                    description: Some("Hyperbolic sine".into()),
                },
                Symbol::Func {
                    name: "cosh".into(),
                    args: 1,
                    variadic: false,
                    callback: |args| f64_calc_1(args, |x| x.cosh()),
                    description: Some("Hyperbolic cosine".into()),
                },
                Symbol::Func {
                    name: "tanh".into(),
                    args: 1,
                    variadic: false,
                    callback: |args| f64_calc_1(args, |x| x.tanh()),
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
                    callback: |args| f64_calc_1(args, |x| x.cbrt()),
                    description: Some("Cube root".into()),
                },
                Symbol::Func {
                    name: "pow".into(),
                    args: 2,
                    variadic: false,
                    callback: |args| {
                        let base = args[0];
                        let exponent = args[1];
                        match panic::catch_unwind(panic::AssertUnwindSafe(|| base.powd(exponent))) {
                            Ok(result) => Ok(result),
                            Err(_) => Err(FuncError::MathError {
                                message: format!("Power operation failed: {}^{}", base, exponent),
                            }),
                        }
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
                    callback: |args| f64_calc_1(args, |x| x.log2()),
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
                            Ok(args[0].log10())
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
                    callback: |args| f64_calc_1(args, |x| x.exp2()),
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
                    callback: |args| f64_calc_2(args, |x, y| x.hypot(y)),
                    description: Some("Euclidean distance sqrt(x²+y²)".into()),
                },
                Symbol::Func {
                    name: "clamp".into(),
                    args: 3,
                    variadic: false,
                    callback: |args| Ok(args[0].clamp(args[1].min(args[2]), args[2].max(args[1]))),
                    description: Some("Constrain value between bounds".into()),
                },
                Symbol::Func {
                    name: "if".into(),
                    args: 3,
                    variadic: false,
                    callback: |args| {
                        if args[0] != Decimal::ZERO {
                            Ok(args[1])
                        } else {
                            Ok(args[2])
                        }
                    },
                    description: Some(
                        "Conditional expression: if(condition, true_value, false_value)".into(),
                    ),
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
            ],
        }
    }

    /// Adds a constant to the table.
    ///
    /// Returns an error if a symbol with the same name already exists.
    pub fn add_const<S: Into<Cow<'static, str>>>(
        &mut self,
        name: S,
        value: Decimal,
    ) -> Result<&mut Self, SymbolError> {
        let name = name.into();
        if self.get(&name).is_some() {
            return Err(SymbolError::DuplicateSymbol(name.to_string()));
        }
        self.symbols.push(Symbol::Const {
            name,
            value,
            description: None,
        });
        Ok(self)
    }

    /// Adds a function to the table.
    ///
    /// # Parameters
    /// - `name`: Function name
    /// - `args`: Minimum number of arguments
    /// - `variadic`: Whether the function accepts additional arguments
    /// - `callback`: Function implementation
    ///
    /// Returns an error if a symbol with the same name already exists.
    pub fn add_func<S: Into<Cow<'static, str>>>(
        &mut self,
        name: S,
        args: usize,
        variadic: bool,
        callback: fn(&[Decimal]) -> Result<Decimal, FuncError>,
    ) -> Result<&mut Self, SymbolError> {
        let name = name.into();
        if self.get(&name).is_some() {
            return Err(SymbolError::DuplicateSymbol(name.to_string()));
        }
        self.symbols.push(Symbol::Func {
            name,
            args,
            variadic,
            callback,
            description: None,
        });
        Ok(self)
    }

    /// Looks up a symbol by name (case-insensitive).
    pub fn get(&self, name: &str) -> Option<&Symbol> {
        self.symbols
            .iter()
            .find(|sym| sym.name().eq_ignore_ascii_case(name))
    }

    /// Looks up a symbol by name and returns its index and reference (case-insensitive).
    pub fn get_with_index(&self, name: &str) -> Option<(usize, &Symbol)> {
        self.symbols
            .iter()
            .enumerate()
            .find(|(_, sym)| sym.name().eq_ignore_ascii_case(name))
    }

    /// Returns a symbol by index.
    pub fn get_by_index(&self, index: usize) -> Option<&Symbol> {
        self.symbols.get(index)
    }

    /// Returns an iterator over all symbols in the table.
    pub fn symbols(&self) -> impl Iterator<Item = &Symbol> {
        self.symbols.iter()
    }
}
