use crate::ast::{BinOp, Expr, ExprKind, UnOp};
use crate::program::Program;
use crate::span::Span;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// IR building errors.
#[derive(Error, Debug, Clone)]
pub enum IrError {
    #[error("Undefined symbol {0}")]
    UndefinedSymbol(String, Span),
}

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

/// Builder for converting AST expressions into bytecode programs.
pub struct IrBuilder {
    prog: Program,
}

impl IrBuilder {
    /// Creates a new IR builder.
    pub fn new() -> Self {
        Self {
            prog: Program::new(),
        }
    }

    /// Builds a bytecode program from an AST expression.
    pub fn build<'src>(mut self, expr: &Expr<'src>) -> Result<Program, IrError> {
        self.emit(expr)?;
        Ok(self.prog)
    }

    fn emit<'src>(&mut self, e: &Expr<'src>) -> Result<(), IrError> {
        match &e.kind {
            ExprKind::Literal(v) => {
                self.prog.code.push(Instr::Push(*v));
            }
            ExprKind::Ident { name, sym_index } => {
                if let Some(idx) = sym_index {
                    self.prog.code.push(Instr::Load(*idx));
                } else {
                    return Err(IrError::UndefinedSymbol(name.to_string(), e.span));
                }
            }
            ExprKind::Unary { op, expr } => {
                self.emit(expr)?;
                match op {
                    UnOp::Neg => self.prog.code.push(Instr::Neg),
                    UnOp::Fact => self.prog.code.push(Instr::Fact),
                }
            }
            ExprKind::Binary { op, left, right } => {
                self.emit(left)?;
                self.emit(right)?;
                self.prog.code.push(match op {
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
                });
            }
            ExprKind::Call {
                name,
                args,
                sym_index,
            } => {
                if let Some(idx) = sym_index {
                    for a in args.iter() {
                        self.emit(a)?;
                    }
                    self.prog.code.push(Instr::Call(*idx, args.len()));
                } else {
                    return Err(IrError::UndefinedSymbol(name.to_string(), e.span));
                }
            }
        }
        Ok(())
    }
}
