//! Numeric type abstraction for the expression solver.
//!
//! This module provides a type alias `Number` that resolves to either `f64` (default)
//! or `Decimal` (when the `decimal` feature is enabled).
//!
//! ## Features
//!
//! - **Default (f64)**: Standard f64 floating-point arithmetic
//!   - Fast and efficient for general-purpose math
//!   - Allows `Inf` and `NaN` results
//!   - Minimal error checking (only prevents panics)
//!
//! - **`decimal`**: 128-bit Decimal for high precision
//!   - Exact decimal representation
//!   - Checked arithmetic with overflow/underflow detection
//!   - Domain validation for all operations
//!   - Ideal for financial calculations
//!
//! ## Type Alias
//!
//! The `Number` type alias resolves to:
//! - `f64` by default
//! - `rust_decimal::Decimal` when `decimal` feature is enabled
//!
//! ## Internal Constants
//!
//! The `consts` module is internal and provides basic numeric constants (`ZERO`, `ONE`)
//! used by the VM. Mathematical constants (pi, e, etc.) are provided through the
//! type-specific symbol table implementations in `symbol/f64.rs` and `symbol/decimal.rs`.

#[cfg(feature = "decimal")]
pub use rust_decimal::Decimal as Number;

#[cfg(not(feature = "decimal"))]
pub type Number = f64;

/// Internal numeric constants used by the VM.
/// Mathematical constants like PI, E, etc. are provided through the symbol table.
pub(crate) mod consts {
    use super::Number;

    #[cfg(feature = "decimal")]
    pub use rust_decimal::Decimal;

    /// Zero constant
    #[cfg(feature = "decimal")]
    pub const ZERO: Number = Decimal::ZERO;

    #[cfg(not(feature = "decimal"))]
    pub const ZERO: Number = 0.0;

    /// One constant
    #[cfg(feature = "decimal")]
    pub const ONE: Number = Decimal::ONE;

    #[cfg(not(feature = "decimal"))]
    pub const ONE: Number = 1.0;
}

/// Helper trait for parsing numbers from strings.
pub trait ParseNumber: Sized {
    /// Parse a number from a string.
    fn parse_number(s: &str) -> Result<Self, String>;
}

// Decimal implementation
#[cfg(feature = "decimal")]
impl ParseNumber for Number {
    fn parse_number(s: &str) -> Result<Self, String> {
        use rust_decimal::Decimal;
        s.parse::<Decimal>()
            .map_err(|e| format!("Failed to parse number: {}", e))
    }
}

// f64 implementation
#[cfg(not(feature = "decimal"))]
impl ParseNumber for Number {
    fn parse_number(s: &str) -> Result<Self, String> {
        s.parse::<f64>()
            .map_err(|e| format!("Failed to parse number: {}", e))
    }
}

/// Convenience macro for creating numeric literals that work with both backends.
///
/// # Examples
///
/// ```
/// use expr_solver::num;
///
/// let x = num!(42);
/// let y = num!(3.14);
/// let z = num!(-10);
/// ```
#[cfg(feature = "decimal")]
#[macro_export]
macro_rules! num {
    ($val:expr) => {{
        use rust_decimal_macros::dec;
        dec!($val)
    }};
}

#[cfg(not(feature = "decimal"))]
#[macro_export]
macro_rules! num {
    ($val:expr) => {
        $val as f64
    };
}
