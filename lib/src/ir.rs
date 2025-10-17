use crate::ast::{BinOp, Expr, ExprKind, UnOp};
use crate::program::Program;
use crate::span::Span;
use crate::symbol::Symbol;
use rust_decimal::Decimal;
use thiserror::Error;

/// IR building errors.
#[derive(Error, Debug, Clone)]
pub enum IrError {
    #[error("Undefined symbol {0}")]
    UndefinedSymbol(String, Span),
}

#[derive(Debug, Clone)]
pub enum Instr<'sym> {
    Push(Decimal),
    Load(&'sym Symbol),
    Neg,
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Fact,
    Call(&'sym Symbol, usize), // Symbol and argument count
    // Comparison operators
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

pub struct IrBuilder<'sym> {
    prog: Program<'sym>,
}

impl<'src, 'sym> IrBuilder<'sym> {
    pub fn new() -> Self {
        Self {
            prog: Program::new(),
        }
    }

    pub fn build(mut self, expr: &Expr<'src, 'sym>) -> Result<Program<'sym>, IrError> {
        self.emit(expr)?;
        Ok(self.prog)
    }

    fn emit(&mut self, e: &Expr<'src, 'sym>) -> Result<(), IrError> {
        match &e.kind {
            ExprKind::Literal(v) => {
                self.prog.code.push(Instr::Push(*v));
            }
            ExprKind::Ident { name, sym } => {
                if sym.is_none() {
                    return Err(IrError::UndefinedSymbol(name.to_string(), e.span));
                }
                self.prog.code.push(Instr::Load(sym.unwrap()));
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
            ExprKind::Call { name, args, sym } => {
                if sym.is_none() {
                    return Err(IrError::UndefinedSymbol(name.to_string(), e.span));
                }
                for a in args.iter() {
                    self.emit(a)?;
                }
                self.prog.code.push(Instr::Call(sym.unwrap(), args.len()));
            }
        }
        Ok(())
    }
}
