//! Built-in JavaScript objects and constructors.
//!
//! This module will contain implementations of all built-in objects:
//! - Object, Function, Array, String, Number, Boolean
//! - Math, JSON, Date, RegExp
//! - Map, Set, WeakMap, WeakSet
//! - Promise, Symbol
//! - Error types
//! - TypedArrays, ArrayBuffer, DataView
//! - Etc.

pub mod console;
pub mod object;

use std::collections::HashMap;
use std::rc::Rc;

use crate::runtime::function::{Callable, NativeFunction};
use crate::runtime::value::Value;

/// Register all built-in functions and objects.
pub fn register_builtins() -> HashMap<String, Value> {
    let mut globals = HashMap::new();

    // Console object
    register_console(&mut globals);

    // Object constructor and methods
    register_object(&mut globals);

    globals
}

/// Create a native function value.
fn make_native(name: &str, arity: i32, func: NativeFunction) -> Value {
    Value::Function(Rc::new(Callable::Native {
        name: name.to_string(),
        arity,
        func,
    }))
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
