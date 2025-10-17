use crate::ir::Instr;
use crate::program::Program;
use crate::symbol::{FuncError, Symbol};
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use std::borrow::Cow;
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
    InvalidLoad { symbol_name: Cow<'static, str> },
    #[error("Invalid call operation: cannot call '{symbol_name}' as a function")]
    InvalidCall { symbol_name: Cow<'static, str> },
    #[error(
        "Stack underflow on function call '{function_name}': expected {expected} arguments, found {found}"
    )]
    CallStackUnderflow {
        function_name: Cow<'static, str>,
        expected: usize,
        found: usize,
    },
    #[error("Invalid factorial: {value} (must be a non-negative integer)")]
    InvalidFactorial { value: Decimal },
    #[error("Arithmetic error: {message}")]
    ArithmeticError { message: String },
    #[error("Function error: {0}")]
    FunctionError(FuncError),
}

/// Stack-based virtual machine for executing bytecode programs.
///
/// The VM evaluates programs by executing bytecode instructions on a stack,
/// performing arithmetic operations and function calls.
#[derive(Debug, Default)]
pub struct Vm;

impl Vm {
    /// Executes a program and returns the result.
    ///
    /// # Errors
    ///
    /// Returns [`VmError`] if execution fails due to:
    /// - Stack underflow
    /// - Division by zero
    /// - Invalid operations (e.g., factorial of non-integer)
    /// - Function errors
    pub fn run(&self, prog: &Program) -> Result<Decimal, VmError> {
        if prog.code.is_empty() {
            return Ok(Decimal::ZERO);
        }

        let mut stack: Vec<Decimal> = Vec::new();

        for op in &prog.code {
            self.execute_instruction(op, &mut stack)?;
        }

        match stack.as_slice() {
            [result] => Ok(*result),
            _ => Err(VmError::InvalidFinalStack { count: stack.len() }),
        }
    }

    fn execute_instruction(&self, op: &Instr, stack: &mut Vec<Decimal>) -> Result<(), VmError> {
        match op {
            Instr::Push(v) => {
                stack.push(*v);
                Ok(())
            }
            Instr::Load(sym) => match sym {
                Symbol::Const { name: _, value, .. } => {
                    stack.push(*value);
                    Ok(())
                }
                _ => Err(VmError::InvalidLoad {
                    symbol_name: Cow::Owned(sym.name().to_string()),
                }),
            },
            Instr::Neg => {
                let v = Self::pop(stack)?;
                stack.push(-v);
                Ok(())
            }
            Instr::Add => self.add_op(stack),
            Instr::Sub => self.sub_op(stack),
            Instr::Mul => self.mul_op(stack),
            Instr::Div => self.div_op(stack),
            Instr::Pow => self.pow_op(stack),
            Instr::Fact => self.fact_op(stack),
            Instr::Call(sym, argc) => self.call_op(sym, *argc, stack),
            // Comparison operators
            Instr::Equal => self.comparison_op(stack, |a, b| a == b),
            Instr::NotEqual => self.comparison_op(stack, |a, b| a != b),
            Instr::Less => self.comparison_op(stack, |a, b| a < b),
            Instr::LessEqual => self.comparison_op(stack, |a, b| a <= b),
            Instr::Greater => self.comparison_op(stack, |a, b| a > b),
            Instr::GreaterEqual => self.comparison_op(stack, |a, b| a >= b),
        }
    }

    fn comparison_op<F>(&self, stack: &mut Vec<Decimal>, f: F) -> Result<(), VmError>
    where
        F: FnOnce(Decimal, Decimal) -> bool,
    {
        let right = Self::pop(stack)?;
        let left = Self::pop(stack)?;
        let result = if f(left, right) {
            Decimal::ONE
        } else {
            Decimal::ZERO
        };
        stack.push(result);
        Ok(())
    }

    fn add_op(&self, stack: &mut Vec<Decimal>) -> Result<(), VmError> {
        let right = Self::pop(stack)?;
        let left = Self::pop(stack)?;
        let result = left
            .checked_add(right)
            .ok_or_else(|| VmError::ArithmeticError {
                message: format!("Addition overflow: {} + {}", left, right),
            })?;
        stack.push(result);
        Ok(())
    }

    fn sub_op(&self, stack: &mut Vec<Decimal>) -> Result<(), VmError> {
        let right = Self::pop(stack)?;
        let left = Self::pop(stack)?;
        let result = left
            .checked_sub(right)
            .ok_or_else(|| VmError::ArithmeticError {
                message: format!("Subtraction overflow: {} - {}", left, right),
            })?;
        stack.push(result);
        Ok(())
    }

    fn mul_op(&self, stack: &mut Vec<Decimal>) -> Result<(), VmError> {
        let right = Self::pop(stack)?;
        let left = Self::pop(stack)?;
        let result = left
            .checked_mul(right)
            .ok_or_else(|| VmError::ArithmeticError {
                message: format!("Multiplication overflow: {} * {}", left, right),
            })?;
        stack.push(result);
        Ok(())
    }

    fn div_op(&self, stack: &mut Vec<Decimal>) -> Result<(), VmError> {
        let right = Self::pop(stack)?;
        let left = Self::pop(stack)?;
        let result = left.checked_div(right).ok_or_else(|| {
            if right.is_zero() {
                VmError::DivisionByZero
            } else {
                VmError::ArithmeticError {
                    message: format!("Division overflow or underflow: {} / {}", left, right),
                }
            }
        })?;
        stack.push(result);
        Ok(())
    }

    fn pow_op(&self, stack: &mut Vec<Decimal>) -> Result<(), VmError> {
        let exponent = Self::pop(stack)?;
        let base = Self::pop(stack)?;

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

        stack.push(result);
        Ok(())
    }

    fn fact_op(&self, stack: &mut Vec<Decimal>) -> Result<(), VmError> {
        let n = Self::pop(stack)?;

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

        stack.push(result);
        Ok(())
    }

    fn call_op(&self, sym: &Symbol, argc: usize, stack: &mut Vec<Decimal>) -> Result<(), VmError> {
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
                        function_name: name.clone(),
                        expected: *min_args,
                        found: argc,
                    });
                }

                // Check if we have enough values on the stack
                if stack.len() < argc {
                    return Err(VmError::CallStackUnderflow {
                        function_name: name.clone(),
                        expected: argc,
                        found: stack.len(),
                    });
                }

                let args_start = stack.len() - argc;
                let args = &stack[args_start..];
                let result = callback(args).map_err(VmError::FunctionError)?;
                stack.truncate(args_start);
                stack.push(result);
                Ok(())
            }
            Symbol::Const { .. } => Err(VmError::InvalidCall {
                symbol_name: Cow::Owned(sym.name().to_string()),
            }),
        }
    }

    fn pop(stack: &mut Vec<Decimal>) -> Result<Decimal, VmError> {
        stack.pop().ok_or(VmError::StackUnderflow)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::symbol::SymTable;
    use std::borrow::Cow;

    fn make(code: Vec<Instr>) -> Program {
        let mut program = Program::new();
        program.code = code;
        program
    }

    #[test]
    fn test_vm_error_stack_underflow() {
        let vm = Vm::default();
        let program = make(
            vec![Instr::Add], // No values on stack
        );

        let result = vm.run(&program);
        assert!(matches!(result, Err(VmError::StackUnderflow)));
    }

    #[test]
    fn test_vm_error_division_by_zero() {
        let vm = Vm::default();
        let program = make(vec![Instr::Push(dec!(5)), Instr::Push(dec!(0)), Instr::Div]);

        let result = vm.run(&program);
        assert!(matches!(result, Err(VmError::DivisionByZero)));
    }

    #[test]
    fn test_vm_error_invalid_final_stack() {
        let vm = Vm::default();
        let program = make(vec![
            Instr::Push(dec!(1)),
            Instr::Push(dec!(2)),
            // No operation to combine them
        ]);

        let result = vm.run(&program);
        assert!(matches!(
            result,
            Err(VmError::InvalidFinalStack { count: 2 })
        ));
    }

    #[test]
    fn test_vm_error_invalid_load() {
        let vm = Vm::default();
        let table = SymTable::stdlib();
        let sin_func = table.get("sin").unwrap();

        let program = make(
            vec![Instr::Load(sin_func)], // Trying to load a function as constant
        );

        let result = vm.run(&program);
        assert!(matches!(
            result,
            Err(VmError::InvalidLoad { symbol_name: _ })
        ));
    }

    #[test]
    fn test_vm_error_invalid_call() {
        let vm = Vm::default();
        let table = SymTable::stdlib();
        let pi_const = table.get("pi").unwrap();

        let program = make(
            vec![Instr::Call(pi_const, 0)], // Trying to call a constant as function
        );

        let result = vm.run(&program);
        assert!(matches!(
            result,
            Err(VmError::InvalidCall { symbol_name: _ })
        ));
    }

    #[test]
    fn test_vm_error_call_stack_underflow() {
        let vm = Vm::default();
        let table = SymTable::stdlib();
        let sin_func = table.get("sin").unwrap();

        let program = make(
            vec![Instr::Call(sin_func, 0)], // No arguments for sin function
        );

        let result = vm.run(&program);
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
                symbol_name: Cow::Borrowed("test"),
            }
            .to_string(),
            "Invalid load operation: cannot load 'test' as a constant"
        );
        assert_eq!(
            VmError::InvalidCall {
                symbol_name: Cow::Borrowed("test"),
            }
            .to_string(),
            "Invalid call operation: cannot call 'test' as a function"
        );
        assert_eq!(
            VmError::CallStackUnderflow {
                function_name: Cow::Borrowed("sin"),
                expected: 1,
                found: 0
            }
            .to_string(),
            "Stack underflow on function call 'sin': expected 1 arguments, found 0"
        );
    }

    #[test]
    fn test_binary_operations() {
        let vm = Vm::default();

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
            let program = make(code);
            assert_eq!(vm.run(&program).unwrap(), expected);
        }
    }
}
