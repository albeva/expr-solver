use clap::{ArgAction, Parser};
use expr_solver::{Number, ParseNumber, Program, SymTable, Symbol};
use std::path::PathBuf;

/// A mathematical expression evaluator with compilation support
#[derive(Parser)]
#[command(version, about, long_about = None, arg_required_else_help = true)]
struct Args {
    /// Expression to evaluate (can be provided as positional argument or with -e/--expression)
    #[arg(short, long, conflicts_with = "input")]
    expression: Option<String>,

    /// Expression to evaluate (positional, alternative to -e/--expression)
    #[arg(conflicts_with_all = ["expression", "input"])]
    expr: Option<String>,

    /// Read compiled expression from binary file
    #[arg(short, long, conflicts_with_all = ["expression", "expr"])]
    input: Option<PathBuf>,

    /// Save compiled expression to binary file
    #[arg(short, long, conflicts_with_all = ["assembly", "symbol_table", "print"])]
    output: Option<PathBuf>,

    /// Define constants (e.g., -D x=5.0)
    #[arg(short = 'D', long, value_parser = parse_key_val, action = ArgAction::Append)]
    define: Vec<(String, Number)>,

    /// List all available functions and constants
    #[arg(short = 't', long, conflicts_with_all=["output", "assembly", "print"])]
    symbol_table: bool,

    /// Print the assembly code
    #[arg(short = 'a', long, conflicts_with_all=["symbol_table", "output", "print"])]
    assembly: bool,

    /// Print syntax-highlighted expression
    #[arg(short = 'p', long, conflicts_with_all=["symbol_table", "output", "assembly"])]
    print: bool,
}

/// Parses a key=value pair for custom constant definitions.
fn parse_key_val(s: &str) -> Result<(String, Number), Box<dyn std::error::Error + Send + Sync>> {
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{s}`"))?;
    let key = s[..pos].parse()?;
    let value = Number::parse_number(&s[pos + 1..])
        .map_err(|e| format!("Failed to parse number: {}", e))?;
    Ok((key, value))
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
    }
}

/// Main execution logic: parses arguments, loads program, and executes requested action.
fn run() -> Result<(), String> {
    let args = Args::parse();

    // Create symbol table with custom constants
    let table = create_symbol_table(&args.define)?;

    // Handle --symbol-table (no program needed)
    if args.symbol_table {
        list_symbol_table(&table);
        return Ok(());
    }

    // Load input from either expression or file
    let program = if let Some(expr) = args.expression.as_ref().or(args.expr.as_ref()) {
        Program::new_from_source(expr).map_err(|err| err.to_string())?
    } else if let Some(file) = &args.input {
        Program::new_from_file(file.to_string_lossy().as_ref()).map_err(|err| err.to_string())?
    } else {
        return Err("no input".to_string());
    };

    // Link the program with the symbol table
    #[allow(unused_mut)] // Mut needed for execute(), but not for assembly printing
    let mut program = program.link(table).map_err(|err| err.to_string())?;

    // Act on the loaded program
    if args.print {
        println!("{}", program.get_string());
    } else if args.assembly {
        print!("{}", program.get_assembly());
    } else if let Some(output_path) = &args.output {
        program
            .save_bytecode_to_file(output_path)
            .map_err(|e| e.to_string())?;
    } else {
        let res = program.execute().map_err(|e| e.to_string())?;
        println!("{res}");
    }

    Ok(())
}

/// Creates a symbol table with standard library and user-defined constants.
fn create_symbol_table(defines: &[(String, Number)]) -> Result<SymTable, String> {
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
            .add_const(name.clone(), *value, false)
            .map_err(|e| format!("Failed to add constant '{}': {}", name, e))?;
    }

    Ok(table)
}

/// Prints all available constants and functions in the symbol table.
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
