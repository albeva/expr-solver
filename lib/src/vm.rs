use crate::ir::Instr;
use crate::symbol::{FuncError, SymTable, Symbol};
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use thiserror::Error;

#[cfg(test)]
use rust_decimal_macros::dec;

/// Virtual machine runtime errors.
#[derive(Error, Debug, Clone)]
pub enum VmError {
    #[error("Stack underflow: attempted to pop from empty stack")]
    StackUnderflow,
    #[error("Division by zero")]
    DivisionByZero,
    #[error("Invalid stack state at program end: expected 1 element, found {count}")]
    InvalidFinalStack { count: usize },
    #[error("Invalid load operation: cannot load '{symbol_name}' as a constant")]
    InvalidLoad { symbol_name: String },
    #[error("Invalid call operation: cannot call '{symbol_name}' as a function")]
    InvalidCall { symbol_name: String },
    #[error(
        "Stack underflow on function call '{function_name}': expected {expected} arguments, found {found}"
    )]
    CallStackUnderflow {
        function_name: String,
        expected: usize,
        found: usize,
    },
    #[error("Invalid factorial: {value} (must be a non-negative integer)")]
    InvalidFactorial { value: Decimal },
    #[error("Arithmetic error: {message}")]
    ArithmeticError { message: String },
    #[error("Function error: {0}")]
    FunctionError(FuncError),
    #[error("Invalid symbol index: {0}")]
    InvalidSymbolIndex(usize),
    #[error("Invalid jump: target {target} is out of bounds (program size: {size})")]
    InvalidJump { target: usize, size: usize },
}

/// Stack-based virtual machine for executing bytecode programs.
///
/// The VM evaluates programs by executing bytecode instructions on a stack,
/// performing arithmetic operations and function calls.
#[derive(Debug)]
pub struct Vm<'vm> {
    bytecode: &'vm [Instr],
    symtable: &'vm SymTable,
    stack: Vec<Decimal>,
    ip: usize,
}

impl<'vm> Vm<'vm> {
    /// Executes bytecode and returns the result.
    ///
    /// # Errors
    ///
    /// Returns [`VmError`] if execution fails due to:
    /// - Stack underflow
    /// - Division by zero
    /// - Invalid operations (e.g., factorial of non-integer)
    /// - Function errors
    /// - Invalid symbol indices
    /// - Invalid jumps
    pub fn run(bytecode: &'vm [Instr], symtable: &'vm SymTable) -> Result<Decimal, VmError> {
        if bytecode.is_empty() {
            return Ok(Decimal::ZERO);
        }

        let mut vm = Vm {
            bytecode,
            symtable,
            stack: Vec::new(),
            ip: 0,
        };

        vm.execute()?;

        match vm.stack.as_slice() {
            [result] => Ok(*result),
            _ => Err(VmError::InvalidFinalStack {
                count: vm.stack.len(),
            }),
        }
    }

    fn execute(&mut self) -> Result<(), VmError> {
        while self.ip < self.bytecode.len() {
            let op = &self.bytecode[self.ip];

            // Check for jump instructions and handle them specially
            match op {
                Instr::Jmp(target) => {
                    self.ip = *target;
                    continue;
                }
                Instr::Jz(target) => {
                    let cond = self.pop()?;
                    if cond == Decimal::ZERO {
                        self.ip = *target;
                        continue;
                    }
                }
                _ => {
                    self.execute_instruction(op)?;
                }
            }

            self.ip += 1;
        }
        Ok(())
    }

    fn execute_instruction(&mut self, op: &Instr) -> Result<(), VmError> {
        match op {
            Instr::Push(v) => {
                self.stack.push(*v);
                Ok(())
            }
            Instr::Load(idx) => {
                let sym = self
                    .symtable
                    .get_by_index(*idx)
                    .ok_or(VmError::InvalidSymbolIndex(*idx))?;
                match sym {
                    Symbol::Const { value, .. } => {
                        self.stack.push(*value);
                        Ok(())
                    }
                    _ => Err(VmError::InvalidLoad {
                        symbol_name: sym.name().to_string(),
                    }),
                }
            }
            Instr::Neg => {
                let v = self.pop()?;
                self.stack.push(-v);
                Ok(())
            }
            Instr::Add => self.add_op(),
            Instr::Sub => self.sub_op(),
            Instr::Mul => self.mul_op(),
            Instr::Div => self.div_op(),
            Instr::Pow => self.pow_op(),
            Instr::Fact => self.fact_op(),
            Instr::Call(idx, argc) => {
                let sym = self
                    .symtable
                    .get_by_index(*idx)
                    .ok_or(VmError::InvalidSymbolIndex(*idx))?;
                self.call_op(sym, *argc)
            }
            // Comparison operators
            Instr::Equal => self.comparison_op(|a, b| a == b),
            Instr::NotEqual => self.comparison_op(|a, b| a != b),
            Instr::Less => self.comparison_op(|a, b| a < b),
            Instr::LessEqual => self.comparison_op(|a, b| a <= b),
            Instr::Greater => self.comparison_op(|a, b| a > b),
            Instr::GreaterEqual => self.comparison_op(|a, b| a >= b),
            // Jump instructions are handled in execute()
            Instr::Jmp(_) | Instr::Jz(_) => {
                unreachable!("Jump instructions should be handled in execute")
            }
        }
    }

    fn comparison_op<F>(&mut self, f: F) -> Result<(), VmError>
    where
        F: FnOnce(Decimal, Decimal) -> bool,
    {
        let right = self.pop()?;
        let left = self.pop()?;
        let result = if f(left, right) {
            Decimal::ONE
        } else {
            Decimal::ZERO
        };
        self.stack.push(result);
        Ok(())
    }

    fn add_op(&mut self) -> Result<(), VmError> {
        let right = self.pop()?;
        let left = self.pop()?;
        let result = left
            .checked_add(right)
            .ok_or_else(|| VmError::ArithmeticError {
                message: format!("Addition overflow: {} + {}", left, right),
            })?;
        self.stack.push(result);
        Ok(())
    }

    fn sub_op(&mut self) -> Result<(), VmError> {
        let right = self.pop()?;
        let left = self.pop()?;
        let result = left
            .checked_sub(right)
            .ok_or_else(|| VmError::ArithmeticError {
                message: format!("Subtraction overflow: {} - {}", left, right),
            })?;
        self.stack.push(result);
        Ok(())
    }

    fn mul_op(&mut self) -> Result<(), VmError> {
        let right = self.pop()?;
        let left = self.pop()?;
        let result = left
            .checked_mul(right)
            .ok_or_else(|| VmError::ArithmeticError {
                message: format!("Multiplication overflow: {} * {}", left, right),
            })?;
        self.stack.push(result);
        Ok(())
    }

    fn div_op(&mut self) -> Result<(), VmError> {
        let right = self.pop()?;
        let left = self.pop()?;
        let result = left.checked_div(right).ok_or_else(|| {
            if right.is_zero() {
                VmError::DivisionByZero
            } else {
                VmError::ArithmeticError {
                    message: format!("Division overflow or underflow: {} / {}", left, right),
                }
            }
        })?;
        self.stack.push(result);
        Ok(())
    }

    fn pow_op(&mut self) -> Result<(), VmError> {
        let exponent = self.pop()?;
        let base = self.pop()?;

        // Use Decimal's powd with error handling
        let result =
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| base.powd(exponent))) {
                Ok(result) => result,
                Err(_) => {
                    return Err(VmError::ArithmeticError {
                        message: format!("Power operation failed: {} ^ {}", base, exponent),
                    });
                }
            };

        self.stack.push(result);
        Ok(())
    }

    fn fact_op(&mut self) -> Result<(), VmError> {
        let n = self.pop()?;

        // Check for negative numbers
        if n.is_sign_negative() {
            return Err(VmError::InvalidFactorial { value: n });
        }

        // Check for non-integer
        if n.fract() != Decimal::ZERO {
            return Err(VmError::InvalidFactorial { value: n });
        }

        // Calculate factorial using safe multiplication with iterator
        let n_u64 = n.to_u64().unwrap();
        let result = (1..=n_u64).try_fold(Decimal::ONE, |acc, i| {
            acc.checked_mul(Decimal::from(i))
                .ok_or_else(|| VmError::ArithmeticError {
                    message: format!("Factorial calculation overflow at {}!", i),
                })
        })?;

        self.stack.push(result);
        Ok(())
    }

    fn call_op(&mut self, sym: &Symbol, argc: usize) -> Result<(), VmError> {
        match sym {
            Symbol::Func {
                name,
                args: min_args,
                variadic,
                callback,
                ..
            } => {
                if argc != *min_args && (!*variadic || argc < *min_args) {
                    return Err(VmError::CallStackUnderflow {
                        function_name: name.to_string(),
                        expected: *min_args,
                        found: argc,
                    });
                }

                // Check if we have enough values on the stack
                if self.stack.len() < argc {
                    return Err(VmError::CallStackUnderflow {
                        function_name: name.to_string(),
                        expected: argc,
                        found: self.stack.len(),
                    });
                }

                let args_start = self.stack.len() - argc;
                let args = &self.stack[args_start..];
                let result = callback(args).map_err(VmError::FunctionError)?;
                self.stack.truncate(args_start);
                self.stack.push(result);
                Ok(())
            }
            Symbol::Const { .. } => Err(VmError::InvalidCall {
                symbol_name: sym.name().to_string(),
            }),
        }
    }

    fn pop(&mut self) -> Result<Decimal, VmError> {
        self.stack.pop().ok_or(VmError::StackUnderflow)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::symbol::SymTable;

    #[test]
    fn test_vm_error_stack_underflow() {
        let table = SymTable::stdlib();
        let bytecode = vec![Instr::Add]; // No values on stack

        let result = Vm::run(&bytecode, &table);
        assert!(matches!(result, Err(VmError::StackUnderflow)));
    }

    #[test]
    fn test_vm_error_division_by_zero() {
        let table = SymTable::stdlib();
        let bytecode = vec![Instr::Push(dec!(5)), Instr::Push(dec!(0)), Instr::Div];

        let result = Vm::run(&bytecode, &table);
        assert!(matches!(result, Err(VmError::DivisionByZero)));
    }

    #[test]
    fn test_vm_error_invalid_final_stack() {
        let table = SymTable::stdlib();
        let bytecode = vec![
            Instr::Push(dec!(1)),
            Instr::Push(dec!(2)),
            // No operation to combine them
        ];

        let result = Vm::run(&bytecode, &table);
        assert!(matches!(
            result,
            Err(VmError::InvalidFinalStack { count: 2 })
        ));
    }

    #[test]
    fn test_vm_error_invalid_load() {
        let table = SymTable::stdlib();
        let (sin_idx, _) = table.get_with_index("sin").unwrap();

        let bytecode = vec![Instr::Load(sin_idx)]; // Trying to load a function as constant

        let result = Vm::run(&bytecode, &table);
        assert!(matches!(
            result,
            Err(VmError::InvalidLoad { symbol_name: _ })
        ));
    }

    #[test]
    fn test_vm_error_invalid_call() {
        let table = SymTable::stdlib();
        let (pi_idx, _) = table.get_with_index("pi").unwrap();

        let bytecode = vec![Instr::Call(pi_idx, 0)]; // Trying to call a constant as function

        let result = Vm::run(&bytecode, &table);
        assert!(matches!(
            result,
            Err(VmError::InvalidCall { symbol_name: _ })
        ));
    }

    #[test]
    fn test_vm_error_call_stack_underflow() {
        let table = SymTable::stdlib();
        let (sin_idx, _) = table.get_with_index("sin").unwrap();

        let bytecode = vec![Instr::Call(sin_idx, 0)]; // No arguments for sin function

        let result = Vm::run(&bytecode, &table);
        assert!(matches!(
            result,
            Err(VmError::CallStackUnderflow {
                function_name: _,
                expected: _,
                found: _
            })
        ));
    }

    #[test]
    fn test_vm_error_display() {
        assert_eq!(
            VmError::StackUnderflow.to_string(),
            "Stack underflow: attempted to pop from empty stack"
        );
        assert_eq!(VmError::DivisionByZero.to_string(), "Division by zero");
        assert_eq!(
            VmError::InvalidFinalStack { count: 3 }.to_string(),
            "Invalid stack state at program end: expected 1 element, found 3"
        );
        assert_eq!(
            VmError::InvalidLoad {
                symbol_name: "test".to_string(),
            }
            .to_string(),
            "Invalid load operation: cannot load 'test' as a constant"
        );
        assert_eq!(
            VmError::InvalidCall {
                symbol_name: "test".to_string(),
            }
            .to_string(),
            "Invalid call operation: cannot call 'test' as a function"
        );
        assert_eq!(
            VmError::CallStackUnderflow {
                function_name: "sin".to_string(),
                expected: 1,
                found: 0
            }
            .to_string(),
            "Stack underflow on function call 'sin': expected 1 arguments, found 0"
        );
    }

    #[test]
    fn test_binary_operations() {
        let table = SymTable::stdlib();

        // Test all binary operations
        let test_cases = vec![
            (
                vec![Instr::Push(dec!(6)), Instr::Push(dec!(2)), Instr::Sub],
                dec!(4),
            ),
            (
                vec![Instr::Push(dec!(3)), Instr::Push(dec!(4)), Instr::Mul],
                dec!(12),
            ),
            (
                vec![Instr::Push(dec!(8)), Instr::Push(dec!(2)), Instr::Div],
                dec!(4),
            ),
        ];

        for (code, expected) in test_cases {
            assert_eq!(Vm::run(&code, &table).unwrap(), expected);
        }
    }
}
