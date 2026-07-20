#[derive(Debug, PartialEq, Eq)]
pub enum TokenKind {
    PLUS,
    MINUS,
    NUM(u32),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c.is_whitespace() {
            continue;
        }
        match c {
            '+' => tokens.push(Token {
                kind: TokenKind::PLUS,
            }),
            '-' => tokens.push(Token {
                kind: TokenKind::MINUS,
            }),
            n if n.is_numeric() => {
                let mut num = 0;
                num += n.to_digit(10).unwrap();

                while let Some(n2) = chars.next_if(|n2| n2.is_numeric()) {
                    num = num * 10 + n2.to_digit(10).unwrap();
                }
                tokens.push(Token {
                    kind: TokenKind::NUM(num),
                })
            }
            _ => {}
        };
    }

    return tokens;
}

#[cfg(test)]
mod tests {
    use crate::token::{Token, TokenKind, tokenize};

    #[test]
    fn a() {
        assert_eq!(
            tokenize("12 + 5 - 1"),
            vec![
                Token {
                    kind: TokenKind::NUM(12)
                },
                Token {
                    kind: TokenKind::PLUS
                },
                Token {
                    kind: TokenKind::NUM(5),
                },
                Token {
                    kind: TokenKind::MINUS
                },
                Token {
                    kind: TokenKind::NUM(1),
                },
            ]
        );
    }
}
