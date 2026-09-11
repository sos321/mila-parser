use crate::parser::ast::*;
use inkwell::IntPredicate;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::{Linkage, Module};
use inkwell::types::{BasicType, BasicTypeEnum};
use inkwell::values::{BasicValueEnum, FunctionValue, PointerValue};
use std::collections::HashMap;

type CompileResult<T> = Result<T, String>;

struct ScopeManager<'ctx> {
    scopes: Vec<HashMap<String, PointerValue<'ctx>>>,
}

impl<'ctx> ScopeManager<'ctx> {
    fn new() -> Self {
        ScopeManager {
            scopes: vec![HashMap::new()],
        }
    }

    fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn leave_scope(&mut self) {
        self.scopes.pop();
    }

    fn add_variable(&mut self, name: &str, ptr: PointerValue<'ctx>) {
        self.scopes
            .last_mut()
            .unwrap()
            .insert(name.to_string(), ptr);
    }

    fn find_variable(&self, name: &str) -> Option<&PointerValue<'ctx>> {
        for scope in self.scopes.iter().rev() {
            if let Some(ptr) = scope.get(name) {
                return Some(ptr);
            }
        }
        None
    }
}

/// The main compiler structure.
pub struct Compiler<'ctx> {
    pub context: &'ctx Context,
    pub builder: Builder<'ctx>,
    pub module: Module<'ctx>,

    scopes: ScopeManager<'ctx>,
    constants: HashMap<String, ConstantLiteral>,
    functions: HashMap<
        String,
        (
            FunctionValue<'ctx>,
            Option<Vec<Parameter>>,
            Option<BaseType>,
        ),
    >,

    current_function_return_block: Option<inkwell::basic_block::BasicBlock<'ctx>>,
}

impl<'ctx> Compiler<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        let builder = context.create_builder();
        let module = context.create_module("main");

        Compiler {
            context,
            builder,
            module,
            scopes: ScopeManager::new(),
            constants: HashMap::new(),
            functions: HashMap::new(),
            current_function_return_block: None,
        }
    }

    pub fn compile_program(&mut self, program: &Program) -> CompileResult<()> {
        // Process constants
        if let Some(const_section) = &program.const_section {
            for decl in &const_section.declarations {
                self.constants
                    .insert(decl.name.name.clone(), decl.value.clone());
            }
        }

        // Process global variables
        if let Some(var_section) = &program.var_section {
            for decl_line in &var_section.declarations {
                self.compile_var_decl(decl_line)?;
            }
        }

        // First pass: Declare all functions and procedures to handle forward calls
        self.declare_builtins();
        for decl in &program.decl_list {
            match decl {
                GlobalDeclaration::Function(f) => self.declare_function(f)?,
                GlobalDeclaration::Procedure(p) => self.declare_procedure(p)?,
            }
        }

        // Second pass: Compile the implementations
        for decl in &program.decl_list {
            match decl {
                GlobalDeclaration::Function(f) => self.compile_function(f)?,
                GlobalDeclaration::Procedure(p) => self.compile_procedure(p)?,
            };
        }

        // Compile the main program body as the `main` function
        let main_fn_type = self.context.i32_type().fn_type(&[], false);
        let main_fn = self.module.add_function("main", main_fn_type, None);
        let entry_block = self.context.append_basic_block(main_fn, "entry");
        self.builder.position_at_end(entry_block);

        self.compile_compound_statement(&program.body)?;

        // Ensure main function returns 0
        self.builder
            .build_return(Some(&self.context.i32_type().const_int(0, false)));

        Ok(())
    }

    fn compile_var_decl(&mut self, decl_line: &VarDeclLine) -> CompileResult<()> {
        let llvm_type = self.to_llvm_type(&decl_line.type_specifier)?;
        for ident in &decl_line.identifiers {
            let alloca = self.builder.build_alloca(llvm_type, &ident.name);
            self.scopes.add_variable(&ident.name, alloca);
        }
        Ok(())
    }

    /// Converts a language type to an LLVM type.
    fn to_llvm_type(&self, ty: &TypeSpecifier) -> CompileResult<BasicTypeEnum<'ctx>> {
        match ty {
            TypeSpecifier::Base(BaseType::Integer) => Ok(self.context.i32_type().into()),
            TypeSpecifier::Array(array_type) => {
                let element_type =
                    self.to_llvm_type(&TypeSpecifier::Base(array_type.element_type.clone()))?;
                let size = (array_type.high_bound - array_type.low_bound + 1) as u32;
                Ok(element_type.array_type(size).into())
            }
        }
    }

    // --- Expression Compilation ---

    fn compile_expression(&mut self, expr: &Expression) -> CompileResult<BasicValueEnum<'ctx>> {
        match expr {
            Expression::Literal(lit) => self.compile_literal(lit),
            Expression::Variable(ident) => {
                let ptr = self
                    .scopes
                    .find_variable(&ident.name)
                    .ok_or_else(|| format!("Variable '{}' not found", ident.name))?;
                Ok(self.builder.build_load(*ptr, &ident.name))
            }
            Expression::FunctionCall { name, arguments } => {
                self.compile_function_call(name, arguments)
            }
            Expression::BinaryOp {
                left,
                operator,
                right,
            } => self.compile_binary_op(left, operator, right),
            Expression::UnaryOp { operator, operand } => self.compile_unary_op(operator, operand),
            Expression::Grouped(inner) => self.compile_expression(inner),
            // ArrayAccess compilation is handled in assignment/load contexts via GEP.
            Expression::ArrayAccess { name, index_expr } => {
                let array_ptr = self.compile_assignment_target(&AssignmentTarget::ArrayAccess {
                    name: name.clone(),
                    index_expr: index_expr.clone(),
                })?;
                Ok(self.builder.build_load(array_ptr, &name.name))
            }
        }
    }

    fn compile_literal(&self, lit: &ConstantLiteral) -> CompileResult<BasicValueEnum<'ctx>> {
        match lit {
            ConstantLiteral::Number { value, .. } => Ok(self
                .context
                .i32_type()
                .const_int(*value as u64, true)
                .into()),
            ConstantLiteral::String { value, .. } => Ok(self
                .builder
                .build_global_string_ptr(value, ".str")
                .as_pointer_value()
                .into()),
        }
    }

    fn compile_binary_op(
        &mut self,
        left: &Expression,
        op: &BinaryOperator,
        right: &Expression,
    ) -> CompileResult<BasicValueEnum<'ctx>> {
        let lhs = self.compile_expression(left)?.into_int_value();
        let rhs = self.compile_expression(right)?.into_int_value();

        let result = match op {
            BinaryOperator::Add => self.builder.build_int_add(lhs, rhs, "add"),
            BinaryOperator::Subtract => self.builder.build_int_sub(lhs, rhs, "sub"),
            BinaryOperator::Multiply => self.builder.build_int_mul(lhs, rhs, "mul"),
            BinaryOperator::Divide => self.builder.build_int_signed_div(lhs, rhs, "div"),
            BinaryOperator::Modulo => self.builder.build_int_signed_rem(lhs, rhs, "mod"),
            BinaryOperator::Equal => {
                self.builder
                    .build_int_compare(IntPredicate::EQ, lhs, rhs, "eq")
            }
            BinaryOperator::NotEqual => {
                self.builder
                    .build_int_compare(IntPredicate::NE, lhs, rhs, "ne")
            }
            BinaryOperator::Less => {
                self.builder
                    .build_int_compare(IntPredicate::SLT, lhs, rhs, "lt")
            }
            BinaryOperator::LessEqual => {
                self.builder
                    .build_int_compare(IntPredicate::SLE, lhs, rhs, "le")
            }
            BinaryOperator::Greater => {
                self.builder
                    .build_int_compare(IntPredicate::SGT, lhs, rhs, "gt")
            }
            BinaryOperator::GreaterEqual => {
                self.builder
                    .build_int_compare(IntPredicate::SGE, lhs, rhs, "ge")
            }
            BinaryOperator::And => self.builder.build_and(lhs, rhs, "and"),
            BinaryOperator::Or => self.builder.build_or(lhs, rhs, "or"),
        };
        Ok(result.unwrap().into())
    }

    fn compile_unary_op(
        &mut self,
        op: &UnaryOperator,
        operand: &Expression,
    ) -> CompileResult<BasicValueEnum<'ctx>> {
        let val = self.compile_expression(operand)?.into_int_value();
        let result = match op {
            UnaryOperator::Plus => val,
            UnaryOperator::Minus => self.builder.build_int_neg(val, "neg"),
            UnaryOperator::Not => self.builder.build_not(val, "not"),
        };
        Ok(result.into())
    }

    fn compile_function_call(
        &mut self,
        name: &Identifier,
        arguments: &[Expression],
    ) -> CompileResult<BasicValueEnum<'ctx>> {
        let (function, _, _) = self
            .functions
            .get(&name.name)
            .ok_or_else(|| format!("Function '{}' not found", name.name))?;

        let mut compiled_args = Vec::new();
        for arg in arguments {
            compiled_args.push(self.compile_expression(arg)?.into());
        }

        let call = self
            .builder
            .build_call(*function, &compiled_args, "calltmp");

        match call.try_as_basic_value().left() {
            Some(value) => Ok(value),
            None => Err("Function call does not return a value.".to_string()),
        }
    }

    // --- Statement Compilation ---

    fn compile_statement(&mut self, stmt: &Statement) -> CompileResult<()> {
        match stmt {
            Statement::Compound(c) => self.compile_compound_statement(c),
            Statement::Assignment {
                target, expression, ..
            } => self.compile_assignment(target, expression),
            Statement::Call {
                name, arguments, ..
            } => {
                self.compile_function_call(name, arguments)?;
                Ok(())
            }
            Statement::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => self.compile_if(condition, then_branch, else_branch),
            Statement::While {
                condition, body, ..
            } => self.compile_while(condition, body),
            Statement::Exit { .. } => {
                let return_block = self.current_function_return_block.ok_or_else(|| {
                    "Exit can only be used inside a function or procedure.".to_string()
                })?;
                self.builder.build_unconditional_branch(return_block);
                Ok(())
            }
            Statement::Writeln { argument, .. } => self.compile_writeln(argument),
            Statement::Empty => Ok(()), // Do nothing
            _ => Err(format!("Unsupported statement: {:?}", stmt)),
        }
    }

    fn compile_compound_statement(&mut self, compound: &CompoundStatement) -> CompileResult<()> {
        for stmt in &compound.statements {
            self.compile_statement(stmt)?;
        }
        Ok(())
    }

    fn compile_assignment(
        &mut self,
        target: &AssignmentTarget,
        expr: &Expression,
    ) -> CompileResult<()> {
        let value_to_store = self.compile_expression(expr)?;
        let target_ptr = self.compile_assignment_target(target)?;
        self.builder.build_store(target_ptr, value_to_store);
        Ok(())
    }

    fn compile_assignment_target(
        &mut self,
        target: &AssignmentTarget,
    ) -> CompileResult<PointerValue<'ctx>> {
        match target {
            AssignmentTarget::Identifier(ident) => self
                .scopes
                .find_variable(&ident.name)
                .cloned()
                .ok_or_else(|| format!("Cannot assign to undeclared variable '{}'", ident.name)),
            AssignmentTarget::ArrayAccess { name, index_expr } => {
                let array_ptr = self
                    .scopes
                    .find_variable(&name.name)
                    .ok_or_else(|| format!("Array '{}' not found", name.name))?;
                let index_val = self.compile_expression(index_expr)?.into_int_value();

                // For Pascal-like arrays, we might need to adjust for the lower bound if it's not 0.
                // Assuming the AST provides this info or we look it up. For simplicity, we assume 0-based for now.
                let zero = self.context.i32_type().const_int(0, false);

                unsafe {
                    Ok(self
                        .builder
                        .build_gep(*array_ptr, &[zero, index_val], "arrayidx")
                        .map_err(|_| "An error has occurred".to_string())?)
                }
            }
        }
    }

    fn compile_if(
        &mut self,
        condition: &Expression,
        then_branch: &Statement,
        else_branch: &Option<Box<Statement>>,
    ) -> CompileResult<()> {
        let cond_val = self.compile_expression(condition)?.into_int_value();

        let parent_fn = self
            .builder
            .get_insert_block()
            .unwrap()
            .get_parent()
            .unwrap();

        let then_bb = self.context.append_basic_block(parent_fn, "then");
        let else_bb = self.context.append_basic_block(parent_fn, "else");
        let merge_bb = self.context.append_basic_block(parent_fn, "if_cont");

        self.builder
            .build_conditional_branch(cond_val, then_bb, else_bb);

        // Compile 'then' block
        self.builder.position_at_end(then_bb);
        self.compile_statement(then_branch)?;
        if self
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            self.builder.build_unconditional_branch(merge_bb);
        }

        // Compile 'else' block
        self.builder.position_at_end(else_bb);
        if let Some(else_stmt) = else_branch {
            self.compile_statement(else_stmt)?;
        }
        if self
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            self.builder.build_unconditional_branch(merge_bb);
        }

        self.builder.position_at_end(merge_bb);
        Ok(())
    }

    fn compile_while(&mut self, condition: &Expression, body: &Statement) -> CompileResult<()> {
        let parent_fn = self
            .builder
            .get_insert_block()
            .unwrap()
            .get_parent()
            .unwrap();

        let cond_bb = self.context.append_basic_block(parent_fn, "while_cond");
        let loop_bb = self.context.append_basic_block(parent_fn, "while_loop");
        let after_bb = self.context.append_basic_block(parent_fn, "after_while");

        self.builder.build_unconditional_branch(cond_bb);
        self.builder.position_at_end(cond_bb);

        let cond_val = self.compile_expression(condition)?.into_int_value();
        self.builder
            .build_conditional_branch(cond_val, loop_bb, after_bb);

        self.builder.position_at_end(loop_bb);
        self.compile_statement(body)?;
        if self
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            self.builder.build_unconditional_branch(cond_bb);
        }

        self.builder.position_at_end(after_bb);
        Ok(())
    }

    // --- Function and Procedure Compilation ---

    fn declare_function(&mut self, func: &FunctionDecl) -> CompileResult<()> {
        let param_types = self.get_param_llvm_types(&func.params)?;
        let ret_type = self.to_llvm_type(&TypeSpecifier::Base(func.return_type.clone()))?;
        let fn_type = ret_type.fn_type(&param_types, false);
        let function = self.module.add_function(&func.name.name, fn_type, None);
        self.functions.insert(
            func.name.name.clone(),
            (
                function,
                Some(func.params.clone()),
                Some(func.return_type.clone()),
            ),
        );
        Ok(())
    }

    fn declare_procedure(&mut self, proc: &ProcedureDecl) -> CompileResult<()> {
        let param_types = self.get_param_llvm_types(&proc.params)?;
        let fn_type = self.context.void_type().fn_type(&param_types, false);
        let function = self.module.add_function(&proc.name.name, fn_type, None);
        self.functions.insert(
            proc.name.name.clone(),
            (function, Some(proc.params.clone()), None),
        );
        Ok(())
    }

    fn compile_function(&mut self, func: &FunctionDecl) -> CompileResult<()> {
        if let Implementation::Implemented { var_section, body } = &func.body {
            let (function, _, _) = self.functions.get(&func.name.name).unwrap();

            let entry = self.context.append_basic_block(*function, "entry");
            let return_block = self.context.append_basic_block(*function, "return");
            self.current_function_return_block = Some(return_block);

            self.builder.position_at_end(entry);
            self.scopes.enter_scope();

            // Allocate space for params and store initial values
            for (i, param) in function.get_param_iter().enumerate() {
                for ident in &func.params[i].identifiers {
                    let alloca = self.builder.build_alloca(param.get_type(), &ident.name);
                    self.builder.build_store(alloca, param);
                    self.scopes.add_variable(&ident.name, alloca);
                }
            }

            // Compile local variables
            if let Some(vars) = var_section {
                for decl_line in &vars.declarations {
                    self.compile_var_decl(decl_line)?;
                }
            }

            self.compile_compound_statement(body)?;

            if self
                .builder
                .get_insert_block()
                .unwrap()
                .get_terminator()
                .is_none()
            {
                self.builder.build_unconditional_branch(return_block);
            }

            self.builder.position_at_end(return_block);
            // TODO: Handle actual return value based on user logic. For now, default.
            let ret_val = self.context.i32_type().const_int(0, false);
            self.builder.build_return(Some(&ret_val));

            self.scopes.leave_scope();
            self.current_function_return_block = None;
        }
        Ok(())
    }

    fn compile_procedure(&mut self, proc: &ProcedureDecl) -> CompileResult<()> {
        if let Implementation::Implemented { var_section, body } = &proc.body {
            let (function, _, _) = self.functions.get(&proc.name.name).unwrap();

            let entry = self.context.append_basic_block(*function, "entry");
            let return_block = self.context.append_basic_block(*function, "return");
            self.current_function_return_block = Some(return_block);

            self.builder.position_at_end(entry);
            self.scopes.enter_scope();

            // Handle params
            for (i, param) in function.get_param_iter().enumerate() {
                for ident in &proc.params[i].identifiers {
                    let alloca = self.builder.build_alloca(param.get_type(), &ident.name);
                    self.builder.build_store(alloca, param);
                    self.scopes.add_variable(&ident.name, alloca);
                }
            }

            // Handle local vars
            if let Some(vars) = var_section {
                for decl_line in &vars.declarations {
                    self.compile_var_decl(decl_line)?;
                }
            }

            self.compile_compound_statement(body)?;

            if self
                .builder
                .get_insert_block()
                .unwrap()
                .get_terminator()
                .is_none()
            {
                self.builder.build_unconditional_branch(return_block);
            }

            self.builder.position_at_end(return_block);
            self.builder.build_return(None);

            self.scopes.leave_scope();
            self.current_function_return_block = None;
        }
        Ok(())
    }

    fn get_param_llvm_types(
        &self,
        params: &[Parameter],
    ) -> CompileResult<Vec<BasicTypeEnum<'ctx>>> {
        let mut types = Vec::new();
        for p in params {
            let llvm_type = self.to_llvm_type(&TypeSpecifier::Base(p.type_specifier.clone()))?;
            for _ in &p.identifiers {
                types.push(llvm_type);
            }
        }
        Ok(types)
    }

    // --- Built-in I/O ---

    fn declare_builtins(&mut self) {
        // `printf(i8*, ...)`
        let i32_type = self.context.i32_type();
        let i8_ptr_type = self.context.i8_type().ptr_type(AddressSpace::from(0u16));
        let printf_type = i32_type.fn_type(&[i8_ptr_type.into()], true);
        let printf_fn = self
            .module
            .add_function("printf", printf_type, Some(Linkage::External));
        self.functions
            .insert("printf".to_string(), (printf_fn, None, None));
    }

    fn compile_writeln(&mut self, arg: &Option<WriteArgument>) -> CompileResult<()> {
        let (printf, _, _) = self.functions.get("printf").unwrap();

        match arg {
            Some(WriteArgument::Expression(expr)) => {
                let format_str = self.builder.build_global_string_ptr("%d\n", ".fmt_d");
                let value = self.compile_expression(expr)?;
                self.builder.build_call(
                    *printf,
                    &[format_str.as_pointer_value().into(), value.into()],
                    "printf_call",
                );
            }
            Some(WriteArgument::StringLiteral { value }) => {
                let full_string = format!("{}\n", value);
                let format_str = self.builder.build_global_string_ptr(&full_string, ".fmt_s");
                self.builder.build_call(
                    *printf,
                    &[format_str.as_pointer_value().into()],
                    "printf_call",
                );
            }
            None => {
                // Just a newline
                let format_str = self.builder.build_global_string_ptr("\n", ".fmt_nl");
                self.builder.build_call(
                    *printf,
                    &[format_str.as_pointer_value().into()],
                    "printf_call",
                );
            }
        }
        Ok(())
    }
}
