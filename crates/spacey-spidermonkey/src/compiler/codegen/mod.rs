//! Code generation from AST to bytecode.
//!
//! This module contains the `Compiler` which transforms parsed JavaScript AST
//! into executable bytecode for the VM.

mod scope;

#[cfg(test)]
mod tests;

pub use scope::{Local, Scope};

use crate::Error;
use crate::ast::*;
use crate::compiler::bytecode::{Bytecode, Instruction, OpCode, Operand};
use crate::runtime::value::Value;

/// Compiles AST to bytecode.
pub struct Compiler {
    /// The bytecode being generated
    pub bytecode: Bytecode,
    /// Current scope for variable resolution
    pub scope: Scope,
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
            Statement::FunctionDeclaration(func_decl) => {
                func_decls.push(func_decl);
            }
            // Recurse into block-like statements
            Statement::Block(block) => {
                for inner_stmt in &block.body {
                    self.collect_hoisted_from_statement(inner_stmt, var_names, func_decls);
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
                // Check init for var declarations
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
                    for inner_stmt in &case.consequent {
                        self.collect_hoisted_from_statement(inner_stmt, var_names, func_decls);
                    }
                }
            }
            Statement::Try(try_stmt) => {
                for inner_stmt in &try_stmt.block.body {
                    self.collect_hoisted_from_statement(inner_stmt, var_names, func_decls);
                }
                if let Some(handler) = &try_stmt.handler {
                    for inner_stmt in &handler.body.body {
                        self.collect_hoisted_from_statement(inner_stmt, var_names, func_decls);
                    }
                }
                if let Some(finalizer) = &try_stmt.finalizer {
                    for inner_stmt in &finalizer.body {
                        self.collect_hoisted_from_statement(inner_stmt, var_names, func_decls);
                    }
                }
            }
            Statement::With(with_stmt) => {
                self.collect_hoisted_from_statement(&with_stmt.body, var_names, func_decls);
            }
            Statement::Labeled(labeled) => {
                self.collect_hoisted_from_statement(&labeled.body, var_names, func_decls);
            }
            // Other statements don't contain nested declarations
            _ => {}
        }
    }

    /// Hoist variable and function declarations.
    fn hoist_declarations(&mut self, statements: &[Statement]) -> Result<(), Error> {
        let var_names = self.collect_hoisted_var_names(statements);

        // Hoist var declarations as undefined
        for name in var_names {
            // Only declare if not already in scope
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

        Ok(())
    }

    // ========================================================================
    // Main Compilation Entry Point
    // ========================================================================

    /// Compiles a program AST to bytecode.
    pub fn compile(&mut self, program: &Program) -> Result<Bytecode, Error> {
        // Hoist declarations first
        self.hoist_declarations(&program.body)?;

        // Compile all statements
        let len = program.body.len();
        for (i, stmt) in program.body.iter().enumerate() {
            let is_last = i == len - 1;
            self.compile_statement(stmt, is_last)?;
        }

        // Ensure there's always a value on stack
        if program.body.is_empty() {
            self.emit(Instruction::simple(OpCode::LoadUndefined));
        }

        self.emit(Instruction::simple(OpCode::Halt));

        Ok(std::mem::take(&mut self.bytecode))
    }

    // ========================================================================
    // Statement Compilation
    // ========================================================================

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
            Statement::BreakLabel(_label) => {
                // Break with label - jumps to labeled statement
                // In full impl, would track labels and emit jump
                self.emit(Instruction::simple(OpCode::Nop));
            }
            Statement::Continue => {
                // Continue is handled by the enclosing loop
                // In full impl, would emit jump to loop start
                self.emit(Instruction::simple(OpCode::Nop));
            }
            Statement::ContinueLabel(_label) => {
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
        let _discriminant_local = self.scope.locals.len() as u16;

        let mut case_jumps = Vec::new();
        let mut default_case = None;

        // First pass: Generate tests and jumps
        for (i, case) in switch_stmt.cases.iter().enumerate() {
            if let Some(test) = &case.test {
                // Duplicate discriminant for comparison
                self.emit(Instruction::simple(OpCode::Dup));
                self.compile_expression(test)?;
                self.emit(Instruction::simple(OpCode::StrictEq));

                // Jump to case body if matches
                case_jumps.push((
                    i,
                    self.emit(Instruction::with_operand(
                        OpCode::JumpIfTrue,
                        Operand::Jump(0),
                    )),
                ));
            } else {
                // Default case
                default_case = Some(i);
            }
        }

        // Jump to default or end if no case matches
        let jump_to_default_or_end =
            self.emit(Instruction::with_operand(OpCode::Jump, Operand::Jump(0)));

        // Second pass: Generate case bodies
        let mut case_body_starts = Vec::new();
        for case in &switch_stmt.cases {
            case_body_starts.push(self.bytecode.instructions.len() as i32);
            for stmt in &case.consequent {
                self.compile_statement(stmt, false)?;
            }
        }

        let end_pos = self.bytecode.instructions.len() as i32;

        // Patch case jumps
        for (case_idx, jump_idx) in case_jumps {
            self.bytecode.instructions[jump_idx].operand =
                Some(Operand::Jump(case_body_starts[case_idx]));
        }

        // Patch default/end jump
        if let Some(default_idx) = default_case {
            self.bytecode.instructions[jump_to_default_or_end].operand =
                Some(Operand::Jump(case_body_starts[default_idx]));
        } else {
            self.bytecode.instructions[jump_to_default_or_end].operand =
                Some(Operand::Jump(end_pos));
        }

        // Pop discriminant
        self.emit(Instruction::simple(OpCode::Pop));

        Ok(())
    }

    fn compile_try_statement(&mut self, try_stmt: &TryStatement) -> Result<(), Error> {
        // Try-catch-finally is complex - for now, we'll just compile the blocks
        // In a full implementation, would use exception handling tables

        // Compile try block
        for stmt in &try_stmt.block.body {
            self.compile_statement(stmt, false)?;
        }

        // Compile catch block if present
        if let Some(handler) = &try_stmt.handler {
            // In a full impl, would:
            // 1. Set up exception handler
            // 2. Bind caught exception to param
            // 3. Handle the exception

            // For now, just compile the catch body
            for stmt in &handler.body.body {
                self.compile_statement(stmt, false)?;
            }
        }

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

    // ========================================================================
    // Expression Compilation
    // ========================================================================

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
                }
            }
            Expression::Member(member) => {
                // obj.prop++ or obj[key]++
                // This is more complex - need to:
                // 1. Evaluate object
                // 2. Evaluate property key
                // 3. Get current value
                // 4. Increment/decrement
                // 5. Store back
                // 6. Return old or new value based on prefix

                // For now, just compile as a simple get
                self.compile_member(member)?;

                // Add 1
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
            }
            _ => {
                return Err(Error::SyntaxError(
                    "Invalid update expression argument".into(),
                ));
            }
        }

        Ok(())
    }

    /// Compile sequence (comma) expressions.
    fn compile_sequence(&mut self, seq: &SequenceExpression) -> Result<(), Error> {
        for (i, expr) in seq.expressions.iter().enumerate() {
            self.compile_expression(expr)?;
            // Pop all but the last result
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

        // Emit new instruction (for now, just call)
        // In a full impl, would create new object with prototype chain
        self.emit(Instruction::with_operand(
            OpCode::Call,
            Operand::ArgCount(new_expr.arguments.len() as u8),
        ));

        Ok(())
    }

    // ========================================================================
    // Utilities
    // ========================================================================

    fn emit(&mut self, instruction: Instruction) -> usize {
        self.bytecode.emit(instruction)
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

