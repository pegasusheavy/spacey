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
