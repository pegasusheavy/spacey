//! Console built-in object.
//!
//! Provides `console.log`, `console.error`, `console.warn`, etc.

use crate::runtime::function::CallFrame;
use crate::runtime::value::Value;

/// Console.log - prints to stdout
pub fn console_log(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let output: Vec<String> = args.iter().map(|v| format!("{}", v)).collect();
    println!("{}", output.join(" "));
    Ok(Value::Undefined)
}

/// Console.error - prints to stderr
pub fn console_error(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let output: Vec<String> = args.iter().map(|v| format!("{}", v)).collect();
    eprintln!("{}", output.join(" "));
    Ok(Value::Undefined)
}

/// Console.warn - prints warning to stderr
pub fn console_warn(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let output: Vec<String> = args.iter().map(|v| format!("{}", v)).collect();
    eprintln!("Warning: {}", output.join(" "));
    Ok(Value::Undefined)
}

/// Console.info - prints info message
pub fn console_info(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let output: Vec<String> = args.iter().map(|v| format!("{}", v)).collect();
    println!("Info: {}", output.join(" "));
    Ok(Value::Undefined)
}

/// Console.debug - prints debug message
pub fn console_debug(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let output: Vec<String> = args.iter().map(|v| format!("{}", v)).collect();
    println!("Debug: {}", output.join(" "));
    Ok(Value::Undefined)
}

/// Console.clear - clears console (ANSI escape)
pub fn console_clear(_frame: &mut CallFrame, _args: &[Value]) -> Result<Value, String> {
    print!("\x1B[2J\x1B[H");
    Ok(Value::Undefined)
}

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
    fn test_console_log_no_args() {
        let mut frame = make_frame();
        let result = console_log(&mut frame, &[]);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Value::Undefined));
    }

    #[test]
    fn test_console_log_single_arg() {
        let mut frame = make_frame();
        let result = console_log(&mut frame, &[Value::String("hello".to_string())]);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Value::Undefined));
    }

    #[test]
    fn test_console_log_multiple_args() {
        let mut frame = make_frame();
        let result = console_log(
            &mut frame,
            &[Value::String("count:".to_string()), Value::Number(42.0)],
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_console_error_no_args() {
        let mut frame = make_frame();
        let result = console_error(&mut frame, &[]);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Value::Undefined));
    }

    #[test]
    fn test_console_error_with_message() {
        let mut frame = make_frame();
        let result = console_error(&mut frame, &[Value::String("error message".to_string())]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_console_warn_no_args() {
        let mut frame = make_frame();
        let result = console_warn(&mut frame, &[]);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Value::Undefined));
    }

    #[test]
    fn test_console_warn_with_message() {
        let mut frame = make_frame();
        let result = console_warn(&mut frame, &[Value::String("warning".to_string())]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_console_info_no_args() {
        let mut frame = make_frame();
        let result = console_info(&mut frame, &[]);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Value::Undefined));
    }

    #[test]
    fn test_console_info_with_message() {
        let mut frame = make_frame();
        let result = console_info(&mut frame, &[Value::String("info".to_string())]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_console_debug_no_args() {
        let mut frame = make_frame();
        let result = console_debug(&mut frame, &[]);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Value::Undefined));
    }

    #[test]
    fn test_console_debug_with_message() {
        let mut frame = make_frame();
        let result = console_debug(&mut frame, &[Value::String("debug".to_string())]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_console_clear() {
        let mut frame = make_frame();
        let result = console_clear(&mut frame, &[]);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Value::Undefined));
    }

    #[test]
    fn test_console_log_different_types() {
        let mut frame = make_frame();

        // Number
        assert!(console_log(&mut frame, &[Value::Number(42.0)]).is_ok());

        // Boolean
        assert!(console_log(&mut frame, &[Value::Boolean(true)]).is_ok());

        // Null
        assert!(console_log(&mut frame, &[Value::Null]).is_ok());

        // Undefined
        assert!(console_log(&mut frame, &[Value::Undefined]).is_ok());
    }
}
