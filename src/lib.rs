#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Plus,
    Subtract,
    Divide,
    OpenBracket,
    CloseBracket,
    Multiply,
    Number(i64),
}

#[derive(Debug, PartialEq)]
pub enum LexError {
    UnexpectedChar { ch: char, pos: usize },
    NumberTooLarge { pos: usize },
}

#[derive(Debug, PartialEq)]
pub enum EvalError {
    DivisionByZero,
}

pub fn lexer(input: &str) -> Result<Vec<Token>, LexError> {
    let mut tokens = Vec::new();
    let mut chars = input.char_indices().peekable();

    while let Some(&(pos, ch)) = chars.peek() {
        let single = match ch {
            '+' => Some(Token::Plus),
            '-' => Some(Token::Subtract),
            '*' => Some(Token::Multiply),
            '/' => Some(Token::Divide),
            '(' => Some(Token::OpenBracket),
            ')' => Some(Token::CloseBracket),
            _ => None,
        };

        if let Some(token) = single {
            tokens.push(token);
            chars.next();
        } else if ch.is_ascii_digit() {
            let mut digits = String::new();
            while let Some(&(_, d)) = chars.peek() {
                if !d.is_ascii_digit() {
                    break;
                }
                digits.push(d);
                chars.next();
            }
            let n = digits
                .parse::<i64>()
                .map_err(|_| LexError::NumberTooLarge { pos })?;
            tokens.push(Token::Number(n));
        } else if ch.is_whitespace() {
            chars.next();
        } else {
            return Err(LexError::UnexpectedChar { ch, pos });
        }
    }

    Ok(tokens)
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    UnexpectedToken(Token),
    UnexpectedEnd,
    MissingClosingBracket, // add more as you need them
}

#[derive(Debug, PartialEq)]
pub enum CalcError {
    Lex(LexError),
    Parse(ParseError),
    Eval(EvalError),
}

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

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    // (10-4)/2
    fn expr(&mut self) -> Result<Expr, ParseError> {
        let mut term = self.term()?; // LEFT

        while let Some(token) = self.peek() {
            match token {
                Token::Plus => {
                    self.advance();
                    term = Expr::Binary(Box::new(term), Op::Add, Box::new(self.term()?));
                }
                Token::Subtract => {
                    self.advance();
                    term = Expr::Binary(Box::new(term), Op::Sub, Box::new(self.term()?));
                }
                _ => return Ok(term),
            }
        }

        Ok(term)
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut factor = self.unary()?; // LEFT

        while let Some(token) = self.peek() {
            match token {
                Token::Multiply => {
                    self.advance();
                    factor = Expr::Binary(Box::new(factor), Op::Mul, Box::new(self.unary()?));
                }
                Token::Divide => {
                    self.advance();
                    factor = Expr::Binary(Box::new(factor), Op::Div, Box::new(self.unary()?));
                }
                _ => return Ok(factor),
            }
        }

        Ok(factor)
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        match self.peek() {
            Some(Token::Subtract) => {
                self.advance();
                let value = self.unary()?;
                Ok(Expr::Negate(Box::new(value)))
            }
            _ => self.factor(),
        }
    }

    fn factor(&mut self) -> Result<Expr, ParseError> {
        match self.peek().cloned() {
            Some(Token::Number(n)) => {
                self.advance();
                Ok(Expr::Number(n))
            }
            Some(Token::OpenBracket) => {
                self.advance(); // consume "("
                let value = self.expr()?; // everything inside the brackets

                match self.peek().cloned() {
                    Some(Token::CloseBracket) => {
                        self.advance(); // consume ")"
                        Ok(value)
                    }
                    Some(t) => Err(ParseError::UnexpectedToken(t)),
                    None => Err(ParseError::MissingClosingBracket),
                }
            }
            Some(t) => Err(ParseError::UnexpectedToken(t)),
            None => Err(ParseError::UnexpectedEnd),
        }
    }
}

pub fn eval(tree: Expr) -> Result<i64, EvalError> {
    match tree {
        Expr::Number(n) => Ok(n),
        Expr::Negate(v) => {
            let value = eval(*v)?;
            Ok(-value)
        }
        Expr::Binary(l, op, r) => {
            let left = eval(*l)?;
            let right = eval(*r)?;

            let calculation = match op {
                Op::Add => left + right,
                Op::Sub => left - right,
                Op::Div => {
                    if right == 0 {
                        return Err(EvalError::DivisionByZero);
                    }
                    left / right
                } // add divide by 0 check
                Op::Mul => left * right,
            };

            Ok(calculation)
        }
    }
}

pub fn evaluate(input: &str) -> Result<i64, CalcError> {
    let lexer_result = lexer(input)?;
    let expr_tree = Parser::new(lexer_result).expr()?;

    Ok(eval(expr_tree)?)
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Number(i64),
    Negate(Box<Expr>),
    Binary(Box<Expr>, Op, Box<Expr>),
}

#[derive(Debug, PartialEq)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
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
    fn parser_evaluates_brackets_first() {
        let mut parser = Parser::new(vec![
            Token::OpenBracket,
            Token::Number(10),
            Token::Subtract,
            Token::Number(4),
            Token::CloseBracket,
            Token::Divide,
            Token::Number(2),
        ]);

        let expr = Expr::Binary(
            Box::new(Expr::Binary(
                Box::new(Expr::Number(10)),
                Op::Sub,
                Box::new(Expr::Number(4)),
            )),
            Op::Div,
            Box::new(Expr::Number(2)),
        );

        assert_eq!(parser.expr(), Ok(expr))
    }

    #[test]
    fn parser_respects_multiplication_precedence() {
        let mut parser = Parser::new(vec![
            Token::Number(2),
            Token::Plus,
            Token::Number(3),
            Token::Multiply,
            Token::Number(4),
        ]);

        // 2 + (3 * 4)
        let expr = Expr::Binary(
            Box::new(Expr::Number(2)),
            Op::Add,
            Box::new(Expr::Binary(
                Box::new(Expr::Number(3)),
                Op::Mul,
                Box::new(Expr::Number(4)),
            )),
        );

        assert_eq!(parser.expr(), Ok(expr))
    }

    #[test]
    fn parser_subtracts_after_division() {
        // 10 - 4 / 2 = 10 - (4 / 2)
        let mut parser = Parser::new(vec![
            Token::Number(10),
            Token::Subtract,
            Token::Number(4),
            Token::Divide,
            Token::Number(2),
        ]);

        let expr = Expr::Binary(
            Box::new(Expr::Number(10)),
            Op::Sub,
            Box::new(Expr::Binary(
                Box::new(Expr::Number(4)),
                Op::Div,
                Box::new(Expr::Number(2)),
            )),
        );

        assert_eq!(parser.expr(), Ok(expr))
    }

    #[test]
    fn parser_subtracts_left_to_right() {
        // (10 - 4) - 3, not 10 - (4 - 3)
        let mut parser = Parser::new(vec![
            Token::Number(10),
            Token::Subtract,
            Token::Number(4),
            Token::Subtract,
            Token::Number(3),
        ]);

        let expr = Expr::Binary(
            Box::new(Expr::Binary(
                Box::new(Expr::Number(10)),
                Op::Sub,
                Box::new(Expr::Number(4)),
            )),
            Op::Sub,
            Box::new(Expr::Number(3)),
        );

        assert_eq!(parser.expr(), Ok(expr))
    }

    #[test]
    fn parser_reads_single_number_as_factor() {
        let mut parser = Parser::new(vec![Token::Number(7)]);

        assert_eq!(parser.factor(), Ok(Expr::Number(7)))
    }

    #[test]
    fn parser_multiplies_in_term() {
        let mut parser = Parser::new(vec![Token::Number(2), Token::Multiply, Token::Number(3)]);

        let expr = Expr::Binary(
            Box::new(Expr::Number(2)),
            Op::Mul,
            Box::new(Expr::Number(3)),
        );

        assert_eq!(parser.term(), Ok(expr))
    }

    #[test]
    fn parser_divides_left_to_right() {
        // (8 / 2) / 2
        let mut parser = Parser::new(vec![
            Token::Number(8),
            Token::Divide,
            Token::Number(2),
            Token::Divide,
            Token::Number(2),
        ]);

        let expr = Expr::Binary(
            Box::new(Expr::Binary(
                Box::new(Expr::Number(8)),
                Op::Div,
                Box::new(Expr::Number(2)),
            )),
            Op::Div,
            Box::new(Expr::Number(2)),
        );

        assert_eq!(parser.term(), Ok(expr))
    }

    #[test]
    fn lexes_every_single_char_token() {
        assert_eq!(
            lexer("+-*/()"),
            Ok(vec![
                Token::Plus,
                Token::Subtract,
                Token::Multiply,
                Token::Divide,
                Token::OpenBracket,
                Token::CloseBracket,
            ])
        );
    }

    #[test]
    fn lexes_multi_digit_numbers_and_skips_whitespace() {
        assert_eq!(
            lexer("(12 + 3) / 456"),
            Ok(vec![
                Token::OpenBracket,
                Token::Number(12),
                Token::Plus,
                Token::Number(3),
                Token::CloseBracket,
                Token::Divide,
                Token::Number(456),
            ])
        );
    }

    #[test]
    fn empty_input_gives_no_tokens() {
        assert_eq!(lexer(""), Ok(vec![]));
    }

    #[test]
    fn rejects_unknown_character_with_position() {
        assert_eq!(
            lexer("1 $ 2"),
            Err(LexError::UnexpectedChar { ch: '$', pos: 2 })
        );
    }

    #[test]
    fn rejects_number_too_large_for_i64() {
        assert_eq!(
            lexer("9223372036854775808"),
            Err(LexError::NumberTooLarge { pos: 0 })
        );
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
}
