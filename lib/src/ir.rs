//! Bytecode instruction definitions shared across v1 and v2.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Bytecode instructions for the stack-based virtual machine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Instr {
    Push(Decimal),
    Load(usize), // Index into SymTable
    Neg,
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Fact,
    Call(usize, usize), // Index into SymTable and argument count
    // Comparison operators
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}
