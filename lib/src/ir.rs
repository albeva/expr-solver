//! Bytecode instruction definitions for the virtual machine.

use crate::ast::{BinOp, UnOp};
use crate::number::Number;
#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};

/// Bytecode instructions for the stack-based virtual machine.
///
/// The VM uses a stack-based architecture with support for user-defined functions
/// through a call stack mechanism.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub enum Instr {
    /// Push a constant value onto the stack
    Push(Number),
    /// Load a symbol value by index and push onto stack
    Load(usize),
    /// Store top of stack to a symbol by index
    Store(usize),
    /// Negate the top of stack
    Neg,
    /// Pop two values, add them, push result
    Add,
    /// Pop two values, subtract them, push result
    Sub,
    /// Pop two values, multiply them, push result
    Mul,
    /// Pop two values, divide them, push result
    Div,
    /// Pop two values, compute power, push result
    Pow,
    /// Pop value, compute factorial, push result
    Fact,
    /// Call a function by symbol index with argument count
    Call(usize, usize),
    /// Compare top two values for equality
    Equal,
    /// Compare top two values for inequality
    NotEqual,
    /// Compare if second < top
    Less,
    /// Compare if second <= top
    LessEqual,
    /// Compare if second > top
    Greater,
    /// Compare if second >= top
    GreaterEqual,
    // Control flow
    /// Unconditional jump to instruction index (used for if-expressions and function skip)
    Jmp(usize),
    /// Jump to instruction index if top of stack is zero (consumes value)
    Jz(usize),
    // Function support
    /// Load a function parameter by index from current call frame
    LoadParam(usize),
    /// Return from function call, popping call frame
    Ret,
}

impl From<UnOp> for Instr {
    fn from(op: UnOp) -> Self {
        match op {
            UnOp::Neg => Instr::Neg,
            UnOp::Fact => Instr::Fact,
        }
    }
}

impl From<BinOp> for Instr {
    fn from(op: BinOp) -> Self {
        match op {
            BinOp::Add => Instr::Add,
            BinOp::Sub => Instr::Sub,
            BinOp::Mul => Instr::Mul,
            BinOp::Div => Instr::Div,
            BinOp::Pow => Instr::Pow,
            BinOp::Equal => Instr::Equal,
            BinOp::NotEqual => Instr::NotEqual,
            BinOp::Less => Instr::Less,
            BinOp::LessEqual => Instr::LessEqual,
            BinOp::Greater => Instr::Greater,
            BinOp::GreaterEqual => Instr::GreaterEqual,
        }
    }
}
