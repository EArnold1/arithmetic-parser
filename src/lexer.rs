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

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Plus => write!(f, "'+'"),
            Token::Subtract => write!(f, "'-'"),
            Token::Divide => write!(f, "'/'"),
            Token::Multiply => write!(f, "'*'"),
            Token::OpenBracket => write!(f, "'('"),
            Token::CloseBracket => write!(f, "')'"),
            Token::Number(n) => write!(f, "number {n}"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum LexError {
    UnexpectedChar { ch: char, pos: usize },
    NumberTooLarge { pos: usize },
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LexError::UnexpectedChar { ch, pos } => {
                write!(f, "unexpected character '{ch}' at position {pos}")
            }
            LexError::NumberTooLarge { pos } => {
                write!(f, "number at position {pos} is too large")
            }
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
