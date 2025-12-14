//! Code generation from AST to bytecode.

use crate::Error;
use crate::ast::*;
use crate::compiler::bytecode::{Bytecode, Instruction, OpCode, Operand};
use crate::runtime::value::Value;

/// A local variable in a scope.
#[derive(Debug, Clone)]
struct Local {
    /// The variable name
    name: String,
    /// The scope depth where this was declared
    depth: usize,
    /// Whether the variable is mutable (var/let vs const)
    mutable: bool,
    /// Whether the variable has been initialized
    initialized: bool,
}

/// A scope for variable resolution.
#[derive(Debug, Default)]
struct Scope {
    /// Local variables in this scope
    locals: Vec<Local>,
    /// Current scope depth (0 = global)
    depth: usize,
}

impl Scope {
    fn new() -> Self {
        Self {
            locals: Vec::new(),
            depth: 0,
        }
    }

    /// Begin a new scope.
    fn begin_scope(&mut self) {
        self.depth += 1;
    }

    /// End the current scope and return the number of locals to pop.
    fn end_scope(&mut self) -> usize {
        let mut count = 0;
        while !self.locals.is_empty() && self.locals.last().unwrap().depth == self.depth {
            self.locals.pop();
            count += 1;
        }
        self.depth -= 1;
        count
    }

    /// Declare a local variable.
    fn declare(&mut self, name: String, mutable: bool) -> Result<usize, Error> {
        // Check for duplicate in same scope
        for local in self.locals.iter().rev() {
            if local.depth < self.depth {
                break;
            }
            if local.name == name {
                return Err(Error::SyntaxError(format!(
                    "Variable '{}' already declared in this scope",
                    name
                )));
            }
        }

        let index = self.locals.len();
        self.locals.push(Local {
            name,
            depth: self.depth,
            mutable,
            initialized: false,
        });
        Ok(index)
    }

    /// Mark a variable as initialized.
    fn mark_initialized(&mut self, index: usize) {
        if index < self.locals.len() {
            self.locals[index].initialized = true;
        }
    }

    /// Resolve a local variable by name, returning its index.
    fn resolve(&self, name: &str) -> Option<usize> {
        for (i, local) in self.locals.iter().enumerate().rev() {
            if local.name == name {
                return Some(i);
            }
        }
        None
    }

    /// Check if a variable is a local (vs global).
    fn is_local(&self, name: &str) -> bool {
        self.resolve(name).is_some()
    }
}

/// Compiles AST to bytecode.
pub struct Compiler {
    bytecode: Bytecode,
    scope: Scope,
}

impl Compiler {
    /// Creates a new compiler.
    pub fn new() -> Self {
        Self {
            bytecode: Bytecode::new(),
            scope: Scope::new(),
        }
    }

    /// Compiles a program to bytecode.
    ///
    /// For REPL/eval, keeps the last expression value on the stack.
    pub fn compile(&mut self, program: &Program) -> Result<Bytecode, Error> {
        let len = program.body.len();

        for (i, statement) in program.body.iter().enumerate() {
            let is_last = i == len - 1;
            self.compile_statement(statement, is_last)?;
        }

        // If program is empty, push undefined as result
        if program.body.is_empty() {
            self.emit(Instruction::simple(OpCode::LoadUndefined));
        }

        self.emit(Instruction::simple(OpCode::Halt));
        Ok(std::mem::take(&mut self.bytecode))
    }

    fn compile_statement(&mut self, stmt: &Statement, keep_value: bool) -> Result<(), Error> {
        match stmt {
            Statement::Expression(expr) => {
                self.compile_expression(&expr.expression)?;
                // Only pop if not the last statement (for REPL eval)
                if !keep_value {
                    self.emit(Instruction::simple(OpCode::Pop));
                }
            }
            Statement::Return(ret) => {
                if let Some(arg) = &ret.argument {
                    self.compile_expression(arg)?;
                } else {
                    self.emit(Instruction::simple(OpCode::LoadUndefined));
                }
                self.emit(Instruction::simple(OpCode::Return));
            }
            Statement::VariableDeclaration(decl) => {
                self.compile_variable_declaration(decl)?;
                // Variable declarations produce undefined
                if keep_value {
                    self.emit(Instruction::simple(OpCode::LoadUndefined));
                }
            }
            Statement::Block(block) => {
                let block_len = block.body.len();
                for (i, stmt) in block.body.iter().enumerate() {
                    let is_last = keep_value && i == block_len - 1;
                    self.compile_statement(stmt, is_last)?;
                }
                if block.body.is_empty() && keep_value {
                    self.emit(Instruction::simple(OpCode::LoadUndefined));
                }
            }
            Statement::If(if_stmt) => {
                self.compile_if_statement(if_stmt, keep_value)?;
            }
            Statement::While(while_stmt) => {
                self.compile_while_statement(while_stmt)?;
                if keep_value {
                    self.emit(Instruction::simple(OpCode::LoadUndefined));
                }
            }
            Statement::For(for_stmt) => {
                self.compile_for_statement(for_stmt)?;
                if keep_value {
                    self.emit(Instruction::simple(OpCode::LoadUndefined));
                }
            }
            Statement::Empty => {
                if keep_value {
                    self.emit(Instruction::simple(OpCode::LoadUndefined));
                }
            }
            _ => {
                // TODO: Implement remaining statements
                if keep_value {
                    self.emit(Instruction::simple(OpCode::LoadUndefined));
                }
            }
        }
        Ok(())
    }

    fn compile_variable_declaration(&mut self, decl: &VariableDeclaration) -> Result<(), Error> {
        let mutable = decl.kind != VariableKind::Const;

        for declarator in &decl.declarations {
            // Declare the variable
            let index = self.scope.declare(declarator.id.name.clone(), mutable)?;

            // Compile initializer if present
            if let Some(init) = &declarator.init {
                self.compile_expression(init)?;
            } else {
                self.emit(Instruction::simple(OpCode::LoadUndefined));
            }

            // Store to local
            self.emit(Instruction::with_operand(
                OpCode::StoreLocal,
                Operand::Local(index as u16),
            ));

            // Mark as initialized
            self.scope.mark_initialized(index);
        }

        Ok(())
    }

    fn compile_if_statement(
        &mut self,
        if_stmt: &IfStatement,
        keep_value: bool,
    ) -> Result<(), Error> {
        // Compile condition
        self.compile_expression(&if_stmt.test)?;

        // Jump to else/end if false
        let jump_to_else = self.emit(Instruction::with_operand(
            OpCode::JumpIfFalse,
            Operand::Jump(0), // Placeholder
        ));

        // Compile then branch
        self.compile_statement(&if_stmt.consequent, keep_value)?;

        if let Some(alternate) = &if_stmt.alternate {
            // Jump over else branch
            let jump_to_end = self.emit(Instruction::with_operand(
                OpCode::Jump,
                Operand::Jump(0), // Placeholder
            ));

            // Patch jump to else
            let else_pos = self.bytecode.instructions.len() as i32;
            self.bytecode.instructions[jump_to_else].operand = Some(Operand::Jump(else_pos));

            // Compile else branch
            self.compile_statement(alternate, keep_value)?;

            // Patch jump to end
            let end_pos = self.bytecode.instructions.len() as i32;
            self.bytecode.instructions[jump_to_end].operand = Some(Operand::Jump(end_pos));
        } else {
            // No else branch
            if keep_value {
                // Jump over the undefined push
                let jump_to_end =
                    self.emit(Instruction::with_operand(OpCode::Jump, Operand::Jump(0)));

                // Patch jump to else (which is the undefined push)
                let else_pos = self.bytecode.instructions.len() as i32;
                self.bytecode.instructions[jump_to_else].operand = Some(Operand::Jump(else_pos));

                // Push undefined for when condition is false
                self.emit(Instruction::simple(OpCode::LoadUndefined));

                // Patch jump to end
                let end_pos = self.bytecode.instructions.len() as i32;
                self.bytecode.instructions[jump_to_end].operand = Some(Operand::Jump(end_pos));
            } else {
                // Patch jump to end
                let end_pos = self.bytecode.instructions.len() as i32;
                self.bytecode.instructions[jump_to_else].operand = Some(Operand::Jump(end_pos));
            }
        }

        Ok(())
    }

    fn compile_while_statement(&mut self, while_stmt: &WhileStatement) -> Result<(), Error> {
        let loop_start = self.bytecode.instructions.len();

        // Compile condition
        self.compile_expression(&while_stmt.test)?;

        // Jump to end if false
        let jump_to_end = self.emit(Instruction::with_operand(
            OpCode::JumpIfFalse,
            Operand::Jump(0),
        ));

        // Compile body (don't keep value)
        self.compile_statement(&while_stmt.body, false)?;

        // Jump back to start
        self.emit(Instruction::with_operand(
            OpCode::Jump,
            Operand::Jump(loop_start as i32),
        ));

        // Patch jump to end
        let end_pos = self.bytecode.instructions.len() as i32;
        self.bytecode.instructions[jump_to_end].operand = Some(Operand::Jump(end_pos));

        Ok(())
    }

    fn compile_for_statement(&mut self, for_stmt: &ForStatement) -> Result<(), Error> {
        // Compile init
        if let Some(init) = &for_stmt.init {
            match init {
                ForInit::Declaration(decl) => {
                    self.compile_variable_declaration(decl)?;
                }
                ForInit::Expression(expr) => {
                    self.compile_expression(expr)?;
                    self.emit(Instruction::simple(OpCode::Pop));
                }
            }
        }

        let loop_start = self.bytecode.instructions.len();

        // Compile test
        let jump_to_end = if let Some(test) = &for_stmt.test {
            self.compile_expression(test)?;
            Some(self.emit(Instruction::with_operand(
                OpCode::JumpIfFalse,
                Operand::Jump(0),
            )))
        } else {
            None
        };

        // Compile body
        self.compile_statement(&for_stmt.body, false)?;

        // Compile update
        if let Some(update) = &for_stmt.update {
            self.compile_expression(update)?;
            self.emit(Instruction::simple(OpCode::Pop));
        }

        // Jump back to test
        self.emit(Instruction::with_operand(
            OpCode::Jump,
            Operand::Jump(loop_start as i32),
        ));

        // Patch jump to end
        if let Some(jump_idx) = jump_to_end {
            let end_pos = self.bytecode.instructions.len() as i32;
            self.bytecode.instructions[jump_idx].operand = Some(Operand::Jump(end_pos));
        }

        Ok(())
    }

    fn compile_expression(&mut self, expr: &Expression) -> Result<(), Error> {
        match expr {
            Expression::Literal(lit) => self.compile_literal(lit),
            Expression::Binary(bin) => self.compile_binary(bin),
            Expression::Unary(un) => self.compile_unary(un),
            Expression::Identifier(id) => self.compile_identifier(id),
            Expression::Assignment(assign) => self.compile_assignment(assign),
            Expression::Call(call) => self.compile_call(call),
            Expression::Member(member) => self.compile_member(member),
            Expression::This => {
                self.emit(Instruction::simple(OpCode::LoadThis));
                Ok(())
            }
            Expression::Array(arr) => self.compile_array(arr),
            Expression::Object(obj) => self.compile_object(obj),
            Expression::Conditional(cond) => self.compile_conditional(cond),
            _ => {
                // For now, just push undefined for unimplemented expressions
                self.emit(Instruction::simple(OpCode::LoadUndefined));
                Ok(())
            }
        }
    }

    fn compile_identifier(&mut self, id: &Identifier) -> Result<(), Error> {
        if let Some(index) = self.scope.resolve(&id.name) {
            // Local variable
            self.emit(Instruction::with_operand(
                OpCode::LoadLocal,
                Operand::Local(index as u16),
            ));
        } else {
            // Global variable
            let name_idx = self.bytecode.add_constant(Value::String(id.name.clone()));
            self.emit(Instruction::with_operand(
                OpCode::LoadGlobal,
                Operand::Property(name_idx),
            ));
        }
        Ok(())
    }

    fn compile_assignment(&mut self, assign: &AssignmentExpression) -> Result<(), Error> {
        // Compile the right-hand side
        self.compile_expression(&assign.right)?;

        // Handle the left-hand side
        match assign.left.as_ref() {
            Expression::Identifier(id) => {
                if let Some(index) = self.scope.resolve(&id.name) {
                    // Local variable
                    self.emit(Instruction::simple(OpCode::Dup)); // Keep value on stack
                    self.emit(Instruction::with_operand(
                        OpCode::StoreLocal,
                        Operand::Local(index as u16),
                    ));
                } else {
                    // Global variable
                    self.emit(Instruction::simple(OpCode::Dup)); // Keep value on stack
                    let name_idx = self.bytecode.add_constant(Value::String(id.name.clone()));
                    self.emit(Instruction::with_operand(
                        OpCode::StoreGlobal,
                        Operand::Property(name_idx),
                    ));
                }
            }
            Expression::Member(member) => {
                // Compile object
                self.compile_expression(&member.object)?;

                // Compile property name
                match &member.property {
                    MemberProperty::Identifier(prop_id) => {
                        let name_idx = self
                            .bytecode
                            .add_constant(Value::String(prop_id.name.clone()));
                        self.emit(Instruction::with_operand(
                            OpCode::SetProperty,
                            Operand::Property(name_idx),
                        ));
                    }
                    MemberProperty::Expression(prop_expr) => {
                        self.compile_expression(prop_expr)?;
                        self.emit(Instruction::simple(OpCode::SetProperty));
                    }
                }
            }
            _ => {
                return Err(Error::SyntaxError("Invalid assignment target".into()));
            }
        }

        Ok(())
    }

    fn compile_call(&mut self, call: &CallExpression) -> Result<(), Error> {
        // Compile callee
        self.compile_expression(&call.callee)?;

        // Compile arguments
        for arg in &call.arguments {
            self.compile_expression(arg)?;
        }

        // Emit call instruction
        self.emit(Instruction::with_operand(
            OpCode::Call,
            Operand::ArgCount(call.arguments.len() as u8),
        ));

        Ok(())
    }

    fn compile_member(&mut self, member: &MemberExpression) -> Result<(), Error> {
        // Compile object
        self.compile_expression(&member.object)?;

        // Compile property access
        match &member.property {
            MemberProperty::Identifier(id) => {
                let name_idx = self.bytecode.add_constant(Value::String(id.name.clone()));
                self.emit(Instruction::with_operand(
                    OpCode::GetProperty,
                    Operand::Property(name_idx),
                ));
            }
            MemberProperty::Expression(expr) => {
                self.compile_expression(expr)?;
                self.emit(Instruction::simple(OpCode::GetProperty));
            }
        }

        Ok(())
    }

    fn compile_array(&mut self, arr: &ArrayExpression) -> Result<(), Error> {
        // Push elements
        for elem in &arr.elements {
            if let Some(expr) = elem {
                self.compile_expression(expr)?;
            } else {
                self.emit(Instruction::simple(OpCode::LoadUndefined));
            }
        }

        // Create array
        self.emit(Instruction::with_operand(
            OpCode::NewArray,
            Operand::ArgCount(arr.elements.len() as u8),
        ));

        Ok(())
    }

    fn compile_object(&mut self, _obj: &ObjectExpression) -> Result<(), Error> {
        // For now, just create empty object
        self.emit(Instruction::simple(OpCode::NewObject));
        Ok(())
    }

    fn compile_conditional(&mut self, cond: &ConditionalExpression) -> Result<(), Error> {
        // Compile condition
        self.compile_expression(&cond.test)?;

        // Jump to alternate if false
        let jump_to_alternate = self.emit(Instruction::with_operand(
            OpCode::JumpIfFalse,
            Operand::Jump(0),
        ));

        // Compile consequent
        self.compile_expression(&cond.consequent)?;

        // Jump over alternate
        let jump_to_end = self.emit(Instruction::with_operand(OpCode::Jump, Operand::Jump(0)));

        // Patch jump to alternate
        let alternate_pos = self.bytecode.instructions.len() as i32;
        self.bytecode.instructions[jump_to_alternate].operand = Some(Operand::Jump(alternate_pos));

        // Compile alternate
        self.compile_expression(&cond.alternate)?;

        // Patch jump to end
        let end_pos = self.bytecode.instructions.len() as i32;
        self.bytecode.instructions[jump_to_end].operand = Some(Operand::Jump(end_pos));

        Ok(())
    }

    fn compile_literal(&mut self, lit: &Literal) -> Result<(), Error> {
        match lit {
            Literal::Number(n) => {
                let idx = self.bytecode.add_constant(Value::Number(*n));
                self.emit(Instruction::with_operand(
                    OpCode::LoadConst,
                    Operand::Constant(idx),
                ));
            }
            Literal::String(s) => {
                let idx = self.bytecode.add_constant(Value::String(s.clone()));
                self.emit(Instruction::with_operand(
                    OpCode::LoadConst,
                    Operand::Constant(idx),
                ));
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
