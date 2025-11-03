//! Pretty printing for expressions with syntax highlighting.
//!
//! This module provides the [`Print`] type for converting compiled programs
//! back to human-readable, syntax-highlighted expressions and assembly.

use crate::ir::Instr;
use crate::program::{Compiled, Linked, Program};
use crate::style::ExprStyle;
use std::collections::HashMap;
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

    /// Core decompilation logic shared between Compiled and Linked.
    /// Takes closures to access symbol name and local flag.
    fn decompile_region<F, G>(
        &self,
        bytecode: &[Instr],
        start: usize,
        end: usize,
        get_symbol_name: &F,
        is_local: &G,
        func_labels: Option<&HashMap<usize, (String, Vec<String>)>>,
        current_params: Option<&Vec<String>>,
    ) -> (String, Vec<String>)
    where
        F: Fn(usize) -> String,
        G: Fn(usize) -> bool,
    {
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
                    let sym_name = get_symbol_name(*idx);
                    let styled = if is_local(*idx) {
                        self.style.local_symbol(&sym_name)
                    } else {
                        self.style.global_symbol(&sym_name)
                    };
                    stack.push(styled.to_string());
                    ip += 1;
                }
                Instr::Store(idx) => {
                    let value = stack.pop().unwrap();
                    let sym_name = get_symbol_name(*idx);
                    let name = self.style.local_symbol(&sym_name);
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
                    let sym_name = get_symbol_name(*idx);
                    let func_name = self.style.function(&sym_name);
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

                    // Recursively decompile then-branch
                    let (then_expr, _) = self.decompile_region(
                        bytecode,
                        ip + 1,
                        jmp_pos,
                        get_symbol_name,
                        is_local,
                        func_labels,
                        current_params,
                    );

                    // Recursively decompile else-branch
                    let (else_expr, _) = self.decompile_region(
                        bytecode,
                        *else_target,
                        end_target,
                        get_symbol_name,
                        is_local,
                        func_labels,
                        current_params,
                    );

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
                Instr::Jmp(target) => {
                    // Check if this is a function-skipping JMP
                    // Function bodies are at ip+1, and JMP targets are after the function
                    if let Some(func_map) = func_labels {
                        let func_start = ip + 1;
                        if let Some((func_name, params)) = func_map.get(&func_start) {
                            // This JMP skips over a function body
                            // Decompile the function

                            // Find the RET instruction
                            let mut func_end = func_start;
                            while func_end < bytecode.len() {
                                if matches!(bytecode[func_end], Instr::Ret) {
                                    break;
                                }
                                func_end += 1;
                            }

                            // Decompile function body with function's params
                            let (body, _) = self.decompile_region(
                                bytecode,
                                func_start,
                                func_end,
                                get_symbol_name,
                                is_local,
                                func_labels,
                                Some(params),
                            );

                            // Format function declaration: name(param1, param2) = body
                            let fname = self.style.function(func_name);
                            let lparen = self.style.delimiter("(");
                            let rparen = self.style.delimiter(")");
                            let comma = self.style.delimiter(", ");
                            let eq = self.style.operator(" = ");

                            let param_names: Vec<String> = params
                                .iter()
                                .map(|p| self.style.local_symbol(p).to_string())
                                .collect();
                            let params_str = param_names.join(&comma.to_string());

                            declarations.push(format!(
                                "{}{}{}{}{}{}",
                                fname, lparen, params_str, rparen, eq, body
                            ));

                            // Jump past the function to continue
                            ip = *target;
                            continue;
                        }
                    }

                    // Otherwise, part of IF expression, already handled
                    ip += 1;
                }
                Instr::LoadParam(idx) => {
                    if let Some(params) = current_params {
                        if let Some(name) = params.get(*idx) {
                            let param = self.style.local_symbol(name);
                            stack.push(param.to_string());
                        } else {
                            // Fallback: use index
                            stack.push(format!("param_{}", idx));
                        }
                    } else {
                        // No params context, shouldn't happen
                        stack.push(format!("param_{}", idx));
                    }
                    ip += 1;
                }
                Instr::Ret => {
                    // End of function - this should be handled by the caller
                    // who set the end boundary
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

    /// Core assembly formatting logic shared between Compiled and Linked.
    fn format_assembly<F>(
        &self,
        bytecode: &[Instr],
        version: &str,
        get_symbol_name: &F,
        func_labels: Option<&HashMap<usize, String>>,
        func_params: Option<&HashMap<usize, Vec<String>>>,
    ) -> String
    where
        F: Fn(usize) -> String,
    {
        let mut out = String::new();

        // Version comment
        let comment = self.style.comment(&format!("; VERSION {}\n", version));
        out.push_str(&comment.to_string());

        // Track current function parameters
        let mut current_params: Option<&Vec<String>> = None;

        // Instructions
        for (i, instr) in bytecode.iter().enumerate() {
            // Check if this address has a function label
            if let Some(labels) = func_labels {
                if let Some(_label) = labels.get(&i) {
                    // Emit label line: "     add(a, b):"
                    let styled_label = self.style.asm_label(_label);
                    let colon = self.style.delimiter(":");
                    let _ = writeln!(out, "     {}{}", styled_label, colon);

                    // Update current function parameters
                    if let Some(params_map) = func_params {
                        current_params = params_map.get(&i);
                    }
                }
            }

            // Check if we're exiting a function (RET instruction)
            if matches!(instr, Instr::Ret) {
                current_params = None;
            }

            let addr = self.style.asm_address(&format!("{:04X} ", i));
            let _ = write!(out, "{}", addr);

            let line = self.format_instruction(instr, get_symbol_name, current_params);
            let _ = writeln!(out, "{}", line);
        }

        out
    }

    fn format_instruction<F>(
        &self,
        instr: &Instr,
        get_symbol_name: &F,
        current_params: Option<&Vec<String>>,
    ) -> String
    where
        F: Fn(usize) -> String,
    {
        match instr {
            Instr::Push(v) => {
                let instr = self.style.keyword("PUSH");
                let value = self.style.number(&v.to_string());
                format!("{} {}", instr, value)
            }
            Instr::Load(idx) => {
                let instr = self.style.keyword("LOAD");
                let sym_name = get_symbol_name(*idx);
                let sym = self.style.global_symbol(&sym_name);
                format!("{} {}", instr, sym)
            }
            Instr::Store(idx) => {
                let instr = self.style.keyword("STORE");
                let sym_name = get_symbol_name(*idx);
                let sym = self.style.local_symbol(&sym_name);
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
                let sym_name = get_symbol_name(*idx);
                let func = self.style.function(&sym_name);
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
            Instr::LoadParam(idx) => {
                let instr = self.style.keyword("LOAD_PARAM");
                let param = if let Some(params) = current_params {
                    if let Some(name) = params.get(*idx) {
                        self.style.local_symbol(name).to_string()
                    } else {
                        self.style.number(&idx.to_string()).to_string()
                    }
                } else {
                    self.style.number(&idx.to_string()).to_string()
                };
                format!("{} {}", instr, param)
            }
            Instr::Ret => {
                let instr = self.style.keyword("RET");
                format!("{}", instr)
            }
        }
    }
}

// ============================================================================
// Expression printing for Compiled programs
// ============================================================================

impl<'p> Print<'p, Compiled> {
    /// Build function metadata map from symbol metadata.
    fn build_func_metadata_compiled(&self) -> HashMap<usize, (String, Vec<String>)> {
        use crate::metadata::SymbolKind;

        let mut func_map = HashMap::new();
        let symbols = self.program.symbols();

        for meta in symbols {
            if let SymbolKind::LocalFunc {
                arity: _,
                params,
                offset,
            } = &meta.kind
            {
                let param_names: Vec<String> = params.iter().map(|p| p.to_string()).collect();
                func_map.insert(*offset, (meta.name.to_string(), param_names));
            }
        }

        func_map
    }

    /// Returns the pretty-printed expression as a string.
    fn get_expr(&self) -> String {
        let bytecode = self.program.bytecode();
        let symbols = self.program.symbols();
        let func_metadata = self.build_func_metadata_compiled();

        let (expr, _) = self.decompile_region(
            bytecode,
            0,
            bytecode.len(),
            &|idx| symbols[idx].name.to_string(),
            &|idx| symbols[idx].local,
            Some(&func_metadata),
            None,
        );
        expr
    }

    /// Returns the pretty-printed assembly as a string.
    pub fn assembly(&self) -> String {
        let bytecode = self.program.bytecode();
        let symbols = self.program.symbols();
        let version = self.program.version();

        self.format_assembly(
            bytecode,
            version,
            &|idx| symbols[idx].name.to_string(),
            None,
            None,
        )
    }
}

impl<'p> fmt::Display for Print<'p, Compiled> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get_expr())
    }
}

// ============================================================================
// Expression and assembly printing for Linked programs
// ============================================================================

impl<'p> Print<'p, Linked> {
    /// Build function metadata map from symbol table.
    fn build_func_metadata_linked(&self) -> HashMap<usize, (String, Vec<String>)> {
        use crate::symbol::Symbol;

        let mut func_map = HashMap::new();

        for symbol in self.program.symtable().symbols() {
            if let Symbol::LocalFunc {
                name,
                params,
                offset,
            } = symbol
            {
                let param_names: Vec<String> = params.iter().map(|p| p.to_string()).collect();
                func_map.insert(*offset, (name.to_string(), param_names));
            }
        }

        func_map
    }

    /// Returns the pretty-printed expression.
    pub fn get_expr(&self) -> String {
        let bytecode = self.program.bytecode();
        let func_metadata = self.build_func_metadata_linked();

        let (expr, _) = self.decompile_region(
            bytecode,
            0,
            bytecode.len(),
            &|idx| self.get_symbol_name(idx).to_string(),
            &|idx| self.is_local_symbol(idx),
            Some(&func_metadata),
            None,
        );
        expr
    }

    /// Build a map of instruction offsets to function labels.
    fn build_function_labels(&self) -> HashMap<usize, String> {
        use crate::symbol::Symbol;

        let mut labels = HashMap::new();

        for symbol in self.program.symtable().symbols() {
            if let Symbol::LocalFunc {
                name,
                params,
                offset,
            } = symbol
            {
                // Format: "add(a, b)"
                let param_list = params
                    .iter()
                    .map(|p| p.as_ref())
                    .collect::<Vec<_>>()
                    .join(", ");
                let label = format!("{}({})", name, param_list);
                labels.insert(*offset, label);
            }
        }

        labels
    }

    /// Build a map of instruction offsets to function parameter names.
    fn build_function_params(&self) -> HashMap<usize, Vec<String>> {
        use crate::symbol::Symbol;

        let mut params_map = HashMap::new();

        for symbol in self.program.symtable().symbols() {
            if let Symbol::LocalFunc { params, offset, .. } = symbol {
                let param_names: Vec<String> = params.iter().map(|p| p.to_string()).collect();
                params_map.insert(*offset, param_names);
            }
        }

        params_map
    }

    /// Returns the pretty-printed assembly as a string.
    pub fn assembly(&self) -> String {
        let bytecode = self.program.bytecode();
        let version = self.program.version();
        let func_labels = self.build_function_labels();
        let func_params = self.build_function_params();

        self.format_assembly(
            bytecode,
            version,
            &|idx| self.get_symbol_name(idx).to_string(),
            Some(&func_labels),
            Some(&func_params),
        )
    }

    fn get_symbol_name(&self, idx: usize) -> &str {
        self.program
            .symtable()
            .get_by_index(idx)
            .map(|s| s.name())
            .expect("Symbol not found")
    }

    fn is_local_symbol(&self, idx: usize) -> bool {
        if let Ok(symbol) = self.program.symtable().get_by_index(idx) {
            symbol.is_local()
        } else {
            false
        }
    }
}

impl<'p> fmt::Display for Print<'p, Linked> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.assembly())
    }
}
