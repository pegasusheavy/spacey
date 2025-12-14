//! Math built-in object (ES3 Section 15.8).
//!
//! The Math object provides mathematical constants and functions.
//! Unlike other built-in objects, Math is not a constructor.

use crate::runtime::function::CallFrame;
use crate::runtime::value::Value;

// ============================================================================
// Math Constants (ES3 Section 15.8.1)
// ============================================================================

/// Math.E - Euler's number (approximately 2.718)
pub const E: f64 = std::f64::consts::E;

/// Math.LN10 - Natural logarithm of 10
pub const LN10: f64 = std::f64::consts::LN_10;

/// Math.LN2 - Natural logarithm of 2
pub const LN2: f64 = std::f64::consts::LN_2;

/// Math.LOG2E - Base 2 logarithm of E
pub const LOG2E: f64 = std::f64::consts::LOG2_E;

/// Math.LOG10E - Base 10 logarithm of E
pub const LOG10E: f64 = std::f64::consts::LOG10_E;

/// Math.PI - Ratio of circumference to diameter
pub const PI: f64 = std::f64::consts::PI;

/// Math.SQRT1_2 - Square root of 1/2
pub const SQRT1_2: f64 = std::f64::consts::FRAC_1_SQRT_2;

/// Math.SQRT2 - Square root of 2
pub const SQRT2: f64 = std::f64::consts::SQRT_2;

// ============================================================================
// Math Functions (ES3 Section 15.8.2)
// ============================================================================

/// Math.abs(x) - Returns the absolute value of x.
pub fn abs(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let x = args.first().map(|v| v.to_number()).unwrap_or(f64::NAN);
    Ok(Value::Number(x.abs()))
}

/// Math.acos(x) - Returns the arc cosine of x.
pub fn acos(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let x = args.first().map(|v| v.to_number()).unwrap_or(f64::NAN);
    Ok(Value::Number(x.acos()))
}

/// Math.asin(x) - Returns the arc sine of x.
pub fn asin(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let x = args.first().map(|v| v.to_number()).unwrap_or(f64::NAN);
    Ok(Value::Number(x.asin()))
}

/// Math.atan(x) - Returns the arc tangent of x.
pub fn atan(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let x = args.first().map(|v| v.to_number()).unwrap_or(f64::NAN);
    Ok(Value::Number(x.atan()))
}

/// Math.atan2(y, x) - Returns the arc tangent of y/x.
pub fn atan2(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let y = args.first().map(|v| v.to_number()).unwrap_or(f64::NAN);
    let x = args.get(1).map(|v| v.to_number()).unwrap_or(f64::NAN);
    Ok(Value::Number(y.atan2(x)))
}

/// Math.ceil(x) - Returns the smallest integer >= x.
pub fn ceil(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let x = args.first().map(|v| v.to_number()).unwrap_or(f64::NAN);
    Ok(Value::Number(x.ceil()))
}

/// Math.cos(x) - Returns the cosine of x.
pub fn cos(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let x = args.first().map(|v| v.to_number()).unwrap_or(f64::NAN);
    Ok(Value::Number(x.cos()))
}

/// Math.exp(x) - Returns E^x.
pub fn exp(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let x = args.first().map(|v| v.to_number()).unwrap_or(f64::NAN);
    Ok(Value::Number(x.exp()))
}

/// Math.floor(x) - Returns the largest integer <= x.
pub fn floor(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let x = args.first().map(|v| v.to_number()).unwrap_or(f64::NAN);
    Ok(Value::Number(x.floor()))
}

/// Math.log(x) - Returns the natural logarithm of x.
pub fn log(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let x = args.first().map(|v| v.to_number()).unwrap_or(f64::NAN);
    Ok(Value::Number(x.ln()))
}

/// Math.max(...values) - Returns the maximum of the arguments.
pub fn max(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Ok(Value::Number(f64::NEG_INFINITY));
    }

    let mut result = f64::NEG_INFINITY;
    for arg in args {
        let n = arg.to_number();
        if n.is_nan() {
            return Ok(Value::Number(f64::NAN));
        }
        if n > result {
            result = n;
        }
    }
    Ok(Value::Number(result))
}

/// Math.min(...values) - Returns the minimum of the arguments.
pub fn min(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Ok(Value::Number(f64::INFINITY));
    }

    let mut result = f64::INFINITY;
    for arg in args {
        let n = arg.to_number();
        if n.is_nan() {
            return Ok(Value::Number(f64::NAN));
        }
        if n < result {
            result = n;
        }
    }
    Ok(Value::Number(result))
}

/// Math.pow(base, exponent) - Returns base^exponent.
pub fn pow(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let base = args.first().map(|v| v.to_number()).unwrap_or(f64::NAN);
    let exp = args.get(1).map(|v| v.to_number()).unwrap_or(f64::NAN);
    Ok(Value::Number(base.powf(exp)))
}

/// Math.random() - Returns a random number in [0, 1).
pub fn random(_frame: &mut CallFrame, _args: &[Value]) -> Result<Value, String> {
    use std::time::{SystemTime, UNIX_EPOCH};

    // Simple LCG random - not cryptographically secure but ES3 compliant
    // In production, would use a proper RNG
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;

    // LCG parameters from Numerical Recipes
    let random = ((seed.wrapping_mul(1103515245).wrapping_add(12345)) >> 16) as f64;
    let result = (random % 32768.0) / 32768.0;

    Ok(Value::Number(result))
}

/// Math.round(x) - Returns the nearest integer to x.
pub fn round(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let x = args.first().map(|v| v.to_number()).unwrap_or(f64::NAN);

    if x.is_nan() || x.is_infinite() || x == 0.0 {
        return Ok(Value::Number(x));
    }

    // ES3 rounds half toward positive infinity
    let result = if x >= 0.0 {
        (x + 0.5).floor()
    } else if x > -0.5 {
        // -0.5 < x < 0 rounds to -0
        -0.0
    } else {
        (x + 0.5).ceil()
    };

    Ok(Value::Number(result))
}

/// Math.sin(x) - Returns the sine of x.
pub fn sin(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let x = args.first().map(|v| v.to_number()).unwrap_or(f64::NAN);
    Ok(Value::Number(x.sin()))
}

/// Math.sqrt(x) - Returns the square root of x.
pub fn sqrt(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let x = args.first().map(|v| v.to_number()).unwrap_or(f64::NAN);
    Ok(Value::Number(x.sqrt()))
}

/// Math.tan(x) - Returns the tangent of x.
pub fn tan(_frame: &mut CallFrame, args: &[Value]) -> Result<Value, String> {
    let x = args.first().map(|v| v.to_number()).unwrap_or(f64::NAN);
    Ok(Value::Number(x.tan()))
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
    fn test_math_constants() {
        assert!((E - 2.718281828).abs() < 0.0001);
        assert!((PI - 3.141592653).abs() < 0.0001);
        assert!((SQRT2 - 1.414213562).abs() < 0.0001);
    }

    #[test]
    fn test_abs() {
        let mut frame = make_frame();
        assert!(
            matches!(abs(&mut frame, &[Value::Number(-5.0)]).unwrap(), Value::Number(n) if n == 5.0)
        );
        assert!(
            matches!(abs(&mut frame, &[Value::Number(5.0)]).unwrap(), Value::Number(n) if n == 5.0)
        );
        assert!(
            matches!(abs(&mut frame, &[Value::Number(0.0)]).unwrap(), Value::Number(n) if n == 0.0)
        );
    }

    #[test]
    fn test_ceil() {
        let mut frame = make_frame();
        assert!(
            matches!(ceil(&mut frame, &[Value::Number(1.1)]).unwrap(), Value::Number(n) if n == 2.0)
        );
        assert!(
            matches!(ceil(&mut frame, &[Value::Number(-1.1)]).unwrap(), Value::Number(n) if n == -1.0)
        );
    }

    #[test]
    fn test_floor() {
        let mut frame = make_frame();
        assert!(
            matches!(floor(&mut frame, &[Value::Number(1.9)]).unwrap(), Value::Number(n) if n == 1.0)
        );
        assert!(
            matches!(floor(&mut frame, &[Value::Number(-1.1)]).unwrap(), Value::Number(n) if n == -2.0)
        );
    }

    #[test]
    fn test_round() {
        let mut frame = make_frame();
        assert!(
            matches!(round(&mut frame, &[Value::Number(1.4)]).unwrap(), Value::Number(n) if n == 1.0)
        );
        assert!(
            matches!(round(&mut frame, &[Value::Number(1.5)]).unwrap(), Value::Number(n) if n == 2.0)
        );
        assert!(
            matches!(round(&mut frame, &[Value::Number(-1.5)]).unwrap(), Value::Number(n) if n == -1.0)
        );
    }

    #[test]
    fn test_max() {
        let mut frame = make_frame();
        let result = max(
            &mut frame,
            &[Value::Number(1.0), Value::Number(3.0), Value::Number(2.0)],
        )
        .unwrap();
        assert!(matches!(result, Value::Number(n) if n == 3.0));

        // Empty returns -Infinity
        assert!(
            matches!(max(&mut frame, &[]).unwrap(), Value::Number(n) if n == f64::NEG_INFINITY)
        );
    }

    #[test]
    fn test_min() {
        let mut frame = make_frame();
        let result = min(
            &mut frame,
            &[Value::Number(1.0), Value::Number(3.0), Value::Number(2.0)],
        )
        .unwrap();
        assert!(matches!(result, Value::Number(n) if n == 1.0));

        // Empty returns Infinity
        assert!(matches!(min(&mut frame, &[]).unwrap(), Value::Number(n) if n == f64::INFINITY));
    }

    #[test]
    fn test_pow() {
        let mut frame = make_frame();
        let result = pow(&mut frame, &[Value::Number(2.0), Value::Number(3.0)]).unwrap();
        assert!(matches!(result, Value::Number(n) if n == 8.0));
    }

    #[test]
    fn test_sqrt() {
        let mut frame = make_frame();
        assert!(
            matches!(sqrt(&mut frame, &[Value::Number(4.0)]).unwrap(), Value::Number(n) if n == 2.0)
        );
        assert!(
            matches!(sqrt(&mut frame, &[Value::Number(9.0)]).unwrap(), Value::Number(n) if n == 3.0)
        );
    }

    #[test]
    fn test_random() {
        let mut frame = make_frame();
        let result = random(&mut frame, &[]).unwrap();
        if let Value::Number(n) = result {
            assert!(n >= 0.0 && n < 1.0);
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_trig_functions() {
        let mut frame = make_frame();
        // sin(0) = 0
        assert!(
            matches!(sin(&mut frame, &[Value::Number(0.0)]).unwrap(), Value::Number(n) if n.abs() < 0.0001)
        );
        // cos(0) = 1
        assert!(
            matches!(cos(&mut frame, &[Value::Number(0.0)]).unwrap(), Value::Number(n) if (n - 1.0).abs() < 0.0001)
        );
        // tan(0) = 0
        assert!(
            matches!(tan(&mut frame, &[Value::Number(0.0)]).unwrap(), Value::Number(n) if n.abs() < 0.0001)
        );
    }
}
