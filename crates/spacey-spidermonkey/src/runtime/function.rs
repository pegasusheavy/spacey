//! JavaScript function representation.

use super::value::Value;
use crate::compiler::Bytecode;

/// A JavaScript function.
#[derive(Debug, Clone)]
pub struct Function {
    /// The function name (if any)
    pub name: Option<String>,
    /// The parameter names
    pub params: Vec<String>,
    /// The compiled bytecode
    pub bytecode: Bytecode,
    /// Number of local variables
    pub local_count: usize,
    /// Captured upvalues (for closures)
    pub upvalues: Vec<Upvalue>,
}

impl Function {
    /// Creates a new function.
    pub fn new(
        name: Option<String>,
        params: Vec<String>,
        bytecode: Bytecode,
        local_count: usize,
    ) -> Self {
        Self {
            name,
            params,
            bytecode,
            local_count,
            upvalues: Vec::new(),
        }
    }

    /// Returns the arity (number of parameters).
    pub fn arity(&self) -> usize {
        self.params.len()
    }
}

/// An upvalue for closure capture.
#[derive(Debug, Clone)]
pub struct Upvalue {
    /// The index in the enclosing scope
    pub index: usize,
    /// Whether this is a local in the immediately enclosing function
    pub is_local: bool,
}

/// A native (Rust) function.
pub type NativeFunction = fn(&mut CallFrame, &[Value]) -> Result<Value, String>;

/// A callable value - either a JS function or a native function.
#[derive(Clone)]
pub enum Callable {
    /// A JavaScript function
    Function(Function),
    /// A native Rust function
    Native {
        /// The function name
        name: String,
        /// The arity (-1 for variadic)
        arity: i32,
        /// The native function pointer
        func: NativeFunction,
    },
}

impl std::fmt::Debug for Callable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Callable::Function(func) => write!(f, "Function({:?})", func.name),
            Callable::Native { name, .. } => write!(f, "NativeFunction({})", name),
        }
    }
}

/// A call frame for function execution.
#[derive(Debug)]
pub struct CallFrame {
    /// The function being executed
    pub function: Function,
    /// Instruction pointer within this function
    pub ip: usize,
    /// Base index in the stack for this frame's locals
    pub base_slot: usize,
    /// Local variables for this frame
    pub locals: Vec<Value>,
}

impl CallFrame {
    /// Creates a new call frame.
    pub fn new(function: Function, base_slot: usize) -> Self {
        let local_count = function.local_count.max(function.params.len());
        Self {
            function,
            ip: 0,
            base_slot,
            locals: vec![Value::Undefined; local_count],
        }
    }

    /// Gets a local variable.
    pub fn get_local(&self, index: usize) -> Value {
        self.locals.get(index).cloned().unwrap_or(Value::Undefined)
    }

    /// Sets a local variable.
    pub fn set_local(&mut self, index: usize, value: Value) {
        if index >= self.locals.len() {
            self.locals.resize(index + 1, Value::Undefined);
        }
        self.locals[index] = value;
    }
}
