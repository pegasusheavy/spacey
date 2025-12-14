//! The bytecode interpreter.

use std::collections::HashMap;
use std::sync::Arc;

use crate::Error;
use crate::compiler::{Bytecode, OpCode, Operand};
use crate::runtime::function::{CallFrame, Callable, Function};
use crate::runtime::value::Value;

/// A saved call frame for restoring after return.
#[derive(Clone)]
struct SavedFrame {
    /// Saved instruction pointer
    ip: usize,
    /// Saved bytecode reference
    bytecode_idx: usize,
    /// Saved locals base
    locals_base: usize,
}

/// Runtime object representation
#[derive(Clone, Debug)]
pub struct RuntimeObject {
    /// Properties stored as key-value pairs
    properties: HashMap<String, Value>,
    /// For arrays: stores elements by index
    array_elements: Vec<Value>,
    /// Whether this is an array
    is_array: bool,
}

impl RuntimeObject {
    /// Create a new empty object
    fn new() -> Self {
        Self {
            properties: HashMap::new(),
            array_elements: Vec::new(),
            is_array: false,
        }
    }

    /// Create a new array with elements
    fn new_array(elements: Vec<Value>) -> Self {
        let len = elements.len();
        let mut obj = Self {
            properties: HashMap::new(),
            array_elements: elements,
            is_array: true,
        };
        obj.properties.insert("length".to_string(), Value::Number(len as f64));
        obj
    }

    /// Get a property by name
    fn get(&self, name: &str) -> Value {
        if self.is_array {
            // Check for numeric index
            if let Ok(idx) = name.parse::<usize>() {
                return self.array_elements.get(idx).cloned().unwrap_or(Value::Undefined);
            }
        }
        self.properties.get(name).cloned().unwrap_or(Value::Undefined)
    }

    /// Set a property
    fn set(&mut self, name: &str, value: Value) {
        if self.is_array {
            if let Ok(idx) = name.parse::<usize>() {
                // Extend array if necessary
                while self.array_elements.len() <= idx {
                    self.array_elements.push(Value::Undefined);
                }
                self.array_elements[idx] = value;
                // Update length
                self.properties.insert("length".to_string(), Value::Number(self.array_elements.len() as f64));
                return;
            }
        }
        self.properties.insert(name.to_string(), value);
    }
}

/// The virtual machine that executes bytecode.
#[derive(Clone)]
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
    native_functions: HashMap<String, Arc<Callable>>,
    /// Heap for runtime objects
    heap: Vec<RuntimeObject>,
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
            heap: Vec::with_capacity(256),
        };
        vm.register_builtins();
        vm
    }

    /// Allocate a new object on the heap, returning its index
    fn alloc_object(&mut self, obj: RuntimeObject) -> usize {
        let idx = self.heap.len();
        self.heap.push(obj);
        idx
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
            .insert(name.to_string(), Arc::new(callable));
    }

    /// Get a native function by name.
    pub fn get_native(&self, name: &str) -> Option<Arc<Callable>> {
        self.native_functions.get(name).cloned()
    }

    /// Call a user-defined function with given arguments.
    fn call_function(&mut self, func: &Function, args: &[Value]) -> Result<Value, Error> {
        // Execute the function's bytecode
        let func_bytecode = &func.bytecode;

        // Save current state
        let saved_ip = self.ip;
        let saved_locals_len = self.locals.len();

        // Initialize locals for the function
        // First set up parameters
        for (i, param) in func.params.iter().enumerate() {
            let value = args.get(i).cloned().unwrap_or(Value::Undefined);
            // Extend locals if needed
            while self.locals.len() <= i + saved_locals_len {
                self.locals.push(Value::Undefined);
            }
            self.locals[i + saved_locals_len] = value;
            let _ = param; // Silence unused warning
        }

        // Allocate remaining locals as undefined
        let total_locals = func.local_count.max(func.params.len());
        while self.locals.len() < saved_locals_len + total_locals {
            self.locals.push(Value::Undefined);
        }

        // Execute function
        self.ip = 0;
        let mut result = Value::Undefined;

        loop {
            if self.ip >= func_bytecode.instructions.len() {
                break;
            }

            let instruction = &func_bytecode.instructions[self.ip];
            self.ip += 1;

            match instruction.opcode {
                OpCode::Return => {
                    result = self.pop().unwrap_or(Value::Undefined);
                    break;
                }

                OpCode::LoadConst => {
                    if let Some(Operand::Constant(idx)) = &instruction.operand {
                        let value = func_bytecode.constants[*idx as usize].clone();
                        self.stack.push(value);
                    }
                }

                OpCode::LoadLocal => {
                    if let Some(Operand::Local(idx)) = &instruction.operand {
                        let local_idx = saved_locals_len + *idx as usize;
                        let value = self
                            .locals
                            .get(local_idx)
                            .cloned()
                            .unwrap_or(Value::Undefined);
                        self.stack.push(value);
                    }
                }

                OpCode::StoreLocal => {
                    if let Some(Operand::Local(idx)) = &instruction.operand {
                        let local_idx = saved_locals_len + *idx as usize;
                        let value = self.pop()?;
                        while self.locals.len() <= local_idx {
                            self.locals.push(Value::Undefined);
                        }
                        self.locals[local_idx] = value;
                    }
                }

                OpCode::LoadUndefined => {
                    self.stack.push(Value::Undefined);
                }

                OpCode::Add => self.binary_add()?,
                OpCode::Sub => self.binary_num_op(|a, b| a - b)?,
                OpCode::Mul => self.binary_num_op(|a, b| a * b)?,
                OpCode::Div => self.binary_num_op(|a, b| a / b)?,
                OpCode::Mod => self.binary_num_op(|a, b| a % b)?,
                OpCode::Pow => self.binary_num_op(|a, b| a.powf(b))?,

                OpCode::Pop => {
                    self.pop()?;
                }

                OpCode::LoadGlobal => {
                    if let Some(Operand::Property(idx)) = &instruction.operand {
                        let name = match &func_bytecode.constants[*idx as usize] {
                            Value::String(s) => s.clone(),
                            _ => return Err(Error::TypeError("Property name must be a string".into())),
                        };
                        let value = self.globals.get(&name).cloned().unwrap_or(Value::Undefined);
                        self.stack.push(value);
                    }
                }

                OpCode::StoreGlobal => {
                    if let Some(Operand::Property(idx)) = &instruction.operand {
                        let name = match &func_bytecode.constants[*idx as usize] {
                            Value::String(s) => s.clone(),
                            _ => return Err(Error::TypeError("Property name must be a string".into())),
                        };
                        let value = self.pop()?;
                        self.globals.insert(name, value);
                    }
                }

                OpCode::Dup => {
                    if let Some(value) = self.stack.last().cloned() {
                        self.stack.push(value);
                    }
                }

                OpCode::LoadTrue => {
                    self.stack.push(Value::Boolean(true));
                }

                OpCode::LoadFalse => {
                    self.stack.push(Value::Boolean(false));
                }

                OpCode::LoadNull => {
                    self.stack.push(Value::Null);
                }

                // Comparison operations
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

                // Jump operations
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

                OpCode::Neg => {
                    if let Some(Value::Number(n)) = self.stack.pop() {
                        self.stack.push(Value::Number(-n));
                    }
                }

                OpCode::Not => {
                    let value = self.pop()?;
                    self.stack.push(Value::Boolean(!value.to_boolean()));
                }

                // Property access
                OpCode::GetProperty => {
                    let prop_name = if let Some(Operand::Property(idx)) = &instruction.operand {
                        match func_bytecode.constants.get(*idx as usize) {
                            Some(Value::String(s)) => s.clone(),
                            _ => {
                                self.stack.push(Value::Undefined);
                                continue;
                            }
                        }
                    } else {
                        match self.stack.pop() {
                            Some(Value::String(s)) => s,
                            Some(Value::Number(n)) => n.to_string(),
                            _ => {
                                self.stack.push(Value::Undefined);
                                continue;
                            }
                        }
                    };

                    let obj = self.stack.pop().unwrap_or(Value::Undefined);
                    let result = match &obj {
                        Value::String(s) => match prop_name.as_str() {
                            "length" => Value::Number(s.len() as f64),
                            _ => {
                                if let Ok(idx) = prop_name.parse::<usize>() {
                                    s.chars()
                                        .nth(idx)
                                        .map(|c| Value::String(c.to_string()))
                                        .unwrap_or(Value::Undefined)
                                } else {
                                    Value::Undefined
                                }
                            }
                        },
                        Value::Object(idx) => {
                            if let Some(obj) = self.heap.get(*idx) {
                                obj.get(&prop_name)
                            } else {
                                Value::Undefined
                            }
                        }
                        Value::NativeObject(props) => {
                            props.get(&prop_name).cloned().unwrap_or(Value::Undefined)
                        }
                        _ => Value::Undefined,
                    };
                    self.stack.push(result);
                }

                // Function calls within functions
                OpCode::Call => {
                    if let Some(Operand::ArgCount(argc)) = &instruction.operand {
                        let argc = *argc as usize;
                        let mut call_args = Vec::with_capacity(argc);
                        for _ in 0..argc {
                            call_args.push(self.pop()?);
                        }
                        call_args.reverse();

                        let callee = self.pop()?;
                        match callee {
                            Value::Function(callable) => {
                                match callable.as_ref() {
                                    crate::runtime::function::Callable::Native { func, .. } => {
                                        let temp_func = Function::new(None, vec![], Bytecode::new(), 0);
                                        let mut frame = CallFrame::new(temp_func, 0);
                                        match func(&mut frame, &call_args) {
                                            Ok(res) => self.stack.push(res),
                                            Err(e) => return Err(Error::TypeError(e)),
                                        }
                                    }
                                    crate::runtime::function::Callable::Function(inner_func) => {
                                        let res = self.call_function(inner_func, &call_args)?;
                                        self.stack.push(res);
                                    }
                                }
                            }
                            _ => return Err(Error::TypeError("Value is not callable".into())),
                        }
                    }
                }

                _ => {
                    // For any other operations, we'd need to handle them
                    // For now, skip unhandled ops in function bodies
                }
            }
        }

        // Restore state
        self.ip = saved_ip;
        self.locals.truncate(saved_locals_len);

        Ok(result)
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
                    if let Some(Operand::ArgCount(count)) = &instruction.operand {
                        let count = *count as usize;
                        // Pop elements from stack (in reverse order)
                        let mut elements = Vec::with_capacity(count);
                        for _ in 0..count {
                            elements.push(self.pop()?);
                        }
                        elements.reverse();
                        // Create array on heap
                        let idx = self.alloc_object(RuntimeObject::new_array(elements));
                        self.stack.push(Value::Object(idx));
                    } else {
                        // Empty array
                        let idx = self.alloc_object(RuntimeObject::new_array(vec![]));
                        self.stack.push(Value::Object(idx));
                    }
                }

                OpCode::NewObject => {
                    let idx = self.alloc_object(RuntimeObject::new());
                    self.stack.push(Value::Object(idx));
                }

                OpCode::GetProperty => {
                    // Get property name from operand or stack
                    let prop_name = if let Some(Operand::Property(idx)) = &instruction.operand {
                        // Property name is in constants
                        match bytecode.constants.get(*idx as usize) {
                            Some(Value::String(s)) => s.clone(),
                            _ => {
                                self.stack.push(Value::Undefined);
                                continue;
                            }
                        }
                    } else {
                        // Property name/index is on stack
                        match self.stack.pop() {
                            Some(Value::String(s)) => s,
                            Some(Value::Number(n)) => n.to_string(),
                            _ => {
                                self.stack.push(Value::Undefined);
                                continue;
                            }
                        }
                    };

                    // Get object from stack
                    let obj = self.stack.pop().unwrap_or(Value::Undefined);

                    // Access property based on value type
                    let result = match &obj {
                        Value::String(s) => {
                            // String properties
                            match prop_name.as_str() {
                                "length" => Value::Number(s.len() as f64),
                                _ => {
                                    // Numeric index access
                                    if let Ok(idx) = prop_name.parse::<usize>() {
                                        s.chars()
                                            .nth(idx)
                                            .map(|c| Value::String(c.to_string()))
                                            .unwrap_or(Value::Undefined)
                                    } else {
                                        Value::Undefined
                                    }
                                }
                            }
                        }
                        Value::Object(idx) => {
                            // Object property access from heap
                            if let Some(obj) = self.heap.get(*idx) {
                                obj.get(&prop_name)
                            } else {
                                Value::Undefined
                            }
                        }
                        Value::NativeObject(props) => {
                            // Native object property access (e.g., console.log)
                            props.get(&prop_name).cloned().unwrap_or(Value::Undefined)
                        }
                        _ => Value::Undefined,
                    };

                    self.stack.push(result);
                }

                OpCode::SetProperty => {
                    // Get property name from operand or stack
                    let prop_name = if let Some(Operand::Property(idx)) = &instruction.operand {
                        match bytecode.constants.get(*idx as usize) {
                            Some(Value::String(s)) => s.clone(),
                            _ => {
                                self.stack.push(Value::Undefined);
                                continue;
                            }
                        }
                    } else {
                        match self.stack.pop() {
                            Some(Value::String(s)) => s,
                            Some(Value::Number(n)) => n.to_string(),
                            _ => {
                                self.stack.push(Value::Undefined);
                                continue;
                            }
                        }
                    };

                    // Get value to set
                    let value = self.pop()?;

                    // Get object
                    let obj = self.stack.pop().unwrap_or(Value::Undefined);

                    // Set property on object in heap
                    if let Value::Object(idx) = obj {
                        if let Some(obj) = self.heap.get_mut(idx) {
                            obj.set(&prop_name, value.clone());
                        }
                        // Push the object back (object is still on stack for chained property access)
                        self.stack.push(Value::Object(idx));
                        // Also push the value (result of assignment)
                        self.stack.push(value);
                    } else {
                        self.stack.push(Value::Undefined);
                    }
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
                                    Callable::Function(func) => {
                                        // Execute the user-defined function
                                        let result = self.call_function(func, &args)?;
                                        self.stack.push(result);
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

                OpCode::ForInInit => {
                    // Pop object to iterate, push keys array and index 0
                    let obj = self.stack.pop().unwrap_or(Value::Undefined);

                    // Get enumerable keys from the object
                    // In a full impl, would use Object.keys or iterate prototype chain
                    let keys = match &obj {
                        Value::Object(_) => {
                            // Placeholder - in full impl would get actual keys
                            vec![]
                        }
                        Value::String(s) => {
                            // String indices
                            (0..s.len()).map(|i| Value::String(i.to_string())).collect()
                        }
                        _ => vec![],
                    };

                    // Push keys and index onto stack for iteration
                    // We use a special representation: index as Number, followed by the obj
                    self.stack.push(obj); // Keep object for reference
                    self.stack.push(Value::Number(0.0)); // Current index
                    self.stack.push(Value::Object(keys.len())); // Keys count
                }

                OpCode::ForInNext => {
                    // Check if there are more keys
                    // Stack: [obj, index, count]
                    if let Some(Operand::Jump(target)) = instruction.operand {
                        let count = match self.stack.pop() {
                            Some(Value::Object(n)) => n,
                            _ => 0,
                        };
                        let index = match self.stack.pop() {
                            Some(Value::Number(n)) => n as usize,
                            _ => 0,
                        };
                        let obj = self.stack.pop().unwrap_or(Value::Undefined);

                        if index >= count {
                            // No more keys, jump to end
                            self.ip = target as usize;
                        } else {
                            // Push next key and restore iteration state
                            self.stack.push(obj.clone());
                            self.stack.push(Value::Number((index + 1) as f64));
                            self.stack.push(Value::Object(count));
                            // Push the key value (index as string for now)
                            self.stack.push(Value::String(index.to_string()));
                        }
                    }
                }

                OpCode::ForInDone => {
                    // Clean up iteration state
                    // Stack should have: [obj, index, count] - pop them all
                    self.stack.pop(); // count
                    self.stack.pop(); // index
                    self.stack.pop(); // obj
                }

                OpCode::LogicalAnd => {
                    let right = self.stack.pop().unwrap_or(Value::Undefined);
                    let left = self.stack.pop().unwrap_or(Value::Undefined);
                    // Short-circuit AND: return left if falsy, else right
                    if left.to_boolean() {
                        self.stack.push(right);
                    } else {
                        self.stack.push(left);
                    }
                }

                OpCode::LogicalOr => {
                    let right = self.stack.pop().unwrap_or(Value::Undefined);
                    let left = self.stack.pop().unwrap_or(Value::Undefined);
                    // Short-circuit OR: return left if truthy, else right
                    if left.to_boolean() {
                        self.stack.push(left);
                    } else {
                        self.stack.push(right);
                    }
                }

                OpCode::TypeOf => {
                    let val = self.stack.pop().unwrap_or(Value::Undefined);
                    self.stack.push(Value::String(val.type_of().to_string()));
                }

                OpCode::InstanceOf => {
                    let right = self.stack.pop().unwrap_or(Value::Undefined);
                    let left = self.stack.pop().unwrap_or(Value::Undefined);
                    // Simplified instanceof - checks if left is an object
                    let result = match (&left, &right) {
                        (Value::Object(_), Value::Function(_)) => true,
                        _ => false,
                    };
                    self.stack.push(Value::Boolean(result));
                }

                OpCode::In => {
                    let right = self.stack.pop().unwrap_or(Value::Undefined);
                    let left = self.stack.pop().unwrap_or(Value::Undefined);

                    // The 'in' operator checks if a property exists on an object
                    let result = match (&left, &right) {
                        (Value::String(prop), Value::Object(idx)) => {
                            // Check if property exists on the object
                            if let Some(obj) = self.heap.get(*idx) {
                                obj.get(prop) != Value::Undefined
                            } else {
                                false
                            }
                        }
                        (Value::Number(n), Value::Object(idx)) => {
                            // Check numeric index
                            let prop = n.to_string();
                            if let Some(obj) = self.heap.get(*idx) {
                                obj.get(&prop) != Value::Undefined
                            } else {
                                false
                            }
                        }
                        _ => false,
                    };
                    self.stack.push(Value::Boolean(result));
                }

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::Compiler;
    use crate::parser::Parser;

    fn eval(src: &str) -> Result<Value, Error> {
        let mut parser = Parser::new(src);
        let program = parser.parse_program()?;
        let mut compiler = Compiler::new();
        let bytecode = compiler.compile(&program)?;
        let mut vm = VM::new();
        vm.execute(&bytecode)
    }

    fn eval_ok(src: &str) -> Value {
        eval(src).expect("Evaluation should succeed")
    }

    #[test]
    fn test_vm_new() {
        let vm = VM::new();
        assert!(vm.stack.is_empty());
        assert!(vm.locals.is_empty());
    }

    #[test]
    fn test_vm_default() {
        let vm = VM::default();
        assert!(vm.stack.is_empty());
    }

    #[test]
    fn test_eval_number() {
        let result = eval_ok("42;");
        assert!(matches!(result, Value::Number(n) if n == 42.0));
    }

    #[test]
    fn test_eval_float() {
        let result = eval_ok("3.14;");
        assert!(matches!(result, Value::Number(n) if (n - 3.14).abs() < 0.001));
    }

    #[test]
    fn test_eval_string() {
        let result = eval_ok("'hello';");
        assert!(matches!(result, Value::String(s) if s == "hello"));
    }

    #[test]
    fn test_eval_boolean_true() {
        let result = eval_ok("true;");
        assert!(matches!(result, Value::Boolean(true)));
    }

    #[test]
    fn test_eval_boolean_false() {
        let result = eval_ok("false;");
        assert!(matches!(result, Value::Boolean(false)));
    }

    #[test]
    fn test_eval_null() {
        let result = eval_ok("null;");
        assert!(matches!(result, Value::Null));
    }

    #[test]
    fn test_eval_add_numbers() {
        let result = eval_ok("1 + 2;");
        assert!(matches!(result, Value::Number(n) if n == 3.0));
    }

    #[test]
    fn test_eval_subtract() {
        let result = eval_ok("5 - 3;");
        assert!(matches!(result, Value::Number(n) if n == 2.0));
    }

    #[test]
    fn test_eval_multiply() {
        let result = eval_ok("4 * 5;");
        assert!(matches!(result, Value::Number(n) if n == 20.0));
    }

    #[test]
    fn test_eval_divide() {
        let result = eval_ok("10 / 2;");
        assert!(matches!(result, Value::Number(n) if n == 5.0));
    }

    #[test]
    fn test_eval_modulo() {
        let result = eval_ok("7 % 3;");
        assert!(matches!(result, Value::Number(n) if n == 1.0));
    }

    #[test]
    fn test_eval_string_concat() {
        let result = eval_ok("'hello' + ' ' + 'world';");
        assert!(matches!(result, Value::String(s) if s == "hello world"));
    }

    #[test]
    fn test_eval_string_number_concat() {
        let result = eval_ok("'count: ' + 42;");
        assert!(matches!(result, Value::String(s) if s == "count: 42"));
    }

    #[test]
    fn test_eval_less_than() {
        assert!(matches!(eval_ok("1 < 2;"), Value::Boolean(true)));
        assert!(matches!(eval_ok("2 < 1;"), Value::Boolean(false)));
    }

    #[test]
    fn test_eval_greater_than() {
        assert!(matches!(eval_ok("2 > 1;"), Value::Boolean(true)));
        assert!(matches!(eval_ok("1 > 2;"), Value::Boolean(false)));
    }

    #[test]
    fn test_eval_less_than_equal() {
        assert!(matches!(eval_ok("1 <= 2;"), Value::Boolean(true)));
        assert!(matches!(eval_ok("2 <= 2;"), Value::Boolean(true)));
        assert!(matches!(eval_ok("3 <= 2;"), Value::Boolean(false)));
    }

    #[test]
    fn test_eval_greater_than_equal() {
        assert!(matches!(eval_ok("2 >= 1;"), Value::Boolean(true)));
        assert!(matches!(eval_ok("2 >= 2;"), Value::Boolean(true)));
        assert!(matches!(eval_ok("1 >= 2;"), Value::Boolean(false)));
    }

    #[test]
    fn test_eval_equal() {
        assert!(matches!(eval_ok("1 == 1;"), Value::Boolean(true)));
        assert!(matches!(eval_ok("1 == 2;"), Value::Boolean(false)));
    }

    #[test]
    fn test_eval_not_equal() {
        assert!(matches!(eval_ok("1 != 2;"), Value::Boolean(true)));
        assert!(matches!(eval_ok("1 != 1;"), Value::Boolean(false)));
    }

    #[test]
    fn test_eval_strict_equal() {
        assert!(matches!(eval_ok("1 === 1;"), Value::Boolean(true)));
        assert!(matches!(eval_ok("1 === 2;"), Value::Boolean(false)));
    }

    #[test]
    fn test_eval_strict_not_equal() {
        assert!(matches!(eval_ok("1 !== 2;"), Value::Boolean(true)));
        assert!(matches!(eval_ok("1 !== 1;"), Value::Boolean(false)));
    }

    #[test]
    fn test_eval_negate() {
        let result = eval_ok("-42;");
        assert!(matches!(result, Value::Number(n) if n == -42.0));
    }

    #[test]
    fn test_eval_not() {
        assert!(matches!(eval_ok("!true;"), Value::Boolean(false)));
        assert!(matches!(eval_ok("!false;"), Value::Boolean(true)));
    }

    #[test]
    fn test_eval_variable() {
        let result = eval_ok("let x = 42; x;");
        assert!(matches!(result, Value::Number(n) if n == 42.0));
    }

    #[test]
    fn test_eval_variable_assignment() {
        let result = eval_ok("let x = 1; x = 2; x;");
        assert!(matches!(result, Value::Number(n) if n == 2.0));
    }

    #[test]
    fn test_eval_multiple_variables() {
        let result = eval_ok("let a = 1; let b = 2; a + b;");
        assert!(matches!(result, Value::Number(n) if n == 3.0));
    }

    #[test]
    fn test_eval_if_true() {
        let result = eval_ok("let x = 0; if (true) { x = 1; } x;");
        assert!(matches!(result, Value::Number(n) if n == 1.0));
    }

    #[test]
    fn test_eval_if_false() {
        let result = eval_ok("let x = 0; if (false) { x = 1; } x;");
        assert!(matches!(result, Value::Number(n) if n == 0.0));
    }

    #[test]
    fn test_eval_if_else() {
        let result = eval_ok("let x = 0; if (false) { x = 1; } else { x = 2; } x;");
        assert!(matches!(result, Value::Number(n) if n == 2.0));
    }

    #[test]
    fn test_eval_while_loop() {
        let result = eval_ok("let x = 0; while (x < 3) { x = x + 1; } x;");
        assert!(matches!(result, Value::Number(n) if n == 3.0));
    }

    #[test]
    fn test_eval_for_loop() {
        let result =
            eval_ok("let sum = 0; for (let i = 0; i < 5; i = i + 1) { sum = sum + i; } sum;");
        assert!(matches!(result, Value::Number(n) if n == 10.0)); // 0+1+2+3+4
    }

    // Note: User-defined function calls are not yet fully supported
    // These tests are placeholders for when they are implemented

    #[test]
    fn test_eval_array_literal() {
        let result = eval_ok("let arr = [1, 2, 3]; arr;");
        // Array should be an object
        assert!(matches!(result, Value::Object(_)));
    }

    #[test]
    fn test_eval_empty_program() {
        let result = eval_ok("");
        assert!(matches!(result, Value::Undefined));
    }

    #[test]
    fn test_eval_expression_precedence() {
        let result = eval_ok("2 + 3 * 4;");
        assert!(matches!(result, Value::Number(n) if n == 14.0)); // 2 + 12
    }

    #[test]
    fn test_vm_register_native() {
        let mut vm = VM::new();
        fn custom_fn(_frame: &mut CallFrame, _args: &[Value]) -> Result<Value, String> {
            Ok(Value::Number(999.0))
        }
        vm.register_native("custom", 0, custom_fn);
        assert!(vm.get_native("custom").is_some());
    }

    #[test]
    fn test_vm_get_native_not_found() {
        let vm = VM::new();
        assert!(vm.get_native("nonexistent").is_none());
    }

    #[test]
    fn test_eval_complex_expression() {
        let result = eval_ok("let x = 5; let y = 3; x * y + 2;");
        assert!(matches!(result, Value::Number(n) if n == 17.0));
    }
}
