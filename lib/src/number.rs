//! Numeric type abstraction for the expression solver.
//!
//! This module provides a type alias `Number` that resolves to either `f64` or `Decimal`
//! depending on the enabled feature flag. This allows the library to be used with either
//! standard floating-point arithmetic (faster, simpler) or high-precision decimal arithmetic.
//!
//! ## Features
//!
//! - `f64-floats` (default): Use standard f64 floating-point arithmetic
//! - `decimal-precision`: Use 128-bit Decimal for high precision
//!
//! ## Type Alias
//!
//! The `Number` type alias resolves to:
//! - `f64` when `f64-floats` is enabled
//! - `rust_decimal::Decimal` when `decimal-precision` is enabled

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
