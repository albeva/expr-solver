//! Bytecode instruction definitions for the virtual machine.

use crate::ast::{BinOp, UnOp};
use crate::number::Number;
#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};

/// Bytecode instructions for the stack-based virtual machine.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub enum Instr {
    Push(Number),
    Load(usize), // Index into SymTable
    Store(usize),
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
