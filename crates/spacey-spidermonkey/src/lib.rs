// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2025 Pegasus Heavy Industries, LLC

//! # spacey-spidermonkey
//!
//! A JavaScript engine inspired by Mozilla's SpiderMonkey, implemented in Rust.
//!
//! ## Overview
//!
//! This crate provides a complete JavaScript execution environment including:
//! - Lexer and parser for ECMAScript 2024+
//! - Bytecode compiler and interpreter
//! - Garbage-collected runtime
//! - Built-in objects and standard library
//! - Optional JIT compilation
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use spacey_spidermonkey::{Engine, Value};
//!
//! let mut engine = Engine::new();
//! let result = engine.eval("1 + 2")?;
//! assert_eq!(result, Value::Number(3.0));
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

// Core modules - to be implemented
pub mod lexer;
pub mod parser;
pub mod ast;
pub mod compiler;
pub mod runtime;
pub mod vm;
pub mod builtins;
pub mod gc;

// Re-exports for convenience
pub use runtime::value::Value;
pub use runtime::context::Context;

/// The main JavaScript engine instance.
///
/// Encapsulates the entire JavaScript execution environment including
/// the heap, global object, and execution state.
pub struct Engine {
    context: Context,
}

impl Engine {
    /// Creates a new JavaScript engine instance with default configuration.
    pub fn new() -> Self {
        Self {
            context: Context::new(),
        }
    }

    /// Evaluates JavaScript source code and returns the result.
    ///
    /// # Arguments
    ///
    /// * `source` - The JavaScript source code to evaluate
    ///
    /// # Returns
    ///
    /// The result of evaluating the expression, or an error if parsing
    /// or execution fails.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut engine = Engine::new();
    /// let result = engine.eval("2 + 2")?;
    /// ```
    pub fn eval(&mut self, source: &str) -> Result<Value, Error> {
        // TODO: Implement evaluation pipeline
        // 1. Lex source into tokens
        // 2. Parse tokens into AST
        // 3. Compile AST to bytecode
        // 4. Execute bytecode in VM
        let _ = source;
        Ok(Value::Undefined)
    }

    /// Evaluates JavaScript source code from a file.
    pub fn eval_file(&mut self, path: &std::path::Path) -> Result<Value, Error> {
        let source = std::fs::read_to_string(path)
            .map_err(|e| Error::Io(e.to_string()))?;
        self.eval(&source)
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur during JavaScript execution.
#[derive(Debug, Clone)]
pub enum Error {
    /// Syntax error during parsing
    SyntaxError(String),
    /// Type error during execution
    TypeError(String),
    /// Reference error (undefined variable)
    ReferenceError(String),
    /// Range error (out of bounds, etc.)
    RangeError(String),
    /// Internal engine error
    InternalError(String),
    /// I/O error
    Io(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::SyntaxError(msg) => write!(f, "SyntaxError: {}", msg),
            Error::TypeError(msg) => write!(f, "TypeError: {}", msg),
            Error::ReferenceError(msg) => write!(f, "ReferenceError: {}", msg),
            Error::RangeError(msg) => write!(f, "RangeError: {}", msg),
            Error::InternalError(msg) => write!(f, "InternalError: {}", msg),
            Error::Io(msg) => write!(f, "IOError: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = Engine::new();
        assert!(matches!(engine.context, _));
    }

    #[test]
    fn test_default_eval_returns_undefined() {
        let mut engine = Engine::new();
        let result = engine.eval("test").unwrap();
        assert!(matches!(result, Value::Undefined));
    }
}


