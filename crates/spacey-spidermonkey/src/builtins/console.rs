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
