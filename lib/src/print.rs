//! Pretty printing for expressions with syntax highlighting.
//!
//! This module provides the [`Print`] type for converting compiled programs
//! back to human-readable, syntax-highlighted expressions and assembly.

use crate::ir::Instr;
use crate::program::{Compiled, Linked, Program};
use crate::style::ExprStyle;
use std::fmt::{self, Write as _};

/// Pretty printer for expressions with syntax highlighting.
///
/// Takes a compiled or linked program and produces a syntax-highlighted string
/// representation. Supports both expression and assembly output formats.
///
/// # Examples
///
/// ```
/// use expr_solver::{Program, Print, SymTable};
///
/// // Print expression
/// let program = Program::new_from_source("let x = 10 then x * 2").unwrap();
/// let printer = Print::new(&program);
/// println!("{}", printer);  // Prints highlighted expression
///
/// // Print assembly
/// let linked = program.link(SymTable::stdlib()).unwrap();
/// let asm_printer = Print::new(&linked);
/// println!("{}", asm_printer.assembly());
/// ```
#[derive(Debug)]
pub struct Print<'p, S> {
    program: &'p Program<'p, S>,
    style: ExprStyle,
}

impl<'p, S> Print<'p, S> {
    /// Creates a new printer with default style.
    pub fn new(program: &'p Program<'p, S>) -> Self {
        Self {
            program,
            style: ExprStyle::default(),
        }
    }

    /// Creates a new printer with custom style.
    pub fn with_style(program: &'p Program<'p, S>, style: ExprStyle) -> Self {
        Self { program, style }
    }
}

// ============================================================================
// Expression printing (for Compiled programs)
// ============================================================================

impl<'p> Print<'p, Compiled> {
    /// Returns the pretty-printed expression as a string.
    ///
    /// Decompiles bytecode into an idealized pretty-printed expression.
    fn get_expr(&self) -> String {
        self.decompile_bytecode()
    }

    /// Decompiles bytecode back to an expression string.
    fn decompile_bytecode(&self) -> String {
        let bytecode = self.program.bytecode();
        let symbols = self.program.symbols();
        let (expr, _) = self.decompile_region(bytecode, symbols, 0, bytecode.len());
        expr
    }

    /// Recursively decompiles a region of bytecode.
    /// Returns (expression_string, declarations_list)
    fn decompile_region(
        &self,
        bytecode: &[Instr],
        symbols: &[crate::metadata::SymbolMetadata],
        start: usize,
        end: usize,
    ) -> (String, Vec<String>) {
        let mut stack: Vec<String> = Vec::new();
        let mut declarations: Vec<String> = Vec::new();
        let mut ip = start;

        while ip < end {
            let instr = &bytecode[ip];
            match instr {
                Instr::Push(value) => {
                    let styled = self.style.number(&value.to_string());
                    stack.push(styled.to_string());
                    ip += 1;
                }
                Instr::Load(idx) => {
                    let meta = &symbols[*idx];
                    let styled = if meta.local {
                        self.style.local_symbol(&meta.name)
                    } else {
                        self.style.global_symbol(&meta.name)
                    };
                    stack.push(styled.to_string());
                    ip += 1;
                }
                Instr::Store(idx) => {
                    // This is for LET declarations
                    let value = stack.pop().unwrap();
                    let meta = &symbols[*idx];
                    let name = self.style.local_symbol(&meta.name);
                    let eq = self.style.operator(" = ");
                    declarations.push(format!("{}{}{}", name, eq, value));
                    ip += 1;
                }
                Instr::Neg => {
                    let operand = stack.pop().unwrap();
                    let op = self.style.operator("-");
                    stack.push(format!("{}{}", op, operand));
                    ip += 1;
                }
                Instr::Add => {
                    self.binary_op(&mut stack, " + ");
                    ip += 1;
                }
                Instr::Sub => {
                    self.binary_op(&mut stack, " - ");
                    ip += 1;
                }
                Instr::Mul => {
                    self.binary_op(&mut stack, " * ");
                    ip += 1;
                }
                Instr::Div => {
                    self.binary_op(&mut stack, " / ");
                    ip += 1;
                }
                Instr::Pow => {
                    self.binary_op(&mut stack, " ^ ");
                    ip += 1;
                }
                Instr::Fact => {
                    let operand = stack.pop().unwrap();
                    let op = self.style.operator("!");
                    stack.push(format!("{}{}", operand, op));
                    ip += 1;
                }
                Instr::Equal => {
                    self.binary_op(&mut stack, " == ");
                    ip += 1;
                }
                Instr::NotEqual => {
                    self.binary_op(&mut stack, " != ");
                    ip += 1;
                }
                Instr::Less => {
                    self.binary_op(&mut stack, " < ");
                    ip += 1;
                }
                Instr::LessEqual => {
                    self.binary_op(&mut stack, " <= ");
                    ip += 1;
                }
                Instr::Greater => {
                    self.binary_op(&mut stack, " > ");
                    ip += 1;
                }
                Instr::GreaterEqual => {
                    self.binary_op(&mut stack, " >= ");
                    ip += 1;
                }
                Instr::Call(idx, argc) => {
                    let meta = &symbols[*idx];
                    let func_name = self.style.function(&meta.name);
                    let args: Vec<_> = (0..*argc)
                        .map(|_| stack.pop().unwrap())
                        .collect::<Vec<_>>()
                        .into_iter()
                        .rev()
                        .collect();
                    let lparen = self.style.delimiter("(");
                    let rparen = self.style.delimiter(")");
                    let comma = self.style.delimiter(", ");
                    stack.push(format!(
                        "{}{}{}{}",
                        func_name,
                        lparen,
                        args.join(&comma.to_string()),
                        rparen
                    ));
                    ip += 1;
                }
                Instr::Jz(else_target) => {
                    // Pattern: JZ else_target, <then_code>, JMP end_target, else_target: <else_code>
                    let condition = stack.pop().unwrap();

                    // Find the JMP instruction (should be right before else_target)
                    let jmp_pos = else_target - 1;
                    let end_target = if let Instr::Jmp(target) = &bytecode[jmp_pos] {
                        *target
                    } else {
                        // Malformed bytecode, use placeholder
                        let if_kw = self.style.keyword("if");
                        let placeholder = self.style.delimiter("(...)");
                        stack.push(format!("{}{}", if_kw, placeholder));
                        ip += 1;
                        continue;
                    };

                    // Recursively decompile then-branch (ip+1 to jmp_pos)
                    let (then_expr, _) = self.decompile_region(bytecode, symbols, ip + 1, jmp_pos);

                    // Recursively decompile else-branch (else_target to end_target)
                    let (else_expr, _) =
                        self.decompile_region(bytecode, symbols, *else_target, end_target);

                    // Construct if(...) expression
                    let if_kw = self.style.keyword("if");
                    let lparen = self.style.delimiter("(");
                    let rparen = self.style.delimiter(")");
                    let comma = self.style.delimiter(", ");

                    stack.push(format!(
                        "{}{}{}{}{}{}{}{}",
                        if_kw, lparen, condition, comma, then_expr, comma, else_expr, rparen
                    ));

                    // Jump past the entire if expression
                    ip = end_target;
                }
                Instr::Jmp(_) => {
                    // JMP without preceding JZ shouldn't happen in well-formed code
                    // This is part of an IF expression, so we should have already handled it
                    ip += 1;
                }
            }
        }

        let body = stack.pop().unwrap_or_default();

        // If we have declarations, wrap in LET...THEN
        let result = if !declarations.is_empty() {
            let let_kw = self.style.keyword("let");
            let then_kw = self.style.keyword(" then ");
            let comma = self.style.delimiter(", ");
            format!(
                "{} {}{}{}",
                let_kw,
                declarations.join(&comma.to_string()),
                then_kw,
                body
            )
        } else {
            body
        };

        (result, declarations)
    }

    fn binary_op(&self, stack: &mut Vec<String>, op: &str) {
        let right = stack.pop().unwrap();
        let left = stack.pop().unwrap();
        let op_styled = self.style.operator(op);
        stack.push(format!("{}{}{}", left, op_styled, right));
    }
}

impl<'p> fmt::Display for Print<'p, Compiled> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get_expr())
    }
}

// ============================================================================
// Assembly printing (for Linked programs)
// ============================================================================

impl<'p> Print<'p, Linked> {
    /// Returns the pretty-printed assembly as a string.
    pub fn assembly(&self) -> String {
        let mut out = String::new();

        // Version comment
        let comment = self
            .style
            .comment(&format!("; VERSION {}\n", self.program.version()));
        out.push_str(&comment.to_string());

        // Instructions
        for (i, instr) in self.program.bytecode().iter().enumerate() {
            let addr = self.style.asm_address(&format!("{:04X} ", i));
            let _ = write!(out, "{}", addr);

            let line = self.format_instruction(instr);
            let _ = writeln!(out, "{}", line);
        }

        out
    }

    /// Returns the pretty-printed expression for a linked program.
    ///
    /// Note: Linked programs don't have source, so this will show a placeholder.
    pub fn get_expr(&self) -> String {
        // For linked programs, we don't have access to the original expression
        // since the bytecode has already been linked with the symbol table
        self.style
            .comment("(expression not available for linked programs)")
            .to_string()
    }

    fn format_instruction(&self, instr: &Instr) -> String {
        match instr {
            Instr::Push(v) => {
                let instr = self.style.keyword("PUSH");
                let value = self.style.number(&v.to_string());
                format!("{} {}", instr, value)
            }
            Instr::Load(idx) => {
                let instr = self.style.keyword("LOAD");
                let sym_name = self.get_symbol_name(*idx);
                let sym = self.style.global_symbol(sym_name);
                format!("{} {}", instr, sym)
            }
            Instr::Store(idx) => {
                let instr = self.style.keyword("STORE");
                let sym_name = self.get_symbol_name(*idx);
                let sym = self.style.local_symbol(sym_name);
                format!("{} {}", instr, sym)
            }
            Instr::Neg => self.style.keyword("NEG").to_string(),
            Instr::Add => self.style.keyword("ADD").to_string(),
            Instr::Sub => self.style.keyword("SUB").to_string(),
            Instr::Mul => self.style.keyword("MUL").to_string(),
            Instr::Div => self.style.keyword("DIV").to_string(),
            Instr::Pow => self.style.keyword("POW").to_string(),
            Instr::Fact => self.style.keyword("FACT").to_string(),
            Instr::Call(idx, argc) => {
                let instr = self.style.keyword("CALL");
                let sym_name = self.get_symbol_name(*idx);
                let func = self.style.function(sym_name);
                let args_label = self.style.comment("args:");
                let args_count = self.style.number(&argc.to_string());
                format!("{} {} {} {}", instr, func, args_label, args_count)
            }
            Instr::Equal => self.style.keyword("EQ").to_string(),
            Instr::NotEqual => self.style.keyword("NEQ").to_string(),
            Instr::Less => self.style.keyword("LT").to_string(),
            Instr::LessEqual => self.style.keyword("LTE").to_string(),
            Instr::Greater => self.style.keyword("GT").to_string(),
            Instr::GreaterEqual => self.style.keyword("GTE").to_string(),
            Instr::Jmp(target) => {
                let instr = self.style.keyword("JMP");
                let addr = self.style.asm_address(&format!("{:04X}", target));
                format!("{} {}", instr, addr)
            }
            Instr::Jz(target) => {
                let instr = self.style.keyword("JZ");
                let addr = self.style.asm_address(&format!("{:04X}", target));
                format!("{} {}", instr, addr)
            }
        }
    }

    fn get_symbol_name(&self, idx: usize) -> &str {
        self.program
            .symtable()
            .get_by_index(idx)
            .map(|s| s.name())
            .expect("Symbol not found in assembly")
    }
}

impl<'p> fmt::Display for Print<'p, Linked> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.assembly())
    }
}
