//! IR (bytecode) builder for compiling AST to bytecode.
//!
//! The [`IrBuilder`] is responsible for the second stage of compilation:
//! transforming the Abstract Syntax Tree (AST) into bytecode instructions
//! and collecting symbol metadata.
//!
//! # Process
//!
//! 1. Traverse the AST recursively
//! 2. Emit bytecode instructions for each expression
//! 3. Collect symbol references (functions and constants)
//! 4. Handle `let` declarations by creating local symbols
//! 5. Return bytecode and symbol metadata for linking
//!
//! # Example
//!
//! ```ignore
//! use expr_solver::ir_builder::IrBuilder;
//! use expr_solver::parser::Parser;
//!
//! let mut parser = Parser::new("let x = 5 then x * 2");
//! let ast = parser.parse().unwrap().unwrap();
//!
//! let (bytecode, symbols) = IrBuilder::new().build(&ast).unwrap();
//! // bytecode contains instructions to compute the expression
//! // symbols contains metadata about 'x' (local) and any other references
//! ```

use crate::SymbolError;
use crate::ast::{Decl, Expr, ExprKind};
use crate::error::IrError;
use crate::ir::Instr;
use crate::metadata::{SymbolKind, SymbolMetadata};
use std::borrow::Cow;
use std::borrow::Cow::Owned;

/// Builder for generating bytecode from an AST.
///
/// The builder maintains state during compilation including the bytecode
/// instructions and symbol metadata table.
#[derive(Debug, Default)]
pub struct IrBuilder {
    bytecode: Vec<Instr>,
    symbols: Vec<SymbolMetadata>,
    locals: Option<Vec<Cow<'static, str>>>,
}

impl IrBuilder {
    /// Creates a new IR builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Builds bytecode from an AST expression.
    ///
    /// This is the main entry point for compilation.
    pub fn build(mut self, ast: &Expr) -> Result<(Vec<Instr>, Vec<SymbolMetadata>), IrError> {
        // Handle let declarations at the top level
        let body = if let ExprKind::Let { decls, body } = &ast.kind {
            self.emit_let(decls)?;
            body
        } else {
            ast
        };

        // Emit the main expression body
        self.emit_expr(body);

        Ok((self.bytecode, self.symbols))
    }

    /// Emits let declarations.
    fn emit_let(&mut self, decls: &[Decl]) -> Result<(), IrError> {
        for decl in decls {
            match decl {
                Decl::Var { name, body } => self.emit_var_decl(name, body)?,
                Decl::Func { name, params, body } => self.emit_func_decl(name, params, body)?,
            }
        }
        Ok(())
    }

    fn emit_var_decl(&mut self, name: &str, expr: &Expr) -> Result<(), IrError> {
        // Check for duplicate declarations
        if self.symbols.iter().any(|meta| meta.name == *name) {
            return Err(SymbolError::DuplicateSymbol(name.to_string()).into());
        }

        // Emit bytecode for the value expression
        self.emit_expr(expr);

        // Declare local symbol
        let idx = self.symbols.len();
        self.symbols.push(SymbolMetadata {
            name: Owned(name.to_string()),
            kind: SymbolKind::Const,
            local: true,
            index: None,
        });

        // Store the value
        self.bytecode.push(Instr::Store(idx));

        Ok(())
    }

    fn emit_func_decl(&mut self, name: &str, params: &[&str], body: &Expr) -> Result<(), IrError> {
        // Check for duplicate declarations
        if self.symbols.iter().any(|meta| meta.name == *name) {
            return Err(SymbolError::DuplicateSymbol(name.to_string()).into());
        }

        self.locals = Some(params.iter().map(|s| Owned(s.to_string())).collect());

        // jmp instruction to skip the function body
        // TODO: functions should bodies should be at the end of bytecode
        //       and execution terminated with HALT.
        let jump_offset = self.bytecode.len();
        self.bytecode.push(Instr::Jmp(0));

        // Function start
        let offset = self.bytecode.len();

        // Declare local symbol
        let idx = self.symbols.len();
        self.symbols.push(SymbolMetadata {
            name: Owned(name.to_string()),
            kind: SymbolKind::LocalFunc {
                arity: params.len(),
                params: Vec::default(),
                offset,
            },
            local: true,
            index: None,
        });

        // the func body
        self.emit_expr(body);

        // ret instruction
        self.bytecode.push(Instr::Ret);

        // patch the jump instruction
        self.bytecode[jump_offset] = Instr::Jmp(self.bytecode.len());

        // assign locals
        let locals = self.locals.take().unwrap();
        match &mut self.symbols[idx].kind {
            SymbolKind::LocalFunc { params, .. } => {
                *params = locals;
            }
            _ => unreachable!(),
        }

        // done
        Ok(())
    }

    /// Emits bytecode for an expression.
    fn emit_expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Literal(v) => {
                self.bytecode.push(Instr::Push(*v));
            }
            ExprKind::Ident { name } => {
                if let Some(locals) = self.locals.as_ref() {
                    if let Some(idx) = locals.iter().position(|l| l.eq_ignore_ascii_case(name)) {
                        self.bytecode.push(Instr::LoadParam(idx));
                        return;
                    }
                }
                let idx = self.get_or_create_symbol(name, SymbolKind::Const);
                self.bytecode.push(Instr::Load(idx));
            }
            ExprKind::Unary { op, expr } => {
                self.emit_expr(expr);
                self.bytecode.push((*op).into());
            }
            ExprKind::Binary { op, left, right } => {
                self.emit_expr(left);
                self.emit_expr(right);
                self.bytecode.push((*op).into());
            }
            ExprKind::Call { name, args } => self.emit_call(name, args),
            ExprKind::If {
                cond,
                then_branch,
                else_branch,
            } => self.emit_if(cond, then_branch, else_branch),
            ExprKind::Let { body, .. } => self.emit_expr(body),
        }
    }

    /// Emits bytecode for an if expression.
    fn emit_if(&mut self, cond: &Expr, then_branch: &Expr, else_branch: &Expr) {
        // Emit condition
        self.emit_expr(cond);

        // Emit Jz to else branch (placeholder, will be backpatched)
        let jz_idx = self.bytecode.len();
        self.bytecode.push(Instr::Jz(0)); // Placeholder

        // Emit then branch
        self.emit_expr(then_branch);

        // Emit Jmp to end (placeholder, will be backpatched)
        let jmp_idx = self.bytecode.len();
        self.bytecode.push(Instr::Jmp(0)); // Placeholder

        // else_start: This is where we jump if condition is false
        let else_start = self.bytecode.len();

        // Emit else branch
        self.emit_expr(else_branch);

        // end: This is where we jump after then branch
        let end = self.bytecode.len();

        // Backpatch the jump targets
        self.bytecode[jz_idx] = Instr::Jz(else_start);
        self.bytecode[jmp_idx] = Instr::Jmp(end);
    }

    /// Emits bytecode for a function call.
    fn emit_call(&mut self, name: &str, args: &[Expr]) {
        // Emit arguments first
        for arg in args {
            self.emit_expr(arg);
        }

        // Get or create symbol index for this function
        let idx = self.get_or_create_symbol(
            name,
            SymbolKind::GlobalFunc {
                arity: args.len(),
                variadic: false,
            },
        );
        self.bytecode.push(Instr::Call(idx, args.len()));
    }

    /// Gets existing symbol index or creates a new one.
    ///
    /// For ~50 symbols, linear search is faster than HashMap overhead.
    fn get_or_create_symbol(&mut self, name: &str, kind: SymbolKind) -> usize {
        // Check if symbol already exists
        if let Some(pos) = self.symbols.iter().position(|s| s.name == name) {
            return pos;
        }

        // Create new symbol entry
        self.symbols.push(SymbolMetadata {
            name: name.to_string().into(),
            kind,
            local: false,
            index: None,
        });
        self.symbols.len() - 1
    }
}
