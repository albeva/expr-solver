use clap::{ArgAction, Parser};
use expr_solver::{Eval, SymTable, Symbol};
use rust_decimal::prelude::*;
use std::path::PathBuf;

/// A mathematical expression evaluator with compilation support
#[derive(Parser)]
#[command(version, about, long_about = None, arg_required_else_help = true)]
struct Args {
    /// Expression to evaluate
    #[arg(short, long, conflicts_with_all = ["input", "expr"])]
    expression: Option<String>,

    /// Expression to evaluate (positional)
    #[arg(conflicts_with_all = ["expression", "input", "symbol_table"])]
    expr: Option<String>,

    /// Read compiled expression from binary file
    #[arg(short, long, conflicts_with_all = ["expression", "expr"])]
    input: Option<PathBuf>,

    /// Save compiled expression to binary file
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Define constants (e.g., -D x=5.0)
    #[arg(short = 'D', long, value_parser = parse_key_val, action = ArgAction::Append)]
    define: Vec<(String, f64)>,

    /// List all available functions and constants
    #[arg(short = 't', long)]
    symbol_table: bool,

    /// Print the assembly code
    #[arg(short = 'a', long, conflicts_with_all=["symbol_table", "output"])]
    assembly: bool,
}

fn parse_key_val(s: &str) -> Result<(String, f64), Box<dyn std::error::Error + Send + Sync>> {
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{s}`"))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}

fn main() {
    match run() {
        Err(err) => {
            eprintln!("{err}");
        }
        _ => {}
    }
}

fn run() -> Result<(), String> {
    let args = Args::parse();

    // Create symbol table with custom constants
    let table = create_symbol_table(&args.define)?;

    // Handle --symbol-table
    if args.symbol_table {
        list_symbol_table(&table);
        return Ok(());
    }

    // load either from string input or a file
    let mut eval = if let Some(expr) = args.expression.as_ref().or(args.expr.as_ref()) {
        Eval::new(expr)
    } else if let Some(input) = &args.input {
        Ok(Eval::new_from_file(input.clone()))
    } else {
        return Err("no input".to_string());
    }?;

    eval.with_table(table);

    if args.assembly {
        let program = eval.build_program()?;
        print!("{}", program.get_assembly());
        return Ok(());
    }

    // save to a file?
    if let Some(output_path) = &args.output {
        eval.compile_to_file(output_path)?
    } else {
        let res = eval.run()?;
        println!("{res}");
    }

    Ok(())
}

fn create_symbol_table(defines: &[(String, f64)]) -> Result<SymTable, String> {
    let mut table = SymTable::stdlib();

    for (name, value) in defines {
        // Validate name (simple identifier check)
        if !name.chars().all(|c| c.is_alphanumeric() || c == '_') || name.is_empty() {
            return Err(format!(
                "Invalid constant name '{}'. Use alphanumeric and underscore only",
                name
            ));
        }

        table
            .add_const(
                name.clone(),
                Decimal::from_f64(*value).unwrap_or(Decimal::ZERO),
            )
            .map_err(|e| format!("Failed to add constant '{}': {}", name, e))?;
    }

    Ok(table)
}

fn list_symbol_table(table: &SymTable) {
    println!("Available constants:");
    for symbol in table.symbols() {
        if let Symbol::Const {
            name, description, ..
        } = symbol
        {
            let desc = description
                .as_ref()
                .map_or("No description", |c| c.as_ref());
            println!("  {:<12} - {}", name, desc);
        }
    }

    println!("\nAvailable functions:");
    for symbol in table.symbols() {
        if let Symbol::Func {
            name,
            args,
            description,
            variadic,
            ..
        } = symbol
        {
            let signature = match args {
                1 => format!("{}(x)", name),
                2 => format!("{}(x,y)", name),
                3 => format!("{}(x,y,z)", name),
                n if !*variadic => format!("{}({} args)", name, n),
                _ => format!("{}(x,...)", name),
            };
            let desc = description
                .as_ref()
                .map_or("No description", |c| c.as_ref());
            println!("  {:<16} - {}", signature, desc);
        }
    }

    println!("\nNote: All function and constant names are case-insensitive.");
}
