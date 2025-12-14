//! Built-in JavaScript objects and constructors.
//!
//! This module contains implementations of ES3 built-in objects:
//! - Global functions (parseInt, parseFloat, isNaN, isFinite, etc.)
//! - Object
//! - Function
//! - Array
//! - String
//! - Number
//! - Boolean
//! - Math
//! - Date (TODO)
//! - RegExp (TODO)
//! - Error types

pub mod array;
pub mod boolean;
pub mod console;
pub mod date;
pub mod error;
pub mod function;
pub mod global;
pub mod math;
pub mod number;
pub mod object;
pub mod regexp;
pub mod string;

use std::collections::HashMap;
use std::sync::Arc;

use crate::runtime::function::{Callable, NativeFunction};
use crate::runtime::value::Value;

/// Register all built-in functions and objects.
pub fn register_builtins() -> HashMap<String, Value> {
    let mut globals = HashMap::new();

    // Global functions (ES3 Section 15.1.2)
    register_global_functions(&mut globals);

    // Console object (non-standard but essential)
    register_console(&mut globals);

    // Object constructor and methods
    register_object(&mut globals);

    // Array constructor and methods
    register_array(&mut globals);

    // String constructor and methods
    register_string(&mut globals);

    // Number constructor and methods
    register_number(&mut globals);

    // Boolean constructor and methods
    register_boolean(&mut globals);

    // Math object
    register_math(&mut globals);

    // Error constructors
    register_errors(&mut globals);

    // Function constructor and methods
    register_function(&mut globals);

    // Date constructor and methods
    register_date(&mut globals);

    // RegExp constructor and methods
    register_regexp(&mut globals);

    globals
}

/// Create a native function value.
fn make_native(name: &str, arity: i32, func: NativeFunction) -> Value {
    Value::Function(Arc::new(Callable::Native {
        name: name.to_string(),
        arity,
        func,
    }))
}

/// Register global functions.
fn register_global_functions(globals: &mut HashMap<String, Value>) {
    // ES3 Section 15.1.2
    globals.insert(
        "parseInt".to_string(),
        make_native("parseInt", 2, global::parse_int),
    );
    globals.insert(
        "parseFloat".to_string(),
        make_native("parseFloat", 1, global::parse_float),
    );
    globals.insert("isNaN".to_string(), make_native("isNaN", 1, global::is_nan));
    globals.insert(
        "isFinite".to_string(),
        make_native("isFinite", 1, global::is_finite),
    );

    // ES3 Section 15.1.3
    globals.insert(
        "encodeURI".to_string(),
        make_native("encodeURI", 1, global::encode_uri),
    );
    globals.insert(
        "decodeURI".to_string(),
        make_native("decodeURI", 1, global::decode_uri),
    );
    globals.insert(
        "encodeURIComponent".to_string(),
        make_native("encodeURIComponent", 1, global::encode_uri_component),
    );
    globals.insert(
        "decodeURIComponent".to_string(),
        make_native("decodeURIComponent", 1, global::decode_uri_component),
    );

    // Global constants
    globals.insert("NaN".to_string(), Value::Number(f64::NAN));
    globals.insert("Infinity".to_string(), Value::Number(f64::INFINITY));
    globals.insert("undefined".to_string(), Value::Undefined);
}

/// Register console methods.
fn register_console(globals: &mut HashMap<String, Value>) {
    // For now, register console.log directly as a global
    // In a full implementation, console would be an object with methods
    globals.insert(
        "console_log".to_string(),
        make_native("console.log", -1, console::console_log),
    );
    globals.insert(
        "console_error".to_string(),
        make_native("console.error", -1, console::console_error),
    );
    globals.insert(
        "console_warn".to_string(),
        make_native("console.warn", -1, console::console_warn),
    );
    globals.insert(
        "console_info".to_string(),
        make_native("console.info", -1, console::console_info),
    );
    globals.insert(
        "console_debug".to_string(),
        make_native("console.debug", -1, console::console_debug),
    );
    globals.insert(
        "console_clear".to_string(),
        make_native("console.clear", 0, console::console_clear),
    );
    globals.insert(
        "print".to_string(),
        make_native("print", -1, console::console_log),
    );
}

/// Register Object constructor and static methods.
fn register_object(globals: &mut HashMap<String, Value>) {
    // Object constructor
    globals.insert(
        "Object".to_string(),
        make_native("Object", -1, object::object_constructor),
    );

    // Object static methods (would be properties of Object in full impl)
    globals.insert(
        "Object_keys".to_string(),
        make_native("Object.keys", 1, object::object_keys),
    );
    globals.insert(
        "Object_values".to_string(),
        make_native("Object.values", 1, object::object_values),
    );
    globals.insert(
        "Object_entries".to_string(),
        make_native("Object.entries", 1, object::object_entries),
    );
    globals.insert(
        "Object_assign".to_string(),
        make_native("Object.assign", -1, object::object_assign),
    );
    globals.insert(
        "Object_create".to_string(),
        make_native("Object.create", 1, object::object_create),
    );
    globals.insert(
        "Object_freeze".to_string(),
        make_native("Object.freeze", 1, object::object_freeze),
    );
    globals.insert(
        "Object_seal".to_string(),
        make_native("Object.seal", 1, object::object_seal),
    );

    // typeof helper
    globals.insert(
        "typeof".to_string(),
        make_native("typeof", 1, object::typeof_value),
    );
}

/// Register Array constructor and methods.
fn register_array(globals: &mut HashMap<String, Value>) {
    // Array constructor
    globals.insert(
        "Array".to_string(),
        make_native("Array", -1, array::array_constructor),
    );
    globals.insert(
        "Array_isArray".to_string(),
        make_native("Array.isArray", 1, array::is_array),
    );

    // Array.prototype methods (would be on prototype in full impl)
    globals.insert(
        "Array_join".to_string(),
        make_native("Array.prototype.join", -1, array::join),
    );
    globals.insert(
        "Array_push".to_string(),
        make_native("Array.prototype.push", -1, array::push),
    );
    globals.insert(
        "Array_pop".to_string(),
        make_native("Array.prototype.pop", 0, array::pop),
    );
    globals.insert(
        "Array_shift".to_string(),
        make_native("Array.prototype.shift", 0, array::shift),
    );
    globals.insert(
        "Array_unshift".to_string(),
        make_native("Array.prototype.unshift", -1, array::unshift),
    );
    globals.insert(
        "Array_slice".to_string(),
        make_native("Array.prototype.slice", 2, array::slice),
    );
    globals.insert(
        "Array_splice".to_string(),
        make_native("Array.prototype.splice", -1, array::splice),
    );
    globals.insert(
        "Array_concat".to_string(),
        make_native("Array.prototype.concat", -1, array::concat),
    );
    globals.insert(
        "Array_reverse".to_string(),
        make_native("Array.prototype.reverse", 0, array::reverse),
    );
    globals.insert(
        "Array_sort".to_string(),
        make_native("Array.prototype.sort", 1, array::sort),
    );
    globals.insert(
        "Array_toString".to_string(),
        make_native("Array.prototype.toString", 0, array::to_string),
    );
}

/// Register String constructor and methods.
fn register_string(globals: &mut HashMap<String, Value>) {
    // String constructor
    globals.insert(
        "String".to_string(),
        make_native("String", 1, string::string_constructor),
    );
    globals.insert(
        "String_fromCharCode".to_string(),
        make_native("String.fromCharCode", -1, string::from_char_code),
    );

    // String.prototype methods
    globals.insert(
        "String_charAt".to_string(),
        make_native("String.prototype.charAt", 1, string::char_at),
    );
    globals.insert(
        "String_charCodeAt".to_string(),
        make_native("String.prototype.charCodeAt", 1, string::char_code_at),
    );
    globals.insert(
        "String_concat".to_string(),
        make_native("String.prototype.concat", -1, string::concat),
    );
    globals.insert(
        "String_indexOf".to_string(),
        make_native("String.prototype.indexOf", 2, string::index_of),
    );
    globals.insert(
        "String_lastIndexOf".to_string(),
        make_native("String.prototype.lastIndexOf", 2, string::last_index_of),
    );
    globals.insert(
        "String_slice".to_string(),
        make_native("String.prototype.slice", 2, string::slice),
    );
    globals.insert(
        "String_split".to_string(),
        make_native("String.prototype.split", 2, string::split),
    );
    globals.insert(
        "String_substring".to_string(),
        make_native("String.prototype.substring", 2, string::substring),
    );
    globals.insert(
        "String_toLowerCase".to_string(),
        make_native("String.prototype.toLowerCase", 0, string::to_lower_case),
    );
    globals.insert(
        "String_toUpperCase".to_string(),
        make_native("String.prototype.toUpperCase", 0, string::to_upper_case),
    );
    globals.insert(
        "String_trim".to_string(),
        make_native("String.prototype.trim", 0, string::trim),
    );
    globals.insert(
        "String_localeCompare".to_string(),
        make_native("String.prototype.localeCompare", 1, string::locale_compare),
    );
}

/// Register Number constructor and methods.
fn register_number(globals: &mut HashMap<String, Value>) {
    // Number constructor
    globals.insert(
        "Number".to_string(),
        make_native("Number", 1, number::number_constructor),
    );

    // Number constants (would be properties of Number in full impl)
    globals.insert(
        "Number_MAX_VALUE".to_string(),
        Value::Number(number::MAX_VALUE),
    );
    globals.insert(
        "Number_MIN_VALUE".to_string(),
        Value::Number(number::MIN_VALUE),
    );
    globals.insert("Number_NaN".to_string(), Value::Number(number::NAN));
    globals.insert(
        "Number_NEGATIVE_INFINITY".to_string(),
        Value::Number(number::NEGATIVE_INFINITY),
    );
    globals.insert(
        "Number_POSITIVE_INFINITY".to_string(),
        Value::Number(number::POSITIVE_INFINITY),
    );

    // Number.prototype methods
    globals.insert(
        "Number_toString".to_string(),
        make_native("Number.prototype.toString", 1, number::to_string),
    );
    globals.insert(
        "Number_valueOf".to_string(),
        make_native("Number.prototype.valueOf", 0, number::value_of),
    );
    globals.insert(
        "Number_toFixed".to_string(),
        make_native("Number.prototype.toFixed", 1, number::to_fixed),
    );
    globals.insert(
        "Number_toExponential".to_string(),
        make_native("Number.prototype.toExponential", 1, number::to_exponential),
    );
    globals.insert(
        "Number_toPrecision".to_string(),
        make_native("Number.prototype.toPrecision", 1, number::to_precision),
    );
}

/// Register Boolean constructor and methods.
fn register_boolean(globals: &mut HashMap<String, Value>) {
    globals.insert(
        "Boolean".to_string(),
        make_native("Boolean", 1, boolean::boolean_constructor),
    );
    globals.insert(
        "Boolean_toString".to_string(),
        make_native("Boolean.prototype.toString", 0, boolean::to_string),
    );
    globals.insert(
        "Boolean_valueOf".to_string(),
        make_native("Boolean.prototype.valueOf", 0, boolean::value_of),
    );
}

/// Register Math object methods.
fn register_math(globals: &mut HashMap<String, Value>) {
    // Math constants
    globals.insert("Math_E".to_string(), Value::Number(math::E));
    globals.insert("Math_LN10".to_string(), Value::Number(math::LN10));
    globals.insert("Math_LN2".to_string(), Value::Number(math::LN2));
    globals.insert("Math_LOG2E".to_string(), Value::Number(math::LOG2E));
    globals.insert("Math_LOG10E".to_string(), Value::Number(math::LOG10E));
    globals.insert("Math_PI".to_string(), Value::Number(math::PI));
    globals.insert("Math_SQRT1_2".to_string(), Value::Number(math::SQRT1_2));
    globals.insert("Math_SQRT2".to_string(), Value::Number(math::SQRT2));

    // Math methods
    globals.insert(
        "Math_abs".to_string(),
        make_native("Math.abs", 1, math::abs),
    );
    globals.insert(
        "Math_acos".to_string(),
        make_native("Math.acos", 1, math::acos),
    );
    globals.insert(
        "Math_asin".to_string(),
        make_native("Math.asin", 1, math::asin),
    );
    globals.insert(
        "Math_atan".to_string(),
        make_native("Math.atan", 1, math::atan),
    );
    globals.insert(
        "Math_atan2".to_string(),
        make_native("Math.atan2", 2, math::atan2),
    );
    globals.insert(
        "Math_ceil".to_string(),
        make_native("Math.ceil", 1, math::ceil),
    );
    globals.insert(
        "Math_cos".to_string(),
        make_native("Math.cos", 1, math::cos),
    );
    globals.insert(
        "Math_exp".to_string(),
        make_native("Math.exp", 1, math::exp),
    );
    globals.insert(
        "Math_floor".to_string(),
        make_native("Math.floor", 1, math::floor),
    );
    globals.insert(
        "Math_log".to_string(),
        make_native("Math.log", 1, math::log),
    );
    globals.insert(
        "Math_max".to_string(),
        make_native("Math.max", -1, math::max),
    );
    globals.insert(
        "Math_min".to_string(),
        make_native("Math.min", -1, math::min),
    );
    globals.insert(
        "Math_pow".to_string(),
        make_native("Math.pow", 2, math::pow),
    );
    globals.insert(
        "Math_random".to_string(),
        make_native("Math.random", 0, math::random),
    );
    globals.insert(
        "Math_round".to_string(),
        make_native("Math.round", 1, math::round),
    );
    globals.insert(
        "Math_sin".to_string(),
        make_native("Math.sin", 1, math::sin),
    );
    globals.insert(
        "Math_sqrt".to_string(),
        make_native("Math.sqrt", 1, math::sqrt),
    );
    globals.insert(
        "Math_tan".to_string(),
        make_native("Math.tan", 1, math::tan),
    );
}

/// Register Error constructors.
fn register_errors(globals: &mut HashMap<String, Value>) {
    globals.insert(
        "Error".to_string(),
        make_native("Error", 1, error::error_constructor),
    );
    globals.insert(
        "EvalError".to_string(),
        make_native("EvalError", 1, error::eval_error_constructor),
    );
    globals.insert(
        "RangeError".to_string(),
        make_native("RangeError", 1, error::range_error_constructor),
    );
    globals.insert(
        "ReferenceError".to_string(),
        make_native("ReferenceError", 1, error::reference_error_constructor),
    );
    globals.insert(
        "SyntaxError".to_string(),
        make_native("SyntaxError", 1, error::syntax_error_constructor),
    );
    globals.insert(
        "TypeError".to_string(),
        make_native("TypeError", 1, error::type_error_constructor),
    );
    globals.insert(
        "URIError".to_string(),
        make_native("URIError", 1, error::uri_error_constructor),
    );
}

/// Register Function constructor and methods.
fn register_function(globals: &mut HashMap<String, Value>) {
    // Function constructor
    globals.insert(
        "Function".to_string(),
        make_native("Function", -1, function::function_constructor),
    );

    // Function.prototype methods
    globals.insert(
        "Function_toString".to_string(),
        make_native("Function.prototype.toString", 0, function::to_string),
    );
    globals.insert(
        "Function_call".to_string(),
        make_native("Function.prototype.call", -1, function::call),
    );
    globals.insert(
        "Function_apply".to_string(),
        make_native("Function.prototype.apply", 2, function::apply),
    );
    globals.insert(
        "Function_bind".to_string(),
        make_native("Function.prototype.bind", -1, function::bind),
    );

    // Function properties
    globals.insert(
        "Function_length".to_string(),
        make_native("Function.prototype.length", 0, function::get_length),
    );
    globals.insert(
        "Function_name".to_string(),
        make_native("Function.prototype.name", 0, function::get_name),
    );
}

/// Register Date constructor and methods.
fn register_date(globals: &mut HashMap<String, Value>) {
    // Date constructor
    globals.insert(
        "Date".to_string(),
        make_native("Date", -1, date::date_constructor),
    );

    // Date static methods
    globals.insert(
        "Date_parse".to_string(),
        make_native("Date.parse", 1, date::parse),
    );
    globals.insert(
        "Date_UTC".to_string(),
        make_native("Date.UTC", -1, date::utc),
    );
    globals.insert(
        "Date_now".to_string(),
        make_native("Date.now", 0, date::now),
    );

    // Date.prototype methods
    globals.insert(
        "Date_toString".to_string(),
        make_native("Date.prototype.toString", 0, date::to_string),
    );
    globals.insert(
        "Date_toDateString".to_string(),
        make_native("Date.prototype.toDateString", 0, date::to_date_string),
    );
    globals.insert(
        "Date_toTimeString".to_string(),
        make_native("Date.prototype.toTimeString", 0, date::to_time_string),
    );
    globals.insert(
        "Date_toISOString".to_string(),
        make_native("Date.prototype.toISOString", 0, date::to_iso_string),
    );
    globals.insert(
        "Date_toJSON".to_string(),
        make_native("Date.prototype.toJSON", 0, date::to_json),
    );
    globals.insert(
        "Date_valueOf".to_string(),
        make_native("Date.prototype.valueOf", 0, date::value_of),
    );
    globals.insert(
        "Date_getTime".to_string(),
        make_native("Date.prototype.getTime", 0, date::get_time),
    );
    globals.insert(
        "Date_getFullYear".to_string(),
        make_native("Date.prototype.getFullYear", 0, date::get_full_year),
    );
    globals.insert(
        "Date_getMonth".to_string(),
        make_native("Date.prototype.getMonth", 0, date::get_month),
    );
    globals.insert(
        "Date_getDate".to_string(),
        make_native("Date.prototype.getDate", 0, date::get_date),
    );
    globals.insert(
        "Date_getDay".to_string(),
        make_native("Date.prototype.getDay", 0, date::get_day),
    );
    globals.insert(
        "Date_getHours".to_string(),
        make_native("Date.prototype.getHours", 0, date::get_hours),
    );
    globals.insert(
        "Date_getMinutes".to_string(),
        make_native("Date.prototype.getMinutes", 0, date::get_minutes),
    );
    globals.insert(
        "Date_getSeconds".to_string(),
        make_native("Date.prototype.getSeconds", 0, date::get_seconds),
    );
    globals.insert(
        "Date_getMilliseconds".to_string(),
        make_native("Date.prototype.getMilliseconds", 0, date::get_milliseconds),
    );
    globals.insert(
        "Date_getTimezoneOffset".to_string(),
        make_native(
            "Date.prototype.getTimezoneOffset",
            0,
            date::get_timezone_offset,
        ),
    );
    globals.insert(
        "Date_setTime".to_string(),
        make_native("Date.prototype.setTime", 1, date::set_time),
    );
    globals.insert(
        "Date_setMilliseconds".to_string(),
        make_native("Date.prototype.setMilliseconds", 1, date::set_milliseconds),
    );
    globals.insert(
        "Date_setSeconds".to_string(),
        make_native("Date.prototype.setSeconds", -1, date::set_seconds),
    );
    globals.insert(
        "Date_setMinutes".to_string(),
        make_native("Date.prototype.setMinutes", -1, date::set_minutes),
    );
    globals.insert(
        "Date_setHours".to_string(),
        make_native("Date.prototype.setHours", -1, date::set_hours),
    );
    globals.insert(
        "Date_setDate".to_string(),
        make_native("Date.prototype.setDate", 1, date::set_date),
    );
    globals.insert(
        "Date_setMonth".to_string(),
        make_native("Date.prototype.setMonth", -1, date::set_month),
    );
    globals.insert(
        "Date_setFullYear".to_string(),
        make_native("Date.prototype.setFullYear", -1, date::set_full_year),
    );
}

/// Register RegExp constructor and methods.
fn register_regexp(globals: &mut HashMap<String, Value>) {
    // RegExp constructor
    globals.insert(
        "RegExp".to_string(),
        make_native("RegExp", 2, regexp::regexp_constructor),
    );

    // RegExp.prototype methods
    globals.insert(
        "RegExp_exec".to_string(),
        make_native("RegExp.prototype.exec", 1, regexp::exec),
    );
    globals.insert(
        "RegExp_test".to_string(),
        make_native("RegExp.prototype.test", 1, regexp::test),
    );
    globals.insert(
        "RegExp_toString".to_string(),
        make_native("RegExp.prototype.toString", 0, regexp::to_string),
    );

    // RegExp.prototype properties
    globals.insert(
        "RegExp_source".to_string(),
        make_native("RegExp.prototype.source", 0, regexp::get_source),
    );
    globals.insert(
        "RegExp_global".to_string(),
        make_native("RegExp.prototype.global", 0, regexp::get_global),
    );
    globals.insert(
        "RegExp_ignoreCase".to_string(),
        make_native("RegExp.prototype.ignoreCase", 0, regexp::get_ignore_case),
    );
    globals.insert(
        "RegExp_multiline".to_string(),
        make_native("RegExp.prototype.multiline", 0, regexp::get_multiline),
    );
    globals.insert(
        "RegExp_lastIndex".to_string(),
        make_native("RegExp.prototype.lastIndex", 0, regexp::get_last_index),
    );

    // String methods that use RegExp
    globals.insert(
        "String_match".to_string(),
        make_native("String.prototype.match", 1, regexp::string_match),
    );
    globals.insert(
        "String_replace".to_string(),
        make_native("String.prototype.replace", 2, regexp::string_replace),
    );
    globals.insert(
        "String_search".to_string(),
        make_native("String.prototype.search", 1, regexp::string_search),
    );
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_builtins() {
        let globals = register_builtins();

        // Check global functions
        assert!(globals.contains_key("parseInt"));
        assert!(globals.contains_key("parseFloat"));
        assert!(globals.contains_key("isNaN"));
        assert!(globals.contains_key("isFinite"));
        assert!(globals.contains_key("encodeURI"));
        assert!(globals.contains_key("decodeURI"));

        // Check global constants
        assert!(globals.contains_key("NaN"));
        assert!(globals.contains_key("Infinity"));
        assert!(globals.contains_key("undefined"));

        // Check constructors
        assert!(globals.contains_key("Object"));
        assert!(globals.contains_key("Array"));
        assert!(globals.contains_key("String"));
        assert!(globals.contains_key("Number"));
        assert!(globals.contains_key("Boolean"));

        // Check Error constructors
        assert!(globals.contains_key("Error"));
        assert!(globals.contains_key("TypeError"));
        assert!(globals.contains_key("ReferenceError"));
        assert!(globals.contains_key("SyntaxError"));
        assert!(globals.contains_key("RangeError"));

        // Check Math
        assert!(globals.contains_key("Math_PI"));
        assert!(globals.contains_key("Math_abs"));
        assert!(globals.contains_key("Math_floor"));
        assert!(globals.contains_key("Math_random"));
    }

    #[test]
    fn test_global_constants() {
        let globals = register_builtins();

        if let Some(Value::Number(n)) = globals.get("NaN") {
            assert!(n.is_nan());
        } else {
            panic!("NaN should be a number");
        }

        if let Some(Value::Number(n)) = globals.get("Infinity") {
            assert!(n.is_infinite() && *n > 0.0);
        } else {
            panic!("Infinity should be a number");
        }

        assert!(matches!(globals.get("undefined"), Some(Value::Undefined)));
    }

    #[test]
    fn test_math_constants() {
        let globals = register_builtins();

        if let Some(Value::Number(n)) = globals.get("Math_PI") {
            assert!((n - std::f64::consts::PI).abs() < 0.0001);
        } else {
            panic!("Math_PI should be a number");
        }

        if let Some(Value::Number(n)) = globals.get("Math_E") {
            assert!((n - std::f64::consts::E).abs() < 0.0001);
        } else {
            panic!("Math_E should be a number");
        }
    }

    #[test]
    fn test_number_constants() {
        let globals = register_builtins();

        if let Some(Value::Number(n)) = globals.get("Number_MAX_VALUE") {
            assert_eq!(*n, f64::MAX);
        } else {
            panic!("Number_MAX_VALUE should be a number");
        }

        if let Some(Value::Number(n)) = globals.get("Number_NaN") {
            assert!(n.is_nan());
        } else {
            panic!("Number_NaN should be a number");
        }
    }

    #[test]
    fn test_console_functions() {
        let globals = register_builtins();
        assert!(globals.contains_key("console_log"));
        assert!(globals.contains_key("console_error"));
        assert!(globals.contains_key("console_warn"));
        assert!(globals.contains_key("print"));
    }

    #[test]
    fn test_all_builtins_are_functions_or_values() {
        let globals = register_builtins();

        for (name, value) in globals.iter() {
            match value {
                Value::Function(_) => {
                    // Functions are expected
                }
                Value::Number(_) => {
                    // Constants like NaN, Infinity, Math_PI
                }
                Value::Undefined => {
                    assert_eq!(name, "undefined");
                }
                _ => {
                    panic!("Unexpected value type for {}: {:?}", name, value);
                }
            }
        }
    }
}
