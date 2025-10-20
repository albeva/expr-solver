//! Numeric type abstraction for the expression solver.
//!
//! This module provides a type alias `Number` that resolves to either `f64` or `Decimal`
//! depending on the enabled feature flag. This allows the library to be used with either
//! standard floating-point arithmetic (faster, simpler) or high-precision decimal arithmetic.
//!
//! ## Features
//!
//! - **`f64-floats`** (default): Standard f64 floating-point arithmetic
//!   - Fast and efficient for general-purpose math
//!   - Allows `Inf` and `NaN` results
//!   - Minimal error checking (only prevents panics)
//!
//! - **`decimal-precision`**: 128-bit Decimal for high precision
//!   - Exact decimal representation
//!   - Checked arithmetic with overflow/underflow detection
//!   - Domain validation for all operations
//!   - Ideal for financial calculations
//!
//! ## Type Alias
//!
//! The `Number` type alias resolves to:
//! - `f64` when `f64-floats` feature is enabled
//! - `rust_decimal::Decimal` when `decimal-precision` feature is enabled
//!
//! ## Internal Constants
//!
//! The `consts` module is internal and provides basic numeric constants (`ZERO`, `ONE`)
//! used by the VM. Mathematical constants (pi, e, etc.) are provided through the
//! type-specific symbol table implementations in `symbol/f64.rs` and `symbol/decimal.rs`.

#[cfg(feature = "decimal-precision")]
pub use rust_decimal::Decimal as Number;

#[cfg(feature = "f64-floats")]
pub type Number = f64;

// Ensure exactly one feature is enabled
#[cfg(all(feature = "f64-floats", feature = "decimal-precision"))]
compile_error!("Cannot enable both 'f64-floats' and 'decimal-precision' features");

#[cfg(not(any(feature = "f64-floats", feature = "decimal-precision")))]
compile_error!("Must enable either 'f64-floats' or 'decimal-precision' feature");

/// Internal numeric constants used by the VM.
/// Mathematical constants like PI, E, etc. are provided through the symbol table.
pub(crate) mod consts {
    use super::Number;

    #[cfg(feature = "decimal-precision")]
    pub use rust_decimal::Decimal;

    /// Zero constant
    #[cfg(feature = "decimal-precision")]
    pub const ZERO: Number = Decimal::ZERO;

    #[cfg(feature = "f64-floats")]
    pub const ZERO: Number = 0.0;

    /// One constant
    #[cfg(feature = "decimal-precision")]
    pub const ONE: Number = Decimal::ONE;

    #[cfg(feature = "f64-floats")]
    pub const ONE: Number = 1.0;
}

/// Helper trait for parsing numbers from strings.
pub trait ParseNumber: Sized {
    /// Parse a number from a string.
    fn parse_number(s: &str) -> Result<Self, String>;
}

#[cfg(feature = "decimal-precision")]
impl ParseNumber for Number {
    fn parse_number(s: &str) -> Result<Self, String> {
        use rust_decimal::prelude::FromStr;
        Number::from_str(s).map_err(|e| e.to_string())
    }
}

#[cfg(feature = "f64-floats")]
impl ParseNumber for Number {
    fn parse_number(s: &str) -> Result<Self, String> {
        s.parse::<f64>().map_err(|e| e.to_string())
    }
}

/// Macro for creating numeric literals in a type-neutral way.
///
/// # Examples
///
/// ```
/// use expr_solver::num;
///
/// let x = num!(42);
/// let y = num!(3.14159);
/// let z = num!(2.5);
/// ```
///
/// This macro resolves to:
/// - `$val as f64` when `f64-floats` is enabled
/// - `dec!($val)` when `decimal-precision` is enabled
#[macro_export]
#[cfg(feature = "decimal-precision")]
macro_rules! num {
    ($val:expr) => {
        rust_decimal_macros::dec!($val)
    };
}

/// Macro for creating numeric literals in a type-neutral way.
///
/// # Examples
///
/// ```
/// use expr_solver::num;
///
/// let x = num!(42);
/// let y = num!(3.14159);
/// let z = num!(2.5);
/// ```
///
/// This macro resolves to:
/// - `$val as f64` when `f64-floats` is enabled
/// - `dec!($val)` when `decimal-precision` is enabled
#[macro_export]
#[cfg(feature = "f64-floats")]
macro_rules! num {
    ($val:expr) => {
        $val as f64
    };
}
