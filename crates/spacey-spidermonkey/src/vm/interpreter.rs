//! The bytecode interpreter.

use crate::compiler::{Bytecode, OpCode, Operand};
use crate::runtime::value::Value;
use crate::Error;

/// The virtual machine that executes bytecode.
pub struct VM {
    /// The value stack
    stack: Vec<Value>,
    /// Instruction pointer
    ip: usize,
}

impl VM {
    /// Creates a new VM.
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(256),
            ip: 0,
        }
    }

    /// Executes bytecode and returns the result.
    pub fn execute(&mut self, bytecode: &Bytecode) -> Result<Value, Error> {
        self.ip = 0;
        self.stack.clear();

        loop {
            if self.ip >= bytecode.instructions.len() {
                break;
            }

            let instruction = &bytecode.instructions[self.ip];
            self.ip += 1;

            match instruction.opcode {
                OpCode::Halt => break,

                OpCode::LoadConst => {
                    if let Some(Operand::Constant(idx)) = &instruction.operand {
                        let value = bytecode.constants[*idx as usize].clone();
                        self.stack.push(value);
                    }
                }

                OpCode::LoadUndefined => self.stack.push(Value::Undefined),
                OpCode::LoadNull => self.stack.push(Value::Null),
                OpCode::LoadTrue => self.stack.push(Value::Boolean(true)),
                OpCode::LoadFalse => self.stack.push(Value::Boolean(false)),

                OpCode::Pop => {
                    self.stack.pop();
                }

                OpCode::Dup => {
                    if let Some(value) = self.stack.last().cloned() {
                        self.stack.push(value);
                    }
                }

                // Arithmetic
                OpCode::Add => self.binary_op(|a, b| a + b)?,
                OpCode::Sub => self.binary_op(|a, b| a - b)?,
                OpCode::Mul => self.binary_op(|a, b| a * b)?,
                OpCode::Div => self.binary_op(|a, b| a / b)?,
                OpCode::Mod => self.binary_op(|a, b| a % b)?,

                OpCode::Neg => {
                    if let Some(Value::Number(n)) = self.stack.pop() {
                        self.stack.push(Value::Number(-n));
                    }
                }

                // Comparison
                OpCode::Lt => self.compare_op(|a, b| a < b)?,
                OpCode::Le => self.compare_op(|a, b| a <= b)?,
                OpCode::Gt => self.compare_op(|a, b| a > b)?,
                OpCode::Ge => self.compare_op(|a, b| a >= b)?,

                OpCode::Eq | OpCode::StrictEq => {
                    let b = self.stack.pop().ok_or(Error::InternalError("Stack underflow".into()))?;
                    let a = self.stack.pop().ok_or(Error::InternalError("Stack underflow".into()))?;
                    self.stack.push(Value::Boolean(a == b));
                }

                OpCode::Ne | OpCode::StrictNe => {
                    let b = self.stack.pop().ok_or(Error::InternalError("Stack underflow".into()))?;
                    let a = self.stack.pop().ok_or(Error::InternalError("Stack underflow".into()))?;
                    self.stack.push(Value::Boolean(a != b));
                }

                OpCode::Not => {
                    if let Some(value) = self.stack.pop() {
                        self.stack.push(Value::Boolean(!value.to_boolean()));
                    }
                }

                OpCode::Return => {
                    return self.stack.pop().ok_or(Error::InternalError("Stack underflow".into()));
                }

                _ => {
                    // TODO: Implement remaining opcodes
                }
            }
        }

        self.stack.pop().ok_or(Error::InternalError("No result".into()))
    }

    fn binary_op<F>(&mut self, op: F) -> Result<(), Error>
    where
        F: Fn(f64, f64) -> f64,
    {
        let b = self.stack.pop().ok_or(Error::InternalError("Stack underflow".into()))?;
        let a = self.stack.pop().ok_or(Error::InternalError("Stack underflow".into()))?;

        match (a, b) {
            (Value::Number(a), Value::Number(b)) => {
                self.stack.push(Value::Number(op(a, b)));
            }
            _ => {
                return Err(Error::TypeError("Expected numbers".into()));
            }
        }

        Ok(())
    }

    fn compare_op<F>(&mut self, op: F) -> Result<(), Error>
    where
        F: Fn(f64, f64) -> bool,
    {
        let b = self.stack.pop().ok_or(Error::InternalError("Stack underflow".into()))?;
        let a = self.stack.pop().ok_or(Error::InternalError("Stack underflow".into()))?;

        match (a, b) {
            (Value::Number(a), Value::Number(b)) => {
                self.stack.push(Value::Boolean(op(a, b)));
            }
            _ => {
                return Err(Error::TypeError("Expected numbers".into()));
            }
        }

        Ok(())
    }
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}


