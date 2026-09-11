## Grammar description for the mila language

### Legend

- Uppercase words are NON-terminals
- Lowercase words are terminals

### CF Grammar

The parser and AST builder are based on this grammar for Mila

```
PROGRAM             -> program identifier semicolon OPT_CONST_SECTION DECL_LIST OPT_VAR_SECTION  begin STATEMENT_LIST end dot

OPT_CONST_SECTION   -> CONST_SECTION
OPT_CONST_SECTION   ->

OPT_VAR_SECTION     -> VAR_SECTION
OPT_VAR_SECTION     ->

DECL_LIST           -> FUNCTION_DECL DECL_LIST
DECL_LIST           -> PROCEDURE_DECL DECL_LIST
DECL_LIST           ->

STATEMENT_LIST      -> STATEMENT STATEMENT_LIST'

STATEMENT_LIST'     -> semicolon STATEMENT STATEMENT_LIST'
STATEMENT_LIST'     ->

CONST_SECTION       -> const CONST_DECL_LIST

CONST_DECL_LIST     -> CONST_DECL CONST_DECL_LIST
CONST_DECL_LIST     ->

CONST_DECL          -> identifier equal CONST_LIT semicolon

CONST_LIT           -> number
CONST_LIT           -> stringliteral

VAR_SECTION         -> var VAR_DECL_LINE_LIST

VAR_DECL_LINE_LIST  -> VAR_DECL_LINE VAR_DECL_LINE_LIST
VAR_DECL_LINE_LIST  ->

VAR_DECL_LINE       -> IDENTIFIER_LIST colon TYPE_SPECIFIER semicolon

IDENTIFIER_LIST     -> identifier IDENTIFIER_LIST'

IDENTIFIER_LIST'    -> comma identifier IDENTIFIER_LIST'
IDENTIFIER_LIST'    ->

TYPE_SPECIFIER      -> BASE_TYPE
TYPE_SPECIFIER      -> ARRAY_TYPE

BASE_TYPE           -> integer

ARRAY_TYPE          -> array leftbracket number doubledot number rightbracket of BASE_TYPE

PROCEDURE_DECL      -> procedure identifier leftparen OPT_PARAMETERS rightparen semicolon BODY

FUNCTION_DECL       -> function identifier leftparen OPT_PARAMETERS rightparen colon BASE_TYPE semicolon BODY

BODY                -> OPT_VAR_SECTION begin OPT_STATEMENT_LIST end semicolon
                    -> forward semicolon

OPT_PARAMETERS      -> PARAMETERS
OPT_PARAMETERS      ->

PARAMETERS          -> PARAMETER PARAMETERS'

PARAMETERS'         -> semicolon PARAMETER PARAMETERS'
PARAMETERS'         ->

PARAMETER           -> IDENTIFIER_LIST colon BASE_TYPE

STATEMENT           -> identifier USE_IDENT_STMT
STATEMENT           -> WHILE_STMT
STATEMENT           -> IF_STMT
STATEMENT           -> FOR_STMT
STATEMENT           -> COMPOUND_STMT
STATEMENT           -> EXIT_STMT
STATEMENT           -> WRITELN_STMT
STATEMENT           -> WRITE_STMT
STATEMENT           -> READLN_STMT
STATEMENT           -> EMPTY_STATEMENT

COMPOUND_STMT       -> begin OPT_STATEMENT_LIST end

USE_IDENT_STMT      -> CALL_STMT
USE_IDENT_STMT      -> ASSIGNMENT_STMT

ASSIGNMENT_STMT     -> OPT_ARRAY_ACCESS assign EXPRESSION

CALL_STMT           -> leftparen OPT_ARGUMENTS rightparen

OPT_ARRAY_ACCESS    -> leftbracket EXPRESSION rightbracket
OPT_ARRAY_ACCESS    ->

WHILE_STMT          -> while EXPRESSION do STATEMENT

IF_STMT             -> if EXPRESSION then STATEMENT OPT_ELSE_CLAUSE

OPT_ELSE_CLAUSE     -> else STATEMENT
OPT_ELSE_CLAUSE     ->

FOR_STMT            -> for identifier assign EXPRESSION FOR_TYPE EXPRESSION do STATEMENT

FOR_TYPE            -> to
FOR_TYPE            -> downto


OPT_ARGUMENTS       -> ARGUMENTS
OPT_ARGUMENTS       ->

EXIT_STMT           -> exit

WRITELN_STMT        -> writeln leftparen WRITE_ARG rightparen

WRITE_STMT          -> write leftparen WRITE_ARG rightparen

WRITE_ARG           -> stringliteral
WRITE_ARG           -> EXPRESSION

READLN_STMT         -> readln leftparen identifier OPT_ARRAY_ACCESS rightparen

EMPTY_STATEMENT     ->

ARGUMENTS           -> EXPRESSION ARGUMENTS'

ARGUMENTS'          -> comma EXPRESSION ARGUMENTS'
ARGUMENTS'          ->

EXPRESSION          -> LOGICAL_TERM EXPRESSION'

EXPRESSION'         -> or LOGICAL_TERM EXPRESSION'
EXPRESSION'         ->

LOGICAL_TERM        -> COMPARISON LOGICAL_TERM'
LOGICAL_TERM'       -> and COMPARISON LOGICAL_TERM'
LOGICAL_TERM'       ->

COMPARISON          -> SIMPLE_EXPRESSION OPT_COMPARISON_RHS

OPT_COMPARISON_RHS  -> REL_OP SIMPLE_EXPRESSION
OPT_COMPARISON_RHS  ->

REL_OP              -> equal
REL_OP              -> notequal
REL_OP              -> less
REL_OP              -> lessequal
REL_OP              -> greater
REL_OP              -> greaterequal

SIMPLE_EXPRESSION   -> OPT_UNARY_OP TERM SIMPLE_EXPRESSION'

OPT_UNARY_OP        -> plus
OPT_UNARY_OP        -> minus
OPT_UNARY_OP        -> not
OPT_UNARY_OP        ->

SIMPLE_EXPRESSION'  -> ADD_OP TERM SIMPLE_EXPRESSION'
SIMPLE_EXPRESSION'  ->

ADD_OP              -> plus
ADD_OP              -> minus

TERM                -> FACTOR TERM'

TERM'               -> MUL_OP FACTOR TERM'
TERM'               ->

MUL_OP              -> multiply
MUL_OP              -> div
MUL_OP              -> mod

FACTOR              -> number
FACTOR              -> indentifier
FACTOR              -> stringliteral
FACTOR              -> ARRAY_ACCESS
FACTOR              -> FUNCTION_CALL
FACTOR              -> leftparen EXPRESSION rightparen

FUNCTION_CALL       -> identifier leftparen OPT_ARGUMENTS rightparen

```
