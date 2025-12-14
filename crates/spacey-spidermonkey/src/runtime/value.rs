//! JavaScript value representation.

use super::function::Callable;
use std::fmt;
use std::sync::Arc;

/// A JavaScript value.
///
/// Values are designed to be thread-safe and can be safely shared
/// between async tasks.
#[derive(Debug, Clone)]
pub enum Value {
    /// undefined
    Undefined,
    /// null
    Null,
    /// Boolean value
    Boolean(bool),
    /// Number (IEEE 754 double)
    Number(f64),
    /// String
    String(String),
    /// Symbol
    Symbol(u64),
    /// BigInt (stored as string for now)
    BigInt(String),
    /// Object reference (placeholder - would be GC handle)
    Object(usize),
    /// Function reference (Arc for thread safety)
    Function(Arc<Callable>),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Undefined, Value::Undefined) => true,
            (Value::Null, Value::Null) => true,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => {
                // Handle NaN comparisons
                if a.is_nan() && b.is_nan() {
                    false
                } else {
                    a == b
                }
            }
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Symbol(a), Value::Symbol(b)) => a == b,
            (Value::BigInt(a), Value::BigInt(b)) => a == b,
            (Value::Object(a), Value::Object(b)) => a == b,
            (Value::Function(a), Value::Function(b)) => Arc::ptr_eq(a, b),
            _ => false,
        }
    }
}

impl Value {
    /// Returns true if this value is undefined.
    pub fn is_undefined(&self) -> bool {
        matches!(self, Value::Undefined)
    }

    /// Returns true if this value is null.
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    /// Returns true if this value is nullish (null or undefined).
    pub fn is_nullish(&self) -> bool {
        matches!(self, Value::Undefined | Value::Null)
    }

    /// Returns true if this value is a function.
    pub fn is_function(&self) -> bool {
        matches!(self, Value::Function(_))
    }

    /// Converts the value to a boolean (ToBoolean).
    pub fn to_boolean(&self) -> bool {
        match self {
            Value::Undefined | Value::Null => false,
            Value::Boolean(b) => *b,
            Value::Number(n) => !n.is_nan() && *n != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Symbol(_) | Value::BigInt(_) | Value::Object(_) | Value::Function(_) => true,
        }
    }

    /// Returns the type of this value as a string.
    pub fn type_of(&self) -> &'static str {
        match self {
            Value::Undefined => "undefined",
            Value::Null => "object", // Historical quirk
            Value::Boolean(_) => "boolean",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Symbol(_) => "symbol",
            Value::BigInt(_) => "bigint",
            Value::Object(_) => "object",
            Value::Function(_) => "function",
        }
    }
}

impl Default for Value {
    fn default() -> Self {
        Value::Undefined
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Undefined => write!(f, "undefined"),
            Value::Null => write!(f, "null"),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Symbol(id) => write!(f, "Symbol({})", id),
            Value::BigInt(n) => write!(f, "{}n", n),
            Value::Object(_) => write!(f, "[object Object]"),
            Value::Function(callable) => match callable.as_ref() {
                Callable::Function(func) => {
                    if let Some(name) = &func.name {
                        write!(f, "[Function: {}]", name)
                    } else {
                        write!(f, "[Function (anonymous)]")
                    }
                }
                Callable::Native { name, .. } => {
                    write!(f, "[Function: {} (native)]", name)
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::Bytecode;
    use crate::runtime::function::{CallFrame, Function};

    fn make_test_function(name: Option<&str>) -> Function {
        Function::new(name.map(|s| s.to_string()), vec![], Bytecode::new(), 0)
    }

    fn native_test_fn(_frame: &mut CallFrame, _args: &[Value]) -> Result<Value, String> {
        Ok(Value::Undefined)
    }

    #[test]
    fn test_value_undefined() {
        let v = Value::Undefined;
        assert!(v.is_undefined());
        assert!(!v.is_null());
        assert!(v.is_nullish());
        assert!(!v.is_function());
        assert_eq!(v.type_of(), "undefined");
        assert!(!v.to_boolean());
        assert_eq!(v.to_string(), "undefined");
    }

    #[test]
    fn test_value_null() {
        let v = Value::Null;
        assert!(!v.is_undefined());
        assert!(v.is_null());
        assert!(v.is_nullish());
        assert!(!v.is_function());
        assert_eq!(v.type_of(), "object"); // Historical quirk
        assert!(!v.to_boolean());
        assert_eq!(v.to_string(), "null");
    }

    #[test]
    fn test_value_boolean() {
        let t = Value::Boolean(true);
        let f = Value::Boolean(false);

        assert!(!t.is_undefined());
        assert!(!t.is_null());
        assert!(!t.is_nullish());
        assert_eq!(t.type_of(), "boolean");
        assert!(t.to_boolean());
        assert_eq!(t.to_string(), "true");

        assert!(!f.to_boolean());
        assert_eq!(f.to_string(), "false");
    }

    #[test]
    fn test_value_number() {
        let zero = Value::Number(0.0);
        let pos = Value::Number(42.0);
        let neg = Value::Number(-1.5);
        let nan = Value::Number(f64::NAN);
        let inf = Value::Number(f64::INFINITY);

        assert_eq!(zero.type_of(), "number");
        assert!(!zero.to_boolean()); // 0 is falsy
        assert!(pos.to_boolean());
        assert!(neg.to_boolean());
        assert!(!nan.to_boolean()); // NaN is falsy
        assert!(inf.to_boolean());

        assert_eq!(pos.to_string(), "42");
        assert_eq!(neg.to_string(), "-1.5");
    }

    #[test]
    fn test_value_string() {
        let empty = Value::String(String::new());
        let hello = Value::String("hello".to_string());

        assert_eq!(empty.type_of(), "string");
        assert!(!empty.to_boolean()); // Empty string is falsy
        assert!(hello.to_boolean());
        assert_eq!(hello.to_string(), "hello");
    }

    #[test]
    fn test_value_symbol() {
        let sym = Value::Symbol(123);
        assert_eq!(sym.type_of(), "symbol");
        assert!(sym.to_boolean()); // Symbols are truthy
        assert_eq!(sym.to_string(), "Symbol(123)");
    }

    #[test]
    fn test_value_bigint() {
        let big = Value::BigInt("12345678901234567890".to_string());
        assert_eq!(big.type_of(), "bigint");
        assert!(big.to_boolean()); // BigInts are truthy
        assert_eq!(big.to_string(), "12345678901234567890n");
    }

    #[test]
    fn test_value_object() {
        let obj = Value::Object(42);
        assert_eq!(obj.type_of(), "object");
        assert!(obj.to_boolean()); // Objects are truthy
        assert_eq!(obj.to_string(), "[object Object]");
    }

    #[test]
    fn test_value_function_named() {
        let func = make_test_function(Some("myFunc"));
        let v = Value::Function(Arc::new(Callable::Function(func)));

        assert!(v.is_function());
        assert_eq!(v.type_of(), "function");
        assert!(v.to_boolean());
        assert_eq!(v.to_string(), "[Function: myFunc]");
    }

    #[test]
    fn test_value_function_anonymous() {
        let func = make_test_function(None);
        let v = Value::Function(Arc::new(Callable::Function(func)));

        assert!(v.is_function());
        assert_eq!(v.to_string(), "[Function (anonymous)]");
    }

    #[test]
    fn test_value_function_native() {
        let native = Callable::Native {
            name: "print".to_string(),
            arity: 1,
            func: native_test_fn,
        };
        let v = Value::Function(Arc::new(native));

        assert!(v.is_function());
        assert_eq!(v.to_string(), "[Function: print (native)]");
    }

    #[test]
    fn test_value_default() {
        let v = Value::default();
        assert!(v.is_undefined());
    }

    #[test]
    fn test_value_equality_undefined() {
        assert_eq!(Value::Undefined, Value::Undefined);
        assert_ne!(Value::Undefined, Value::Null);
    }

    #[test]
    fn test_value_equality_null() {
        assert_eq!(Value::Null, Value::Null);
        assert_ne!(Value::Null, Value::Undefined);
    }

    #[test]
    fn test_value_equality_boolean() {
        assert_eq!(Value::Boolean(true), Value::Boolean(true));
        assert_eq!(Value::Boolean(false), Value::Boolean(false));
        assert_ne!(Value::Boolean(true), Value::Boolean(false));
    }

    #[test]
    fn test_value_equality_number() {
        assert_eq!(Value::Number(42.0), Value::Number(42.0));
        assert_ne!(Value::Number(42.0), Value::Number(43.0));

        // NaN is not equal to itself
        assert_ne!(Value::Number(f64::NAN), Value::Number(f64::NAN));
    }

    #[test]
    fn test_value_equality_string() {
        assert_eq!(
            Value::String("hello".to_string()),
            Value::String("hello".to_string())
        );
        assert_ne!(
            Value::String("hello".to_string()),
            Value::String("world".to_string())
        );
    }

    #[test]
    fn test_value_equality_symbol() {
        assert_eq!(Value::Symbol(1), Value::Symbol(1));
        assert_ne!(Value::Symbol(1), Value::Symbol(2));
    }

    #[test]
    fn test_value_equality_bigint() {
        assert_eq!(
            Value::BigInt("123".to_string()),
            Value::BigInt("123".to_string())
        );
        assert_ne!(
            Value::BigInt("123".to_string()),
            Value::BigInt("456".to_string())
        );
    }

    #[test]
    fn test_value_equality_object() {
        assert_eq!(Value::Object(1), Value::Object(1));
        assert_ne!(Value::Object(1), Value::Object(2));
    }

    #[test]
    fn test_value_equality_function() {
        let func1 = Arc::new(Callable::Function(make_test_function(None)));
        let func2 = func1.clone();
        let func3 = Arc::new(Callable::Function(make_test_function(None)));

        // Same Arc pointer
        assert_eq!(Value::Function(func1.clone()), Value::Function(func2));
        // Different Arc pointers
        assert_ne!(Value::Function(func1), Value::Function(func3));
    }

    #[test]
    fn test_value_equality_different_types() {
        assert_ne!(Value::Undefined, Value::Boolean(false));
        assert_ne!(Value::Null, Value::Number(0.0));
        assert_ne!(Value::Boolean(true), Value::Number(1.0));
        assert_ne!(Value::String("42".to_string()), Value::Number(42.0));
    }

    #[test]
    fn test_value_debug() {
        // Test that Debug is implemented
        let v = Value::Number(42.0);
        let debug_str = format!("{:?}", v);
        assert!(debug_str.contains("Number"));
        assert!(debug_str.contains("42"));
    }

    #[test]
    fn test_value_clone() {
        let original = Value::String("test".to_string());
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }
}
