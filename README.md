# arithmetic-parser

A small lexer, recursive-descent parser and evaluator for integer arithmetic.

```
$ cargo run -- "2 + 3 * 4"
14
$ cargo run            # starts a REPL; Ctrl-D to quit
> (10 - 4) / 2
3
```

## Pipeline

| Stage | Module | Input | Output |
|---|---|---|---|
| Lex | `lexer.rs` | `&str` | `Vec<Token>` |
| Parse | `parser.rs` | `Vec<Token>` | `Expr` tree |
| Eval | `eval.rs` | `Expr` | `i64` |

`evaluate(input)` in `lib.rs` runs all three and wraps their errors in `CalcError`.

## Terms

**Lexer** (or tokenizer): turns raw text into a flat list of tokens, the smallest meaningful pieces. `"(12 + 3)"` becomes `OpenBracket, Number(12), Plus, Number(3), CloseBracket`. It skips whitespace and rejects characters that don't belong, but knows nothing about whether the tokens make sense together.

**Parser**: reads the token list and checks it follows the grammar below, producing a tree (`Expr`) that records the structure of the expression: which operation applies to which operands. `2 + 3 * 4` becomes `Add(2, Mul(3, 4))`. It does not calculate anything; that is `eval`'s job.

## Grammar

```
expr    = term   { ("+" | "-") term }
term    = unary  { ("*" | "/") unary }
unary   = "-" unary | factor
factor  = NUMBER | "(" expr ")"
```

Each rule is one method on `Parser`. `{ ... }` means "zero or more", and the loops make `+ - * /` left-associative: `10 - 4 - 3` is `(10 - 4) - 3`.

Precedence comes from the nesting of the rules: `term` is called from inside `expr`, so `*` and `/` bind tighter than `+` and `-`. Unary minus binds tighter still and can repeat (`--5`). Brackets return the inner tree without an extra node.

`NUMBER` is one or more ASCII digits that fit in an `i64`; whitespace is ignored.

## Errors

| Error | Raised when |
|---|---|
| `LexError::UnexpectedChar` | the input has a character that is not a digit, operator, bracket or whitespace |
| `LexError::NumberTooLarge` | a literal does not fit in an `i64` |
| `ParseError::UnexpectedToken` | a token is in the wrong place, or is left over after a complete expression (`1 2`) |
| `ParseError::UnexpectedEnd` | the input stops mid-expression (`1 +`) |
| `ParseError::MissingClosingBracket` | a `(` is never closed |
| `EvalError::DivisionByZero` | a divisor evaluates to `0` |
| `EvalError::Overflow` | any step does not fit in an `i64` (`9223372036854775807 + 1`) |

## Known limitation

Very deep bracket nesting (around 100,000 levels) overflows the stack, because the parser and `eval` are recursive.
