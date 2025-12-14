//! JavaScript function representation.
//!
//! This module provides:
//! - Function objects with parameters, bytecode, and closures
//! - CallFrame for execution context with `this` and `arguments`
//! - Native function support

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
    /// Whether this is a strict mode function
    pub strict: bool,
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
            strict: false,
        }
    }

    /// Creates a new strict mode function.
    pub fn new_strict(
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
            strict: true,
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

/// The `arguments` object for a function call (ES3 Section 10.1.8).
#[derive(Debug, Clone)]
pub struct Arguments {
    /// The actual arguments passed to the function
    pub values: Vec<Value>,
    /// The callee function (arguments.callee)
    pub callee: Option<Value>,
    /// The number of arguments (arguments.length)
    pub length: usize,
}

impl Arguments {
    /// Creates a new arguments object.
    pub fn new(values: Vec<Value>, callee: Option<Value>) -> Self {
        let length = values.len();
        Self {
            values,
            callee,
            length,
        }
    }

    /// Gets an argument by index.
    pub fn get(&self, index: usize) -> Value {
        self.values.get(index).cloned().unwrap_or(Value::Undefined)
    }

    /// Gets the length.
    pub fn len(&self) -> usize {
        self.length
    }

    /// Checks if empty.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

impl Default for Arguments {
    fn default() -> Self {
        Self::new(vec![], None)
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
    /// The `this` value for this call
    pub this_value: Value,
    /// The `arguments` object for this call
    pub arguments: Arguments,
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
            this_value: Value::Undefined,
            arguments: Arguments::default(),
        }
    }

    /// Creates a new call frame with this binding and arguments.
    pub fn with_this_and_args(
        function: Function,
        base_slot: usize,
        this_value: Value,
        args: Vec<Value>,
        callee: Option<Value>,
    ) -> Self {
        let local_count = function.local_count.max(function.params.len());
        let arguments = Arguments::new(args, callee);
        Self {
            function,
            ip: 0,
            base_slot,
            locals: vec![Value::Undefined; local_count],
            this_value,
            arguments,
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

    /// Gets the `this` value.
    pub fn get_this(&self) -> Value {
        self.this_value.clone()
    }

    /// Gets an argument by index.
    pub fn get_argument(&self, index: usize) -> Value {
        self.arguments.get(index)
    }

    /// Gets the number of arguments.
    pub fn argument_count(&self) -> usize {
        self.arguments.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::Bytecode;
    use std::sync::Arc;

    fn make_function(name: Option<&str>, params: Vec<&str>, local_count: usize) -> Function {
        Function::new(
            name.map(|s| s.to_string()),
            params.into_iter().map(|s| s.to_string()).collect(),
            Bytecode::new(),
            local_count,
        )
    }

    // ========================================================================
    // Function Tests
    // ========================================================================

    #[test]
    fn test_function_new() {
        let func = make_function(Some("test"), vec!["a", "b"], 3);
        assert_eq!(func.name, Some("test".to_string()));
        assert_eq!(func.params, vec!["a".to_string(), "b".to_string()]);
        assert_eq!(func.local_count, 3);
        assert!(func.upvalues.is_empty());
        assert!(!func.strict);
    }

    #[test]
    fn test_function_new_strict() {
        let func = Function::new_strict(
            Some("strict_test".to_string()),
            vec!["x".to_string()],
            Bytecode::new(),
            1,
        );
        assert!(func.strict);
    }

    #[test]
    fn test_function_arity() {
        let func0 = make_function(None, vec![], 0);
        let func1 = make_function(None, vec!["x"], 0);
        let func3 = make_function(None, vec!["a", "b", "c"], 0);

        assert_eq!(func0.arity(), 0);
        assert_eq!(func1.arity(), 1);
        assert_eq!(func3.arity(), 3);
    }

    #[test]
    fn test_function_clone() {
        let func = make_function(Some("clone_test"), vec!["x"], 1);
        let cloned = func.clone();

        assert_eq!(func.name, cloned.name);
        assert_eq!(func.params, cloned.params);
        assert_eq!(func.local_count, cloned.local_count);
    }

    // ========================================================================
    // Upvalue Tests
    // ========================================================================

    #[test]
    fn test_upvalue() {
        let upvalue = Upvalue {
            index: 5,
            is_local: true,
        };

        assert_eq!(upvalue.index, 5);
        assert!(upvalue.is_local);

        let cloned = upvalue.clone();
        assert_eq!(cloned.index, 5);
    }

    // ========================================================================
    // Callable Tests
    // ========================================================================

    #[test]
    fn test_callable_function() {
        let func = make_function(Some("myFunc"), vec![], 0);
        let callable = Callable::Function(func);

        let debug = format!("{:?}", callable);
        assert!(debug.contains("myFunc"));
    }

    #[test]
    fn test_callable_native() {
        fn native_fn(_frame: &mut CallFrame, _args: &[Value]) -> Result<Value, String> {
            Ok(Value::Number(42.0))
        }

        let callable = Callable::Native {
            name: "native_test".to_string(),
            arity: 0,
            func: native_fn,
        };

        let debug = format!("{:?}", callable);
        assert!(debug.contains("native_test"));
    }

    #[test]
    fn test_callable_clone() {
        let func = make_function(Some("cloneable"), vec![], 0);
        let callable = Callable::Function(func);
        let cloned = callable.clone();

        match cloned {
            Callable::Function(f) => assert_eq!(f.name, Some("cloneable".to_string())),
            _ => panic!("Expected Function"),
        }
    }

    // ========================================================================
    // Arguments Tests
    // ========================================================================

    #[test]
    fn test_arguments_new() {
        let args = Arguments::new(vec![Value::Number(1.0), Value::Number(2.0)], None);
        assert_eq!(args.len(), 2);
        assert!(!args.is_empty());
    }

    #[test]
    fn test_arguments_default() {
        let args = Arguments::default();
        assert_eq!(args.len(), 0);
        assert!(args.is_empty());
    }

    #[test]
    fn test_arguments_get() {
        let args = Arguments::new(
            vec![Value::Number(10.0), Value::String("hello".to_string())],
            None,
        );
        assert_eq!(args.get(0), Value::Number(10.0));
        assert_eq!(args.get(1), Value::String("hello".to_string()));
        assert_eq!(args.get(2), Value::Undefined); // Out of bounds
    }

    #[test]
    fn test_arguments_with_callee() {
        let func = make_function(Some("test"), vec![], 0);
        let callee = Value::Function(Arc::new(Callable::Function(func)));
        let args = Arguments::new(vec![], Some(callee.clone()));
        assert!(args.callee.is_some());
    }

    // ========================================================================
    // CallFrame Tests
    // ========================================================================

    #[test]
    fn test_call_frame_new() {
        let func = make_function(Some("frame_test"), vec!["x", "y"], 5);
        let frame = CallFrame::new(func, 10);

        assert_eq!(frame.ip, 0);
        assert_eq!(frame.base_slot, 10);
        assert_eq!(frame.locals.len(), 5); // max(local_count, params.len())
        assert_eq!(frame.this_value, Value::Undefined);
        assert!(frame.arguments.is_empty());
    }

    #[test]
    fn test_call_frame_with_this_and_args() {
        let func = make_function(Some("method"), vec!["a"], 2);
        let this = Value::Object(123);
        let args = vec![Value::Number(42.0)];
        let frame = CallFrame::with_this_and_args(func, 0, this.clone(), args, None);

        assert_eq!(frame.get_this(), this);
        assert_eq!(frame.argument_count(), 1);
        assert_eq!(frame.get_argument(0), Value::Number(42.0));
    }

    #[test]
    fn test_call_frame_locals_from_params() {
        // When params > local_count, locals should be sized by params
        let func = make_function(None, vec!["a", "b", "c", "d"], 2);
        let frame = CallFrame::new(func, 0);

        assert_eq!(frame.locals.len(), 4); // max(2, 4) = 4
    }

    #[test]
    fn test_call_frame_get_local() {
        let func = make_function(None, vec![], 3);
        let mut frame = CallFrame::new(func, 0);

        // Initial values are undefined
        assert_eq!(frame.get_local(0), Value::Undefined);
        assert_eq!(frame.get_local(1), Value::Undefined);
        assert_eq!(frame.get_local(2), Value::Undefined);

        // Out of bounds returns undefined
        assert_eq!(frame.get_local(100), Value::Undefined);

        // Set and get
        frame.set_local(1, Value::Number(42.0));
        assert_eq!(frame.get_local(1), Value::Number(42.0));
    }

    #[test]
    fn test_call_frame_set_local_expands() {
        let func = make_function(None, vec![], 2);
        let mut frame = CallFrame::new(func, 0);

        assert_eq!(frame.locals.len(), 2);

        // Setting beyond current size should expand
        frame.set_local(5, Value::Boolean(true));

        assert!(frame.locals.len() >= 6);
        assert_eq!(frame.get_local(5), Value::Boolean(true));

        // Intermediate values should be undefined
        assert_eq!(frame.get_local(3), Value::Undefined);
    }

    #[test]
    fn test_call_frame_debug() {
        let func = make_function(Some("debug_test"), vec![], 1);
        let frame = CallFrame::new(func, 0);

        let debug = format!("{:?}", frame);
        assert!(debug.contains("CallFrame"));
    }

    #[test]
    fn test_native_function_type() {
        fn my_native(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
            if args.is_empty() {
                Err("Need at least one argument".to_string())
            } else {
                Ok(args[0].clone())
            }
        }

        let callable = Callable::Native {
            name: "identity".to_string(),
            arity: 1,
            func: my_native,
        };

        match callable {
            Callable::Native { name, arity, .. } => {
                assert_eq!(name, "identity");
                assert_eq!(arity, 1);
            }
            _ => panic!("Expected Native"),
        }
    }
}
