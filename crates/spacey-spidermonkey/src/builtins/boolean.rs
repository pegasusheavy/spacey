//! Boolean built-in object (ES3 Section 15.6).
//!
//! Provides Boolean constructor and prototype methods.

use crate::runtime::function::CallFrame;
use crate::runtime::value::Value;

// ============================================================================
// Boolean Constructor (ES3 Section 15.6.1-2)
// ============================================================================

/// Boolean() constructor - converts value to boolean.
pub fn boolean_constructor(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let b = args.first().map(|v| v.to_boolean()).unwrap_or(false);
    Ok(Value::Boolean(b))
}

// ============================================================================
// Boolean.prototype Methods (ES3 Section 15.6.4)
// ============================================================================

/// Boolean.prototype.toString() - Returns "true" or "false".
///
/// ES3 Section 15.6.4.2
pub fn to_string(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let b = get_this_boolean(args)?;
    Ok(Value::String(if b {
        "true".to_string()
    } else {
        "false".to_string()
    }))
}

/// Boolean.prototype.valueOf() - Returns the boolean value.
///
/// ES3 Section 15.6.4.3
pub fn value_of(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let b = get_this_boolean(args)?;
    Ok(Value::Boolean(b))
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Get the boolean value from 'this' (first argument).
fn get_this_boolean(args: &[Value]) -> Result<bool, String> {
    match args.first() {
        Some(Value::Boolean(b)) => Ok(*b),
        Some(v) => Ok(v.to_boolean()),
        None => Ok(false),
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::Bytecode;
    use crate::runtime::function::Function;

    fn make_frame() -> CallFrame {
        let func = Function::new(None, vec![], Bytecode::new(), 0);
        CallFrame::new(func, 0)
    }

    #[test]
    fn test_boolean_constructor_true() {
        let mut frame = make_frame();
        assert!(matches!(
            boolean_constructor(&mut frame, &[Value::Number(1.0)]).unwrap(),
            Value::Boolean(true)
        ));
        assert!(matches!(
            boolean_constructor(&mut frame, &[Value::String("hello".to_string())]).unwrap(),
            Value::Boolean(true)
        ));
        assert!(matches!(
            boolean_constructor(&mut frame, &[Value::Object(0)]).unwrap(),
            Value::Boolean(true)
        ));
    }

    #[test]
    fn test_boolean_constructor_false() {
        let mut frame = make_frame();
        assert!(matches!(
            boolean_constructor(&mut frame, &[Value::Number(0.0)]).unwrap(),
            Value::Boolean(false)
        ));
        assert!(matches!(
            boolean_constructor(&mut frame, &[Value::String("".to_string())]).unwrap(),
            Value::Boolean(false)
        ));
        assert!(matches!(
            boolean_constructor(&mut frame, &[Value::Null]).unwrap(),
            Value::Boolean(false)
        ));
        assert!(matches!(
            boolean_constructor(&mut frame, &[Value::Undefined]).unwrap(),
            Value::Boolean(false)
        ));
        assert!(matches!(
            boolean_constructor(&mut frame, &[Value::Number(f64::NAN)]).unwrap(),
            Value::Boolean(false)
        ));
    }

    #[test]
    fn test_boolean_constructor_empty() {
        let mut frame = make_frame();
        assert!(matches!(
            boolean_constructor(&mut frame, &[]).unwrap(),
            Value::Boolean(false)
        ));
    }

    #[test]
    fn test_to_string() {
        let mut frame = make_frame();
        assert!(matches!(
            to_string(&mut frame, &[Value::Boolean(true)]).unwrap(),
            Value::String(s) if s == "true"
        ));
        assert!(matches!(
            to_string(&mut frame, &[Value::Boolean(false)]).unwrap(),
            Value::String(s) if s == "false"
        ));
    }

    #[test]
    fn test_value_of() {
        let mut frame = make_frame();
        assert!(matches!(
            value_of(&mut frame, &[Value::Boolean(true)]).unwrap(),
            Value::Boolean(true)
        ));
        assert!(matches!(
            value_of(&mut frame, &[Value::Boolean(false)]).unwrap(),
            Value::Boolean(false)
        ));
    }

    #[test]
    fn test_coercion_in_prototype() {
        let mut frame = make_frame();
        // Non-boolean values should be coerced
        assert!(matches!(
            to_string(&mut frame, &[Value::Number(1.0)]).unwrap(),
            Value::String(s) if s == "true"
        ));
        assert!(matches!(
            value_of(&mut frame, &[Value::Number(0.0)]).unwrap(),
            Value::Boolean(false)
        ));
    }
}
