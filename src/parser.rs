use crate::lexer::Token;

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

#[derive(Debug, PartialEq)]
pub enum ParseError {
    UnexpectedToken(Token),
    UnexpectedEnd,
    MissingClosingBracket,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedToken(t) => write!(f, "unexpected {t}"),
            ParseError::UnexpectedEnd => write!(f, "unexpected end of input"),
            ParseError::MissingClosingBracket => write!(f, "missing closing bracket"),
        }
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

    /// Parses the whole token stream into an expression tree, rejecting any
    /// tokens left over after a complete expression (e.g. `1 2`).
    pub fn parse(&mut self) -> Result<Expr, ParseError> {
        let tree = self.expr()?;

        match self.peek().cloned() {
            None => Ok(tree),
            Some(t) => Err(ParseError::UnexpectedToken(t)),
        }
    }

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
