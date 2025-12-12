//! Token definitions for the JavaScript lexer.

/// A span in the source code, representing a range of characters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// Start byte offset (inclusive)
    pub start: usize,
    /// End byte offset (exclusive)
    pub end: usize,
}

impl Span {
    /// Creates a new span.
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Returns the length of this span in bytes.
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    /// Returns true if this span is empty.
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

/// A token produced by the lexer.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// The kind of token
    pub kind: TokenKind,
    /// The span in the source code
    pub span: Span,
}

impl Token {
    /// Creates a new token.
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

/// The different kinds of tokens in JavaScript.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literals
    /// Numeric literal (integer or floating point)
    Number(f64),
    /// BigInt literal
    BigInt(String),
    /// String literal
    String(String),
    /// Template literal part
    Template(String),
    /// Regular expression literal
    RegExp { pattern: String, flags: String },
    /// Boolean true
    True,
    /// Boolean false
    False,
    /// null
    Null,

    // Identifiers and Keywords
    /// Identifier
    Identifier(String),
    /// Private identifier (#name)
    PrivateIdentifier(String),

    // Keywords
    Await,
    Break,
    Case,
    Catch,
    Class,
    Const,
    Continue,
    Debugger,
    Default,
    Delete,
    Do,
    Else,
    Enum,
    Export,
    Extends,
    Finally,
    For,
    Function,
    If,
    Import,
    In,
    Instanceof,
    Let,
    New,
    Return,
    Static,
    Super,
    Switch,
    This,
    Throw,
    Try,
    Typeof,
    Var,
    Void,
    While,
    With,
    Yield,
    Async,

    // Punctuation
    /// {
    LeftBrace,
    /// }
    RightBrace,
    /// (
    LeftParen,
    /// )
    RightParen,
    /// [
    LeftBracket,
    /// ]
    RightBracket,
    /// .
    Dot,
    /// ...
    Ellipsis,
    /// ;
    Semicolon,
    /// ,
    Comma,
    /// <
    LessThan,
    /// >
    GreaterThan,
    /// <=
    LessThanEqual,
    /// >=
    GreaterThanEqual,
    /// ==
    EqualEqual,
    /// !=
    NotEqual,
    /// ===
    StrictEqual,
    /// !==
    StrictNotEqual,
    /// +
    Plus,
    /// -
    Minus,
    /// *
    Star,
    /// /
    Slash,
    /// %
    Percent,
    /// **
    StarStar,
    /// ++
    PlusPlus,
    /// --
    MinusMinus,
    /// <<
    LeftShift,
    /// >>
    RightShift,
    /// >>>
    UnsignedRightShift,
    /// &
    Ampersand,
    /// |
    Pipe,
    /// ^
    Caret,
    /// !
    Bang,
    /// ~
    Tilde,
    /// &&
    AmpersandAmpersand,
    /// ||
    PipePipe,
    /// ??
    QuestionQuestion,
    /// ?
    Question,
    /// ?.
    QuestionDot,
    /// :
    Colon,
    /// =
    Equal,
    /// +=
    PlusEqual,
    /// -=
    MinusEqual,
    /// *=
    StarEqual,
    /// /=
    SlashEqual,
    /// %=
    PercentEqual,
    /// **=
    StarStarEqual,
    /// <<=
    LeftShiftEqual,
    /// >>=
    RightShiftEqual,
    /// >>>=
    UnsignedRightShiftEqual,
    /// &=
    AmpersandEqual,
    /// |=
    PipeEqual,
    /// ^=
    CaretEqual,
    /// &&=
    AmpersandAmpersandEqual,
    /// ||=
    PipePipeEqual,
    /// ??=
    QuestionQuestionEqual,
    /// =>
    Arrow,

    // Special
    /// End of file
    Eof,
    /// Invalid token (for error recovery)
    Invalid,
}

impl TokenKind {
    /// Returns true if this token is a keyword.
    pub fn is_keyword(&self) -> bool {
        matches!(
            self,
            TokenKind::Await
                | TokenKind::Break
                | TokenKind::Case
                | TokenKind::Catch
                | TokenKind::Class
                | TokenKind::Const
                | TokenKind::Continue
                | TokenKind::Debugger
                | TokenKind::Default
                | TokenKind::Delete
                | TokenKind::Do
                | TokenKind::Else
                | TokenKind::Enum
                | TokenKind::Export
                | TokenKind::Extends
                | TokenKind::Finally
                | TokenKind::For
                | TokenKind::Function
                | TokenKind::If
                | TokenKind::Import
                | TokenKind::In
                | TokenKind::Instanceof
                | TokenKind::Let
                | TokenKind::New
                | TokenKind::Return
                | TokenKind::Static
                | TokenKind::Super
                | TokenKind::Switch
                | TokenKind::This
                | TokenKind::Throw
                | TokenKind::Try
                | TokenKind::Typeof
                | TokenKind::Var
                | TokenKind::Void
                | TokenKind::While
                | TokenKind::With
                | TokenKind::Yield
                | TokenKind::Async
        )
    }

    /// Returns true if this token is a literal.
    pub fn is_literal(&self) -> bool {
        matches!(
            self,
            TokenKind::Number(_)
                | TokenKind::BigInt(_)
                | TokenKind::String(_)
                | TokenKind::Template(_)
                | TokenKind::RegExp { .. }
                | TokenKind::True
                | TokenKind::False
                | TokenKind::Null
        )
    }
}


