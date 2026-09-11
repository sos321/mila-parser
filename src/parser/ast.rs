#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub name: Identifier,
    pub const_section: Option<ConstSection>,
    pub var_section: Option<VarSection>,
    pub decl_list: Vec<GlobalDeclaration>,
    pub body: CompoundStatement,
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Identifier {
    pub name: String,
    pub line: u32,
}

impl Identifier {
    pub fn new(name: String, line: u32) -> Self {
        Identifier { name, line }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConstSection {
    pub declarations: Vec<ConstDecl>,
    pub line: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConstDecl {
    pub name: Identifier,
    pub value: ConstantLiteral,
    pub line: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConstantLiteral {
    Number { value: i32, line: u32 },
    String { value: String, line: u32 },
}

#[derive(Debug, Clone, PartialEq)]
pub struct VarSection {
    pub declarations: Vec<VarDeclLine>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VarDeclLine {
    pub identifiers: Vec<Identifier>,
    pub type_specifier: TypeSpecifier,
    pub line: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeSpecifier {
    Base(BaseType),
    Array(ArrayType),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BaseType {
    Integer,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArrayType {
    pub low_bound: i32,
    pub high_bound: i32,
    pub element_type: BaseType,
    pub line: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GlobalDeclaration {
    Function(FunctionDecl),
    Procedure(ProcedureDecl),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Implementation {
    Forwarded,
    Implemented {
        var_section: Option<VarSection>,
        body: CompoundStatement,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDecl {
    pub name: Identifier,
    pub params: Vec<Parameter>,
    pub return_type: BaseType,
    pub body: Implementation,
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProcedureDecl {
    pub name: Identifier,
    pub params: Vec<Parameter>,
    pub body: Implementation,
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub identifiers: Vec<Identifier>,
    pub type_specifier: BaseType,
    pub line: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompoundStatement {
    pub statements: Vec<Statement>,
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Assignment {
        target: AssignmentTarget,
        expression: Expression,
        line: u32,
    },
    Call {
        name: Identifier,
        arguments: Vec<Expression>,
        line: u32,
    },
    If {
        condition: Expression,
        then_branch: Box<Statement>,
        else_branch: Option<Box<Statement>>,
        start: u32,
    },
    While {
        condition: Expression,
        body: Box<Statement>,
        start: u32,
    },
    For {
        loop_var: Identifier,
        start_expr: Expression,
        for_type: ForType,
        end_expr: Expression,
        body: Box<Statement>,
        start: u32,
    },
    Compound(CompoundStatement),
    Exit {
        line: u32,
    },
    Writeln {
        argument: Option<WriteArgument>,
        line: u32,
    },
    Write {
        argument: WriteArgument,
        line: u32,
    },
    Readln {
        target: AssignmentTarget,
        line: u32,
    },
    Empty,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WriteArgument {
    StringLiteral { value: String },
    Expression(Box<Expression>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AssignmentTarget {
    Identifier(Identifier),
    ArrayAccess {
        name: Identifier,
        index_expr: Box<Expression>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ForType {
    To,
    Downto,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Literal(ConstantLiteral),
    Variable(Identifier),
    ArrayAccess {
        name: Identifier,
        index_expr: Box<Expression>,
    },
    FunctionCall {
        name: Identifier,
        arguments: Vec<Expression>,
    },
    BinaryOp {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
    },
    UnaryOp {
        operator: UnaryOperator,
        operand: Box<Expression>,
    },
    Grouped(Box<Expression>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    Or,
    And,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Plus,
    Minus,
    Not,
}
