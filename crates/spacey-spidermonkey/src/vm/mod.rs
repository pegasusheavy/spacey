//! The bytecode virtual machine.
//!
//! This module contains the VM that executes compiled bytecode, along with
//! supporting modules for method dispatch and runtime objects.
//!
//! ## Structure
//!
//! - `interpreter` - Main VM execution engine (monolithic, contains all logic)
//!
//! ## Submodules (for future refactoring)
//!
//! The following submodules contain extracted functionality that can be used
//! to gradually modularize the interpreter:
//!
//! - `comparison` - Abstract equality comparison (ES3 11.9.3)
//! - `date_methods` - Date object method implementations
//! - `number_methods` - Number object method implementations
//! - `string_methods` - String object method implementations
//! - `regexp_methods` - RegExp object method implementations
//! - `runtime_object` - Runtime object representation

mod interpreter;

// Submodules with extracted functionality
pub mod comparison;
pub mod date_methods;
pub mod number_methods;
pub mod regexp_methods;
pub mod runtime_object;
pub mod string_methods;

// Re-export public API
pub use interpreter::VM;

// Re-export types from submodules for external use
pub use runtime_object::RuntimeObject as RuntimeObjectNew;
