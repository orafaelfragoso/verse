## 17. EBNF grammar sketch

This grammar is a readable sketch of the core language, not a complete formal
definition. The lexer turns leading whitespace into `INDENT` and `DEDENT`
tokens and ends logical lines with `NEWLINE`, suppressing it inside brackets,
after a trailing binary operator, and before a line that begins with `.`
or `|` (§3.4). `BLANK` is an empty line, significant only after a module
doc. Text in `"quotes"` is a literal token; `{ x }` means zero or more, `[ x ]`
means optional, `|` separates alternatives. Rules marked *(semantic)* are
checked after parsing.

```ebnf
(* ---------- Files and declarations ---------- *)
file          = { NEWLINE } [ doc BLANK ] { import } { top_item } ;
                                    (* doc + blank line = module doc (§14.3) *)
top_item      = declaration | statement | test_block ;
declaration   = [ doc ] [ "export" ]
                ( fn_decl | type_decl | alias_decl | trait_decl | binding )
              | [ doc ] impl_block
              | [ doc ] extern_block ;
doc           = DOC_COMMENT { DOC_COMMENT } ;     (* directly above its item *)

import        = "import" module_path [ "as" IDENT ] NEWLINE
              | "import" ( import_list | "(" import_list [ "," ] ")" )
                "from" module_path NEWLINE ;
                (* the line always starts with `import`; a trailing `from` on
                   the same logical line makes the names items of that module,
                   so `import a as b` is a module and `import a as b from m` an
                   item. An import is one line; only the parenthesised list may
                   span lines (§3.4). *)
import_list   = import_item { "," import_item } ;
import_item   = name [ "as" name ] ;              (* same kind on both sides *)
name          = IDENT | TYPE_IDENT ;
module_path   = IDENT { "." IDENT } ;

(* ---------- Functions ---------- *)
fn_decl       = "fn" IDENT [ generics ] "(" [ params ] ")" [ ":" type ] block ;
fn_sig        = "fn" IDENT [ generics ] "(" [ params ] ")" [ ":" type ] NEWLINE ;
params        = param { "," param } [ "," ] ;
param         = [ "mut" | "take" ] "self"
              | [ "mut" | "take" ] IDENT [ ":" type ] [ "=" expr ] ;
                (* types optional (§4.2); `mut`/`take` always explicit *)
generics      = "<" generic_param { "," generic_param } ">" ;
generic_param = TYPE_IDENT [ ":" bound { "+" bound } ] ;
bound         = TYPE_IDENT [ type_args ] ;

(* ---------- Types ---------- *)
type_decl     = "type" TYPE_IDENT [ generics ]
                ( NEWLINE INDENT field { field } DEDENT    (* record: nothing after the name *)
                | "=" variant { "|" variant } NEWLINE ) ;  (* new variants; never a field block *)
                (* (semantic) a variant name may not be a type already in scope:
                   `type Meters = Float` is rejected with "use alias" *)
field         = [ doc ] [ "export" ] IDENT ( ":" type [ "=" expr ] | "=" expr ) NEWLINE ;
                (* the annotation may be omitted only when a default implies the type *)
variant       = TYPE_IDENT [ "(" field_list ")" ] ;
field_list    = IDENT ":" type { "," IDENT ":" type } [ "," ] ;
alias_decl    = "alias" TYPE_IDENT [ generics ] "=" type NEWLINE ;  (* incl. unions *)

type          = union_type ;
union_type    = optional_type { "|" optional_type } ;   (* normalized, §4.6 *)
optional_type = base_type { "?" } ;                       (* T?? is not T? *)
base_type     = { IDENT "." } TYPE_IDENT [ type_args ]      (* `time.Duration` *)
              | "(" type "," type { "," type } ")"               (* tuple   *)
              | "(" labeled { "," labeled } ")"                   (* labeled *)
              | "fn" "(" [ type { "," type } ] ")" ":" type
              | "(" type ")" ;
labeled       = IDENT ":" type ;
type_args     = "<" type { "," type } ">" ;

(* ---------- Foreign functions (§12.6) ---------- *)
extern_block  = "extern" IDENT NEWLINE INDENT { extern_fn } DEDENT ;
                (* IDENT names a `[plugins.<name>]` section of verse.toml *)
extern_fn     = [ doc ] [ "export" ] "fn" name [ "as" IDENT ]
                "(" [ params ] ")" [ ":" type ] NEWLINE ;
                (* (semantic) every parameter has a type; no `take`, no
                   `self`, no defaults; only the plain-data types of §12.6 *)

(* ---------- Traits and impls ---------- *)
trait_decl    = "trait" TYPE_IDENT [ generics ] [ ":" bound { "+" bound } ]
                NEWLINE INDENT { [ doc ] ( fn_sig | fn_decl ) } DEDENT ;
impl_block    = "impl" [ generics ] [ bound "for" ] type
                NEWLINE INDENT { [ doc ] [ "export" ] fn_decl } DEDENT ;

(* ---------- Statements ---------- *)
block         = NEWLINE INDENT statement { statement } DEDENT ;
statement     = binding | assign_stmt | if_stmt | loop_stmt
              | with_stmt | match_expr | exit_stmt
              | "pass" NEWLINE | assert_stmt | fn_decl | expr NEWLINE ;
exit_stmt     = exit [ "if" expr ] NEWLINE ;           (* postfix guard: one line, no else;
                                                           an `if` right after an exit is
                                                           always a guard: write
                                                           `return (if c then a else b)` *)
exit          = "return" [ expr ] | "break" [ IDENT ] | "continue" [ IDENT ] ;
                (* (semantic) an `expr` statement may not have a fallible type
                   or contain fallible results (§8.4); use `?`, handle it, or `_ =` *)
binding       = [ "mut" ] pattern [ ":" type ] "=" expr NEWLINE
              | [ "mut" ] pattern [ ":" type ] "=" or_expr "otherwise" exit NEWLINE ;
                (* (semantic) a refutable pattern is allowed only in the second form.
                   `x = e` with a plain name is a *binding* when `x` is not yet
                   visible, and an *assignment* (assign_stmt) when `x` is a visible
                   `mut` binding; on a visible immutable name it is an error.
                   A name is bound once per function: no shadowing (§3.9);
                   "visible" means locals only. A pattern always binds new
                   names; the exception is a tuple made only of visible `mut`
                   names, a parallel assignment (right side evaluated first,
                   so `(a, b) = (b, a)` swaps); mixing new and existing names
                   is an error.
                   Constructor patterns start with an uppercase name (§3.5), so
                   `Circle(r) = …` is a pattern and `f(r) = …` is not valid.
                   `let` is reserved: "`let` is not used in Verse; write `x = …`" *)
assign_stmt   = place assign_op expr NEWLINE ;
assign_op     = "=" | "+=" | "-=" | "*=" | "/=" | "%=" ;
place         = ( IDENT | "self" ) { "." IDENT | "[" expr "]" } ;

if_stmt       = "if" expr block
                { "else" "if" expr block } [ "else" block ] ;
loop_stmt     = [ IDENT ":" ] ( for_stmt | while_stmt ) ;          (* loop label *)
for_stmt      = "for" pattern "in" expr block ;
while_stmt    = "while" expr block ;
with_stmt     = "with" resource { "," resource } block ;
resource      = expr "as" [ "mut" ] IDENT ;  (* (semantic) expr's type must implement
                                           Cleanup; a fallible acquisition needs `?`;
                                           the name is mutable only with `mut` *)
assert_stmt   = "assert" expr [ "," expr ] NEWLINE ;
test_block    = "test" TEXT block ;

(* ---------- Pattern matching ---------- *)
match_expr    = "match" expr NEWLINE INDENT arm { arm } DEDENT
              | "match" NEWLINE INDENT cond_arm { cond_arm } "_" "->" arm_body DEDENT ;
                                                        (* subject-less: final `_` required *)
cond_arm      = arm_expr "->" arm_body ;                (* a Bool; `expr - lambda`, so a
                                                           lone name is not a lambda *)
arm_body      = ( arm_expr | exit | "pass" | place assign_op expr ) NEWLINE | block ;
                                                        (* `_ -> pass`, `none -> return 0`,
                                                           `x -> total += 1` *)
arm           = top_pat [ "if" arm_expr ] "->" arm_body ;
arm_expr      = expr - lambda ;          (* a lambda body must be parenthesized *)
top_pat       = ( "_" | IDENT ) ":" type                       (* typed: top level only *)
              | pattern ;
pattern       = present_pat { "|" present_pat } ;
present_pat   = single_pat { "?" } ;                           (* `user?`, `e?`, `none?` *)
single_pat    = literal | "none" | "_" | IDENT
              | TYPE_IDENT [ "(" [ sub_pats ] ")" ]            (* variant / record *)
              | "(" pattern "," pattern { "," pattern } ")"    (* tuple   *)
              | "[" [ list_pats ] "]"                          (* list    *)
              | literal ( ".." | "..=" ) literal ;             (* range   *)
sub_pats      = sub_pat { "," sub_pat } ;
sub_pat       = IDENT ":" [ field_pat ]       (* always a field; `name:` alone puns *)
              | pattern ;                     (* positional *)
field_pat     = IDENT ":" type | pattern ;    (* `body: b: Text` = typed binding *)
list_pats     = pattern { "," pattern } [ "," ".." IDENT ] | ".." IDENT ;

(* ---------- Expressions ---------- *)
expr          = lambda | cond_expr | otherwise_expr ;
lambda        = lambda_params "->" ( expr | block ) ;
                (* closing rule: a block lambda opened inside "(" on the same line
                   must be the last argument; its DEDENT also closes that "(" *)
lambda_params = IDENT | "(" [ IDENT [ ":" type ] { "," IDENT [ ":" type ] } ] ")" ;
cond_expr     = "if" expr "then" expr "else" expr ;      (* one-line conditional *)
otherwise_expr= or_expr { "otherwise" fallback } [ "otherwise" exit ] ;
                                                         (* fallback, loosest; an exit
                                                            only as the whole last operand *)
fallback      = postfix | "-" ( INT | FLOAT ) ;          (* literal, call, name, field,
                                                            index, lifted chain, ( … );
                                                            anything else: add brackets *)
or_expr       = and_expr { "or" and_expr } ;             (* Bool only *)
and_expr      = not_expr { "and" not_expr } ;
not_expr      = "not" not_expr | comparison ;
comparison    = range_expr { comp_op range_expr }
              | range_expr "is" top_pat ;
comp_op       = "==" | "!=" | "<" | "<=" | ">" | ">=" | "in" | "not" "in" ;
range_expr    = additive [ ( ".." | "..=" ) additive ] ;  (* both ends required *)
additive      = term { ( "+" | "-" ) term } ;
term          = unary { ( "*" | "/" | "%" ) unary } ;
unary         = "-" unary
              | "spawn" postfix          (* (semantic) the postfix must end in a call *)
              | power ;
power         = postfix [ "**" unary ] ; (* right-assoc; tighter than unary minus *)
postfix       = TYPE_IDENT "." IDENT type_args call { postfix_op }
              | primary { postfix_op } ;
postfix_op    = call | index | "." ( IDENT | INT | TYPE_IDENT ) | "?" ;
                (* (semantic) lifted chains: a "." on a `T?` looks through the
                   optional only when the chain is consumed by `otherwise`, a
                   postfix "?", an `is` / `match` subject, or a position whose
                   expected type is optional (§8.1); it short-circuits on the
                   first `none` and yields one flattened `?` level. Elsewhere a
                   "." on a `T?` is an error. There is no `?.` token. *)
                (* `. TYPE_IDENT`: qualified names such as `Shape.Circle(2.0)`
                   and `time.Duration` *)
                (* `TYPE_IDENT . IDENT type_args call`: explicit type arguments
                   only on an associated-function call whose receiver is a
                   capitalised type name, as in `Channel.new<Int>(capacity: 10)`;
                   a lowercase receiver never takes them, so `a.b < c > (d)` is
                   always a chained comparison. Free generic functions take no
                   explicit type arguments in 0.1 (§10.5). They are normally inferred
                   from an annotation instead:
                   `(tx, rx): (Sender<Int>, Receiver<Int>) = Channel.new(capacity: 10)` *)
                (* (semantic) postfix "?" is an ordinary postfix operator applied
                   left to right, but a "?" immediately followed by "." is an
                   error ("`?.` is not Verse"): write `(f(x)?).email` to
                   propagate then read *)
                (* (semantic) no two postfix "?" in a row: write (x?)? *)
                (* x.f(y) may call a free function f(x, y) (uniform call syntax);
                   a method always wins; ambiguity is an error *)
call          = "(" [ args ] ")" ;
args          = arg { "," arg } [ "," ] ;
arg           = IDENT ":" [ expr ]       (* named; `name:` alone puns `name: name` *)
              | [ "mut" ] expr ;         (* `mut` marks an exclusive borrow *)
index         = "[" ( expr | slice ) "]" ;
slice         = additive ( ".." | "..=" ) [ additive ]    (* open end: only in slices *)
              | ( ".." | "..=" ) additive ;

primary       = literal | IDENT | "self" | TYPE_IDENT [ type_args ]
              | "(" expr ")" | tuple | list | map | set
              | if_expr | match_expr | key_path ;
key_path      = "." ( IDENT | INT ) { "." ( IDENT | INT ) | call } ;
                (* only where an expression starts: after "(", "," or an operator;
                   plain access chains only *)
if_expr       = "if" expr block "else" ( block | if_expr ) ;   (* block form *)
tuple         = "(" arg "," [ arg { "," arg } ] [ "," ] ")" ;
list          = "[" [ elem { "," elem } [ "," ] ] "]" ;
elem          = ".." expr | expr ;                             (* prefix `..` spreads *)
map           = "{" ":" "}"                                    (* empty map; `{}` is an error *)
              | "{" entry { "," entry } [ "," ] "}" ;
entry         = ".." expr | expr ":" expr ;                    (* later keys win *)
set           = "set" "{" [ elem { "," elem } [ "," ] ] "}" ;  (* `set` is reserved *)
literal       = INT | FLOAT | TEXT | RAW_TEXT | CHAR | BYTES
              | "true" | "false" | "none" ;
```

Lexical tokens, informally:

```ebnf
(* The first character decides the kind of name (§3.5).                     *)
TYPE_IDENT  = upper { letter | digit | "_" } ;        (* upper = Unicode Lu or Lt  *)
IDENT       = ( other_letter | "_" ) { letter | digit | "_" } ;
                          (* other_letter = any letter that is not Lu/Lt,
                             including lowercase and uncased letters (e.g. 数) *)
INT         = digits | "0x" hex { hex | "_" }
            | "0o" oct { oct | "_" } | "0b" bin { bin | "_" } ;
FLOAT       = digits "." digits [ exponent ]
            | digits exponent ;                        (* 3.14, 6.02e23, 1e-9 *)
              (* right after ".", a digit sequence is always one INT, never a
                 FLOAT: `t.0.1` is INT 0 then INT 1. `10.seconds()` is INT 10,
                 ".", IDENT; `10.` with no digit after the dot is an error *)
exponent    = ( "e" | "E" ) [ "+" | "-" ] digits ;
digits      = digit { digit | "_" } ;
TEXT        = '"' { char | escape | interp } '"'
            | '"""' NEWLINE { line } '"""' ;
interp      = "{" expr [ ":" format ] "}" ;
                          (* (semantic) a TEXT inside `expr` may not itself
                             contain `interp`: nesting is one level deep *)
              (* the format ":" is the first colon not inside ( ), [ ], { }
                 or a nested text literal; an expression with its own
                 top-level colon, such as `v is n: Int`, needs ( ) *)
RAW_TEXT    = 'r"' { char } '"' | 'r"""' NEWLINE { line } '"""' ;
CHAR        = "'" ( char | escape ) "'" ;
BYTES       = 'b"' { byte_char | escape } '"' ;
COMMENT     = "#" { any } NEWLINE ;
DOC_COMMENT = "##" { any } NEWLINE ;
```

---

