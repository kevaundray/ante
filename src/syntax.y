%{

#include <stdlib.h>
#include <stdio.h>
#include <tokens.h>

int yylex();
void yyerror(const char *msg);

#define YYERROR_VERBOSE

%}

%token Ident

/*types*/
%token I8
%token I16
%token I32
%token I64
%token U8
%token U16
%token U32
%token U64
%token ISz
%token Usz
%token F32
%token F64
%token C8
%token C32
%token Bool
%token Void

/*operators*/
%token Operator
%token Eq
%token NotEq
%token AddEq
%token SubEq
%token MulEq
%token DivEq
%token GrtrEq
%token LesrEq
%token Or
%token And
%token Range
%token RangeBX
%token RangeEX
%token RangeX

/*literals*/
%token True
%token False
%token IntLit
%token FltLit
%token StrLit

/*keywords*/
%token Return
%token If
%token Elif
%token Else
%token For
%token While
%token Do
%token In
%token Continue
%token Break
%token Import
%token Match
%token Data
%token Enum

/*modifiers*/
%token Pub
%token Pri
%token Pro
%token Const
%token Ext
%token Dyn
%token Pathogen

/*other*/
%token Where
%token Infect
%token Cleanse
%token Ct

%token Newline
%token Indent
%token Unindent


/*
    Now to manually fix all shift/reduce conflicts
*/
%precedence ','

%precedence '.'
/*
    Used in type casting, high precedence to operate
    before . so that examples like Type(param, param).field
    can be used.
*/
%precedence I8
%precedence I16
%precedence I32
%precedence I64
%precedence U8
%precedence U16
%precedence U32
%precedence U64
%precedence ISz
%precedence Usz
%precedence F32
%precedence F64 
%precedence C8
%precedence C16
%precedence C32
%precedence C64
%precedence Bool
%precedence Void

%precedence '*'

%precedence '('
%precedence '['

/*
    All shift/reduce conflicts should be manually dealt with.
*/
%expect 0
%start maybe_statement_list
%%

maybe_statement_list: statement_list
                    | %empty
                    ;

statement_list: statement_list statement { puts("statement_list"); }
              | statement { puts("statement_list: statement"); }
              ;

statement: var_decl
         | var_assign
         | fn_decl
         | fn_call { puts("statement: fn_call"); }
         | ret_stmt
         | while_loop
         | for_loop
         | if_stmt
         | Newline
         ;

lit_type: I8
        | I16
        | I32
        | I64
        | U8
        | U16
        | U32
        | U64
        | ISz
        | Usz
        | F32
        | F64
        | C8
        | C16
        | C32
        | C64
        | Bool
        | Void
        ;

type: type '*'
    | type '[' maybe_expr ']'
    | lit_type
    ;

type_expr: type_expr ',' type
         | type_expr '|' type
         | type
         ;

modifier: Pub
        | Pri
        | Pro
        | Const
        | Ext
        | Dyn
        | Pathogen
        ;

maybe_modifier_list: modifier_list
                   | %empty
                   ;

modifier_list: modifier_list modifier
             | modifier
             ;

decl: maybe_modifier_list type_expr Ident

var_decl: decl '=' expr
        | decl { puts("decl"); }
        ;

var_assign: var '=' expr
          ;

block: Indent statement_list Unindent
     ;

params: params ',' type_expr Ident
      | type_expr Ident
      ;

maybe_params: params
            | %empty
            ;

fn_decl: decl ':' maybe_params block { puts("fn_decl"); }
       | decl '(' maybe_expr ')' ':' params block
       ;

fn_call: Ident '(' maybe_expr ')' { puts("fn_call"); }
       ;

ret_stmt: Return expr
        ;

maybe_else: Else block
          | %empty
          ;

elif_list: elif_list Elif block
         | Elif block
         ;

maybe_elif_list: elif_list
               | %empty
               ;

if_stmt: If expr block maybe_elif_list maybe_else
       ;

while_loop: While expr block
          ;

for_loop: For var_decl In expr block
        ;

bin_op: '+'
      | '-'
      | '*'
      | '/'
      | '%'
      | '^'
      | '|'
      | '&'
      | '<'
      | '>'
      | '.'
      | Eq
      | NotEq
      | AddEq
      | SubEq
      | MulEq
      | DivEq
      | GrtrEq
      | LesrEq
      | Or
      | And
      | Range
      | RangeEX
      | RangeBX
      | RangeX
      ;

var: Ident '[' expr ']'
   | Ident { puts("var"); }
   ;

val: fn_call
   | '(' expr ')'
   | var
   | IntLit
   | FltLit
   | StrLit
   | True
   | False
   ;

maybe_expr: expr { puts("maybe_expr: true"); }
          | %empty { puts("maybe_expr: false"); }
          ;

expr: l_expr val
    | val
    ;

l_expr: l_expr val bin_op
      | val bin_op
      ;


%%

void yyerror(const char *s){
    fprintf(stderr, "%s\nerrtok = %d\n", s, yychar);
}

