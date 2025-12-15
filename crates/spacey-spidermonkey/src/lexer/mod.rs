//! Lexical analysis (tokenization) for JavaScript source code.
//!
//! The lexer transforms JavaScript source text into a stream of tokens
//! that can be consumed by the parser.
//!
//! ## Structure
//!
//! - `scanner.rs` - Main `Scanner` struct that produces tokens
//! - `token.rs` - `Token` and `TokenKind` definitions
//!
//! ## Documentation Submodules
//!
//! - `operators` - Multi-character operator scanning
//! - `literals` - Number, string, and identifier literals
//!
//! ## Usage
//!
//! ```rust
//! use spacey_spidermonkey::lexer::{Scanner, TokenKind};
//!
//! let mut scanner = Scanner::new("let x = 42;");
//!
//! loop {
//!     let token = scanner.next_token();
//!     if matches!(token.kind, TokenKind::Eof) {
//!         break;
//!     }
//!     println!("{:?}", token.kind);
//! }
//! ```

mod scanner;
mod token;

// Documentation and test submodules
pub mod literals;
pub mod operators;

pub use scanner::Scanner;
pub use token::{Span, Token, TokenKind};
