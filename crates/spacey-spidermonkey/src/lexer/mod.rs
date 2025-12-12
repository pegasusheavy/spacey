//! Lexical analysis (tokenization) for JavaScript source code.
//!
//! The lexer transforms JavaScript source text into a stream of tokens
//! that can be consumed by the parser.

mod token;
mod scanner;

pub use token::{Token, TokenKind, Span};
pub use scanner::Scanner;


