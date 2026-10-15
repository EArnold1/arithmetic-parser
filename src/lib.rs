#[derive(Debug, PartialEq)]
pub enum Token {
    Plus,
    Number(u32),
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
        if ch.is_ascii_digit() {
            let mut digits = String::new();
            while let Some(&(_, d)) = chars.peek() {
                if !d.is_ascii_digit() {
                    break;
                }
                digits.push(d);
                chars.next();
            }
            let n = digits
                .parse::<u32>()
                .map_err(|_| LexError::NumberTooLarge { pos })?;
            tokens.push(Token::Number(n));
        } else if ch == '+' {
            tokens.push(Token::Plus);
            chars.next();
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
    fn lexes_multi_digit_number_plus_number() {
        assert_eq!(
            lexer("12+3"),
            Ok(vec![Token::Number(12), Token::Plus, Token::Number(3)])
        );
    }

    #[test]
    fn lexes_single_digit_addition() {
        assert_eq!(
            lexer("1+2"),
            Ok(vec![Token::Number(1), Token::Plus, Token::Number(2)])
        );
    }

    #[test]
    fn lexes_lone_number_without_operator() {
        assert_eq!(lexer("100"), Ok(vec![Token::Number(100)]));
    }

    #[test]
    fn empty_input_gives_no_tokens() {
        assert_eq!(lexer(""), Ok(vec![]));
    }
    #[test]
    fn skips_whitespace() {
        assert_eq!(
            lexer("1 + 2"),
            Ok(vec![Token::Number(1), Token::Plus, Token::Number(2)])
        );
    }

    #[test]
    fn rejects_unknown_character() {
        assert_eq!(
            lexer("1 $ 2"),
            Err(LexError::UnexpectedChar { ch: '$', pos: 2 })
        );
    }

    #[test]
    fn rejects_huge_number() {
        assert_eq!(
            lexer("99999999999"),
            Err(LexError::NumberTooLarge { pos: 0 })
        );
    }
}
