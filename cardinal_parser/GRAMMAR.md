# Cardinal Language Grammar

The following document outlines the overall grammar that describes what constitutes valid
syntax in the Cardinal programming language.

## Notation

This document uses a dialect based upon Extended Backus-Naur Form (EBNF) productions to represent
the syntactic structure of tbe language.

|   Symbol    | Name          | Description                                                                  |
|:-----------:|:--------------|:-----------------------------------------------------------------------------|
|   `rule`    | rule          | References a rule.                                                           |
|     `=`     | definition    | Denotes the start of a rule definition.                                      |
|     `;`     | termination   | Denotes the end of a rule definition.                                        |
|     `,`     | concatenation | Denotes that one rule must be followed by another rule (i.e. a logical AND). |
|    `\|`     | alternation   | Denotes that one rule can be replaced with another (i.e. a logical OR).      |
|   `rule?`   | optionality   | The annotated rule may occur 0 or 1 times.                                   |
|   `rule*`   | repetition    | The annotated rule may occur 0 or more times.                                |
|   `rule+`   | repetition    | The annotated rule may occur 1 or more times.                                |
|  `( ... )`  | grouping      | Groups multiple elements within parenthesis.                                 |
|  `'blah'`   | terminal      | The literal content in quotes.                                               |
|  `"blah"`   | terminal      | The literal content in quotes.                                               |
|   `0x2a`    | byte terminal | The literal byte value as a character. Represented in hexadecimal.           |
| `(* ... *)` | comment       | No meaning other than existing for documentation purposes.                   |
|  `? ... ?`  | special       | A special sequence that has a description between the question marks.        |

## Lexical grammar

The following section documents the lexical grammar of this language. This is used to produce
initial tokens that can be fed through a recursive descent parser later. Any rules without an
underscore at the start are considered to be context-free tokens that can be emitted by the lexer.

All characters in terminals are considered to be UTF-8 characters unless otherwise
specified.

### Preface

The following productions are not tokens, and are used only to structure other rules here.

```
_BIN_DIGIT = '0' | '1' ;

_OCT_DIGIT = '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' ;

_DEC_DIGIT = '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' ;

_HEX_DIGIT = '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9'
           | 'a' | 'b' | 'c' | 'd' | 'e' | 'f'
           | 'A' | 'B' | 'C' | 'D' | 'E' | 'F'
           ;

_ALPHA = 'a' | 'b' | 'c' | 'd' | 'e' | 'f' | 'g' | 'h' | 'i' | 'j' | 'k' 
       | 'l' | 'm' | 'n' | 'o' | 'p' | 'q' | 'r' | 's' | 't' | 'u' | 'v' 
       | 'w' | 'x' | 'y' | 'z' | 'A' | 'B' | 'C' | 'D' | 'E' | 'F' | 'G' 
       | 'H' | 'I' | 'J' | 'K' | 'L' | 'M' | 'N' | 'O' | 'P' | 'Q' | 'R'
       | 'S' | 'T' | 'U' | 'V' | 'W' | 'X' | 'Y' | 'Z' 
       ;

_ALNUM = _ALPHA | _DEC_DIGIT ;

_BACKSLASH = 0x5c ;  (* ascii backslash \ character *)
```

Prior to each token, the lexer must check for whitespace characters, and **drop** them.

The presence of these characters must only be considered **prior** to each token. They can have a
different meaning if midway through parsing a different production.

```
_WHITESPACE = 0x20  (* ascii whitespace *)
            | 0x0d  (* ascii carriage return \r *)
            | 0x0a  (* ascii line feed \n *)
            | 0x09  (* ascii horizontal tab \t *)
            ;
```

### Tokens

The language defines the following tokens:

```
(* keywords *)

FN       = 'fn' ;
LET      = 'let' ;
IF       = 'if' ;
ELSE     = 'else' ;
WHILE    = 'while' ;
FOR      = 'for' ;
BREAK    = 'break' ;
CONTINUE = 'continue' ;
RETURN   = 'return' ;


(* operators *)

ADD = '+' ;
SUB = '-' ;
MUL = '*' ;
DIV = '/' ;
MOD = '%' ;
POW = '**' ;

ADD_ASSIGN = '+=' ;
SUB_ASSIGN = '-=' ;
MUL_ASSIGN = '*=' ;
DIV_ASSIGN = '/=' ;
MOD_ASSIGN = '%=' ;
POW_ASSIGN = '**=' ;

BIT_AND = '&' ;
BIT_OR  = '|' ;
BIT_XOR = '^' ;
BIT_NOT = '~' ;
BIT_SHL = '<<' ;
BIT_SHR = '>>' ;

BIT_AND_ASSIGN = '&=' ;
BIT_OR_ASSIGN  = '|°' ;
BIT_XOR_ASSIGN = '^°' ;
BIT_SHL_ASSIGN = '<<=' ;
BIT_SHL_ASSIGN = '>>=' ;

BOOL_AND = '&&' ;
BOOL_OR  = '||' ;
BOOL_NOT = '!' ;

EQ   = '==' ;
NEQ  = '!=' ;
LT   = '<' ;
LTEQ = '<=' ;
GT   = '>' ;
GTEQ = '>' ;

ASSIGN = '=' ; 


(* blocks *)

LEFT_PAREN    = '(' ;
RIGHT_PAREN   = ')' ;
LEFT_BRACKET  = '[' ;
RIGHT_BRACKET = ']' ;
LEFT_BRACE    = '{' ;
RIGHT_BRACE   = '}' ;


(* other symbols *)

SEMI          = ';' ;
PERIOD        = '.' ;
COMMA         = ',' ;
NAMESPACE_SEP = '::' ;


(* boolean literals *)

BOOL_LIT = 'true' | 'false' ;


(* int literals *)

INT_LIT      = _BIN_INT_LIT | _OCT_INT_LIT | _HEX_INT_LIT | _DEC_INT_LIT ;
_BIN_INT_LIT = ( '0b' | '0B' ) , _BIN_DIGIT+ ;
_OCT_INT_LIT = ( '0o' | '0O' ) , _OCT_DIGIT+ ;
_HEX_INT_LIT = ( '0x' | '0X' ) , _HEX_DIGIT+ ;
_DEC_INT_LIT = _DEC_DIGIT + ;


(* float literals *)

FLOAT_LIT  = _DEC_DIGIT+ , ( '.' , _DEC_DIGIT* )? , _FLOAT_EXP
           | '.' , _DEC_DIGIT+ , _FLOAT_EXP?
           | _DEC_DIGIT+ , '.' , _DEC_DIGIT*
           ;
_FLOAT_EXP = ( 'e' | 'E' ) , ( '+' | '-' ) , _DEC_DIGIT+ ;


(* strings - note that these are parsed fully during the later parsing phase *)

STR_LIT  = '"' , ( _STR_ESC | _STR_CHR )* , '"' ;
_STR_ESC = _BACKSLASH , ( '"' | _BACKSLASH ) ;
_STR_CHR = ? any UTF-8 codepoint except 0xa linefeed \n and 0xd carriage return \r ? ;


(* identifiers *)

IDENT = ( _ALPHA | '_' ) , ( _ALNUM | '_' )* ;
```

## Structural Grammar

The following grammar is implemented by a parser, consuming tokens emitted by the lexical layer.

### Identifiers

```
ident = IDENT , ( NAMESPACE_SEP , IDENT )+
      | IDENT
      ;
```

### Expressions

```
expr = assign_expr ;

assign_expr = bool_or_expr , ASSIGN , assign_expr
            | bool_or_expr , ADD_ASSIGN , assign_expr
            | bool_or_expr , SUB_ASSIGN , assign_expr
            | bool_or_expr , MUL_ASSIGN , assign_expr
            | bool_or_expr , DIV_ASSIGN , assign_expr
            | bool_or_expr , MOD_ASSIGN , assign_expr
            | bool_or_expr , POW_ASSIGN , assign_expr
            | bool_or_expr , BIT_AND_ASSIGN , assign_expr
            | bool_or_expr , BIT_OR_ASSIGN , assign_expr
            | bool_or_expr , BIT_XOR_ASSIGN , assign_expr
            | bool_or_expr , BIT_SHL_ASSIGN , assign_expr
            | bool_or_expr , BIT_SHR_ASSIGN , assign_expr
            | bool_or_expr
            ;
              
bool_or_expr = bool_and_expr , BIT_OR , bool_or_expr
             | bool_and_expr
             ;
               
bool_and_expr = eq_expr , BIT_AND , bool_and_expr
              | eq_expr
              ;
                
eq_expr = comp_expr , EQ , eq_expr
        | comp_expr , NEQ , eq_expr
        | comp_expr
        ;
          
comp_expr = bitshift_expr , LT , comp_expr
          | bitshift_expr , LTEQ , comp_expr
          | bitshift_expr , GT , comp_expr
          | bitshift_expr , GTEQ , comp_expr
          | bitshift_expr
          ;
            
bitshift_expr = sum_expr , BIT_SHL , bitshift_expr
              | sum_expr , BIT_SHR , bitshift_expr
              | sum_expr
              ;
                
sum_expr = factor_expr , ADD , sum_expr
         | factor_expr , SUB , sum_expr
         | factor_expr
         ;
           
factor_expr = unary_expr , MUL , factor_expr
            | unary_expr , SUB , factor_expr
            | unary_expr
            ;
              
unary_expr = ADD , unary_expr
           | SUB , unary_expr
           | BIT_NOT , unary_expr
           | BOOL_NOT , unary_expr
           | pow_expr
           ;
             
pow_expr = pow_expr , POW , primary_expr
         | primary_expr
         ;

primary_expr = atom , ( member_access_expr | index_expr | func_call_expr )*
             ;

member_access_expr = PERIOD , ident ;

index_expr = LEFT_BRACKET , expr , RIGHT_BRACKET ;

func_call_expr = LEFT_PAREN , arg_list , RIGHT_PAREN ;
arg_list       = expr , ( COMMA , expr )* , COMMA? ;

atom = LEFT_PAREN , expr , RIGHT_PAREN
     | ident
     | BOOL_LIT
     | INT_LIT
     | FLOAT_LIT
     | str_lit  ? see below for notes ?
     ;
```

### String Literals

String literals are consumed from the `STR_LIT` token, but are handled in the parser layer to fully
resolve their contents. This allows for implementing more advanced features like string
interpolation in the future.

Specifically, the `str_lit` parser rule is a `STR_LIT` token with the following transformations
applied:

- `\n` translates to an ASCII linefeed 0x0A.
- `\r` translates to an ASCII carriage return 0x0D.
- `\t` translates to an ASCII horizontal tab 0x09.
- `\\` translates to a single backslash `\`.
- `\"` translates to a literal double quote `"`.
- `\uXXXX` where `XXXX` is a hexadecimal number translates to that codepoint in the UTF-8
  plane.
- Any other sequence after a `\` is deemed to be invalid.
- The string contents are expected to be UTF-8 encoded sequences. Anything else is deemed
  garbage.
