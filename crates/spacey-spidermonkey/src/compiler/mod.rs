//! Bytecode compiler for JavaScript.
//!
//! Transforms AST into bytecode that can be executed by the VM.
//!
//! # Module Structure
//!
//! - `bytecode`: Bytecode definitions and instructions
//! - `codegen`: Code generation from AST
//!   - `codegen::scope`: Scope management for variable resolution

pub mod bytecode;
pub mod codegen;

pub use bytecode::{Bytecode, Instruction, OpCode, Operand};
pub use codegen::Compiler;
