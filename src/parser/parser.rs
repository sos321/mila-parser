use super::ast::*;
use super::errors::*;
use crate::lexer::Lexer;
use crate::lexer::{Token, TokenKind};

pub struct Parser<'a> {
    lexer: std::iter::Peekable<Lexer<'a>>,
    current_token: Token,
    last_consumed_token: Option<Token>,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        let mut lexer = lexer.peekable();

        // laod first token
        let first_token = lexer
            .next()
            .unwrap_or_else(|| Token::new(TokenKind::EndOfFile, "".to_string(), 0));

        Parser {
            lexer,
            current_token: first_token,
            last_consumed_token: None,
        }
    }

    fn consume(&mut self) -> Result<Token, ParseError> {
        // check for end of input
        if self.current_token.kind == TokenKind::EndOfFile {
            return Err(ParseError::new_with_token(
                "Unexpected end of input.".to_string(),
                self.last_consumed_token
                    .clone()
                    .unwrap_or_else(|| self.current_token.clone()),
            ));
        }

        // consume token
        let consumed = self.current_token.clone();
        self.last_consumed_token = Some(consumed.clone());
        self.current_token = self.lexer.next().unwrap();

        Ok(consumed)
    }

    fn consume_token_type(&mut self, expected_kind: TokenKind) -> Result<Token, ParseError> {
        // check for type match
        if std::mem::discriminant(&self.current_token.kind)
            == std::mem::discriminant(&expected_kind)
        {
            self.consume()
        } else {
            Err(ParseError::new(
                format!(
                    "Expected token {:?} but found {:?} ('{}')",
                    expected_kind, self.current_token.kind, self.current_token.lexeme,
                ),
                Some(self.current_token.line),
            ))
        }
    }

    fn consume_end(&mut self) -> Result<bool, ParseError> {
        if self.peek() != TokenKind::EndOfFile {
            Err(ParseError::new(
                format!("Expected EOF but found {:?}", self.current_token.kind),
                Some(self.current_token.line),
            ))
        } else {
            Ok(true)
        }
    }

    fn peek(&self) -> TokenKind {
        self.current_token.kind.clone()
    }

    fn parse_number(&self, s_val: String, base: u32) -> Result<i32, ParseError> {
        i32::from_str_radix(&s_val, base)
            .map_err(|e| ParseError::new(format!("Invalid number '{}': {}", s_val, e), None))
    }

    // PROGRAM -> program identifier semicolon OPT_CONST_SECTION OPT_VAR_SECTION DECL_LIST begin STATEMENT_LIST end dot
    pub fn parse_program(&mut self) -> Result<Program, ParseError> {
        let program_token = self.consume_token_type(TokenKind::Program)?;
        let name = self.expect_identifier()?;
        self.consume_token_type(TokenKind::Semicolon)?;

        let const_section = self.parse_opt_const_section()?;
        let decl_list = self.parse_decl_list()?;

        let var_section = self.parse_opt_var_section()?;
        let begin_token = self.consume_token_type(TokenKind::Begin)?;
        let statements = self.parse_statement_list(false)?;
        let end_token = self.consume_token_type(TokenKind::End)?;
        self.consume_token_type(TokenKind::Dot)?;

        let body = CompoundStatement {
            statements,
            start: begin_token.line,
            end: end_token.line,
        };

        self.consume_end()?;

        Ok(Program {
            name,
            const_section: const_section,
            var_section: var_section,
            decl_list,
            body,
            start: program_token.line,
            end: end_token.line,
        })
    }

    // basic types
    fn expect_identifier(&mut self) -> Result<Identifier, ParseError> {
        let token = self.consume()?;
        if let TokenKind::Identifier(name) = token.kind {
            Ok(Identifier::new(name, token.line))
        } else {
            Err(ParseError::new_with_token(
                format!("Expected Identifier, found {:?}", token.kind),
                token,
            ))
        }
    }

    fn expect_number_literal(&mut self) -> Result<(i32, u32), ParseError> {
        let token = self.consume()?;
        if let TokenKind::Number { value, base } = token.kind {
            Ok((self.parse_number(value, base)?, token.line))
        } else if TokenKind::Minus == token.kind {
            let (num, line) = self.expect_number_literal()?;

            Ok((-num, line))
        } else {
            Err(ParseError::new_with_token(
                format!("Expected Number, found {:?}", token.kind),
                token,
            ))
        }
    }

    fn expect_string_literal(&mut self) -> Result<(String, u32), ParseError> {
        let token = self.consume()?;
        if let TokenKind::StringLiteral(val) = token.kind {
            Ok((val, token.line))
        } else {
            Err(ParseError::new_with_token(
                format!("Expected String Literal, found {:?}", token.kind),
                token,
            ))
        }
    }

    fn is_ident_kind(&self) -> bool {
        std::mem::discriminant(&self.peek())
            == std::mem::discriminant(&TokenKind::Identifier("".to_string()))
    }
    // CONSTS SECTION

    // OPT_CONST_SECTION -> CONST_SECTION | epsilon
    fn parse_opt_const_section(&mut self) -> Result<Option<ConstSection>, ParseError> {
        if self.peek() == TokenKind::Const {
            self.parse_const_section().map(Some)
        } else {
            Ok(None)
        }
    }

    // CONST_SECTION -> const CONST_DECL_LIST
    fn parse_const_section(&mut self) -> Result<ConstSection, ParseError> {
        let const_token = self.consume_token_type(TokenKind::Const)?;
        let mut declarations = Vec::new();

        if self.is_ident_kind() {
            // continue for identifiers
            declarations.push(self.parse_const_decl()?);
            while std::mem::discriminant(&self.peek())
                == std::mem::discriminant(&TokenKind::Identifier("".to_string()))
            {
                declarations.push(self.parse_const_decl()?);
            }
        }
        // const section after 'const' cannot be empty
        else {
            return Err(ParseError::new_with_token(
                "Expected constant declaration after 'const' keyword.".to_string(),
                self.current_token.clone(),
            ));
        }
        Ok(ConstSection {
            declarations,
            line: const_token.line,
        })
    }

    // CONST_DECL -> identifier equal CONST_LIT semicolon
    fn parse_const_decl(&mut self) -> Result<ConstDecl, ParseError> {
        let name = self.expect_identifier()?;
        let eq_token = self.consume_token_type(TokenKind::Equal)?;
        let value = self.parse_const_lit()?;
        self.consume_token_type(TokenKind::Semicolon)?;
        Ok(ConstDecl {
            name,
            value,
            line: eq_token.line,
        })
    }

    // CONST_LIT -> number | stringliteral
    fn parse_const_lit(&mut self) -> Result<ConstantLiteral, ParseError> {
        match self.peek() {
            TokenKind::Number { .. } => {
                let (val, line) = self.expect_number_literal()?;
                Ok(ConstantLiteral::Number { value: val, line })
            }
            TokenKind::StringLiteral(_) => {
                let (val, line) = self.expect_string_literal()?;
                Ok(ConstantLiteral::String { value: val, line })
            }
            _ => Err(ParseError::new_with_token(
                "Expected number or string literal for constant value".to_string(),
                self.current_token.clone(),
            )),
        }
    }

    // VARS SECTION

    // OPT_VAR_SECTION -> VAR_SECTION | epsilon
    fn parse_opt_var_section(&mut self) -> Result<Option<VarSection>, ParseError> {
        if self.peek() == TokenKind::Var {
            self.parse_var_section().map(Some)
        } else {
            Ok(None)
        }
    }

    // VAR_SECTION -> var VAR_DECL_LINE_LIST
    fn parse_var_section(&mut self) -> Result<VarSection, ParseError> {
        self.consume_token_type(TokenKind::Var)?;
        let mut declarations = Vec::new();

        if self.is_ident_kind() {
            declarations.push(self.parse_var_decl_line()?);

            // continue for indentifiers
            while self.is_ident_kind() || self.peek() == TokenKind::Var {
                if self.peek() == TokenKind::Var {
                    self.consume()?;
                }

                declarations.push(self.parse_var_decl_line()?);
            }
        } else {
            return Err(ParseError::new_with_token(
                "Expected variable declaration after 'var' keyword.".to_string(),
                self.current_token.clone(),
            ));
        }
        Ok(VarSection { declarations })
    }

    // VAR_DECL_LINE -> IDENTIFIER_LIST colon TYPE_SPECIFIER semicolon
    fn parse_var_decl_line(&mut self) -> Result<VarDeclLine, ParseError> {
        let identifiers = self.parse_identifier_list()?;
        self.consume_token_type(TokenKind::Colon)?;
        let type_specifier = self.parse_type_specifier()?;
        let semi_token = self.consume_token_type(TokenKind::Semicolon)?;

        Ok(VarDeclLine {
            identifiers,
            type_specifier,
            line: semi_token.line,
        })
    }

    fn parse_identifier_list(&mut self) -> Result<Vec<Identifier>, ParseError> {
        let mut identifiers = Vec::new();

        identifiers.push(self.expect_identifier()?);
        while self.peek() == TokenKind::Comma {
            self.consume()?;
            identifiers.push(self.expect_identifier()?);
        }

        Ok(identifiers)
    }

    fn parse_type_specifier(&mut self) -> Result<TypeSpecifier, ParseError> {
        match self.peek() {
            TokenKind::Integer => self.parse_base_type().map(TypeSpecifier::Base),
            TokenKind::Array => self.parse_array_type().map(TypeSpecifier::Array),
            _ => Err(ParseError::new_with_token(
                "Expected 'integer' or 'array' for type specifier".to_string(),
                self.current_token.clone(),
            )),
        }
    }

    // BASE TYPES

    // Integer
    fn parse_base_type(&mut self) -> Result<BaseType, ParseError> {
        self.consume_token_type(TokenKind::Integer)?;
        Ok(BaseType::Integer)
    }

    // Array
    fn parse_array_type(&mut self) -> Result<ArrayType, ParseError> {
        self.consume_token_type(TokenKind::Array)?;
        self.consume_token_type(TokenKind::LeftBracket)?;

        let (low_bound, _) = self.expect_number_literal()?;

        self.consume_token_type(TokenKind::DoubleDot)?;

        let (high_bound, h_line) = self.expect_number_literal()?;

        self.consume_token_type(TokenKind::RightBracket)?;
        self.consume_token_type(TokenKind::Of)?;
        let element_type = self.parse_base_type()?;

        Ok(ArrayType {
            low_bound,
            high_bound,
            element_type,
            line: h_line,
        })
    }

    // FUNCTIONS & PROCEDURES

    // DECL_LIST -> FUNCTION_DECL DECL_LIST | PROCEDURE_DECL DECL_LIST | epsilon
    fn parse_decl_list(&mut self) -> Result<Vec<GlobalDeclaration>, ParseError> {
        let mut declarations = Vec::new();

        // parse all statements
        while matches!(self.peek(), TokenKind::Function | TokenKind::Procedure) {
            match self.peek() {
                TokenKind::Function => {
                    declarations.push(GlobalDeclaration::Function(self.parse_function_decl()?))
                }
                TokenKind::Procedure => {
                    declarations.push(GlobalDeclaration::Procedure(self.parse_procedure_decl()?))
                }
                _ => unreachable!(),
            }
        }
        Ok(declarations)
    }

    // FUNCTION_DECL -> function identifier leftparen OPT_PARAMETERS rightparen colon BASE_TYPE semicolon OPT_VAR_SECTION begin OPT_STATEMENT_LIST end semicolon
    fn parse_function_decl(&mut self) -> Result<FunctionDecl, ParseError> {
        let function_token = self.consume_token_type(TokenKind::Function)?;
        let name = self.expect_identifier()?;
        self.consume_token_type(TokenKind::LeftParen)?;

        let params = self.parse_opt_parameters()?;

        self.consume_token_type(TokenKind::RightParen)?;
        self.consume_token_type(TokenKind::Colon)?;

        let base_type = self.parse_base_type()?;

        self.consume_token_type(TokenKind::Semicolon)?;

        // check for forward declaration
        if self.peek() == TokenKind::Forward {
            self.consume()?;
            self.consume_token_type(TokenKind::Semicolon)?;

            return Ok(FunctionDecl {
                name,
                params: params,
                return_type: base_type,
                body: Implementation::Forwarded,
                start: function_token.line,
                end: function_token.line,
            });
        }

        let var_section = self.parse_opt_var_section()?;

        let begin_token = self.consume_token_type(TokenKind::Begin)?;
        let statements = self.parse_statement_list(true)?;
        let end_token = self.consume_token_type(TokenKind::End)?;

        let body = CompoundStatement {
            statements,
            start: begin_token.line,
            end: end_token.line,
        };

        let semicolon_token = self.consume_token_type(TokenKind::Semicolon)?;

        Ok(FunctionDecl {
            name,
            params: params,
            return_type: base_type,
            body: Implementation::Implemented { var_section, body },
            start: function_token.line,
            end: semicolon_token.line,
        })
    }

    // PROCEDURE_DECL -> procedure identifier leftparen OPT_PARAMETERS rightparen semicolon OPT_VAR_SECTION begin OPT_STATEMENT_LIST end semicolon
    fn parse_procedure_decl(&mut self) -> Result<ProcedureDecl, ParseError> {
        let proc_token = self.consume_token_type(TokenKind::Procedure)?;
        let name = self.expect_identifier()?;
        self.consume_token_type(TokenKind::LeftParen)?;

        let params = self.parse_opt_parameters()?;

        self.consume_token_type(TokenKind::RightParen)?;
        self.consume_token_type(TokenKind::Semicolon)?;

        // check for forward declaration
        if self.peek() == TokenKind::Forward {
            self.consume()?;
            self.consume_token_type(TokenKind::Semicolon)?;

            return Ok(ProcedureDecl {
                name,
                params,
                body: Implementation::Forwarded,
                start: proc_token.line,
                end: proc_token.line,
            });
        }

        let var_section = self.parse_opt_var_section()?;

        let begin_token = self.consume_token_type(TokenKind::Begin)?;
        let statements = self.parse_statement_list(true)?;
        let end_token = self.consume_token_type(TokenKind::End)?;

        let body = CompoundStatement {
            statements,
            start: begin_token.line,
            end: end_token.line,
        };

        let semicolon_token = self.consume_token_type(TokenKind::Semicolon)?;

        Ok(ProcedureDecl {
            name,
            params,
            body: Implementation::Implemented { var_section, body },
            start: proc_token.line,
            end: semicolon_token.line,
        })
    }

    // PARAMETERS

    // OPT_PARAMETERS -> PARAMETERS | epsilon
    fn parse_opt_parameters(&mut self) -> Result<Vec<Parameter>, ParseError> {
        if let TokenKind::Identifier(_) = self.peek() {
            self.parse_parameters()
        } else {
            Ok(Vec::new())
        }
    }

    // PARAMETERS -> PARAMETER PARAMETERS'
    fn parse_parameters(&mut self) -> Result<Vec<Parameter>, ParseError> {
        let mut params = Vec::new();
        params.push(self.parse_parameter()?);

        // read all params
        while self.peek() == TokenKind::Semicolon {
            self.consume()?;
            params.push(self.parse_parameter()?);
        }

        Ok(params)
    }

    // PARAMETER -> IDENTIFIER_LIST colon BASE_TYPE
    fn parse_parameter(&mut self) -> Result<Parameter, ParseError> {
        let identifiers = self.parse_identifier_list()?;
        let colon_token = self.consume_token_type(TokenKind::Colon)?;
        let type_specifier = self.parse_base_type()?;

        Ok(Parameter {
            identifiers,
            type_specifier,
            line: colon_token.line,
        })
    }

    // STATEMENTS

    // STATEMENT_LIST -> STATEMENT STATEMENT_LIST'
    fn parse_statement_list(&mut self, optional: bool) -> Result<Vec<Statement>, ParseError> {
        let mut statements = Vec::new();

        // optional statements
        if self.peek() == TokenKind::End && optional {
            return Ok(statements);
        }

        statements.push(self.parse_statement()?);

        // parse all statements
        while self.peek() == TokenKind::Semicolon {
            self.consume_token_type(TokenKind::Semicolon)?;

            // optional semicolon at end
            if self.peek() == TokenKind::End {
                return Ok(statements);
            }

            statements.push(self.parse_statement()?);
        }

        Ok(statements)
    }

    // STATEMENT -> ...
    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        match self.peek() {
            TokenKind::Identifier(_) => self.parse_identifier_initiated_statement(),
            TokenKind::Begin => self.parse_compound_statement().map(Statement::Compound),
            TokenKind::If => self.parse_if_statement(),
            TokenKind::While => self.parse_while_statement(),
            TokenKind::For => self.parse_for_statement(),
            TokenKind::Exit => {
                let exit_token = self.consume_token_type(TokenKind::Exit)?;
                Ok(Statement::Exit {
                    line: exit_token.line,
                })
            }
            TokenKind::WriteLn => self.parse_writeln_statement(),
            TokenKind::Write => self.parse_write_statement(),
            TokenKind::ReadLn => self.parse_readln_statement(),
            _ => Ok(Statement::Empty),
        }
    }

    // call or assignment
    fn parse_identifier_initiated_statement(&mut self) -> Result<Statement, ParseError> {
        let name = self.expect_identifier()?;

        match self.peek() {
            // call
            TokenKind::LeftParen => {
                let left_paren_token = self.consume()?;
                let arguments = self.parse_opt_arguments()?;

                if self.peek() == TokenKind::RightParen {
                    self.consume()?;
                } else {
                    return Err(ParseError::new_with_token(
                        "Expected ')' after call arguments".to_string(),
                        self.current_token.clone(),
                    ));
                };

                Ok(Statement::Call {
                    name,
                    arguments,
                    line: left_paren_token.line,
                })
            }

            // assignment
            TokenKind::Assign | TokenKind::LeftBracket => {
                let target = self.parse_assignment_target_suffix(name)?;
                let assign_token = self.consume_token_type(TokenKind::Assign)?;
                let expression = self.parse_expression()?;

                Ok(Statement::Assignment {
                    target,
                    expression,
                    line: assign_token.line,
                })
            }
            _ => Err(ParseError::new_with_token(
                format!(
                    "Expected '(', ':=', or '[' after identifier '{}' in statement",
                    name.name
                ),
                self.current_token.clone(),
            )),
        }
    }

    // OPT_ARGUMENTS -> ARGUMENTS | epsilon
    fn parse_opt_arguments(&mut self) -> Result<Vec<Expression>, ParseError> {
        if self.peek() == TokenKind::RightParen || !self.is_start_of_expression() {
            Ok(Vec::new())
        } else {
            self.parse_arguments()
        }
    }

    fn parse_arguments(&mut self) -> Result<Vec<Expression>, ParseError> {
        let mut args = Vec::new();

        args.push(self.parse_expression()?);

        while self.peek() == TokenKind::Comma {
            self.consume()?;
            args.push(self.parse_expression()?);
        }

        Ok(args)
    }

    fn is_start_of_expression(&self) -> bool {
        matches!(
            self.peek(),
            TokenKind::Number { .. }
                | TokenKind::Identifier(_)
                | TokenKind::StringLiteral(_)
                | TokenKind::LeftParen
                | TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Not,
        )
    }

    // COMPOUND_STMT
    fn parse_compound_statement(&mut self) -> Result<CompoundStatement, ParseError> {
        let begin_token = self.consume_token_type(TokenKind::Begin)?;
        let statements = self.parse_statement_list(true)?;
        let end_token = self.consume_token_type(TokenKind::End)?;
        Ok(CompoundStatement {
            statements,
            start: begin_token.line,
            end: end_token.line,
        })
    }

    // IF_STMT
    fn parse_if_statement(&mut self) -> Result<Statement, ParseError> {
        let if_token = self.consume_token_type(TokenKind::If)?;

        let condition = self.parse_expression()?;

        self.consume_token_type(TokenKind::Then)?;

        let then_branch = Box::new(self.parse_statement()?);

        // match else to the closest if
        let else_branch = if self.peek() == TokenKind::Else {
            self.consume_token_type(TokenKind::Else)?;
            Some(Box::new(self.parse_statement()?))
        } else {
            None
        };

        Ok(Statement::If {
            condition,
            then_branch,
            else_branch,
            start: if_token.line,
        })
    }

    // WHILE_STMT
    fn parse_while_statement(&mut self) -> Result<Statement, ParseError> {
        let while_token = self.consume_token_type(TokenKind::While)?;
        let condition = self.parse_expression()?;
        self.consume_token_type(TokenKind::Do)?;

        let body = Box::new(self.parse_statement()?);
        Ok(Statement::While {
            condition,
            body,
            start: while_token.line,
        })
    }

    // FOR_STMT
    fn parse_for_statement(&mut self) -> Result<Statement, ParseError> {
        let for_token = self.consume_token_type(TokenKind::For)?;

        let loop_var = self.expect_identifier()?;

        self.consume_token_type(TokenKind::Assign)?;

        let start_expr = self.parse_expression()?;

        let (for_type, _) = match self.peek() {
            TokenKind::To => (ForType::To, self.consume_token_type(TokenKind::To)?),
            TokenKind::Downto => (ForType::Downto, self.consume_token_type(TokenKind::Downto)?),
            _ => {
                return Err(ParseError::new_with_token(
                    "Expected 'to' or 'downto' in for loop".to_string(),
                    self.current_token.clone(),
                ));
            }
        };

        let end_expr = self.parse_expression()?;

        self.consume_token_type(TokenKind::Do)?;
        let body = Box::new(self.parse_statement()?);

        Ok(Statement::For {
            loop_var,
            start_expr,
            for_type,
            end_expr,
            body,
            start: for_token.line,
        })
    }

    // WRITELN_STMT
    fn parse_writeln_statement(&mut self) -> Result<Statement, ParseError> {
        let writeln_token = self.consume_token_type(TokenKind::WriteLn)?;
        self.consume_token_type(TokenKind::LeftParen)?;

        let mut argument = None;

        // check if there are any arguments
        if self.peek() != TokenKind::RightParen {
            argument = Some(self.parse_write_arg()?);
        }

        self.consume_token_type(TokenKind::RightParen)?;

        Ok(Statement::Writeln {
            argument,
            line: writeln_token.line,
        })
    }

    // WRITE_STMT
    fn parse_write_statement(&mut self) -> Result<Statement, ParseError> {
        let write_token = self.consume_token_type(TokenKind::Write)?;
        self.consume_token_type(TokenKind::LeftParen)?;

        let argument = self.parse_write_arg()?;

        self.consume_token_type(TokenKind::RightParen)?;
        Ok(Statement::Write {
            argument,
            line: write_token.line,
        })
    }

    // WRITE_ARG -> stringliteral | identifier
    fn parse_write_arg(&mut self) -> Result<WriteArgument, ParseError> {
        if std::mem::discriminant(&self.peek())
            == std::mem::discriminant(&TokenKind::StringLiteral("".to_string()))
        {
            let (s, _) = self.expect_string_literal()?;
            Ok(WriteArgument::StringLiteral { value: s })
        } else if self.is_start_of_expression() {
            Ok(WriteArgument::Expression(Box::new(
                self.parse_expression()?,
            )))
        } else {
            Err(ParseError::new_with_token(
                "Expected string literal or expression for write argument".to_string(),
                self.current_token.clone(),
            ))
        }
    }

    // READLN_STMT
    fn parse_readln_statement(&mut self) -> Result<Statement, ParseError> {
        let readln_token = self.consume_token_type(TokenKind::ReadLn)?;
        self.consume_token_type(TokenKind::LeftParen)?;

        let initial_ident_token = self.current_token.clone();

        if let TokenKind::Identifier(_) = initial_ident_token.kind {
            let ident_name = self.expect_identifier()?;
            let target = self.parse_assignment_target_suffix(ident_name)?;
            self.consume_token_type(TokenKind::RightParen)?;
            Ok(Statement::Readln {
                target,
                line: readln_token.line,
            })
        } else {
            Err(ParseError::new_with_token(
                "Expected identifier for readln target".to_string(),
                initial_ident_token,
            ))
        }
    }

    fn parse_assignment_target_suffix(
        &mut self,
        name: Identifier,
    ) -> Result<AssignmentTarget, ParseError> {
        if self.peek() == TokenKind::LeftBracket {
            self.consume_token_type(TokenKind::LeftBracket)?;
            let index_expr = Box::new(self.parse_expression()?);
            self.consume_token_type(TokenKind::RightBracket)?;

            Ok(AssignmentTarget::ArrayAccess { name, index_expr })
        }
        // it is not an array access
        else {
            Ok(AssignmentTarget::Identifier(name))
        }
    }

    // EXPRESSIONS

    // EXPRESSION -> LOGICAL_TERM EXPRESSION'
    fn parse_expression(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_logical_term()?;

        while self.peek() == TokenKind::Or {
            self.consume_token_type(TokenKind::Or)?;
            let right = self.parse_logical_term()?;

            left = Expression::BinaryOp {
                left: Box::new(left),
                operator: BinaryOperator::Or,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    // LOGICAL_TERM -> COMPARISON LOGICAL_TERM'
    fn parse_logical_term(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_comparison()?;
        while self.peek() == TokenKind::And {
            self.consume_token_type(TokenKind::And)?;
            let right = self.parse_comparison()?;

            left = Expression::BinaryOp {
                left: Box::new(left),
                operator: BinaryOperator::And,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    // COMPARISON -> SIMPLE_EXPRESSION OPT_COMPARISON_RHS
    fn parse_comparison(&mut self) -> Result<Expression, ParseError> {
        let left = self.parse_simple_expression()?;
        if self.is_rel_op(&self.peek()) {
            let op_token = self.consume()?;
            let operator = self.to_binary_operator(&op_token.kind).unwrap();
            let right = self.parse_simple_expression()?;

            Ok(Expression::BinaryOp {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            })
        } else {
            Ok(left)
        }
    }

    fn is_rel_op(&self, tt: &TokenKind) -> bool {
        matches!(
            tt,
            TokenKind::Equal
                | TokenKind::NotEqual
                | TokenKind::Less
                | TokenKind::LessEqual
                | TokenKind::Greater
                | TokenKind::GreaterEqual
        )
    }

    fn to_binary_operator(&self, tt: &TokenKind) -> Option<BinaryOperator> {
        match tt {
            TokenKind::Or => Some(BinaryOperator::Or),
            TokenKind::And => Some(BinaryOperator::And),
            TokenKind::Equal => Some(BinaryOperator::Equal),
            TokenKind::NotEqual => Some(BinaryOperator::NotEqual),
            TokenKind::Less => Some(BinaryOperator::Less),
            TokenKind::LessEqual => Some(BinaryOperator::LessEqual),
            TokenKind::Greater => Some(BinaryOperator::Greater),
            TokenKind::GreaterEqual => Some(BinaryOperator::GreaterEqual),
            TokenKind::Plus => Some(BinaryOperator::Add),
            TokenKind::Minus => Some(BinaryOperator::Subtract),
            TokenKind::Multiply => Some(BinaryOperator::Multiply),
            TokenKind::Div => Some(BinaryOperator::Divide),
            TokenKind::Mod => Some(BinaryOperator::Modulo),
            _ => None,
        }
    }

    fn to_unary_operator(&self, tt: &TokenKind) -> Option<UnaryOperator> {
        match tt {
            TokenKind::Plus => Some(UnaryOperator::Plus),
            TokenKind::Minus => Some(UnaryOperator::Minus),
            TokenKind::Not => Some(UnaryOperator::Not),
            _ => None,
        }
    }

    // SIMPLE_EXPRESSION   -> OPT_UNARY_OP TERM SIMPLE_EXPRESSION'
    // OPT_UNARY_OP        -> plus | minus | not | epsilon
    fn parse_simple_expression(&mut self) -> Result<Expression, ParseError> {
        let unary_op_token_opt = if matches!(self.peek(), TokenKind::Plus | TokenKind::Minus) {
            Some(self.consume()?)
        } else {
            None
        };

        let mut term = self.parse_term()?;

        if let Some(op_token) = unary_op_token_opt {
            let operator = self.to_unary_operator(&op_token.kind).unwrap();
            term = Expression::UnaryOp {
                operator,
                operand: Box::new(term),
            };
        }

        let mut left = term;

        while matches!(self.peek(), TokenKind::Plus | TokenKind::Minus) {
            let op_token = self.consume()?;
            let operator = self.to_binary_operator(&op_token.kind).unwrap();
            let right_term = self.parse_term()?;

            left = Expression::BinaryOp {
                left: Box::new(left),
                operator,
                right: Box::new(right_term),
            };
        }

        Ok(left)
    }

    // TERM -> FACTOR TERM'
    fn parse_term(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_factor()?;

        while matches!(
            self.peek(),
            TokenKind::Multiply | TokenKind::Div | TokenKind::Mod
        ) {
            let op_token = self.consume()?;
            let operator = self.to_binary_operator(&op_token.kind).unwrap();
            let right_factor = self.parse_factor()?;

            left = Expression::BinaryOp {
                left: Box::new(left),
                operator,
                right: Box::new(right_factor),
            };
        }
        Ok(left)
    }

    // FACTOR -> number | identifier | stringliteral | ARRAY_ACCESS | FUNCTION_CALL | leftparen EXPRESSION rightparen
    // ARRAY_ACCESS and FUNCTION_CALL
    fn parse_factor(&mut self) -> Result<Expression, ParseError> {
        match self.peek().clone() {
            TokenKind::Number { .. } => {
                let (val, line) = self.expect_number_literal()?;
                Ok(Expression::Literal(ConstantLiteral::Number {
                    value: val,
                    line,
                }))
            }
            TokenKind::StringLiteral(_) => {
                let (val, line) = self.expect_string_literal()?;
                Ok(Expression::Literal(ConstantLiteral::String {
                    value: val,
                    line,
                }))
            }
            TokenKind::Identifier(_) => {
                let name = self.expect_identifier()?;

                // check for function call or array access
                match self.peek() {
                    // call
                    TokenKind::LeftParen => {
                        self.consume_token_type(TokenKind::LeftParen)?;

                        let arguments = self.parse_opt_arguments()?;

                        if self.peek() == TokenKind::RightParen {
                            self.consume_token_type(TokenKind::RightParen)?;
                        } else {
                            return Err(ParseError::new_with_token(
                                "Expected ')' after function call arguments".to_string(),
                                self.current_token.clone(),
                            ));
                        };

                        Ok(Expression::FunctionCall { name, arguments })
                    }
                    TokenKind::LeftBracket => {
                        // Array access
                        self.consume_token_type(TokenKind::LeftBracket)?;
                        let index_expr = Box::new(self.parse_expression()?);
                        self.consume_token_type(TokenKind::RightBracket)?;
                        Ok(Expression::ArrayAccess { name, index_expr })
                    }
                    _ => Ok(Expression::Variable(name)),
                }
            }

            // bracketed expr
            TokenKind::LeftParen => {
                self.consume_token_type(TokenKind::LeftParen)?;
                let expr = self.parse_expression()?;
                self.consume_token_type(TokenKind::RightParen)?;
                Ok(Expression::Grouped(Box::new(expr)))
            }
            _ => Err(ParseError::new_with_token(
                format!("Unexpected token in factor: {:?}", self.current_token.kind),
                self.current_token.clone(),
            )),
        }
    }
}
