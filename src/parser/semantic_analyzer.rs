use super::ast::*;
use super::errors::SemanticError;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolType {
    Integer,
    StringLiteral,
    Boolean,
    Array {
        element_type: Box<SymbolType>,
        low_bound: i32,
        high_bound: i32,
    },
    Procedure {
        param_types: Vec<SymbolType>,
    },
    Function {
        param_types: Vec<SymbolType>,
        return_type: Box<SymbolType>,
    },
    Unknown,
}

pub struct Symbol {
    pub name: String,
    pub symbol_type: SymbolType,
    pub kind: SymbolKind,
    pub scope_level: usize,
    pub declared_at: u32,
}

#[derive(Debug, Clone)]
pub enum SymbolKind {
    Constant,
    Variable,
    Function,
    Procedure,
    Parameter,
}

pub struct SymbolTable {
    scopes: Vec<HashMap<String, Symbol>>,
    current_scope_level: usize,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            scopes: vec![HashMap::new()],
            current_scope_level: 0,
        }
    }

    // go into next scope
    pub fn next_scope(&mut self) {
        self.current_scope_level += 1;
        self.scopes.push(HashMap::new());
    }

    // return to last scope
    pub fn leave_scope(&mut self) {
        if self.current_scope_level > 0 {
            self.current_scope_level -= 1;
            self.scopes.pop();
        }
    }

    pub fn define_symbol(&mut self, symbol: Symbol) -> Result<(), SemanticError> {
        let name = symbol.name.clone();

        if self.scopes[self.current_scope_level].contains_key(&name) {
            Err(SemanticError::new(
                format!("Symbol '{}' already defined in current scope.", name),
                symbol.declared_at,
            ))
        } else {
            self.scopes[self.current_scope_level].insert(name, symbol);
            Ok(())
        }
    }

    fn check_symbol(&self, name: &str) -> Option<&Symbol> {
        // search for symbol in all scopes
        for i in 0..self.current_scope_level {
            if let Some(Symbol) = self.scopes[i].get(name) {
                return Some(Symbol);
            }
        }

        None
    }
}

pub struct SemanticAnalyzer {
    symbol_table: SymbolTable,
    last_return: Option<SymbolType>,
    // current_function_return_type: Option<SymbolType> // To check 'exit' or return statements
}

impl SemanticAnalyzer {
    fn new() -> Self {
        SemanticAnalyzer {
            symbol_table: SymbolTable::new(),
            last_return: None,
        }
    }

    pub fn analyze(&mut self, program: Program) -> Result<(), SemanticError> {
        self.visit_program(program)?;

        Ok(())
    }

    fn visit_program(&mut self, program: Program) -> Result<(), SemanticError> {
        if let Some(consts) = program.const_section {
            self.visit_consts(&consts)?;
        }
        if let Some(vars) = program.var_section {
            self.visit_vars(&vars)?;
        }

        // function and process declaration
        /*
            This could be made into a single pass parser thanks to the forward declarations of all functions
            It is not implemented here
        */
        for decl in &program.decl_list {
            match decl {
                GlobalDeclaration::Function(func) => self.declare_func(func)?,
                GlobalDeclaration::Procedure(proc) => self.declare_proc(proc)?,
            }
        }

        // now everything should be declared
        for decl in &program.decl_list {
            match decl {
                GlobalDeclaration::Function(func) => self.visit_func(func)?,
                GlobalDeclaration::Procedure(proc) => self.visit_proc(proc)?,
            }
        }

        // main block
        self.visit_block(program.body, None);

        Ok(())
    }

    fn resolve_base_type(&self, base_type: &BaseType) -> SymbolType {
        match base_type {
            BaseType::Integer { .. } => SymbolType::Integer,
        }
    }

    fn resolve_type(&self, type_spec: &TypeSpecifier) -> SymbolType {
        match type_spec {
            TypeSpecifier::Base(bt) => self.resolve_base_type(bt),
            TypeSpecifier::Array(at) => SymbolType::Array {
                element_type: Box::new(self.resolve_base_type(&at.element_type)),
                low_bound: at.low_bound,
                high_bound: at.high_bound,
            },
        }
    }

    fn visit_consts(&mut self, consts: &ConstSection) -> Result<(), SemanticError> {
        for c in &consts.declarations {
            let const_type = match c.value {
                ConstantLiteral::Number { .. } => SymbolType::Integer,
                ConstantLiteral::String { .. } => SymbolType::StringLiteral,
            };

            self.symbol_table.define_symbol(Symbol {
                name: c.name.name.clone(),
                symbol_type: const_type,
                kind: SymbolKind::Constant,
                scope_level: self.symbol_table.current_scope_level,
                declared_at: c.line,
            })?;
        }

        Ok(())
    }

    fn visit_vars(&mut self, vars: &VarSection) -> Result<(), SemanticError> {
        for v_line in &vars.declarations {
            let var_type = self.resolve_type(&v_line.type_specifier);

            for v in &v_line.identifiers {
                self.symbol_table.define_symbol(Symbol {
                    name: v.name.clone(),
                    symbol_type: var_type.clone(),
                    kind: SymbolKind::Variable,
                    scope_level: self.symbol_table.current_scope_level,
                    declared_at: v.line,
                })?;
            }
        }

        Ok(())
    }

    fn declare_func(&mut self, func: &FunctionDecl) -> Result<(), SemanticError> {
        let return_type = self.resolve_base_type(&func.return_type);
        let mut param_types = Vec::new();

        for p_line in &func.params {
            let p_type = self.resolve_base_type(&p_line.type_specifier);
            for _ in &p_line.identifiers {
                param_types.push(p_type.clone());
            }

            self.symbol_table.define_symbol(Symbol {
                name: func.name.name.clone(),
                symbol_type: SymbolType::Function {
                    param_types: param_types.clone(),
                    return_type: Box::new(return_type.clone()),
                },
                kind: SymbolKind::Function,
                scope_level: self.symbol_table.current_scope_level,
                declared_at: func.start,
            })?;
        }

        Ok(())
    }

    fn declare_proc(&mut self, proc: &ProcedureDecl) -> Result<(), SemanticError> {
        let mut param_types = Vec::new();

        for p_line in &proc.params {
            let p_type = self.resolve_base_type(&p_line.type_specifier);
            for _ in &p_line.identifiers {
                param_types.push(p_type.clone());
            }

            self.symbol_table.define_symbol(Symbol {
                name: proc.name.name.clone(),
                symbol_type: SymbolType::Procedure {
                    param_types: param_types.clone(),
                },
                kind: SymbolKind::Procedure,
                scope_level: self.symbol_table.current_scope_level,
                declared_at: proc.start,
            })?;
        }

        Ok(())
    }

    fn visit_func(&mut self, func: &FunctionDecl) -> Result<(), SemanticError> {
        self.symbol_table.next_scope();

        // declare params
        for p_line in &func.params {
            let param_type = self.resolve_base_type(&p_line.type_specifier);
            for p in &p_line.identifiers {
                self.symbol_table.define_symbol(Symbol {
                    name: p.name.clone(),
                    symbol_type: param_type.clone(),
                    kind: SymbolKind::Parameter,
                    scope_level: self.symbol_table.current_scope_level,
                    declared_at: p.line,
                })?;
            }
        }

        // vars
        if let Some(vars) = &func
        Ok(())
    }
}
