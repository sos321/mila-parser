use super::ast::*;
use std::fmt::Write;

macro_rules! output {
    ($out:expr, $prefix:expr, $connector:expr, $fmt:literal $(, $args:expr )* ) => {
        writeln!($out, concat!("{}{}", $fmt), $prefix, $connector $(, $args )* ).unwrap();
    };
}

macro_rules! output_line {
    ($out:expr, $prefix:expr, $connector:expr, $line:expr, $fmt:literal $(, $args:expr )*) => {
        writeln!($out, concat!("{}{}", $fmt, " <line:{}>"), $prefix, $connector $(, $args )*, $line).unwrap();
    };
}

macro_rules! output_lines {
    ($out:expr, $prefix:expr, $connector:expr, $from:expr, $to:expr, $fmt:literal $(, $args:expr )*) => {
        writeln!($out, concat!("{}{}", $fmt, " <line:{}-{}>"), $prefix, $connector $(, $args )*, $from, $to).unwrap();
    };
}

pub trait PrettyPrint {
    fn pretty(&self, prefix: &str, is_last: bool, out: &mut String);
}

impl Program {
    // called on program
    pub fn print_ast(&self) {
        let mut buf = String::new();

        self.pretty("", true, &mut buf);

        print!("{}", buf);
    }
}

fn move_prefix(prefix: &str, is_last: bool) -> String {
    if is_last {
        return format!("{}   ", prefix);
    } else {
        return format!("{}|  ", prefix);
    };
}

fn decide_connector(is_last: bool) -> &'static str {
    if is_last { "`- " } else { "|- " }
}

// Operators
impl PrettyPrint for BinaryOperator {
    fn pretty(&self, prefix: &str, is_last: bool, out: &mut String) {
        let connector = decide_connector(is_last);

        output!(out, prefix, connector, "BinaryOp {:?}", self);
    }
}

impl PrettyPrint for UnaryOperator {
    fn pretty(&self, prefix: &str, is_last: bool, out: &mut String) {
        let connector = decide_connector(is_last);

        output!(out, prefix, connector, "UnaryOp {:?}", self);
    }
}

// Identifier
impl PrettyPrint for Identifier {
    fn pretty(&self, prefix: &str, is_last: bool, out: &mut String) {
        let connector = decide_connector(is_last);

        output_line!(
            out,
            prefix,
            connector,
            self.line,
            "Identifier \"{}\"",
            self.name
        );
    }
}

// Literals
impl PrettyPrint for ConstantLiteral {
    fn pretty(&self, prefix: &str, is_last: bool, out: &mut String) {
        let connector = decide_connector(is_last);

        // based on type
        match self {
            ConstantLiteral::Number { value, line } => {
                output_line!(out, prefix, connector, line, "NumberLiteral {}", value);
            }

            ConstantLiteral::String { value, line } => {
                output_line!(out, prefix, connector, line, "StringLiteral \"{}\"", value);
            }
        }
    }
}

// Program
impl PrettyPrint for Program {
    fn pretty(&self, prefix: &str, is_last: bool, out: &mut String) {
        let connector = "";

        output_lines!(out, prefix, connector, self.start, self.end, "Program");

        // children ident
        let child_prefix = move_prefix(prefix, is_last);

        // name
        self.name.pretty(&child_prefix, false, out);

        // const section
        match &self.const_section {
            Some(cs) => cs.pretty(&child_prefix, false, out),
            None => {
                writeln!(out, "{}|- ConstSection <none>", child_prefix).unwrap();
            }
        }

        // var section
        match &self.var_section {
            Some(vs) => vs.pretty(&child_prefix, false, out),
            None => {
                writeln!(out, "{}|- VarSection <none>", child_prefix).unwrap();
            }
        }

        // declarations list
        if self.decl_list.is_empty() {
            writeln!(out, "{}|- GlobalDeclarations <empty>", child_prefix).unwrap();
        } else {
            for (i, decl) in self.decl_list.iter().enumerate() {
                decl.pretty(&child_prefix, i == (self.decl_list.len() - 1), out);
            }
        }

        // main body - always at the end
        self.body.pretty(&child_prefix, true, out);
    }
}

// Variable declarations
impl PrettyPrint for ConstSection {
    fn pretty(&self, prefix: &str, is_last: bool, out: &mut String) {
        let connector = decide_connector(is_last);

        output_line!(out, prefix, connector, self.line, "ConstSection");

        let child_prefix = move_prefix(prefix, is_last);

        if self.declarations.is_empty() {
            writeln!(out, "{}|- <empty>", child_prefix).unwrap();
        } else {
            for (i, decl) in self.declarations.iter().enumerate() {
                decl.pretty(&child_prefix, i == (self.declarations.len() - 1), out);
            }
        }
    }
}

impl PrettyPrint for ConstDecl {
    fn pretty(&self, prefix: &str, is_last: bool, out: &mut String) {
        let connector = decide_connector(is_last);

        output_line!(
            out,
            prefix,
            connector,
            self.line,
            "ConstDecl \"{}\"",
            self.name.name
        );

        let child_prefix = move_prefix(prefix, is_last);

        // name
        self.name.pretty(&child_prefix, false, out);

        // value
        self.value.pretty(&child_prefix, true, out);
    }
}

impl PrettyPrint for VarSection {
    fn pretty(&self, prefix: &str, is_last: bool, out: &mut String) {
        let connector = decide_connector(is_last);
        output!(out, prefix, connector, "VarSection");

        let child_prefix = move_prefix(prefix, is_last);

        // handle empty
        if self.declarations.is_empty() {
            writeln!(out, "{}|- <empty>", child_prefix).unwrap();
        } else {
            // for each variable line
            for (i, decl_line) in self.declarations.iter().enumerate() {
                decl_line.pretty(&child_prefix, i == (self.declarations.len() - 1), out);
            }
        }
    }
}

impl PrettyPrint for VarDeclLine {
    fn pretty(&self, prefix: &str, is_last: bool, out: &mut String) {
        let connector = decide_connector(is_last);

        output_line!(out, prefix, connector, self.line, "VarDeclLine");

        let child_prefix = move_prefix(prefix, is_last);

        // identifiers
        if self.identifiers.is_empty() {
            writeln!(out, "{}|- Identifiers <none>", child_prefix).unwrap();
        } else {
            for (i, id) in self.identifiers.iter().enumerate() {
                id.pretty(&child_prefix, i == (self.identifiers.len() - 1), out);
            }
        }

        // type specifier
        self.type_specifier.pretty(&child_prefix, true, out);
    }
}

impl PrettyPrint for TypeSpecifier {
    fn pretty(&self, prefix: &str, is_last: bool, out: &mut String) {
        let connector = decide_connector(is_last);

        match self {
            TypeSpecifier::Base(bt) => {
                output!(out, prefix, connector, "BaseType {:?}", bt);
            }
            TypeSpecifier::Array(arr) => {
                output!(out, prefix, connector, "ArrayType");

                let child_prefix = move_prefix(prefix, is_last);
                arr.pretty(&child_prefix, true, out);
            }
        }
    }
}

impl PrettyPrint for BaseType {
    fn pretty(&self, prefix: &str, is_last: bool, out: &mut String) {
        let connector = decide_connector(is_last);

        output!(out, prefix, connector, "BaseType {:?}", self);
    }
}

impl PrettyPrint for ArrayType {
    fn pretty(&self, prefix: &str, is_last: bool, out: &mut String) {
        let connector = decide_connector(is_last);

        output_line!(
            out,
            prefix,
            connector,
            self.line,
            "ArrayType [{}..{}]",
            self.low_bound,
            self.high_bound
        );

        let child_prefix = move_prefix(prefix, is_last);

        // type
        self.element_type.pretty(&child_prefix, true, out);
    }
}

// Functions & Procedures
impl PrettyPrint for GlobalDeclaration {
    fn pretty(&self, prefix: &str, is_last: bool, out: &mut String) {
        match self {
            GlobalDeclaration::Function(fd) => fd.pretty(prefix, is_last, out),
            GlobalDeclaration::Procedure(pd) => pd.pretty(prefix, is_last, out),
        }
    }
}

impl PrettyPrint for FunctionDecl {
    fn pretty(&self, prefix: &str, is_last: bool, out: &mut String) {
        let connector = decide_connector(is_last);

        output_lines!(
            out,
            prefix,
            connector,
            self.start,
            self.end,
            "FunctionDecl \"{}\"",
            self.name.name
        );

        let child_prefix = move_prefix(prefix, is_last);

        match &self.body {
            Implementation::Forwarded => {
                writeln!(out, "{}`- Forwarded", child_prefix).unwrap();
            }
            Implementation::Implemented { var_section, body } => {
                // name
                self.name.pretty(&child_prefix, false, out);
                // params
                if self.params.is_empty() {
                    writeln!(out, "{}|- Params <none>", child_prefix).unwrap();
                } else {
                    for (i, param) in self.params.iter().enumerate() {
                        param.pretty(&child_prefix, i == (self.params.len() - 1), out);
                    }
                }

                // return type
                writeln!(out, "{}|- ReturnType {:?}", child_prefix, self.return_type).unwrap();

                // variables
                match &var_section {
                    Some(vs) => vs.pretty(&child_prefix, false, out),
                    None => writeln!(out, "{}|- VarSection <none>", child_prefix).unwrap(),
                }

                // body
                body.pretty(&child_prefix, true, out);
            }
        }
    }
}

impl PrettyPrint for ProcedureDecl {
    fn pretty(&self, prefix: &str, is_last: bool, out: &mut String) {
        let connector = decide_connector(is_last);

        output_lines!(
            out,
            prefix,
            connector,
            self.start,
            self.end,
            "ProcedureDecl \"{}\"",
            self.name.name
        );

        let child_prefix = move_prefix(prefix, is_last);

        match &self.body {
            Implementation::Forwarded => {
                writeln!(out, "{}`- Forwarded", child_prefix).unwrap();
            }
            Implementation::Implemented { var_section, body } => {
                // name
                self.name.pretty(&child_prefix, false, out);
                // params
                if self.params.is_empty() {
                    writeln!(out, "{}|- Params <none>", child_prefix).unwrap();
                } else {
                    for (i, param) in self.params.iter().enumerate() {
                        param.pretty(&child_prefix, i == (self.params.len() - 1), out);
                    }
                }

                // variables
                match &var_section {
                    Some(vs) => vs.pretty(&child_prefix, false, out),
                    None => writeln!(out, "{}|- VarSection <none>", child_prefix).unwrap(),
                }

                // body
                body.pretty(&child_prefix, true, out);
            }
        }
    }
}

impl PrettyPrint for Parameter {
    fn pretty(&self, prefix: &str, is_last: bool, out: &mut String) {
        let connector = decide_connector(is_last);

        output_line!(out, prefix, connector, self.line, "Parameter");

        let child_prefix = move_prefix(prefix, is_last);

        // identifiers
        if self.identifiers.is_empty() {
            writeln!(out, "{}|- Identifiers <none>", child_prefix).unwrap();
        } else {
            for (i, id) in self.identifiers.iter().enumerate() {
                id.pretty(&child_prefix, i == (self.identifiers.len() - 1), out);
            }
        }

        // base type
        writeln!(
            out,
            "{}`- TypeSpecifier BaseType {:?}",
            child_prefix, self.type_specifier
        )
        .unwrap();
    }
}

// Statements
impl PrettyPrint for CompoundStatement {
    fn pretty(&self, prefix: &str, is_last: bool, out: &mut String) {
        let connector = decide_connector(is_last);

        output_lines!(
            out,
            prefix,
            connector,
            self.start,
            self.end,
            "CompoundStatement"
        );

        let child_prefix = move_prefix(prefix, is_last);

        if self.statements.is_empty() {
            writeln!(out, "{}|- <empty>", child_prefix).unwrap();
        } else {
            for (i, stmt) in self.statements.iter().enumerate() {
                stmt.pretty(&child_prefix, i == (self.statements.len() - 1), out);
            }
        }
    }
}

impl PrettyPrint for Statement {
    fn pretty(&self, prefix: &str, is_last: bool, out: &mut String) {
        let connector = decide_connector(is_last);
        let child_prefix = move_prefix(prefix, is_last);

        // all of the statements
        match self {
            Statement::Assignment {
                target,
                expression,
                line,
            } => {
                output_line!(out, prefix, connector, line, "Assignment");

                target.pretty(&child_prefix, false, out);
                expression.pretty(&child_prefix, true, out);
            }
            Statement::Call {
                name,
                arguments,
                line,
            } => {
                output_line!(out, prefix, connector, line, "Call \"{}\"", name.name);

                if arguments.is_empty() {
                    writeln!(out, "{}|- Args <none>", child_prefix).unwrap();
                } else {
                    for (i, arg) in arguments.iter().enumerate() {
                        arg.pretty(&child_prefix, i == (arguments.len() - 1), out);
                    }
                }
            }
            Statement::If {
                condition,
                then_branch,
                else_branch,
                start,
            } => {
                output_line!(out, prefix, connector, start, "If");

                // condition
                writeln!(out, "{}|- Condition", child_prefix).unwrap();
                condition.pretty(&format!("{}|  ", child_prefix), true, out);

                // then branch
                then_branch.pretty(&child_prefix, else_branch.is_none(), out);

                // else branch
                if let Some(eb) = else_branch {
                    eb.pretty(&child_prefix, true, out);
                }
            }
            Statement::While {
                condition,
                body,
                start,
            } => {
                output_line!(out, prefix, connector, start, "While");

                writeln!(out, "{}|- Condition", child_prefix).unwrap();
                condition.pretty(&format!("{}|  ", child_prefix), true, out);
                body.pretty(&child_prefix, true, out);
            }
            Statement::For {
                loop_var,
                start_expr,
                for_type,
                end_expr,
                body,
                start,
            } => {
                output_line!(out, prefix, connector, start, "For \"{}\"", loop_var.name);

                // start expression
                writeln!(out, "{}|- StartExpr", child_prefix).unwrap();
                start_expr.pretty(&format!("{}|  ", child_prefix), true, out);

                // for type
                writeln!(out, "{}|- ForType {:?}", child_prefix, for_type).unwrap();

                // end
                writeln!(out, "{}|- EndExpr", child_prefix).unwrap();
                end_expr.pretty(&format!("{}|  ", child_prefix), true, out);

                // body
                body.pretty(&child_prefix, true, out);
            }
            Statement::Compound(cs) => {
                output!(out, prefix, connector, "Compound");

                cs.pretty(&child_prefix, true, out);
            }
            Statement::Exit { line } => {
                output_line!(out, prefix, connector, line, "Exit");
            }
            Statement::Writeln { argument, line } => {
                output_line!(out, prefix, connector, line, "Writeln");

                if let Some(arg) = argument {
                    arg.pretty(&child_prefix, true, out);
                }
            }
            Statement::Write { argument, line } => {
                output_line!(out, prefix, connector, line, "Write");

                argument.pretty(&child_prefix, true, out);
            }
            Statement::Readln { target, line } => {
                output_line!(out, prefix, connector, line, "Readln");

                target.pretty(&child_prefix, true, out);
            }
            Statement::Empty => {
                output!(out, prefix, connector, "Empty");
            }
        }
    }
}

impl PrettyPrint for WriteArgument {
    fn pretty(&self, prefix: &str, is_last: bool, out: &mut String) {
        let connector = decide_connector(is_last);
        match self {
            WriteArgument::StringLiteral { value } => {
                output!(out, prefix, connector, "StringLiteral \"{}\"", value);
            }
            WriteArgument::Expression(expr) => {
                output!(out, prefix, connector, "ExprArg");

                let child_prefix = move_prefix(prefix, is_last);
                expr.pretty(&child_prefix, true, out);
            }
        }
    }
}

impl PrettyPrint for AssignmentTarget {
    fn pretty(&self, prefix: &str, is_last: bool, out: &mut String) {
        let connector = decide_connector(is_last);
        match self {
            AssignmentTarget::Identifier(id) => {
                output_line!(
                    out,
                    prefix,
                    connector,
                    id.line,
                    "TargetIdentifier \"{}\"",
                    id.name
                );
            }
            AssignmentTarget::ArrayAccess { name, index_expr } => {
                output_line!(
                    out,
                    prefix,
                    connector,
                    name.line,
                    "ArrayAccess \"{}\"",
                    name.name
                );

                let child_prefix = move_prefix(prefix, is_last);
                index_expr.pretty(&child_prefix, true, out);
            }
        }
    }
}

impl PrettyPrint for ForType {
    fn pretty(&self, prefix: &str, is_last: bool, out: &mut String) {
        let connector = decide_connector(is_last);

        output!(out, prefix, connector, "ForType {:?}", self);
    }
}

// Expressions
impl PrettyPrint for Expression {
    fn pretty(&self, prefix: &str, is_last: bool, out: &mut String) {
        let connector = decide_connector(is_last);
        let child_prefix = move_prefix(prefix, is_last);

        match self {
            Expression::Literal(lit) => {
                output!(out, prefix, connector, "Literal");
                lit.pretty(&child_prefix, true, out);
            }
            Expression::Variable(id) => {
                output_line!(out, prefix, connector, id.line, "Variable \"{}\"", id.name);
            }
            Expression::ArrayAccess { name, index_expr } => {
                output_line!(
                    out,
                    prefix,
                    connector,
                    name.line,
                    "ArrayAccess \"{}\"",
                    name.name
                );

                index_expr.pretty(&child_prefix, true, out);
            }
            Expression::FunctionCall { name, arguments } => {
                output_line!(
                    out,
                    prefix,
                    connector,
                    name.line,
                    "FunctionCall \"{}\"",
                    name.name
                );

                if arguments.is_empty() {
                    writeln!(out, "{}|- Args <none>", child_prefix).unwrap();
                } else {
                    for (i, arg) in arguments.iter().enumerate() {
                        arg.pretty(&child_prefix, i == (arguments.len() - 1), out);
                    }
                }
            }
            Expression::BinaryOp {
                left,
                operator,
                right,
            } => {
                output!(out, prefix, connector, "BinaryOp {:?}", operator);

                left.pretty(&child_prefix, false, out);
                right.pretty(&child_prefix, true, out);
            }
            Expression::UnaryOp { operator, operand } => {
                output!(out, prefix, connector, "UnaryOp {:?}", operator);

                operand.pretty(&child_prefix, true, out);
            }
            Expression::Grouped(expr) => {
                output!(out, prefix, connector, "Grouped");

                expr.pretty(&child_prefix, true, out);
            }
        }
    }
}
