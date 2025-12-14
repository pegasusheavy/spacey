//! Asynchronous JavaScript engine APIs.
//!
//! This module provides async/await compatible APIs for the JavaScript engine,
//! enabling non-blocking file I/O and concurrent evaluation.
//!
//! # Features
//!
//! - Async file evaluation with `eval_file_async`
//! - Parallel module loading
//! - Non-blocking execution
//!
//! # Example
//!
//! ```ignore
//! use spacey_spidermonkey::AsyncEngine;
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut engine = AsyncEngine::new();
//!     let result = engine.eval_file_async("script.js").await.unwrap();
//!     println!("{}", result);
//! }
//! ```

#[cfg(feature = "async")]
use tokio::fs;
#[cfg(feature = "async")]
use tokio::sync::RwLock;

use std::path::Path;
use std::sync::Arc;

use crate::Error;
use crate::compiler::Compiler;
use crate::parser::Parser;
use crate::runtime::value::Value;
use crate::vm::VM;

/// An asynchronous JavaScript engine.
///
/// This engine wraps the synchronous engine and provides async APIs
/// for file I/O and concurrent operations.
#[cfg(feature = "async")]
pub struct AsyncEngine {
    /// The underlying VM (thread-safe wrapper)
    vm: Arc<RwLock<VM>>,
}

#[cfg(feature = "async")]
impl AsyncEngine {
    /// Creates a new async engine.
    pub fn new() -> Self {
        Self {
            vm: Arc::new(RwLock::new(VM::new())),
        }
    }

    /// Evaluates JavaScript source code asynchronously.
    ///
    /// The parsing and compilation happen synchronously, but the VM
    /// execution can be awaited.
    pub async fn eval(&self, source: &str) -> Result<Value, Error> {
        // Parse and compile synchronously (CPU-bound)
        let mut parser = Parser::new(source);
        let ast = parser.parse_program()?;

        let mut compiler = Compiler::new();
        let bytecode = compiler.compile(&ast)?;

        // Execute with write lock
        let mut vm = self.vm.write().await;
        vm.execute(&bytecode)
    }

    /// Evaluates a JavaScript file asynchronously.
    ///
    /// Uses tokio's async file I/O for non-blocking reads.
    pub async fn eval_file(&self, path: impl AsRef<Path>) -> Result<Value, Error> {
        let path = path.as_ref();

        // Async file read
        let source = fs::read_to_string(path)
            .await
            .map_err(|e| Error::Io(format!("Failed to read {}: {}", path.display(), e)))?;

        self.eval(&source).await
    }

    /// Evaluates multiple JavaScript files in parallel.
    ///
    /// Each file is loaded asynchronously, and all are evaluated
    /// in the order they complete.
    pub async fn eval_files(&self, paths: &[impl AsRef<Path>]) -> Vec<Result<Value, Error>> {
        let futures: Vec<_> = paths.iter().map(|p| self.eval_file(p)).collect();

        futures::future::join_all(futures).await
    }

    /// Evaluates multiple JavaScript files and returns results in order.
    ///
    /// Unlike `eval_files`, this preserves the original order of results.
    pub async fn eval_files_ordered(
        &self,
        paths: &[impl AsRef<Path>],
    ) -> Vec<Result<Value, Error>> {
        let mut results = Vec::with_capacity(paths.len());
        for path in paths {
            results.push(self.eval_file(path).await);
        }
        results
    }

    /// Gets a clone of the VM for inspection.
    pub async fn vm(&self) -> VM {
        self.vm.read().await.clone()
    }
}

#[cfg(feature = "async")]
impl Default for AsyncEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// A task executor for running JavaScript in a thread pool.
#[cfg(all(feature = "async", feature = "parallel"))]
pub struct ParallelExecutor {
    /// Thread pool for CPU-bound work
    pool: rayon::ThreadPool,
}

#[cfg(all(feature = "async", feature = "parallel"))]
impl ParallelExecutor {
    /// Creates a new parallel executor with the default number of threads.
    pub fn new() -> Self {
        Self {
            pool: rayon::ThreadPoolBuilder::new()
                .build()
                .expect("Failed to create thread pool"),
        }
    }

    /// Creates a parallel executor with a specific number of threads.
    pub fn with_threads(num_threads: usize) -> Self {
        Self {
            pool: rayon::ThreadPoolBuilder::new()
                .num_threads(num_threads)
                .build()
                .expect("Failed to create thread pool"),
        }
    }

    /// Compiles multiple source files in parallel.
    ///
    /// Returns compiled bytecode for each source.
    pub fn compile_parallel(
        &self,
        sources: &[&str],
    ) -> Vec<Result<crate::compiler::Bytecode, Error>> {
        use rayon::prelude::*;

        self.pool.install(|| {
            sources
                .par_iter()
                .map(|source| {
                    let mut parser = Parser::new(source);
                    let ast = parser.parse_program()?;
                    let mut compiler = Compiler::new();
                    compiler.compile(&ast)
                })
                .collect()
        })
    }

    /// Parses multiple source files in parallel.
    ///
    /// Returns ASTs for each source.
    pub fn parse_parallel(&self, sources: &[&str]) -> Vec<Result<crate::ast::Program, Error>> {
        use rayon::prelude::*;

        self.pool.install(|| {
            sources
                .par_iter()
                .map(|source| {
                    let mut parser = Parser::new(source);
                    parser.parse_program()
                })
                .collect()
        })
    }
}

#[cfg(all(feature = "async", feature = "parallel"))]
impl Default for ParallelExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(all(test, feature = "async"))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_eval() {
        let engine = AsyncEngine::new();
        let result = engine.eval("1 + 2;").await.unwrap();
        assert!(matches!(result, Value::Number(n) if n == 3.0));
    }

    #[tokio::test]
    async fn test_async_eval_string() {
        let engine = AsyncEngine::new();
        let result = engine.eval("\"hello\";").await.unwrap();
        assert!(matches!(result, Value::String(s) if s == "hello"));
    }
}
