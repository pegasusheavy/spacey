//! The scanner that produces tokens from source text.

use super::{Span, Token, TokenKind};

/// A scanner that tokenizes JavaScript source code.
pub struct Scanner<'a> {
    source: &'a str,
    chars: std::iter::Peekable<std::str::CharIndices<'a>>,
    current_pos: usize,
}

impl<'a> Scanner<'a> {
    /// Creates a new scanner for the given source code.
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.char_indices().peekable(),
            current_pos: 0,
        }
    }

    /// Returns the next token from the source.
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace_and_comments();

        let start = self.current_pos;

        let Some((_pos, ch)) = self.advance() else {
            return Token::new(TokenKind::Eof, Span::new(start, start));
        };

        let kind = match ch {
            // Single-character tokens
            '{' => TokenKind::LeftBrace,
            '}' => TokenKind::RightBrace,
            '(' => TokenKind::LeftParen,
            ')' => TokenKind::RightParen,
            '[' => TokenKind::LeftBracket,
            ']' => TokenKind::RightBracket,
            ';' => TokenKind::Semicolon,
            ',' => TokenKind::Comma,
            ':' => TokenKind::Colon,
            '~' => TokenKind::Tilde,

            // Multi-character tokens
            '.' => self.scan_dot(),
            '+' => self.scan_plus(),
            '-' => self.scan_minus(),
            '*' => self.scan_star(),
            '/' => self.scan_slash(),
            '%' => self.scan_percent(),
            '<' => self.scan_less_than(),
            '>' => self.scan_greater_than(),
            '=' => self.scan_equal(),
            '!' => self.scan_bang(),
            '&' => self.scan_ampersand(),
            '|' => self.scan_pipe(),
            '^' => self.scan_caret(),
            '?' => self.scan_question(),

            // String literals
            '"' | '\'' => self.scan_string(ch),

            // Template literals
            '`' => self.scan_template(),

            // Numbers
            '0'..='9' => self.scan_number(ch),

            // Identifiers and keywords
            _ if is_id_start(ch) => self.scan_identifier(ch),

            // Private identifiers
            '#' => self.scan_private_identifier(),

            _ => TokenKind::Invalid,
        };

        Token::new(kind, Span::new(start, self.current_pos))
    }

    fn advance(&mut self) -> Option<(usize, char)> {
        let result = self.chars.next();
        if let Some((pos, ch)) = result {
            self.current_pos = pos + ch.len_utf8();
        }
        result
    }

    fn peek(&mut self) -> Option<char> {
        self.chars.peek().map(|(_, ch)| *ch)
    }

    fn peek_next(&self) -> Option<char> {
        let mut iter = self.chars.clone();
        iter.next();
        iter.next().map(|(_, ch)| ch)
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            match self.peek() {
                Some(' ' | '\t' | '\n' | '\r') => {
                    self.advance();
                }
                Some('/') => {
                    match self.peek_next() {
                        Some('/') => {
                            // Single-line comment: skip until end of line
                            self.advance(); // consume first '/'
                            self.advance(); // consume second '/'
                            while let Some(ch) = self.peek() {
                                if ch == '\n' || ch == '\r' {
                                    break;
                                }
                                self.advance();
                            }
                        }
                        Some('*') => {
                            // Multi-line comment: skip until */
                            self.advance(); // consume '/'
                            self.advance(); // consume '*'
                            let mut prev = ' ';
                            while let Some(ch) = self.peek() {
                                self.advance();
                                if prev == '*' && ch == '/' {
                                    break;
                                }
                                prev = ch;
                            }
                        }
                        _ => break, // Not a comment, it's a division operator
                    }
                }
                _ => break,
            }
        }
    }

    fn scan_dot(&mut self) -> TokenKind {
        if self.peek() == Some('.') {
            self.advance();
            if self.peek() == Some('.') {
                self.advance();
                TokenKind::Ellipsis
            } else {
                // Invalid: ".." is not valid
                TokenKind::Invalid
            }
        } else {
            TokenKind::Dot
        }
    }

    fn scan_plus(&mut self) -> TokenKind {
        match self.peek() {
            Some('+') => {
                self.advance();
                TokenKind::PlusPlus
            }
            Some('=') => {
                self.advance();
                TokenKind::PlusEqual
            }
            _ => TokenKind::Plus,
        }
    }

    fn scan_minus(&mut self) -> TokenKind {
        match self.peek() {
            Some('-') => {
                self.advance();
                TokenKind::MinusMinus
            }
            Some('=') => {
                self.advance();
                TokenKind::MinusEqual
            }
            _ => TokenKind::Minus,
        }
    }

    fn scan_star(&mut self) -> TokenKind {
        match self.peek() {
            Some('*') => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::StarStarEqual
                } else {
                    TokenKind::StarStar
                }
            }
            Some('=') => {
                self.advance();
                TokenKind::StarEqual
            }
            _ => TokenKind::Star,
        }
    }

    fn scan_slash(&mut self) -> TokenKind {
        match self.peek() {
            Some('=') => {
                self.advance();
                TokenKind::SlashEqual
            }
            _ => TokenKind::Slash,
        }
    }

    fn scan_percent(&mut self) -> TokenKind {
        if self.peek() == Some('=') {
            self.advance();
            TokenKind::PercentEqual
        } else {
            TokenKind::Percent
        }
    }

    fn scan_less_than(&mut self) -> TokenKind {
        match self.peek() {
            Some('<') => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::LeftShiftEqual
                } else {
                    TokenKind::LeftShift
                }
            }
            Some('=') => {
                self.advance();
                TokenKind::LessThanEqual
            }
            _ => TokenKind::LessThan,
        }
    }

    fn scan_greater_than(&mut self) -> TokenKind {
        match self.peek() {
            Some('>') => {
                self.advance();
                match self.peek() {
                    Some('>') => {
                        self.advance();
                        if self.peek() == Some('=') {
                            self.advance();
                            TokenKind::UnsignedRightShiftEqual
                        } else {
                            TokenKind::UnsignedRightShift
                        }
                    }
                    Some('=') => {
                        self.advance();
                        TokenKind::RightShiftEqual
                    }
                    _ => TokenKind::RightShift,
                }
            }
            Some('=') => {
                self.advance();
                TokenKind::GreaterThanEqual
            }
            _ => TokenKind::GreaterThan,
        }
    }

    fn scan_equal(&mut self) -> TokenKind {
        match self.peek() {
            Some('=') => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::StrictEqual
                } else {
                    TokenKind::EqualEqual
                }
            }
            Some('>') => {
                self.advance();
                TokenKind::Arrow
            }
            _ => TokenKind::Equal,
        }
    }

    fn scan_bang(&mut self) -> TokenKind {
        match self.peek() {
            Some('=') => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::StrictNotEqual
                } else {
                    TokenKind::NotEqual
                }
            }
            _ => TokenKind::Bang,
        }
    }

    fn scan_ampersand(&mut self) -> TokenKind {
        match self.peek() {
            Some('&') => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::AmpersandAmpersandEqual
                } else {
                    TokenKind::AmpersandAmpersand
                }
            }
            Some('=') => {
                self.advance();
                TokenKind::AmpersandEqual
            }
            _ => TokenKind::Ampersand,
        }
    }

    fn scan_pipe(&mut self) -> TokenKind {
        match self.peek() {
            Some('|') => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::PipePipeEqual
                } else {
                    TokenKind::PipePipe
                }
            }
            Some('=') => {
                self.advance();
                TokenKind::PipeEqual
            }
            _ => TokenKind::Pipe,
        }
    }

    fn scan_caret(&mut self) -> TokenKind {
        if self.peek() == Some('=') {
            self.advance();
            TokenKind::CaretEqual
        } else {
            TokenKind::Caret
        }
    }

    fn scan_question(&mut self) -> TokenKind {
        match self.peek() {
            Some('?') => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::QuestionQuestionEqual
                } else {
                    TokenKind::QuestionQuestion
                }
            }
            Some('.') => {
                self.advance();
                TokenKind::QuestionDot
            }
            _ => TokenKind::Question,
        }
    }

    fn scan_string(&mut self, quote: char) -> TokenKind {
        let mut value = String::new();

        loop {
            match self.advance() {
                None => return TokenKind::Invalid, // Unterminated string
                Some((_, ch)) if ch == quote => break,
                Some((_, '\\')) => {
                    // Handle escape sequences
                    if let Some((_, escaped)) = self.advance() {
                        match escaped {
                            'n' => value.push('\n'),
                            'r' => value.push('\r'),
                            't' => value.push('\t'),
                            '\\' => value.push('\\'),
                            '\'' => value.push('\''),
                            '"' => value.push('"'),
                            '0' => value.push('\0'),
                            // TODO: Unicode escapes, hex escapes
                            _ => value.push(escaped),
                        }
                    }
                }
                Some((_, ch)) => value.push(ch),
            }
        }

        TokenKind::String(value)
    }

    fn scan_template(&mut self) -> TokenKind {
        let mut value = String::new();

        loop {
            match self.advance() {
                None => return TokenKind::Invalid, // Unterminated template
                Some((_, '`')) => break,
                Some((_, '$')) if self.peek() == Some('{') => {
                    // TODO: Handle template expressions
                    self.advance();
                    return TokenKind::Template(value);
                }
                Some((_, '\\')) => {
                    if let Some((_, escaped)) = self.advance() {
                        match escaped {
                            'n' => value.push('\n'),
                            'r' => value.push('\r'),
                            't' => value.push('\t'),
                            '\\' => value.push('\\'),
                            '`' => value.push('`'),
                            '$' => value.push('$'),
                            _ => value.push(escaped),
                        }
                    }
                }
                Some((_, ch)) => value.push(ch),
            }
        }

        TokenKind::Template(value)
    }

    fn scan_number(&mut self, first: char) -> TokenKind {
        let mut value = String::from(first);

        // Handle hex, octal, binary
        if first == '0' {
            match self.peek() {
                Some('x' | 'X') => return self.scan_hex_number(),
                Some('o' | 'O') => return self.scan_octal_number(),
                Some('b' | 'B') => return self.scan_binary_number(),
                _ => {}
            }
        }

        // Integer part
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() || ch == '_' {
                if ch != '_' {
                    value.push(ch);
                }
                self.advance();
            } else {
                break;
            }
        }

        // Fractional part
        if self.peek() == Some('.') {
            value.push('.');
            self.advance();
            while let Some(ch) = self.peek() {
                if ch.is_ascii_digit() || ch == '_' {
                    if ch != '_' {
                        value.push(ch);
                    }
                    self.advance();
                } else {
                    break;
                }
            }
        }

        // Exponent part
        if matches!(self.peek(), Some('e' | 'E')) {
            value.push('e');
            self.advance();
            if matches!(self.peek(), Some('+' | '-')) {
                value.push(self.advance().unwrap().1);
            }
            while let Some(ch) = self.peek() {
                if ch.is_ascii_digit() {
                    value.push(ch);
                    self.advance();
                } else {
                    break;
                }
            }
        }

        // BigInt suffix
        if self.peek() == Some('n') {
            self.advance();
            return TokenKind::BigInt(value);
        }

        match value.parse::<f64>() {
            Ok(n) => TokenKind::Number(n),
            Err(_) => TokenKind::Invalid,
        }
    }

    fn scan_hex_number(&mut self) -> TokenKind {
        self.advance(); // consume 'x'
        let mut value = String::new();

        while let Some(ch) = self.peek() {
            if ch.is_ascii_hexdigit() || ch == '_' {
                if ch != '_' {
                    value.push(ch);
                }
                self.advance();
            } else {
                break;
            }
        }

        if self.peek() == Some('n') {
            self.advance();
            return TokenKind::BigInt(format!("0x{}", value));
        }

        match u64::from_str_radix(&value, 16) {
            Ok(n) => TokenKind::Number(n as f64),
            Err(_) => TokenKind::Invalid,
        }
    }

    fn scan_octal_number(&mut self) -> TokenKind {
        self.advance(); // consume 'o'
        let mut value = String::new();

        while let Some(ch) = self.peek() {
            if ch.is_digit(8) || ch == '_' {
                if ch != '_' {
                    value.push(ch);
                }
                self.advance();
            } else {
                break;
            }
        }

        if self.peek() == Some('n') {
            self.advance();
            return TokenKind::BigInt(format!("0o{}", value));
        }

        match u64::from_str_radix(&value, 8) {
            Ok(n) => TokenKind::Number(n as f64),
            Err(_) => TokenKind::Invalid,
        }
    }

    fn scan_binary_number(&mut self) -> TokenKind {
        self.advance(); // consume 'b'
        let mut value = String::new();

        while let Some(ch) = self.peek() {
            if ch == '0' || ch == '1' || ch == '_' {
                if ch != '_' {
                    value.push(ch);
                }
                self.advance();
            } else {
                break;
            }
        }

        if self.peek() == Some('n') {
            self.advance();
            return TokenKind::BigInt(format!("0b{}", value));
        }

        match u64::from_str_radix(&value, 2) {
            Ok(n) => TokenKind::Number(n as f64),
            Err(_) => TokenKind::Invalid,
        }
    }

    fn scan_identifier(&mut self, first: char) -> TokenKind {
        let mut name = String::from(first);

        while let Some(ch) = self.peek() {
            if is_id_continue(ch) {
                name.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        // Check for keywords
        match name.as_str() {
            "await" => TokenKind::Await,
            "break" => TokenKind::Break,
            "case" => TokenKind::Case,
            "catch" => TokenKind::Catch,
            "class" => TokenKind::Class,
            "const" => TokenKind::Const,
            "continue" => TokenKind::Continue,
            "debugger" => TokenKind::Debugger,
            "default" => TokenKind::Default,
            "delete" => TokenKind::Delete,
            "do" => TokenKind::Do,
            "else" => TokenKind::Else,
            "enum" => TokenKind::Enum,
            "export" => TokenKind::Export,
            "extends" => TokenKind::Extends,
            "false" => TokenKind::False,
            "finally" => TokenKind::Finally,
            "for" => TokenKind::For,
            "function" => TokenKind::Function,
            "if" => TokenKind::If,
            "import" => TokenKind::Import,
            "in" => TokenKind::In,
            "instanceof" => TokenKind::Instanceof,
            "let" => TokenKind::Let,
            "new" => TokenKind::New,
            "null" => TokenKind::Null,
            "return" => TokenKind::Return,
            "static" => TokenKind::Static,
            "super" => TokenKind::Super,
            "switch" => TokenKind::Switch,
            "this" => TokenKind::This,
            "throw" => TokenKind::Throw,
            "true" => TokenKind::True,
            "try" => TokenKind::Try,
            "typeof" => TokenKind::Typeof,
            "var" => TokenKind::Var,
            "void" => TokenKind::Void,
            "while" => TokenKind::While,
            "with" => TokenKind::With,
            "yield" => TokenKind::Yield,
            "async" => TokenKind::Async,
            _ => TokenKind::Identifier(name),
        }
    }

    fn scan_private_identifier(&mut self) -> TokenKind {
        let mut name = String::new();

        while let Some(ch) = self.peek() {
            if is_id_continue(ch) {
                name.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        if name.is_empty() {
            TokenKind::Invalid
        } else {
            TokenKind::PrivateIdentifier(name)
        }
    }
}

/// Checks if a character can start an identifier.
fn is_id_start(ch: char) -> bool {
    ch == '_' || ch == '$' || unicode_xid::UnicodeXID::is_xid_start(ch)
}

/// Checks if a character can continue an identifier.
fn is_id_continue(ch: char) -> bool {
    ch == '_' || ch == '$' || unicode_xid::UnicodeXID::is_xid_continue(ch)
}

impl<'a> Iterator for Scanner<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.next_token();
        if token.kind == TokenKind::Eof {
            None
        } else {
            Some(token)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_tokens() {
        let mut scanner = Scanner::new("{ } ( )");
        assert!(matches!(scanner.next_token().kind, TokenKind::LeftBrace));
        assert!(matches!(scanner.next_token().kind, TokenKind::RightBrace));
        assert!(matches!(scanner.next_token().kind, TokenKind::LeftParen));
        assert!(matches!(scanner.next_token().kind, TokenKind::RightParen));
    }

    #[test]
    fn test_numbers() {
        let mut scanner = Scanner::new("42 3.14 0xff 0b1010");
        assert!(matches!(scanner.next_token().kind, TokenKind::Number(n) if n == 42.0));
        assert!(matches!(scanner.next_token().kind, TokenKind::Number(n) if n == 3.14));
        assert!(matches!(scanner.next_token().kind, TokenKind::Number(n) if n == 255.0));
        assert!(matches!(scanner.next_token().kind, TokenKind::Number(n) if n == 10.0));
    }

    #[test]
    fn test_strings() {
        let mut scanner = Scanner::new(r#""hello" 'world'"#);
        assert!(matches!(scanner.next_token().kind, TokenKind::String(s) if s == "hello"));
        assert!(matches!(scanner.next_token().kind, TokenKind::String(s) if s == "world"));
    }

    #[test]
    fn test_keywords() {
        let mut scanner = Scanner::new("function const let var");
        assert!(matches!(scanner.next_token().kind, TokenKind::Function));
        assert!(matches!(scanner.next_token().kind, TokenKind::Const));
        assert!(matches!(scanner.next_token().kind, TokenKind::Let));
        assert!(matches!(scanner.next_token().kind, TokenKind::Var));
    }

    #[test]
    fn test_identifiers() {
        let mut scanner = Scanner::new("foo _bar $baz");
        assert!(matches!(scanner.next_token().kind, TokenKind::Identifier(s) if s == "foo"));
        assert!(matches!(scanner.next_token().kind, TokenKind::Identifier(s) if s == "_bar"));
        assert!(matches!(scanner.next_token().kind, TokenKind::Identifier(s) if s == "$baz"));
    }

    #[test]
    fn test_single_line_comments() {
        let mut scanner = Scanner::new("42 // this is a comment\n43");
        assert!(matches!(scanner.next_token().kind, TokenKind::Number(n) if n == 42.0));
        assert!(matches!(scanner.next_token().kind, TokenKind::Number(n) if n == 43.0));
    }

    #[test]
    fn test_multi_line_comments() {
        let mut scanner = Scanner::new("1 /* comment */ 2 /* multi\nline\ncomment */ 3");
        assert!(matches!(scanner.next_token().kind, TokenKind::Number(n) if n == 1.0));
        assert!(matches!(scanner.next_token().kind, TokenKind::Number(n) if n == 2.0));
        assert!(matches!(scanner.next_token().kind, TokenKind::Number(n) if n == 3.0));
    }

    #[test]
    fn test_division_vs_comment() {
        let mut scanner = Scanner::new("6 / 2");
        assert!(matches!(scanner.next_token().kind, TokenKind::Number(n) if n == 6.0));
        assert!(matches!(scanner.next_token().kind, TokenKind::Slash));
        assert!(matches!(scanner.next_token().kind, TokenKind::Number(n) if n == 2.0));
    }
}
