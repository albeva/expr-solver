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

/// Mathematical constants for the selected numeric type.
pub mod consts {
    use super::Number;

    #[cfg(feature = "decimal-precision")]
    pub use rust_decimal::Decimal;

    #[cfg(feature = "decimal-precision")]
    use rust_decimal_macros::dec;

    /// π (pi) constant
    #[cfg(feature = "decimal-precision")]
    pub const PI: Number = Decimal::PI;

    #[cfg(feature = "f64-floats")]
    pub const PI: Number = std::f64::consts::PI;

    /// Euler's number (e)
    #[cfg(feature = "decimal-precision")]
    pub const E: Number = Decimal::E;

    #[cfg(feature = "f64-floats")]
    pub const E: Number = std::f64::consts::E;

    /// 2π (tau)
    #[cfg(feature = "decimal-precision")]
    pub const TAU: Number = Decimal::TWO_PI;

    #[cfg(feature = "f64-floats")]
    pub const TAU: Number = std::f64::consts::TAU;

    /// Natural logarithm of 2
    #[cfg(feature = "decimal-precision")]
    pub const LN_2: Number = dec!(0.6931471805599453094172321);

    #[cfg(feature = "f64-floats")]
    pub const LN_2: Number = std::f64::consts::LN_2;

    /// Natural logarithm of 10
    #[cfg(feature = "decimal-precision")]
    pub const LN_10: Number = dec!(2.3025850929940456840179915);

    #[cfg(feature = "f64-floats")]
    pub const LN_10: Number = std::f64::consts::LN_10;

    /// Square root of 2
    #[cfg(feature = "decimal-precision")]
    pub const SQRT_2: Number = dec!(1.4142135623730950488016887);

    #[cfg(feature = "f64-floats")]
    pub const SQRT_2: Number = std::f64::consts::SQRT_2;

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

    /// Two constant
    #[cfg(feature = "decimal-precision")]
    pub const TWO: Number = Decimal::TWO;

    #[cfg(feature = "f64-floats")]
    pub const TWO: Number = 2.0;

    /// Negative one constant
    #[cfg(feature = "decimal-precision")]
    pub const NEG_ONE: Number = Decimal::NEGATIVE_ONE;

    #[cfg(feature = "f64-floats")]
    pub const NEG_ONE: Number = -1.0;
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
