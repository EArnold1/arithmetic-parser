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
    DivisionByZero,
    MissingClosingBracket, // add more as you need them
}

#[derive(Debug, PartialEq)]
pub enum CalcError {
    Lex(LexError),
    Parse(ParseError),
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

    fn expr(&mut self) -> Result<i64, ParseError> {
        let mut term = self.term()?;

        while let Some(token) = self.peek() {
            match token {
                Token::Plus => {
                    self.advance();
                    term += self.term()?;
                }
                Token::Subtract => {
                    self.advance();
                    term -= self.term()?;
                }
                _ => return Ok(term),
            }
        }

        Ok(term)
    }

    fn term(&mut self) -> Result<i64, ParseError> {
        let mut factor = self.unary()?;

        while let Some(token) = self.peek() {
            match token {
                Token::Multiply => {
                    self.advance();
                    factor *= self.unary()?;
                }
                Token::Divide => {
                    self.advance();
                    let new_factor = self.unary()?;
                    if new_factor == 0 {
                        return Err(ParseError::DivisionByZero);
                    }
                    factor /= new_factor;
                }
                _ => return Ok(factor),
            }
        }

        Ok(factor)
    }

    fn unary(&mut self) -> Result<i64, ParseError> {
        match self.peek() {
            Some(Token::Subtract) => {
                self.advance();
                let value = self.unary()?;
                Ok(-value)
            }
            _ => self.factor(),
        }
    }

    fn factor(&mut self) -> Result<i64, ParseError> {
        match self.peek().cloned() {
            Some(Token::Number(n)) => {
                self.advance();
                Ok(n)
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

pub fn evaluate(input: &str) -> Result<i64, CalcError> {
    Ok(Parser::new(lexer(input)?).expr()?)
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
        assert_eq!(parser.expr(), Ok(3_i64))
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
        assert_eq!(parser.expr(), Ok(14_i64))
    }

    #[test]
    fn parser_subtracts_after_division() {
        // 10 - 4 / 2 = 10 - 2 = 8
        let mut parser = Parser::new(vec![
            Token::Number(10),
            Token::Subtract,
            Token::Number(4),
            Token::Divide,
            Token::Number(2),
        ]);
        assert_eq!(parser.expr(), Ok(8_i64))
    }

    #[test]
    fn parser_subtracts_left_to_right() {
        // (10 - 4) - 3 = 3, not 10 - (4 - 3) = 9
        let mut parser = Parser::new(vec![
            Token::Number(10),
            Token::Subtract,
            Token::Number(4),
            Token::Subtract,
            Token::Number(3),
        ]);
        assert_eq!(parser.expr(), Ok(3_i64))
    }

    #[test]
    fn parser_reads_single_number_as_factor() {
        let mut parser = Parser::new(vec![Token::Number(7)]);

        assert_eq!(parser.factor(), Ok(7_i64))
    }

    #[test]
    fn parser_multiplies_in_term() {
        let mut parser = Parser::new(vec![Token::Number(2), Token::Multiply, Token::Number(3)]);
        assert_eq!(parser.term(), Ok(6_i64))
    }

    #[test]
    fn parser_divides_left_to_right() {
        let mut parser = Parser::new(vec![
            Token::Number(8),
            Token::Divide,
            Token::Number(2),
            Token::Divide,
            Token::Number(2),
        ]);
        assert_eq!(parser.term(), Ok(2_i64))
    }

    #[test]
    fn parser_rejects_division_by_zero() {
        let mut parser = Parser::new(vec![Token::Number(7), Token::Divide, Token::Number(0)]);
        assert_eq!(parser.term(), Err(ParseError::DivisionByZero))
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
            Err(CalcError::Parse(ParseError::DivisionByZero))
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
