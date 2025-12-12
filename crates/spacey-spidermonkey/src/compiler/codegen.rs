//! Code generation from AST to bytecode.

use crate::ast::*;
use crate::compiler::bytecode::{Bytecode, Instruction, OpCode, Operand};
use crate::runtime::value::Value;
use crate::Error;

/// Compiles AST to bytecode.
pub struct Compiler {
    bytecode: Bytecode,
}

impl Compiler {
    /// Creates a new compiler.
    pub fn new() -> Self {
        Self {
            bytecode: Bytecode::new(),
        }
    }

    /// Compiles a program to bytecode.
    pub fn compile(&mut self, program: &Program) -> Result<Bytecode, Error> {
        for statement in &program.body {
            self.compile_statement(statement)?;
        }
        self.emit(Instruction::simple(OpCode::Halt));
        Ok(std::mem::take(&mut self.bytecode))
    }

    fn compile_statement(&mut self, stmt: &Statement) -> Result<(), Error> {
        match stmt {
            Statement::Expression(expr) => {
                self.compile_expression(&expr.expression)?;
                self.emit(Instruction::simple(OpCode::Pop));
            }
            Statement::Return(ret) => {
                if let Some(arg) = &ret.argument {
                    self.compile_expression(arg)?;
                } else {
                    self.emit(Instruction::simple(OpCode::LoadUndefined));
                }
                self.emit(Instruction::simple(OpCode::Return));
            }
            _ => {
                // TODO: Implement other statements
            }
        }
        Ok(())
    }

    fn compile_expression(&mut self, expr: &Expression) -> Result<(), Error> {
        match expr {
            Expression::Literal(lit) => self.compile_literal(lit),
            Expression::Binary(bin) => self.compile_binary(bin),
            Expression::Unary(un) => self.compile_unary(un),
            _ => {
                // TODO: Implement other expressions
                Ok(())
            }
        }
    }

    fn compile_literal(&mut self, lit: &Literal) -> Result<(), Error> {
        match lit {
            Literal::Number(n) => {
                let idx = self.bytecode.add_constant(Value::Number(*n));
                self.emit(Instruction::with_operand(OpCode::LoadConst, Operand::Constant(idx)));
            }
            Literal::String(s) => {
                let idx = self.bytecode.add_constant(Value::String(s.clone()));
                self.emit(Instruction::with_operand(OpCode::LoadConst, Operand::Constant(idx)));
            }
            Literal::Boolean(true) => {
                self.emit(Instruction::simple(OpCode::LoadTrue));
            }
            Literal::Boolean(false) => {
                self.emit(Instruction::simple(OpCode::LoadFalse));
            }
            Literal::Null => {
                self.emit(Instruction::simple(OpCode::LoadNull));
            }
            Literal::Undefined => {
                self.emit(Instruction::simple(OpCode::LoadUndefined));
            }
            _ => {}
        }
        Ok(())
    }

    fn compile_binary(&mut self, bin: &BinaryExpression) -> Result<(), Error> {
        self.compile_expression(&bin.left)?;
        self.compile_expression(&bin.right)?;

        let opcode = match bin.operator {
            BinaryOperator::Add => OpCode::Add,
            BinaryOperator::Subtract => OpCode::Sub,
            BinaryOperator::Multiply => OpCode::Mul,
            BinaryOperator::Divide => OpCode::Div,
            BinaryOperator::Modulo => OpCode::Mod,
            BinaryOperator::LessThan => OpCode::Lt,
            BinaryOperator::LessThanEqual => OpCode::Le,
            BinaryOperator::GreaterThan => OpCode::Gt,
            BinaryOperator::GreaterThanEqual => OpCode::Ge,
            BinaryOperator::Equal => OpCode::Eq,
            BinaryOperator::NotEqual => OpCode::Ne,
            BinaryOperator::StrictEqual => OpCode::StrictEq,
            BinaryOperator::StrictNotEqual => OpCode::StrictNe,
            _ => return Err(Error::InternalError("Unsupported operator".into())),
        };

        self.emit(Instruction::simple(opcode));
        Ok(())
    }

    fn compile_unary(&mut self, un: &UnaryExpression) -> Result<(), Error> {
        self.compile_expression(&un.argument)?;

        let opcode = match un.operator {
            UnaryOperator::Minus => OpCode::Neg,
            UnaryOperator::LogicalNot => OpCode::Not,
            UnaryOperator::BitwiseNot => OpCode::BitNot,
            _ => return Err(Error::InternalError("Unsupported operator".into())),
        };

        self.emit(Instruction::simple(opcode));
        Ok(())
    }

    fn emit(&mut self, instruction: Instruction) -> usize {
        self.bytecode.emit(instruction)
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}


