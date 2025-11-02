use crate::ir::Instr;
use crate::number::Number;
use crate::symbol::{FuncError, Symbol};
use crate::symtable::SymTable;
use thiserror::Error;

/// Virtual machine runtime errors.
#[derive(Error, Debug, Clone)]
pub enum VmError {
    #[error("Stack underflow: attempted to pop from empty stack")]
    StackUnderflow,
    #[error("Division by zero")]
    DivisionByZero,
    #[error("Invalid stack state at program end: expected 1 element, found {count}")]
    InvalidFinalStack { count: usize },
    #[error("Invalid factorial: {value} (must be a non-negative integer)")]
    InvalidFactorial { value: Number },
    #[error("Arithmetic error: {message}")]
    ArithmeticError { message: String },
    #[error("Function error: {0}")]
    FunctionError(FuncError),
}

/// Stack-based virtual machine for executing bytecode programs.
///
/// The VM evaluates programs by executing bytecode instructions on a stack,
/// performing arithmetic operations and function calls.
///
/// ## Error Handling
///
/// Behavior varies based on the numeric backend:
///
/// - **f64 mode**: Relaxed error handling. Only catches errors that would cause panics.
///   Allows `Inf` and `NaN` results from operations like `1/0` or `sqrt(-1)`.
///
/// - **Decimal mode**: Strict error handling. All arithmetic operations are checked
///   for overflow/underflow. Returns errors for domain violations.
#[derive(Debug)]
pub struct Vm<'vm> {
    bytecode: &'vm [Instr],
    symtable: &'vm mut SymTable,
    stack: Vec<Number>,
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
    pub fn run(bytecode: &'vm [Instr], symtable: &'vm mut SymTable) -> Result<Number, VmError> {
        use crate::number::consts;

        if bytecode.is_empty() {
            return Ok(consts::ZERO);
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
        use crate::number::consts;

        while self.ip < self.bytecode.len() {
            let op = &self.bytecode[self.ip];

            match op {
                Instr::Jmp(target) => {
                    self.ip = *target;
                    continue;
                }
                Instr::Jz(target) => {
                    let cond = self.pop()?;
                    if cond == consts::ZERO {
                        self.ip = *target;
                        continue;
                    }
                }
                Instr::Push(v) => {
                    self.stack.push(*v);
                }
                Instr::Load(idx) => {
                    let sym = self.symtable.get_by_index(*idx).unwrap();
                    match sym {
                        Symbol::Const { value, .. } => {
                            self.stack.push(*value);
                        }
                        _ => unreachable!(),
                    }
                }
                Instr::Store(idx) => {
                    let top = self.pop()?;
                    let sym = self.symtable.get_mut_by_index(*idx).unwrap();
                    match sym {
                        Symbol::Const { value, .. } => {
                            *value = top;
                        }
                        _ => unreachable!(),
                    }
                }
                Instr::Neg => {
                    let v = self.pop()?;
                    self.stack.push(-v);
                }
                Instr::Add => self.add_op()?,
                Instr::Sub => self.sub_op()?,
                Instr::Mul => self.mul_op()?,
                Instr::Div => self.div_op()?,
                Instr::Pow => self.pow_op()?,
                Instr::Fact => self.fact_op()?,
                Instr::Call(idx, argc) => self.call_op(*idx, *argc)?,
                Instr::Equal => self.comparison_op(|a, b| a == b)?,
                Instr::NotEqual => self.comparison_op(|a, b| a != b)?,
                Instr::Less => self.comparison_op(|a, b| a < b)?,
                Instr::LessEqual => self.comparison_op(|a, b| a <= b)?,
                Instr::Greater => self.comparison_op(|a, b| a > b)?,
                Instr::GreaterEqual => self.comparison_op(|a, b| a >= b)?,
                Instr::LoadParam(_) => {
                    unimplemented!()
                }
                Instr::Ret => {
                    unimplemented!()
                }
            }

            self.ip += 1;
        }
        Ok(())
    }

    fn comparison_op<F>(&mut self, f: F) -> Result<(), VmError>
    where
        F: FnOnce(Number, Number) -> bool,
    {
        use crate::number::consts;

        let right = self.pop()?;
        let left = self.pop()?;
        let result = if f(left, right) {
            consts::ONE
        } else {
            consts::ZERO
        };
        self.stack.push(result);
        Ok(())
    }

    // Decimal mode: use checked arithmetic for safety
    #[cfg(feature = "decimal")]
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

    #[cfg(not(feature = "decimal"))]
    fn add_op(&mut self) -> Result<(), VmError> {
        let right = self.pop()?;
        let left = self.pop()?;
        self.stack.push(left + right);
        Ok(())
    }

    #[cfg(feature = "decimal")]
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

    #[cfg(not(feature = "decimal"))]
    fn sub_op(&mut self) -> Result<(), VmError> {
        let right = self.pop()?;
        let left = self.pop()?;
        self.stack.push(left - right);
        Ok(())
    }

    #[cfg(feature = "decimal")]
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

    #[cfg(not(feature = "decimal"))]
    fn mul_op(&mut self) -> Result<(), VmError> {
        let right = self.pop()?;
        let left = self.pop()?;
        self.stack.push(left * right);
        Ok(())
    }

    #[cfg(feature = "decimal")]
    fn div_op(&mut self) -> Result<(), VmError> {
        use crate::number::consts;

        let right = self.pop()?;
        let left = self.pop()?;
        let result = left.checked_div(right).ok_or_else(|| {
            if right == consts::ZERO {
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

    #[cfg(not(feature = "decimal"))]
    fn div_op(&mut self) -> Result<(), VmError> {
        let right = self.pop()?;
        let left = self.pop()?;
        // Check for division by zero to prevent undefined behavior
        if right == 0.0 {
            return Err(VmError::DivisionByZero);
        }
        self.stack.push(left / right);
        Ok(())
    }

    #[cfg(feature = "decimal")]
    fn pow_op(&mut self) -> Result<(), VmError> {
        use rust_decimal::prelude::{FromPrimitive, ToPrimitive};

        let exponent = self.pop()?;
        let base = self.pop()?;

        // Convert to f64, compute power, convert back
        // This is a limitation of rust_decimal which doesn't have decimal exponent support
        let base_f64 = base.to_f64().ok_or_else(|| VmError::ArithmeticError {
            message: format!("Failed to convert base {} to f64", base),
        })?;
        let exp_f64 = exponent.to_f64().ok_or_else(|| VmError::ArithmeticError {
            message: format!("Failed to convert exponent {} to f64", exponent),
        })?;

        let result_f64 = base_f64.powf(exp_f64);

        let result = Number::from_f64(result_f64).ok_or_else(|| VmError::ArithmeticError {
            message: format!(
                "Power operation result cannot be represented: {} ^ {}",
                base, exponent
            ),
        })?;

        self.stack.push(result);
        Ok(())
    }

    #[cfg(not(feature = "decimal"))]
    fn pow_op(&mut self) -> Result<(), VmError> {
        let exponent = self.pop()?;
        let base = self.pop()?;
        self.stack.push(base.powf(exponent));
        Ok(())
    }

    #[cfg(feature = "decimal")]
    fn fact_op(&mut self) -> Result<(), VmError> {
        use crate::number::consts;
        use rust_decimal::prelude::*;

        let n = self.pop()?;

        // Check for negative numbers
        if n.is_sign_negative() {
            return Err(VmError::InvalidFactorial { value: n });
        }

        // Check for non-integer
        if n.fract() != consts::ZERO {
            return Err(VmError::InvalidFactorial { value: n });
        }

        // Calculate factorial using safe multiplication with iterator
        let n_u64 = n.to_u64().unwrap();
        let result = (1..=n_u64).try_fold(consts::ONE, |acc, i| {
            acc.checked_mul(Number::from(i))
                .ok_or_else(|| VmError::ArithmeticError {
                    message: format!("Factorial calculation overflow at {}!", i),
                })
        })?;

        self.stack.push(result);
        Ok(())
    }

    #[cfg(not(feature = "decimal"))]
    fn fact_op(&mut self) -> Result<(), VmError> {
        let n = self.pop()?;

        // Check for negative numbers
        if n < 0.0 {
            return Err(VmError::InvalidFactorial { value: n });
        }

        // Check for non-integer
        if n.fract() != 0.0 {
            return Err(VmError::InvalidFactorial { value: n });
        }

        // Calculate factorial
        let n_u64 = n as u64;
        let mut result = 1.0;
        for i in 1..=n_u64 {
            result *= i as f64;
        }

        self.stack.push(result);
        Ok(())
    }

    fn call_op(&mut self, idx: usize, argc: usize) -> Result<(), VmError> {
        match self.symtable.get_by_index(idx).unwrap() {
            Symbol::Func { callback, .. } => {
                let args_start = self.stack.len() - argc;
                let args = &self.stack[args_start..];
                let result = callback(args).map_err(VmError::FunctionError)?;
                self.stack.truncate(args_start);
                self.stack.push(result);
                Ok(())
            }
            Symbol::Const { .. } => unreachable!(),
            Symbol::LocalFunc { .. } => unimplemented!(),
        }
    }

    fn pop(&mut self) -> Result<Number, VmError> {
        self.stack.pop().ok_or(VmError::StackUnderflow)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::num;
    use crate::symtable::SymTable;

    #[test]
    fn test_vm_error_stack_underflow() {
        let mut table = SymTable::stdlib();
        let bytecode = vec![Instr::Add]; // No values on stack

        let result = Vm::run(&bytecode, &mut table);
        assert!(matches!(result, Err(VmError::StackUnderflow)));
    }

    #[test]
    fn test_vm_error_division_by_zero() {
        let mut table = SymTable::stdlib();
        let bytecode = vec![Instr::Push(num!(5)), Instr::Push(num!(0)), Instr::Div];

        let result = Vm::run(&bytecode, &mut table);
        assert!(matches!(result, Err(VmError::DivisionByZero)));
    }

    #[test]
    fn test_vm_error_invalid_final_stack() {
        let mut table = SymTable::stdlib();
        let bytecode = vec![
            Instr::Push(num!(1)),
            Instr::Push(num!(2)),
            // No operation to combine them
        ];

        let result = Vm::run(&bytecode, &mut table);
        assert!(matches!(
            result,
            Err(VmError::InvalidFinalStack { count: 2 })
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
    }

    #[test]
    fn test_binary_operations() {
        let mut table = SymTable::stdlib();

        // Test all binary operations
        let test_cases = vec![
            (
                vec![Instr::Push(num!(6)), Instr::Push(num!(2)), Instr::Sub],
                num!(4),
            ),
            (
                vec![Instr::Push(num!(3)), Instr::Push(num!(4)), Instr::Mul],
                num!(12),
            ),
            (
                vec![Instr::Push(num!(8)), Instr::Push(num!(2)), Instr::Div],
                num!(4),
            ),
        ];

        for (code, expected) in test_cases {
            let result = Vm::run(&code, &mut table).unwrap();
            assert_eq!(result, expected);
        }
    }
}
