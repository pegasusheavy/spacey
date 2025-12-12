//! The main parser implementation.

use crate::ast::*;
use crate::lexer::{Scanner, Token, TokenKind};
use crate::Error;

/// A recursive descent parser for JavaScript.
pub struct Parser<'a> {
    scanner: Scanner<'a>,
    current: Token,
    previous: Token,
}

impl<'a> Parser<'a> {
    /// Creates a new parser for the given source code.
    pub fn new(source: &'a str) -> Self {
        let mut scanner = Scanner::new(source);
        let current = scanner.next_token();
        Self {
            scanner,
            current,
            previous: Token::new(TokenKind::Eof, crate::lexer::Span::new(0, 0)),
        }
    }

    /// Parses the source code into a Program AST node.
    pub fn parse_program(&mut self) -> Result<Program, Error> {
        let mut body = Vec::new();

        while !self.is_at_end() {
            body.push(self.parse_statement()?);
        }

        Ok(Program { body })
    }

    /// Parses a single statement.
    pub fn parse_statement(&mut self) -> Result<Statement, Error> {
        match &self.current.kind {
            TokenKind::Var | TokenKind::Let | TokenKind::Const => {
                self.parse_variable_declaration()
            }
            TokenKind::Function => self.parse_function_declaration(),
            TokenKind::If => self.parse_if_statement(),
            TokenKind::While => self.parse_while_statement(),
            TokenKind::For => self.parse_for_statement(),
            TokenKind::Return => self.parse_return_statement(),
            TokenKind::LeftBrace => self.parse_block_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_variable_declaration(&mut self) -> Result<Statement, Error> {
        let kind = match &self.current.kind {
            TokenKind::Var => VariableKind::Var,
            TokenKind::Let => VariableKind::Let,
            TokenKind::Const => VariableKind::Const,
            _ => return Err(Error::SyntaxError("Expected variable keyword".into())),
        };
        self.advance();

        let mut declarations = Vec::new();

        loop {
            let id = self.expect_identifier()?;
            let init = if self.check(&TokenKind::Equal) {
                self.advance();
                Some(self.parse_expression()?)
            } else {
                None
            };

            declarations.push(VariableDeclarator { id, init });

            if !self.check(&TokenKind::Comma) {
                break;
            }
            self.advance();
        }

        self.expect(&TokenKind::Semicolon)?;

        Ok(Statement::VariableDeclaration(VariableDeclaration {
            kind,
            declarations,
        }))
    }

    fn parse_function_declaration(&mut self) -> Result<Statement, Error> {
        self.advance(); // consume 'function'

        let id = self.expect_identifier()?;
        self.expect(&TokenKind::LeftParen)?;

        let params = self.parse_parameters()?;

        self.expect(&TokenKind::RightParen)?;
        self.expect(&TokenKind::LeftBrace)?;

        let body = self.parse_function_body()?;

        self.expect(&TokenKind::RightBrace)?;

        Ok(Statement::FunctionDeclaration(FunctionDeclaration {
            id,
            params,
            body,
            is_async: false,
            is_generator: false,
        }))
    }

    fn parse_parameters(&mut self) -> Result<Vec<Identifier>, Error> {
        let mut params = Vec::new();

        if !self.check(&TokenKind::RightParen) {
            loop {
                params.push(self.expect_identifier()?);
                if !self.check(&TokenKind::Comma) {
                    break;
                }
                self.advance();
            }
        }

        Ok(params)
    }

    fn parse_function_body(&mut self) -> Result<Vec<Statement>, Error> {
        let mut body = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            body.push(self.parse_statement()?);
        }

        Ok(body)
    }

    fn parse_if_statement(&mut self) -> Result<Statement, Error> {
        self.advance(); // consume 'if'
        self.expect(&TokenKind::LeftParen)?;
        let test = self.parse_expression()?;
        self.expect(&TokenKind::RightParen)?;
        let consequent = Box::new(self.parse_statement()?);
        let alternate = if self.check(&TokenKind::Else) {
            self.advance();
            Some(Box::new(self.parse_statement()?))
        } else {
            None
        };

        Ok(Statement::If(IfStatement {
            test,
            consequent,
            alternate,
        }))
    }

    fn parse_while_statement(&mut self) -> Result<Statement, Error> {
        self.advance(); // consume 'while'
        self.expect(&TokenKind::LeftParen)?;
        let test = self.parse_expression()?;
        self.expect(&TokenKind::RightParen)?;
        let body = Box::new(self.parse_statement()?);

        Ok(Statement::While(WhileStatement { test, body }))
    }

    fn parse_for_statement(&mut self) -> Result<Statement, Error> {
        self.advance(); // consume 'for'
        self.expect(&TokenKind::LeftParen)?;

        // Parse init
        let init = if self.check(&TokenKind::Semicolon) {
            None
        } else if matches!(
            self.current.kind,
            TokenKind::Var | TokenKind::Let | TokenKind::Const
        ) {
            Some(ForInit::Declaration(Box::new(
                self.parse_variable_declaration_no_semi()?,
            )))
        } else {
            Some(ForInit::Expression(self.parse_expression()?))
        };
        self.expect(&TokenKind::Semicolon)?;

        // Parse test
        let test = if self.check(&TokenKind::Semicolon) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        self.expect(&TokenKind::Semicolon)?;

        // Parse update
        let update = if self.check(&TokenKind::RightParen) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        self.expect(&TokenKind::RightParen)?;

        let body = Box::new(self.parse_statement()?);

        Ok(Statement::For(ForStatement {
            init,
            test,
            update,
            body,
        }))
    }

    fn parse_variable_declaration_no_semi(&mut self) -> Result<VariableDeclaration, Error> {
        let kind = match &self.current.kind {
            TokenKind::Var => VariableKind::Var,
            TokenKind::Let => VariableKind::Let,
            TokenKind::Const => VariableKind::Const,
            _ => return Err(Error::SyntaxError("Expected variable keyword".into())),
        };
        self.advance();

        let mut declarations = Vec::new();

        loop {
            let id = self.expect_identifier()?;
            let init = if self.check(&TokenKind::Equal) {
                self.advance();
                Some(self.parse_expression()?)
            } else {
                None
            };

            declarations.push(VariableDeclarator { id, init });

            if !self.check(&TokenKind::Comma) {
                break;
            }
            self.advance();
        }

        Ok(VariableDeclaration { kind, declarations })
    }

    fn parse_return_statement(&mut self) -> Result<Statement, Error> {
        self.advance(); // consume 'return'
        let argument = if self.check(&TokenKind::Semicolon) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        self.expect(&TokenKind::Semicolon)?;

        Ok(Statement::Return(ReturnStatement { argument }))
    }

    fn parse_block_statement(&mut self) -> Result<Statement, Error> {
        self.advance(); // consume '{'
        let mut body = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            body.push(self.parse_statement()?);
        }

        self.expect(&TokenKind::RightBrace)?;

        Ok(Statement::Block(BlockStatement { body }))
    }

    fn parse_expression_statement(&mut self) -> Result<Statement, Error> {
        let expression = self.parse_expression()?;
        self.expect(&TokenKind::Semicolon)?;
        Ok(Statement::Expression(ExpressionStatement { expression }))
    }

    /// Parses an expression.
    pub fn parse_expression(&mut self) -> Result<Expression, Error> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<Expression, Error> {
        let expr = self.parse_logical_or()?;

        if self.check(&TokenKind::Equal) {
            self.advance();
            let value = self.parse_assignment()?;
            return Ok(Expression::Assignment(AssignmentExpression {
                operator: AssignmentOperator::Assign,
                left: Box::new(expr),
                right: Box::new(value),
            }));
        }

        Ok(expr)
    }

    fn parse_logical_or(&mut self) -> Result<Expression, Error> {
        let mut left = self.parse_logical_and()?;

        while self.check(&TokenKind::PipePipe) {
            self.advance();
            let right = self.parse_logical_and()?;
            left = Expression::Binary(BinaryExpression {
                operator: BinaryOperator::LogicalOr,
                left: Box::new(left),
                right: Box::new(right),
            });
        }

        Ok(left)
    }

    fn parse_logical_and(&mut self) -> Result<Expression, Error> {
        let mut left = self.parse_equality()?;

        while self.check(&TokenKind::AmpersandAmpersand) {
            self.advance();
            let right = self.parse_equality()?;
            left = Expression::Binary(BinaryExpression {
                operator: BinaryOperator::LogicalAnd,
                left: Box::new(left),
                right: Box::new(right),
            });
        }

        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expression, Error> {
        let mut left = self.parse_comparison()?;

        loop {
            let operator = match &self.current.kind {
                TokenKind::EqualEqual => BinaryOperator::Equal,
                TokenKind::NotEqual => BinaryOperator::NotEqual,
                TokenKind::StrictEqual => BinaryOperator::StrictEqual,
                TokenKind::StrictNotEqual => BinaryOperator::StrictNotEqual,
                _ => break,
            };
            self.advance();
            let right = self.parse_comparison()?;
            left = Expression::Binary(BinaryExpression {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            });
        }

        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expression, Error> {
        let mut left = self.parse_additive()?;

        loop {
            let operator = match &self.current.kind {
                TokenKind::LessThan => BinaryOperator::LessThan,
                TokenKind::LessThanEqual => BinaryOperator::LessThanEqual,
                TokenKind::GreaterThan => BinaryOperator::GreaterThan,
                TokenKind::GreaterThanEqual => BinaryOperator::GreaterThanEqual,
                _ => break,
            };
            self.advance();
            let right = self.parse_additive()?;
            left = Expression::Binary(BinaryExpression {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            });
        }

        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expression, Error> {
        let mut left = self.parse_multiplicative()?;

        loop {
            let operator = match &self.current.kind {
                TokenKind::Plus => BinaryOperator::Add,
                TokenKind::Minus => BinaryOperator::Subtract,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            left = Expression::Binary(BinaryExpression {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            });
        }

        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expression, Error> {
        let mut left = self.parse_unary()?;

        loop {
            let operator = match &self.current.kind {
                TokenKind::Star => BinaryOperator::Multiply,
                TokenKind::Slash => BinaryOperator::Divide,
                TokenKind::Percent => BinaryOperator::Modulo,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            left = Expression::Binary(BinaryExpression {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            });
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expression, Error> {
        let operator = match &self.current.kind {
            TokenKind::Bang => Some(UnaryOperator::LogicalNot),
            TokenKind::Minus => Some(UnaryOperator::Minus),
            TokenKind::Plus => Some(UnaryOperator::Plus),
            TokenKind::Typeof => Some(UnaryOperator::Typeof),
            TokenKind::Void => Some(UnaryOperator::Void),
            TokenKind::Delete => Some(UnaryOperator::Delete),
            _ => None,
        };

        if let Some(op) = operator {
            self.advance();
            let argument = self.parse_unary()?;
            return Ok(Expression::Unary(UnaryExpression {
                operator: op,
                argument: Box::new(argument),
            }));
        }

        self.parse_call()
    }

    fn parse_call(&mut self) -> Result<Expression, Error> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.check(&TokenKind::LeftParen) {
                self.advance();
                let arguments = self.parse_arguments()?;
                self.expect(&TokenKind::RightParen)?;
                expr = Expression::Call(CallExpression {
                    callee: Box::new(expr),
                    arguments,
                });
            } else if self.check(&TokenKind::Dot) {
                self.advance();
                let property = self.expect_identifier()?;
                expr = Expression::Member(MemberExpression {
                    object: Box::new(expr),
                    property: MemberProperty::Identifier(property),
                    computed: false,
                });
            } else if self.check(&TokenKind::LeftBracket) {
                self.advance();
                let property = self.parse_expression()?;
                self.expect(&TokenKind::RightBracket)?;
                expr = Expression::Member(MemberExpression {
                    object: Box::new(expr),
                    property: MemberProperty::Expression(Box::new(property)),
                    computed: true,
                });
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_arguments(&mut self) -> Result<Vec<Expression>, Error> {
        let mut args = Vec::new();

        if !self.check(&TokenKind::RightParen) {
            loop {
                args.push(self.parse_expression()?);
                if !self.check(&TokenKind::Comma) {
                    break;
                }
                self.advance();
            }
        }

        Ok(args)
    }

    fn parse_primary(&mut self) -> Result<Expression, Error> {
        match &self.current.kind {
            TokenKind::Number(n) => {
                let value = *n;
                self.advance();
                Ok(Expression::Literal(Literal::Number(value)))
            }
            TokenKind::String(s) => {
                let value = s.clone();
                self.advance();
                Ok(Expression::Literal(Literal::String(value)))
            }
            TokenKind::True => {
                self.advance();
                Ok(Expression::Literal(Literal::Boolean(true)))
            }
            TokenKind::False => {
                self.advance();
                Ok(Expression::Literal(Literal::Boolean(false)))
            }
            TokenKind::Null => {
                self.advance();
                Ok(Expression::Literal(Literal::Null))
            }
            TokenKind::Identifier(name) => {
                let id = Identifier { name: name.clone() };
                self.advance();
                Ok(Expression::Identifier(id))
            }
            TokenKind::This => {
                self.advance();
                Ok(Expression::This)
            }
            TokenKind::LeftParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(&TokenKind::RightParen)?;
                Ok(expr)
            }
            TokenKind::LeftBracket => self.parse_array_literal(),
            TokenKind::LeftBrace => self.parse_object_literal(),
            _ => Err(Error::SyntaxError(format!(
                "Unexpected token: {:?}",
                self.current.kind
            ))),
        }
    }

    fn parse_array_literal(&mut self) -> Result<Expression, Error> {
        self.advance(); // consume '['
        let mut elements = Vec::new();

        while !self.check(&TokenKind::RightBracket) && !self.is_at_end() {
            if self.check(&TokenKind::Comma) {
                elements.push(None); // Hole in array
            } else {
                elements.push(Some(self.parse_expression()?));
            }

            if !self.check(&TokenKind::RightBracket) {
                self.expect(&TokenKind::Comma)?;
            }
        }

        self.expect(&TokenKind::RightBracket)?;

        Ok(Expression::Array(ArrayExpression { elements }))
    }

    fn parse_object_literal(&mut self) -> Result<Expression, Error> {
        self.advance(); // consume '{'
        let mut properties = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            let key = self.expect_identifier()?;
            self.expect(&TokenKind::Colon)?;
            let value = self.parse_expression()?;

            properties.push(Property {
                key: PropertyKey::Identifier(key),
                value,
                shorthand: false,
            });

            if !self.check(&TokenKind::RightBrace) {
                self.expect(&TokenKind::Comma)?;
            }
        }

        self.expect(&TokenKind::RightBrace)?;

        Ok(Expression::Object(ObjectExpression { properties }))
    }

    // Helper methods

    fn advance(&mut self) {
        self.previous = std::mem::replace(&mut self.current, self.scanner.next_token());
    }

    fn check(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(&self.current.kind) == std::mem::discriminant(kind)
    }

    fn expect(&mut self, kind: &TokenKind) -> Result<(), Error> {
        if self.check(kind) {
            self.advance();
            Ok(())
        } else {
            Err(Error::SyntaxError(format!(
                "Expected {:?}, found {:?}",
                kind, self.current.kind
            )))
        }
    }

    fn expect_identifier(&mut self) -> Result<Identifier, Error> {
        if let TokenKind::Identifier(name) = &self.current.kind {
            let id = Identifier { name: name.clone() };
            self.advance();
            Ok(id)
        } else {
            Err(Error::SyntaxError(format!(
                "Expected identifier, found {:?}",
                self.current.kind
            )))
        }
    }

    fn is_at_end(&self) -> bool {
        matches!(self.current.kind, TokenKind::Eof)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_variable_declaration() {
        let mut parser = Parser::new("let x = 42;");
        let program = parser.parse_program().unwrap();
        assert_eq!(program.body.len(), 1);
    }

    #[test]
    fn test_parse_function_declaration() {
        let mut parser = Parser::new("function add(a, b) { return a + b; }");
        let program = parser.parse_program().unwrap();
        assert_eq!(program.body.len(), 1);
    }

    #[test]
    fn test_parse_binary_expression() {
        let mut parser = Parser::new("1 + 2 * 3;");
        let program = parser.parse_program().unwrap();
        assert_eq!(program.body.len(), 1);
    }
}


