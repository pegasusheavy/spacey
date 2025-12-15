//! TypeScript support for spacey-node.
//!
//! This module provides TypeScript transpilation using SWC, supporting both
//! transpile-only (fast) and type-checking modes.
//!
//! ## Features
//!
//! - Fast SWC-based transpilation
//! - tsconfig.json support
//! - JSX/TSX support
//! - Source map generation
//! - Optional type checking
//!
//! ## Usage
//!
//! ```rust,ignore
//! use spacey_node::typescript::{TypeScriptTranspiler, TsConfig};
//!
//! let config = TsConfig::load("./tsconfig.json")?;
//! let transpiler = TypeScriptTranspiler::new(config);
//!
//! let output = transpiler.transpile("const x: number = 42;", "file.ts")?;
//! println!("{}", output.code);
//! ```

mod config;
mod source_map;
mod transpiler;

pub use config::{CompilerOptions, JsxMode, ModuleKind, TsConfig, TsTarget};
pub use source_map::SourceMapSupport;
pub use transpiler::{TranspileOutput, TypeScriptTranspiler};

/// Check if a file extension indicates a TypeScript file.
pub fn is_typescript_file(path: &std::path::Path) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some("ts" | "tsx" | "mts" | "cts") => true,
        _ => false,
    }
}

/// Check if a file extension indicates a JSX/TSX file.
pub fn is_jsx_file(path: &std::path::Path) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some("jsx" | "tsx") => true,
        _ => false,
    }
}

/// TypeScript file extensions in resolution order.
pub const TS_EXTENSIONS: &[&str] = &[".ts", ".tsx", ".mts", ".cts"];

/// All supported extensions including JavaScript.
pub const ALL_EXTENSIONS: &[&str] = &[
    ".ts", ".tsx", ".mts", ".cts", ".js", ".jsx", ".mjs", ".cjs",
];

