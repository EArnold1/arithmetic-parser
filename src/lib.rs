#[derive(Debug, PartialEq)]
pub enum Token {
    Plus,
    Number(u32),
}

pub fn lexer(input: &str) -> Vec<Token> {
    let input = input.trim();
    let mut tokens = Vec::new();
    let mut temp = String::new();

    for element in input.chars() {
        if element.is_ascii_digit() {
            temp.push(element);
            continue;
        }

        match element {
            '+' => {
                if !temp.is_empty() {
                    let num = temp.parse::<u32>().unwrap();
                    temp.clear();
                    tokens.push(Token::Number(num));
                }
                tokens.push(Token::Plus);
            }
            _ => unimplemented!(),
        }
    }

    if !temp.is_empty() {
        let num = temp.parse::<u32>().unwrap();
        temp.clear();
        tokens.push(Token::Number(num));
    }

    tokens
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_addition() {
        let input = "12+3";
        let value = lexer(input);

        assert_eq!(
            value,
            vec![Token::Number(12_u32), Token::Plus, Token::Number(3_u32)]
        )
    }

    #[test]
    fn test_add_two_values() {
        let input = "1+2";
        let value = lexer(input);

        assert_eq!(
            value,
            vec![Token::Number(1_u32), Token::Plus, Token::Number(2_u32)]
        )
    }

    #[test]
    fn test_with_no_operator() {
        let input = "100";
        let value = lexer(input);

        assert_eq!(value, vec![Token::Number(100_u32)])
    }

    #[test]
    fn test_with_an_empty_input() {
        let input = "";
        let value = lexer(input);

        assert_eq!(value, vec![])
    }
}
