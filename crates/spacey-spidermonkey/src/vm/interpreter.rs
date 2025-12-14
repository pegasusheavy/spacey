//! The bytecode interpreter.

use std::collections::HashMap;
use std::rc::Rc;

use crate::Error;
use crate::compiler::{Bytecode, OpCode, Operand};
use crate::runtime::function::{CallFrame, Callable, Function};
use crate::runtime::value::Value;

/// A saved call frame for restoring after return.
struct SavedFrame {
    /// Saved instruction pointer
    ip: usize,
    /// Saved bytecode reference
    bytecode_idx: usize,
    /// Saved locals base
    locals_base: usize,
}

/// The virtual machine that executes bytecode.
pub struct VM {
    /// The value stack
    stack: Vec<Value>,
    /// Local variables
    locals: Vec<Value>,
    /// Global variables
    globals: HashMap<String, Value>,
    /// Instruction pointer
    ip: usize,
    /// Call stack for function calls
    call_stack: Vec<SavedFrame>,
    /// Native functions
    native_functions: HashMap<String, Rc<Callable>>,
}

impl VM {
    /// Creates a new VM.
    pub fn new() -> Self {
        let mut vm = Self {
            stack: Vec::with_capacity(256),
            locals: Vec::with_capacity(64),
            globals: HashMap::new(),
            ip: 0,
            call_stack: Vec::with_capacity(64),
            native_functions: HashMap::new(),
        };
        vm.register_builtins();
        vm
    }

    /// Register built-in native functions.
    fn register_builtins(&mut self) {
        // Register all builtins from the builtins module
        let builtins = crate::builtins::register_builtins();
        for (name, value) in builtins {
            self.globals.insert(name, value);
        }
    }

    /// Register a native function.
    pub fn register_native(
        &mut self,
        name: &str,
        arity: i32,
        func: fn(&mut CallFrame, &[Value]) -> Result<Value, String>,
    ) {
        let callable = Callable::Native {
            name: name.to_string(),
            arity,
            func,
        };
        self.native_functions
            .insert(name.to_string(), Rc::new(callable));
    }

    /// Get a native function by name.
    pub fn get_native(&self, name: &str) -> Option<Rc<Callable>> {
        self.native_functions.get(name).cloned()
    }

    /// Executes bytecode and returns the result.
    pub fn execute(&mut self, bytecode: &Bytecode) -> Result<Value, Error> {
        self.ip = 0;
        self.stack.clear();
        self.locals.clear();

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

                // Local variable operations
                OpCode::LoadLocal => {
                    if let Some(Operand::Local(idx)) = &instruction.operand {
                        let value = self
                            .locals
                            .get(*idx as usize)
                            .cloned()
                            .unwrap_or(Value::Undefined);
                        self.stack.push(value);
                    }
                }

                OpCode::StoreLocal => {
                    if let Some(Operand::Local(idx)) = &instruction.operand {
                        let value = self.pop()?;
                        let idx = *idx as usize;
                        if idx >= self.locals.len() {
                            self.locals.resize(idx + 1, Value::Undefined);
                        }
                        self.locals[idx] = value;
                    }
                }

                // Global variable operations
                OpCode::LoadGlobal => {
                    if let Some(Operand::Property(idx)) = &instruction.operand {
                        if let Value::String(name) = &bytecode.constants[*idx as usize] {
                            let value = self.globals.get(name).cloned().unwrap_or(Value::Undefined);
                            self.stack.push(value);
                        }
                    }
                }

                OpCode::StoreGlobal => {
                    if let Some(Operand::Property(idx)) = &instruction.operand {
                        if let Value::String(name) = &bytecode.constants[*idx as usize] {
                            let value = self.pop()?;
                            self.globals.insert(name.clone(), value);
                        }
                    }
                }

                OpCode::Pop => {
                    self.stack.pop();
                }

                OpCode::Dup => {
                    if let Some(value) = self.stack.last().cloned() {
                        self.stack.push(value);
                    }
                }

                // Arithmetic
                OpCode::Add => self.binary_add()?,
                OpCode::Sub => self.binary_num_op(|a, b| a - b)?,
                OpCode::Mul => self.binary_num_op(|a, b| a * b)?,
                OpCode::Div => self.binary_num_op(|a, b| a / b)?,
                OpCode::Mod => self.binary_num_op(|a, b| a % b)?,
                OpCode::Pow => self.binary_num_op(|a, b| a.powf(b))?,

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
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.stack.push(Value::Boolean(a == b));
                }

                OpCode::Ne | OpCode::StrictNe => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.stack.push(Value::Boolean(a != b));
                }

                OpCode::Not => {
                    let value = self.pop()?;
                    self.stack.push(Value::Boolean(!value.to_boolean()));
                }

                // Bitwise
                OpCode::BitNot => {
                    let value = self.pop()?;
                    if let Value::Number(n) = value {
                        self.stack.push(Value::Number(!(n as i32) as f64));
                    } else {
                        return Err(Error::TypeError("Expected number".into()));
                    }
                }

                OpCode::BitAnd => self.bitwise_op(|a, b| a & b)?,
                OpCode::BitOr => self.bitwise_op(|a, b| a | b)?,
                OpCode::BitXor => self.bitwise_op(|a, b| a ^ b)?,
                OpCode::Shl => self.bitwise_op(|a, b| a << (b & 0x1f))?,
                OpCode::Shr => self.bitwise_op(|a, b| a >> (b & 0x1f))?,
                OpCode::Ushr => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    if let (Value::Number(a), Value::Number(b)) = (a, b) {
                        let result = (a as u32) >> ((b as u32) & 0x1f);
                        self.stack.push(Value::Number(result as f64));
                    } else {
                        return Err(Error::TypeError("Expected numbers".into()));
                    }
                }

                // Control flow
                OpCode::Jump => {
                    if let Some(Operand::Jump(target)) = &instruction.operand {
                        self.ip = *target as usize;
                    }
                }

                OpCode::JumpIfFalse => {
                    if let Some(Operand::Jump(target)) = &instruction.operand {
                        let condition = self.pop()?;
                        if !condition.to_boolean() {
                            self.ip = *target as usize;
                        }
                    }
                }

                OpCode::JumpIfTrue => {
                    if let Some(Operand::Jump(target)) = &instruction.operand {
                        let condition = self.pop()?;
                        if condition.to_boolean() {
                            self.ip = *target as usize;
                        }
                    }
                }

                // Object/array operations
                OpCode::NewArray => {
                    // For now, just push an object reference
                    if let Some(Operand::ArgCount(_)) = &instruction.operand {
                        // TODO: Actually create array with elements from stack
                        self.stack.push(Value::Object(0));
                    }
                }

                OpCode::NewObject => {
                    self.stack.push(Value::Object(0));
                }

                OpCode::GetProperty | OpCode::SetProperty => {
                    // TODO: Implement proper property access
                    self.stack.push(Value::Undefined);
                }

                OpCode::LoadThis => {
                    // TODO: Implement proper 'this' binding
                    self.stack.push(Value::Undefined);
                }

                OpCode::Call => {
                    if let Some(Operand::ArgCount(argc)) = &instruction.operand {
                        let argc = *argc as usize;

                        // Collect arguments from stack
                        let mut args = Vec::with_capacity(argc);
                        for _ in 0..argc {
                            args.push(self.pop()?);
                        }
                        args.reverse(); // Arguments were pushed in order

                        // Get callee
                        let callee = self.pop()?;

                        match callee {
                            Value::Function(callable) => {
                                match callable.as_ref() {
                                    Callable::Native { func, .. } => {
                                        // Create a temporary call frame for native function
                                        let temp_func =
                                            Function::new(None, vec![], Bytecode::new(), 0);
                                        let mut frame = CallFrame::new(temp_func, 0);

                                        // Call native function
                                        match func(&mut frame, &args) {
                                            Ok(result) => self.stack.push(result),
                                            Err(e) => return Err(Error::TypeError(e)),
                                        }
                                    }
                                    Callable::Function(_func) => {
                                        // TODO: Implement JS function calls
                                        // This would involve:
                                        // 1. Save current frame
                                        // 2. Create new frame
                                        // 3. Set up locals with arguments
                                        // 4. Execute function bytecode
                                        self.stack.push(Value::Undefined);
                                    }
                                }
                            }
                            _ => {
                                return Err(Error::TypeError("Value is not callable".into()));
                            }
                        }
                    }
                }

                OpCode::Return => {
                    return self.pop();
                }

                OpCode::Nop => {}

                _ => {
                    // TODO: Implement remaining opcodes
                }
            }
        }

        // Return the top of stack, or undefined if empty
        Ok(self.stack.pop().unwrap_or(Value::Undefined))
    }

    fn pop(&mut self) -> Result<Value, Error> {
        self.stack
            .pop()
            .ok_or_else(|| Error::InternalError("Stack underflow".into()))
    }

    fn binary_add(&mut self) -> Result<(), Error> {
        let b = self.pop()?;
        let a = self.pop()?;

        let result = match (&a, &b) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a + b),
            (Value::String(a), Value::String(b)) => Value::String(format!("{}{}", a, b)),
            (Value::String(a), b) => Value::String(format!("{}{}", a, b)),
            (a, Value::String(b)) => Value::String(format!("{}{}", a, b)),
            _ => Value::Number(f64::NAN),
        };

        self.stack.push(result);
        Ok(())
    }

    fn binary_num_op<F>(&mut self, op: F) -> Result<(), Error>
    where
        F: Fn(f64, f64) -> f64,
    {
        let b = self.pop()?;
        let a = self.pop()?;

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
        let b = self.pop()?;
        let a = self.pop()?;

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

    fn bitwise_op<F>(&mut self, op: F) -> Result<(), Error>
    where
        F: Fn(i32, i32) -> i32,
    {
        let b = self.pop()?;
        let a = self.pop()?;

        match (a, b) {
            (Value::Number(a), Value::Number(b)) => {
                let result = op(a as i32, b as i32);
                self.stack.push(Value::Number(result as f64));
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
