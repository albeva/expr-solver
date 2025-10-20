//! Bytecode instruction definitions for the virtual machine.

use crate::number::Number;
#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};

/// Bytecode instructions for the stack-based virtual machine.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub enum Instr {
    Push(Number),
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
    // Control flow
    Jmp(usize), // Unconditional jump to instruction index
    Jz(usize),  // Jump to instruction index if top of stack is zero (consumes value)
}
