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

    // ========================================================================
    // Hoisting (ES3 Section 10.1.3)
    // ========================================================================

    /// Performs hoisting for a list of statements.
    /// Returns var_names (function declarations handled in-place)
    fn collect_hoisted_var_names(&self, statements: &[Statement]) -> Vec<String> {
        let mut var_names = Vec::new();
        let mut func_decls: Vec<&FunctionDeclaration> = Vec::new();

        for stmt in statements {
            self.collect_hoisted_from_statement(stmt, &mut var_names, &mut func_decls);
        }

        var_names
    }

    /// Recursively collect hoisted declarations from a statement.
    fn collect_hoisted_from_statement<'a>(
        &self,
        stmt: &'a Statement,
        var_names: &mut Vec<String>,
        func_decls: &mut Vec<&'a FunctionDeclaration>,
    ) {
        match stmt {
            Statement::VariableDeclaration(decl) => {
                // Only hoist 'var' declarations, not 'let' or 'const'
                if decl.kind == VariableKind::Var {
                    for declarator in &decl.declarations {
                        let name = &declarator.id.name;
                        if !var_names.contains(name) {
                            var_names.push(name.clone());
                        }
                    }
                }
            }
            Statement::FunctionDeclaration(func) => {
                func_decls.push(func);
            }
            Statement::Block(block) => {
                // var hoists out of blocks
                for s in &block.body {
                    self.collect_hoisted_from_statement(s, var_names, func_decls);
                }
            }
            Statement::If(if_stmt) => {
                self.collect_hoisted_from_statement(&if_stmt.consequent, var_names, func_decls);
                if let Some(alt) = &if_stmt.alternate {
                    self.collect_hoisted_from_statement(alt, var_names, func_decls);
                }
            }
            Statement::While(while_stmt) => {
                self.collect_hoisted_from_statement(&while_stmt.body, var_names, func_decls);
            }
            Statement::DoWhile(do_while) => {
                self.collect_hoisted_from_statement(&do_while.body, var_names, func_decls);
            }
            Statement::For(for_stmt) => {
                // Hoist var in init
                if let Some(ForInit::Declaration(decl)) = &for_stmt.init {
                    if decl.kind == VariableKind::Var {
                        for declarator in &decl.declarations {
                            let name = &declarator.id.name;
                            if !var_names.contains(name) {
                                var_names.push(name.clone());
                            }
                        }
                    }
                }
                self.collect_hoisted_from_statement(&for_stmt.body, var_names, func_decls);
            }
            Statement::ForIn(for_in) => {
                if let ForInLeft::Declaration(decl) = &for_in.left {
                    if decl.kind == VariableKind::Var {
                        for declarator in &decl.declarations {
                            let name = &declarator.id.name;
                            if !var_names.contains(name) {
                                var_names.push(name.clone());
                            }
                        }
                    }
                }
                self.collect_hoisted_from_statement(&for_in.body, var_names, func_decls);
            }
            Statement::ForOf(for_of) => {
                if let ForInLeft::Declaration(decl) = &for_of.left {
                    if decl.kind == VariableKind::Var {
                        for declarator in &decl.declarations {
                            let name = &declarator.id.name;
                            if !var_names.contains(name) {
                                var_names.push(name.clone());
                            }
                        }
                    }
                }
                self.collect_hoisted_from_statement(&for_of.body, var_names, func_decls);
            }
            Statement::Switch(switch_stmt) => {
                for case in &switch_stmt.cases {
                    for s in &case.consequent {
                        self.collect_hoisted_from_statement(s, var_names, func_decls);
                    }
                }
            }
            Statement::Try(try_stmt) => {
                for s in &try_stmt.block.body {
                    self.collect_hoisted_from_statement(s, var_names, func_decls);
                }
                if let Some(handler) = &try_stmt.handler {
                    for s in &handler.body.body {
                        self.collect_hoisted_from_statement(s, var_names, func_decls);
                    }
                }
                if let Some(finalizer) = &try_stmt.finalizer {
                    for s in &finalizer.body {
                        self.collect_hoisted_from_statement(s, var_names, func_decls);
                    }
                }
            }
            _ => {}
        }
    }

    /// Hoists variable declarations by pre-declaring them.
    fn hoist_declarations(&mut self, statements: &[Statement]) -> Result<(), Error> {
        let var_names = self.collect_hoisted_var_names(statements);

        // Declare all hoisted vars (initialized to undefined)
        for name in var_names {
            if self.scope.resolve(&name).is_none() {
                let index = self.scope.declare(name, true)?;
                // Initialize to undefined
                self.emit(Instruction::simple(OpCode::LoadUndefined));
                self.emit(Instruction::with_operand(
                    OpCode::StoreLocal,
                    Operand::Local(index as u16),
                ));
                self.scope.mark_initialized(index);
            }
        }

        // Function declarations are hoisted completely - compile them first
        // Note: In a full implementation, this would compile the function body
        // and store the function object. For now, we just declare them.

        Ok(())
    }

    // ========================================================================
    // Main Compilation Entry Point
    // ========================================================================

    /// Compiles a program to bytecode.
    ///
    /// For REPL/eval, keeps the last expression value on the stack.
    pub fn compile(&mut self, program: &Program) -> Result<Bytecode, Error> {
        // Hoist declarations first
        self.hoist_declarations(&program.body)?;
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
            Statement::ForIn(for_in) => {
                self.compile_for_in_statement(for_in)?;
                if keep_value {
                    self.emit(Instruction::simple(OpCode::LoadUndefined));
                }
            }
            Statement::ForOf(for_of) => {
                self.compile_for_of_statement(for_of)?;
                if keep_value {
                    self.emit(Instruction::simple(OpCode::LoadUndefined));
                }
            }
            Statement::DoWhile(do_while) => {
                self.compile_do_while_statement(do_while)?;
                if keep_value {
                    self.emit(Instruction::simple(OpCode::LoadUndefined));
                }
            }
            Statement::Switch(switch_stmt) => {
                self.compile_switch_statement(switch_stmt)?;
                if keep_value {
                    self.emit(Instruction::simple(OpCode::LoadUndefined));
                }
            }
            Statement::Try(try_stmt) => {
                self.compile_try_statement(try_stmt)?;
                if keep_value {
                    self.emit(Instruction::simple(OpCode::LoadUndefined));
                }
            }
            Statement::Throw(throw_stmt) => {
                self.compile_expression(&throw_stmt.argument)?;
                self.emit(Instruction::simple(OpCode::Throw));
            }
            Statement::FunctionDeclaration(func_decl) => {
                self.compile_function_declaration(func_decl)?;
                if keep_value {
                    self.emit(Instruction::simple(OpCode::LoadUndefined));
                }
            }
            Statement::Empty => {
                if keep_value {
                    self.emit(Instruction::simple(OpCode::LoadUndefined));
                }
            }
            Statement::Break => {
                // Break is handled by the enclosing loop
                // In full impl, would emit jump to loop end
                self.emit(Instruction::simple(OpCode::Nop));
            }
            Statement::BreakLabel(label) => {
                // Break with label - jumps to labeled statement
                // In full impl, would track labels and emit jump
                self.emit(Instruction::simple(OpCode::Nop));
            }
            Statement::Continue => {
                // Continue is handled by the enclosing loop
                // In full impl, would emit jump to loop start
                self.emit(Instruction::simple(OpCode::Nop));
            }
            Statement::ContinueLabel(label) => {
                // Continue with label - jumps to labeled loop
                // In full impl, would track labels and emit jump
                self.emit(Instruction::simple(OpCode::Nop));
            }
            Statement::With(with_stmt) => {
                self.compile_with_statement(with_stmt)?;
                if keep_value {
                    self.emit(Instruction::simple(OpCode::LoadUndefined));
                }
            }
            Statement::Labeled(labeled) => {
                self.compile_labeled_statement(labeled)?;
                if keep_value {
                    self.emit(Instruction::simple(OpCode::LoadUndefined));
                }
            }
            Statement::Debugger => {
                // Debugger statement - emit a nop (no-op)
                // In a full impl, would trigger debugger breakpoint
                self.emit(Instruction::simple(OpCode::Nop));
                if keep_value {
                    self.emit(Instruction::simple(OpCode::LoadUndefined));
                }
            }
        }
        Ok(())
    }

    /// Compile with statement (ES3 Section 12.10).
    fn compile_with_statement(&mut self, with_stmt: &WithStatement) -> Result<(), Error> {
        // Compile the object expression
        self.compile_expression(&with_stmt.object)?;

        // In a full implementation:
        // 1. Push the object onto the scope chain
        // 2. Compile the body with the modified scope
        // 3. Pop the object from the scope chain
        //
        // For now, we just pop the object and compile the body normally
        // This is not fully compliant but avoids scope chain complexity
        self.emit(Instruction::simple(OpCode::Pop));

        // Compile the body
        self.compile_statement(&with_stmt.body, false)?;

        Ok(())
    }

    /// Compile labeled statement (ES3 Section 12.12).
    fn compile_labeled_statement(&mut self, labeled: &LabeledStatement) -> Result<(), Error> {
        // Labels are used for break/continue targets
        // In a full implementation, would track label positions
        // For now, just compile the body
        self.compile_statement(&labeled.body, false)?;
        Ok(())
    }

    fn compile_variable_declaration(&mut self, decl: &VariableDeclaration) -> Result<(), Error> {
        let mutable = decl.kind != VariableKind::Const;
        let is_var = decl.kind == VariableKind::Var;

        for declarator in &decl.declarations {
            // Check if already hoisted (for var declarations)
            let index = if is_var {
                if let Some(existing) = self.scope.resolve(&declarator.id.name) {
                    // Variable was already hoisted, just use existing slot
                    existing
                } else {
                    // Not hoisted yet (shouldn't happen with hoisting, but handle it)
                    self.scope.declare(declarator.id.name.clone(), mutable)?
                }
            } else {
                // let/const: always declare fresh (may shadow)
                self.scope.declare(declarator.id.name.clone(), mutable)?
            };

            // Compile initializer if present
            if let Some(init) = &declarator.init {
                self.compile_expression(init)?;
            } else if !is_var {
                // let/const without initializer gets undefined
                self.emit(Instruction::simple(OpCode::LoadUndefined));
            } else {
                // var without initializer - already initialized to undefined by hoisting
                // Skip storing undefined again
                continue;
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

    fn compile_for_in_statement(&mut self, for_in: &ForInStatement) -> Result<(), Error> {
        // Compile the object to iterate over
        self.compile_expression(&for_in.right)?;

        // Initialize iteration (pushes keys array and index 0)
        self.emit(Instruction::simple(OpCode::ForInInit));

        let loop_start = self.bytecode.instructions.len();

        // Check if there are more keys, get next key if so
        let jump_to_end = self.emit(Instruction::with_operand(
            OpCode::ForInNext,
            Operand::Jump(0), // Placeholder - jumps to end if done
        ));

        // Store key in variable
        match &for_in.left {
            ForInLeft::Declaration(decl) => {
                // Declare variable if needed
                if !decl.declarations.is_empty() {
                    let var_name = &decl.declarations[0].id.name;
                    let mutable = decl.kind != VariableKind::Const;

                    // Check if already declared in this scope
                    if self.scope.resolve(var_name).is_none() {
                        let index = self.scope.declare(var_name.clone(), mutable)?;
                        self.emit(Instruction::with_operand(
                            OpCode::StoreLocal,
                            Operand::Local(index as u16),
                        ));
                        self.scope.mark_initialized(index);
                    } else {
                        let index = self.scope.resolve(var_name).unwrap();
                        self.emit(Instruction::with_operand(
                            OpCode::StoreLocal,
                            Operand::Local(index as u16),
                        ));
                    }
                }
            }
            ForInLeft::Expression(expr) => {
                // Store to existing variable
                match expr {
                    Expression::Identifier(id) => {
                        if let Some(index) = self.scope.resolve(&id.name) {
                            self.emit(Instruction::with_operand(
                                OpCode::StoreLocal,
                                Operand::Local(index as u16),
                            ));
                        } else {
                            let name_idx =
                                self.bytecode.add_constant(Value::String(id.name.clone()));
                            self.emit(Instruction::with_operand(
                                OpCode::StoreGlobal,
                                Operand::Property(name_idx),
                            ));
                        }
                    }
                    _ => {
                        // For member expressions, would need to handle differently
                        self.emit(Instruction::simple(OpCode::Pop));
                    }
                }
            }
        }

        // Compile body
        self.compile_statement(&for_in.body, false)?;

        // Jump back to start
        self.emit(Instruction::with_operand(
            OpCode::Jump,
            Operand::Jump(loop_start as i32),
        ));

        // Patch jump to end
        let end_pos = self.bytecode.instructions.len() as i32;
        self.bytecode.instructions[jump_to_end].operand = Some(Operand::Jump(end_pos));

        // Clean up iteration state
        self.emit(Instruction::simple(OpCode::ForInDone));

        Ok(())
    }

    fn compile_for_of_statement(&mut self, for_of: &ForOfStatement) -> Result<(), Error> {
        // For-of is ES6, but we can support it with similar logic
        // For now, emit a simple stub that works like for-in
        // In a full implementation, would use Symbol.iterator

        // Compile the iterable
        self.compile_expression(&for_of.right)?;

        // Initialize iteration
        self.emit(Instruction::simple(OpCode::ForInInit));

        let loop_start = self.bytecode.instructions.len();

        let jump_to_end = self.emit(Instruction::with_operand(
            OpCode::ForInNext,
            Operand::Jump(0),
        ));

        // Store value in variable
        match &for_of.left {
            ForInLeft::Declaration(decl) => {
                if !decl.declarations.is_empty() {
                    let var_name = &decl.declarations[0].id.name;
                    let mutable = decl.kind != VariableKind::Const;

                    if self.scope.resolve(var_name).is_none() {
                        let index = self.scope.declare(var_name.clone(), mutable)?;
                        self.emit(Instruction::with_operand(
                            OpCode::StoreLocal,
                            Operand::Local(index as u16),
                        ));
                        self.scope.mark_initialized(index);
                    } else {
                        let index = self.scope.resolve(var_name).unwrap();
                        self.emit(Instruction::with_operand(
                            OpCode::StoreLocal,
                            Operand::Local(index as u16),
                        ));
                    }
                }
            }
            ForInLeft::Expression(expr) => match expr {
                Expression::Identifier(id) => {
                    if let Some(index) = self.scope.resolve(&id.name) {
                        self.emit(Instruction::with_operand(
                            OpCode::StoreLocal,
                            Operand::Local(index as u16),
                        ));
                    } else {
                        let name_idx = self.bytecode.add_constant(Value::String(id.name.clone()));
                        self.emit(Instruction::with_operand(
                            OpCode::StoreGlobal,
                            Operand::Property(name_idx),
                        ));
                    }
                }
                _ => {
                    self.emit(Instruction::simple(OpCode::Pop));
                }
            },
        }

        self.compile_statement(&for_of.body, false)?;

        self.emit(Instruction::with_operand(
            OpCode::Jump,
            Operand::Jump(loop_start as i32),
        ));

        let end_pos = self.bytecode.instructions.len() as i32;
        self.bytecode.instructions[jump_to_end].operand = Some(Operand::Jump(end_pos));

        self.emit(Instruction::simple(OpCode::ForInDone));

        Ok(())
    }

    fn compile_do_while_statement(&mut self, do_while: &DoWhileStatement) -> Result<(), Error> {
        let loop_start = self.bytecode.instructions.len();

        // Compile body first
        self.compile_statement(&do_while.body, false)?;

        // Compile condition
        self.compile_expression(&do_while.test)?;

        // Jump back to start if true
        self.emit(Instruction::with_operand(
            OpCode::JumpIfTrue,
            Operand::Jump(loop_start as i32),
        ));

        Ok(())
    }

    fn compile_switch_statement(&mut self, switch_stmt: &SwitchStatement) -> Result<(), Error> {
        // Compile discriminant
        self.compile_expression(&switch_stmt.discriminant)?;

        // Store discriminant in a temp slot by duplicating
        let discriminant_local = self.scope.locals.len() as u16;

        let mut case_jumps = Vec::new();
        let mut default_case = None;

        // First pass: Generate tests and jumps
        for (i, case) in switch_stmt.cases.iter().enumerate() {
            if let Some(test) = &case.test {
                // Duplicate discriminant for comparison
                self.emit(Instruction::simple(OpCode::Dup));
                self.compile_expression(test)?;
                self.emit(Instruction::simple(OpCode::StrictEq));

                // Jump to case body if equal
                let jump = self.emit(Instruction::with_operand(
                    OpCode::JumpIfTrue,
                    Operand::Jump(0), // Placeholder
                ));
                case_jumps.push((i, jump));
            } else {
                // Default case
                default_case = Some(i);
            }
        }

        // Jump to default or end
        let jump_to_default = self.emit(Instruction::with_operand(OpCode::Jump, Operand::Jump(0)));

        // Second pass: Compile case bodies
        let mut case_starts = Vec::new();
        for case in &switch_stmt.cases {
            case_starts.push(self.bytecode.instructions.len());
            for stmt in &case.consequent {
                self.compile_statement(stmt, false)?;
            }
        }

        let end_pos = self.bytecode.instructions.len() as i32;

        // Patch case jumps
        for (case_idx, jump_idx) in case_jumps {
            self.bytecode.instructions[jump_idx].operand =
                Some(Operand::Jump(case_starts[case_idx] as i32));
        }

        // Patch default jump
        if let Some(default_idx) = default_case {
            self.bytecode.instructions[jump_to_default].operand =
                Some(Operand::Jump(case_starts[default_idx] as i32));
        } else {
            self.bytecode.instructions[jump_to_default].operand = Some(Operand::Jump(end_pos));
        }

        // Pop discriminant
        self.emit(Instruction::simple(OpCode::Pop));

        Ok(())
    }

    fn compile_try_statement(&mut self, try_stmt: &TryStatement) -> Result<(), Error> {
        // Simplified try-catch implementation
        // In a full implementation, would set up exception handlers

        // Compile try block
        for stmt in &try_stmt.block.body {
            self.compile_statement(stmt, false)?;
        }

        // Jump over catch if no exception
        let jump_over_catch = self.emit(Instruction::with_operand(OpCode::Jump, Operand::Jump(0)));

        // Compile catch block if present
        if let Some(handler) = &try_stmt.handler {
            // Declare catch parameter if present
            if let Some(param) = &handler.param {
                let index = self.scope.declare(param.name.clone(), true)?;
                // The exception value would be on the stack
                self.emit(Instruction::with_operand(
                    OpCode::StoreLocal,
                    Operand::Local(index as u16),
                ));
                self.scope.mark_initialized(index);
            }

            // Compile handler body
            for stmt in &handler.body.body {
                self.compile_statement(stmt, false)?;
            }
        }

        let after_catch = self.bytecode.instructions.len() as i32;
        self.bytecode.instructions[jump_over_catch].operand = Some(Operand::Jump(after_catch));

        // Compile finally block if present
        if let Some(finalizer) = &try_stmt.finalizer {
            for stmt in &finalizer.body {
                self.compile_statement(stmt, false)?;
            }
        }

        Ok(())
    }

    fn compile_function_declaration(
        &mut self,
        func_decl: &FunctionDeclaration,
    ) -> Result<(), Error> {
        // Create a new compiler for the function body
        let mut func_compiler = Compiler::new();
        func_compiler.scope.begin_scope();

        // Declare parameters as locals
        let param_names: Vec<String> = func_decl.params.iter().map(|p| p.name.clone()).collect();
        for param in &param_names {
            func_compiler.scope.declare(param.clone(), true)?;
        }

        // Compile function body
        for (i, stmt) in func_decl.body.iter().enumerate() {
            let is_last = i == func_decl.body.len() - 1;
            func_compiler.compile_statement(stmt, is_last)?;
        }

        // Implicit return undefined if no explicit return
        func_compiler.emit(Instruction::simple(OpCode::LoadUndefined));
        func_compiler.emit(Instruction::simple(OpCode::Return));

        let local_count = func_compiler.scope.locals.len();
        func_compiler.scope.end_scope();

        // Create the function object
        let func_obj = crate::runtime::function::Function::new(
            Some(func_decl.id.name.clone()),
            param_names,
            std::mem::take(&mut func_compiler.bytecode),
            local_count,
        );

        // Create callable and wrap in Value
        let callable = crate::runtime::function::Callable::Function(func_obj);
        let func_value = Value::Function(std::sync::Arc::new(callable));
        let idx = self.bytecode.add_constant(func_value);

        // Store function in scope
        let local_idx = self.scope.declare(func_decl.id.name.clone(), true)?;
        self.emit(Instruction::with_operand(
            OpCode::LoadConst,
            Operand::Constant(idx),
        ));
        self.emit(Instruction::with_operand(
            OpCode::StoreLocal,
            Operand::Local(local_idx as u16),
        ));
        self.scope.mark_initialized(local_idx);

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
            Expression::Update(update) => self.compile_update(update),
            Expression::Sequence(seq) => self.compile_sequence(seq),
            Expression::Function(func) => self.compile_function_expr(func),
            Expression::Arrow(arrow) => self.compile_arrow(arrow),
            Expression::New(new_expr) => self.compile_new(new_expr),
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

    fn compile_object(&mut self, obj: &ObjectExpression) -> Result<(), Error> {
        // Create new object
        self.emit(Instruction::simple(OpCode::NewObject));

        // Set properties
        for prop in &obj.properties {
            // Duplicate object reference for each property set
            self.emit(Instruction::simple(OpCode::Dup));

            // Compile property key
            let key_name = match &prop.key {
                PropertyKey::Identifier(id) => id.name.clone(),
                PropertyKey::Literal(lit) => match lit {
                    Literal::String(s) => s.clone(),
                    Literal::Number(n) => n.to_string(),
                    _ => return Err(Error::SyntaxError("Invalid property key".into())),
                },
                PropertyKey::Computed(_) => {
                    return Err(Error::SyntaxError("Computed properties not yet supported".into()))
                }
            };

            // Compile property value
            self.compile_expression(&prop.value)?;

            // Emit SetProperty
            let key_idx = self.bytecode.add_constant(Value::String(key_name));
            self.emit(Instruction::with_operand(
                OpCode::SetProperty,
                Operand::Property(key_idx),
            ));

            // Pop the duplicated object (SetProperty leaves value on stack, we want object)
            self.emit(Instruction::simple(OpCode::Pop));
        }

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
        // Handle short-circuit operators specially
        match bin.operator {
            BinaryOperator::LogicalAnd => {
                return self.compile_logical_and(bin);
            }
            BinaryOperator::LogicalOr => {
                return self.compile_logical_or(bin);
            }
            _ => {}
        }

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
            // Bitwise operators
            BinaryOperator::BitwiseAnd => OpCode::BitAnd,
            BinaryOperator::BitwiseOr => OpCode::BitOr,
            BinaryOperator::BitwiseXor => OpCode::BitXor,
            BinaryOperator::LeftShift => OpCode::Shl,
            BinaryOperator::RightShift => OpCode::Shr,
            BinaryOperator::UnsignedRightShift => OpCode::Ushr,
            // These are handled above
            BinaryOperator::LogicalAnd | BinaryOperator::LogicalOr => unreachable!(),
            _ => return Err(Error::InternalError("Unsupported operator".into())),
        };

        self.emit(Instruction::simple(opcode));
        Ok(())
    }

    /// Compile logical AND with short-circuit evaluation.
    fn compile_logical_and(&mut self, bin: &BinaryExpression) -> Result<(), Error> {
        // Evaluate left side
        self.compile_expression(&bin.left)?;

        // Duplicate for the result if falsy
        self.emit(Instruction::simple(OpCode::Dup));

        // If falsy, jump to end (short-circuit)
        let jump_if_false = self.emit(Instruction::with_operand(
            OpCode::JumpIfFalse,
            Operand::Jump(0), // Placeholder
        ));

        // Pop the duplicated value (we'll use right side result)
        self.emit(Instruction::simple(OpCode::Pop));

        // Evaluate right side
        self.compile_expression(&bin.right)?;

        // Patch the jump
        let end_pos = self.bytecode.instructions.len() as i32;
        self.bytecode.instructions[jump_if_false].operand = Some(Operand::Jump(end_pos));

        Ok(())
    }

    /// Compile logical OR with short-circuit evaluation.
    fn compile_logical_or(&mut self, bin: &BinaryExpression) -> Result<(), Error> {
        // Evaluate left side
        self.compile_expression(&bin.left)?;

        // Duplicate for the result if truthy
        self.emit(Instruction::simple(OpCode::Dup));

        // If truthy, jump to end (short-circuit)
        let jump_if_true = self.emit(Instruction::with_operand(
            OpCode::JumpIfTrue,
            Operand::Jump(0), // Placeholder
        ));

        // Pop the duplicated value (we'll use right side result)
        self.emit(Instruction::simple(OpCode::Pop));

        // Evaluate right side
        self.compile_expression(&bin.right)?;

        // Patch the jump
        let end_pos = self.bytecode.instructions.len() as i32;
        self.bytecode.instructions[jump_if_true].operand = Some(Operand::Jump(end_pos));

        Ok(())
    }

    fn compile_unary(&mut self, un: &UnaryExpression) -> Result<(), Error> {
        self.compile_expression(&un.argument)?;

        let opcode = match un.operator {
            UnaryOperator::Minus => OpCode::Neg,
            UnaryOperator::LogicalNot => OpCode::Not,
            UnaryOperator::BitwiseNot => OpCode::BitNot,
            UnaryOperator::Typeof => OpCode::TypeOf,
            UnaryOperator::Void => {
                // void evaluates expression and returns undefined
                self.emit(Instruction::simple(OpCode::Pop));
                self.emit(Instruction::simple(OpCode::LoadUndefined));
                return Ok(());
            }
            UnaryOperator::Delete => {
                // Delete is complex - for now, just return true
                self.emit(Instruction::simple(OpCode::Pop));
                self.emit(Instruction::simple(OpCode::LoadTrue));
                return Ok(());
            }
            UnaryOperator::Plus => {
                // Unary + converts to number - we'll emit a ToNumber conversion
                // For now, just leave value on stack (it's already compiled)
                return Ok(());
            }
        };

        self.emit(Instruction::simple(opcode));
        Ok(())
    }

    /// Compile ++/-- expressions (ES3 Section 11.4.4-5, 11.3.1-2)
    fn compile_update(&mut self, update: &UpdateExpression) -> Result<(), Error> {
        // Get the variable being updated
        match update.argument.as_ref() {
            Expression::Identifier(id) => {
                if update.prefix {
                    // ++x or --x: increment/decrement first, then return new value
                    // Load current value
                    self.compile_identifier(id)?;

                    // Add/subtract 1
                    let one_idx = self.bytecode.add_constant(Value::Number(1.0));
                    self.emit(Instruction::with_operand(
                        OpCode::LoadConst,
                        Operand::Constant(one_idx),
                    ));

                    match update.operator {
                        UpdateOperator::Increment => {
                            self.emit(Instruction::simple(OpCode::Add));
                        }
                        UpdateOperator::Decrement => {
                            self.emit(Instruction::simple(OpCode::Sub));
                        }
                    }

                    // Duplicate result (one for storage, one for return)
                    self.emit(Instruction::simple(OpCode::Dup));

                    // Store back
                    if let Some(index) = self.scope.resolve(&id.name) {
                        self.emit(Instruction::with_operand(
                            OpCode::StoreLocal,
                            Operand::Local(index as u16),
                        ));
                    } else {
                        let name_idx = self.bytecode.add_constant(Value::String(id.name.clone()));
                        self.emit(Instruction::with_operand(
                            OpCode::StoreGlobal,
                            Operand::Property(name_idx),
                        ));
                    }
                } else {
                    // x++ or x--: return old value, then increment/decrement
                    // Load current value
                    self.compile_identifier(id)?;

                    // Duplicate for return value
                    self.emit(Instruction::simple(OpCode::Dup));

                    // Add/subtract 1
                    let one_idx = self.bytecode.add_constant(Value::Number(1.0));
                    self.emit(Instruction::with_operand(
                        OpCode::LoadConst,
                        Operand::Constant(one_idx),
                    ));

                    match update.operator {
                        UpdateOperator::Increment => {
                            self.emit(Instruction::simple(OpCode::Add));
                        }
                        UpdateOperator::Decrement => {
                            self.emit(Instruction::simple(OpCode::Sub));
                        }
                    }

                    // Store new value
                    if let Some(index) = self.scope.resolve(&id.name) {
                        self.emit(Instruction::with_operand(
                            OpCode::StoreLocal,
                            Operand::Local(index as u16),
                        ));
                    } else {
                        let name_idx = self.bytecode.add_constant(Value::String(id.name.clone()));
                        self.emit(Instruction::with_operand(
                            OpCode::StoreGlobal,
                            Operand::Property(name_idx),
                        ));
                    }
                    // Old value is still on stack from the Dup
                }
            }
            Expression::Member(member) => {
                // For member expressions like obj.prop++ or obj[key]++
                // This is more complex - for now, emit a placeholder
                self.compile_expression(&member.object)?;
                match &member.property {
                    MemberProperty::Identifier(prop_id) => {
                        let name_idx = self
                            .bytecode
                            .add_constant(Value::String(prop_id.name.clone()));
                        self.emit(Instruction::with_operand(
                            OpCode::GetProperty,
                            Operand::Property(name_idx),
                        ));
                    }
                    MemberProperty::Expression(prop_expr) => {
                        self.compile_expression(prop_expr)?;
                        self.emit(Instruction::simple(OpCode::GetProperty));
                    }
                }
                // TODO: Implement full member update (requires saving object/property)
            }
            _ => {
                return Err(Error::SyntaxError(
                    "Invalid left-hand side in update expression".into(),
                ));
            }
        }
        Ok(())
    }

    /// Compile sequence (comma) expressions
    fn compile_sequence(&mut self, seq: &SequenceExpression) -> Result<(), Error> {
        for (i, expr) in seq.expressions.iter().enumerate() {
            self.compile_expression(expr)?;
            // Pop all but the last value
            if i < seq.expressions.len() - 1 {
                self.emit(Instruction::simple(OpCode::Pop));
            }
        }
        Ok(())
    }

    /// Compile function expressions
    fn compile_function_expr(&mut self, func: &FunctionExpression) -> Result<(), Error> {
        // Create a new compiler for the function body
        let mut func_compiler = Compiler::new();
        func_compiler.scope.begin_scope();

        // Declare parameters as locals
        let param_names: Vec<String> = func.params.iter().map(|p| p.name.clone()).collect();
        for param in &param_names {
            func_compiler.scope.declare(param.clone(), true)?;
        }

        // Compile function body
        for (i, stmt) in func.body.iter().enumerate() {
            let is_last = i == func.body.len() - 1;
            func_compiler.compile_statement(stmt, is_last)?;
        }

        // Implicit return undefined if no explicit return
        func_compiler.emit(Instruction::simple(OpCode::LoadUndefined));
        func_compiler.emit(Instruction::simple(OpCode::Return));

        let local_count = func_compiler.scope.locals.len();
        func_compiler.scope.end_scope();

        // Create the function object
        let func_obj = crate::runtime::function::Function::new(
            func.id.as_ref().map(|id| id.name.clone()),
            param_names,
            std::mem::take(&mut func_compiler.bytecode),
            local_count,
        );

        // Create callable and wrap in Value
        let callable = crate::runtime::function::Callable::Function(func_obj);
        let func_value = Value::Function(std::sync::Arc::new(callable));
        let idx = self.bytecode.add_constant(func_value);
        self.emit(Instruction::with_operand(
            OpCode::LoadConst,
            Operand::Constant(idx),
        ));

        Ok(())
    }

    /// Compile arrow function expressions
    fn compile_arrow(&mut self, arrow: &ArrowFunctionExpression) -> Result<(), Error> {
        // Arrow functions are similar to function expressions
        let mut func_compiler = Compiler::new();
        func_compiler.scope.begin_scope();

        let param_names: Vec<String> = arrow.params.iter().map(|p| p.name.clone()).collect();
        for param in &param_names {
            func_compiler.scope.declare(param.clone(), true)?;
        }

        match &arrow.body {
            ArrowBody::Expression(expr) => {
                func_compiler.compile_expression(expr)?;
                func_compiler.emit(Instruction::simple(OpCode::Return));
            }
            ArrowBody::Block(stmts) => {
                for (i, stmt) in stmts.iter().enumerate() {
                    let is_last = i == stmts.len() - 1;
                    func_compiler.compile_statement(stmt, is_last)?;
                }
                func_compiler.emit(Instruction::simple(OpCode::LoadUndefined));
                func_compiler.emit(Instruction::simple(OpCode::Return));
            }
        }

        let local_count = func_compiler.scope.locals.len();
        func_compiler.scope.end_scope();

        // Create the function object
        let func_obj = crate::runtime::function::Function::new(
            None, // Arrow functions are always anonymous
            param_names,
            std::mem::take(&mut func_compiler.bytecode),
            local_count,
        );

        let callable = crate::runtime::function::Callable::Function(func_obj);
        let func_value = Value::Function(std::sync::Arc::new(callable));
        let idx = self.bytecode.add_constant(func_value);
        self.emit(Instruction::with_operand(
            OpCode::LoadConst,
            Operand::Constant(idx),
        ));

        Ok(())
    }

    /// Compile new expressions
    fn compile_new(&mut self, new_expr: &NewExpression) -> Result<(), Error> {
        // Compile the constructor
        self.compile_expression(&new_expr.callee)?;

        // Compile arguments
        for arg in &new_expr.arguments {
            self.compile_expression(arg)?;
        }

        // Emit call with argument count
        // In a full impl, would use a separate NewCall opcode
        self.emit(Instruction::with_operand(
            OpCode::Call,
            Operand::ArgCount(new_expr.arguments.len() as u8),
        ));

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    fn compile_source(src: &str) -> Result<Bytecode, Error> {
        let mut parser = Parser::new(src);
        let program = parser.parse_program()?;
        let mut compiler = Compiler::new();
        compiler.compile(&program)
    }

    fn compile_ok(src: &str) -> Bytecode {
        compile_source(src).expect("Compilation should succeed")
    }

    #[test]
    fn test_compiler_new() {
        let compiler = Compiler::new();
        assert!(compiler.bytecode.instructions.is_empty());
    }

    #[test]
    fn test_compiler_default() {
        let compiler = Compiler::default();
        assert!(compiler.bytecode.instructions.is_empty());
    }

    #[test]
    fn test_compile_empty_program() {
        let bytecode = compile_ok("");
        // Should emit LoadUndefined and Halt
        assert!(!bytecode.instructions.is_empty());
    }

    #[test]
    fn test_compile_number_literal() {
        let bytecode = compile_ok("42;");
        assert!(!bytecode.instructions.is_empty());
    }

    #[test]
    fn test_compile_string_literal() {
        let bytecode = compile_ok("'hello';");
        assert!(!bytecode.instructions.is_empty());
    }

    #[test]
    fn test_compile_boolean_literals() {
        compile_ok("true;");
        compile_ok("false;");
    }

    #[test]
    fn test_compile_null_undefined() {
        compile_ok("null;");
    }

    #[test]
    fn test_compile_binary_add() {
        let bytecode = compile_ok("1 + 2;");
        assert!(!bytecode.instructions.is_empty());
    }

    #[test]
    fn test_compile_binary_sub() {
        compile_ok("5 - 3;");
    }

    #[test]
    fn test_compile_binary_mul() {
        compile_ok("2 * 3;");
    }

    #[test]
    fn test_compile_binary_div() {
        compile_ok("10 / 2;");
    }

    #[test]
    fn test_compile_comparison_operators() {
        compile_ok("1 < 2;");
        compile_ok("1 > 2;");
        compile_ok("1 <= 2;");
        compile_ok("1 >= 2;");
        compile_ok("1 == 2;");
        compile_ok("1 != 2;");
        compile_ok("1 === 2;");
        compile_ok("1 !== 2;");
    }

    #[test]
    fn test_compile_unary_minus() {
        compile_ok("-42;");
    }

    #[test]
    fn test_compile_unary_not() {
        compile_ok("!true;");
    }

    #[test]
    fn test_compile_variable_declaration() {
        compile_ok("let x = 1;");
        compile_ok("const y = 2;");
        compile_ok("var z = 3;");
    }

    #[test]
    fn test_compile_variable_use() {
        compile_ok("let x = 1; x;");
    }

    #[test]
    fn test_compile_variable_assignment() {
        compile_ok("let x = 1; x = 2;");
    }

    #[test]
    fn test_compile_multiple_variables() {
        compile_ok("let a = 1; let b = 2; a + b;");
    }

    #[test]
    fn test_compile_if_statement() {
        compile_ok("if (true) { 1; }");
    }

    #[test]
    fn test_compile_if_else() {
        compile_ok("if (true) { 1; } else { 2; }");
    }

    #[test]
    fn test_compile_while_loop() {
        compile_ok("while (false) { 1; }");
    }

    #[test]
    fn test_compile_for_loop() {
        compile_ok("for (let i = 0; i < 10; i = i + 1) { }");
    }

    #[test]
    fn test_compile_function_declaration() {
        compile_ok("function f() { return 1; }");
    }

    #[test]
    fn test_compile_function_call() {
        compile_ok("function f() { return 1; } f();");
    }

    #[test]
    fn test_compile_function_with_params() {
        compile_ok("function add(a, b) { return a + b; }");
    }

    #[test]
    fn test_compile_return_statement() {
        compile_ok("function f() { return; }");
        compile_ok("function f() { return 42; }");
    }

    #[test]
    fn test_compile_block_statement() {
        compile_ok("{ let x = 1; }");
    }

    #[test]
    fn test_compile_nested_blocks() {
        compile_ok("{ let x = 1; { let y = 2; } }");
    }

    #[test]
    fn test_compile_expression_statement() {
        compile_ok("1 + 2;");
        compile_ok("f();");
    }

    #[test]
    fn test_compile_array_literal() {
        compile_ok("let arr = [1, 2, 3];");
    }

    #[test]
    fn test_compile_object_literal() {
        compile_ok("let obj = { x: 1 };");
    }

    #[test]
    fn test_compile_member_expression() {
        compile_ok("let obj = { x: 1 }; obj.x;");
    }

    #[test]
    fn test_compile_complex_expression() {
        compile_ok("1 + 2 * 3;");
        compile_ok("(1 < 2) === true;");
    }

    // Note: Logical operators and conditional expressions are not yet fully supported
    // These tests are placeholders for when they are implemented

    #[test]
    fn test_scope_declare() {
        let mut scope = Scope::new();
        let idx = scope.declare("x".to_string(), true).unwrap();
        assert_eq!(idx, 0);
    }

    #[test]
    fn test_scope_resolve() {
        let mut scope = Scope::new();
        scope.declare("x".to_string(), true).unwrap();
        assert_eq!(scope.resolve("x"), Some(0));
        assert_eq!(scope.resolve("y"), None);
    }

    #[test]
    fn test_scope_duplicate_error() {
        let mut scope = Scope::new();
        scope.declare("x".to_string(), true).unwrap();
        let result = scope.declare("x".to_string(), true);
        assert!(result.is_err());
    }

    #[test]
    fn test_scope_mark_initialized() {
        let mut scope = Scope::new();
        let idx = scope.declare("x".to_string(), true).unwrap();
        assert!(!scope.locals[idx].initialized);
        scope.mark_initialized(idx);
        assert!(scope.locals[idx].initialized);
    }

    #[test]
    fn test_scope_begin_end() {
        let mut scope = Scope::new();
        assert_eq!(scope.depth, 0);
        scope.begin_scope();
        assert_eq!(scope.depth, 1);
        scope.declare("x".to_string(), true).unwrap();
        let popped = scope.end_scope();
        assert_eq!(popped, 1);
        assert_eq!(scope.depth, 0);
    }

    #[test]
    fn test_scope_nested() {
        let mut scope = Scope::new();
        scope.declare("outer".to_string(), true).unwrap();
        scope.begin_scope();
        scope.declare("inner".to_string(), true).unwrap();
        assert_eq!(scope.resolve("outer"), Some(0));
        assert_eq!(scope.resolve("inner"), Some(1));
        scope.end_scope();
        assert_eq!(scope.resolve("outer"), Some(0));
        assert_eq!(scope.resolve("inner"), None);
    }

    #[test]
    fn test_scope_is_local() {
        let mut scope = Scope::new();
        assert!(!scope.is_local("x"));
        scope.declare("x".to_string(), true).unwrap();
        assert!(scope.is_local("x"));
    }

    #[test]
    fn test_scope_shadowing() {
        let mut scope = Scope::new();
        scope.declare("x".to_string(), true).unwrap();
        scope.begin_scope();
        // Can declare same name in inner scope
        let result = scope.declare("x".to_string(), true);
        assert!(result.is_ok());
    }
}
