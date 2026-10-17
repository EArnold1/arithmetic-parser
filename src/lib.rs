mod eval;
mod lexer;
mod parser;

pub use eval::{EvalError, eval};
pub use lexer::{LexError, Token, lexer};
pub use parser::{Expr, Op, ParseError, Parser};

#[derive(Debug, PartialEq)]
pub enum CalcError {
    Lex(LexError),
    Parse(ParseError),
    Eval(EvalError),
}

impl std::fmt::Display for CalcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CalcError::Lex(e) => write!(f, "lex error: {e}"),
            CalcError::Parse(e) => write!(f, "parse error: {e}"),
            CalcError::Eval(e) => write!(f, "eval error: {e}"),
        }
    }
}

impl std::error::Error for CalcError {}

impl From<LexError> for CalcError {
    fn from(e: LexError) -> Self {
        CalcError::Lex(e)
    }
}

impl From<ParseError> for CalcError {
    fn from(e: ParseError) -> Self {
        CalcError::Parse(e)
    }
}

impl From<EvalError> for CalcError {
    fn from(e: EvalError) -> Self {
        CalcError::Eval(e)
    }
}

pub fn evaluate(input: &str) -> Result<i64, CalcError> {
    let lexer_result = lexer(input)?;
    let expr_tree = Parser::new(lexer_result).parse()?;

    Ok(eval(expr_tree)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unary_minus_at_start_negates_number() {
        assert_eq!(evaluate("-5"), Ok(-5));
    }

    #[test]
    fn unary_minus_after_operator_negates_right_operand() {
        assert_eq!(evaluate("3 * -2"), Ok(-6));
    }

    #[test]
    fn evaluates_with_precedence_and_whitespace() {
        assert_eq!(evaluate("2 + 3 * 4"), Ok(14));
        assert_eq!(evaluate("10-4/2"), Ok(8));
        assert_eq!(evaluate("10-4-3"), Ok(3));
    }

    #[test]
    fn evaluates_nested_brackets() {
        assert_eq!(evaluate("((1 + 2) * (3 + 4))"), Ok(21));
    }

    #[test]
    fn evaluate_surfaces_lex_errors() {
        assert_eq!(
            evaluate("1 $ 2"),
            Err(CalcError::Lex(LexError::UnexpectedChar { ch: '$', pos: 2 }))
        );
    }

    #[test]
    fn evaluate_surfaces_parse_errors() {
        assert_eq!(
            evaluate("1 / 0"),
            Err(CalcError::Eval(EvalError::DivisionByZero))
        );
        assert_eq!(
            evaluate("(1 + 2"),
            Err(CalcError::Parse(ParseError::MissingClosingBracket))
        );
        assert_eq!(
            evaluate("1 +"),
            Err(CalcError::Parse(ParseError::UnexpectedEnd))
        );
        assert_eq!(
            evaluate("*"),
            Err(CalcError::Parse(ParseError::UnexpectedToken(
                Token::Multiply
            )))
        );
    }

    #[test]
    fn evaluate_rejects_tokens_left_after_a_complete_expression() {
        assert_eq!(
            evaluate("1 2"),
            Err(CalcError::Parse(ParseError::UnexpectedToken(
                Token::Number(2)
            )))
        );
    }
}
