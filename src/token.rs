#[derive(Debug, PartialEq, Eq)]
pub enum TokenKind {
    PLUS,
    MINUS,
    MUL,
    DIV,
    LPAREN,
    RPAREN,
    LBRACE,
    RBRACE,
    FN,
    IDENTIFIER(String),
    NUM(i32),
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
            '*' => tokens.push(Token {
                kind: TokenKind::MUL,
            }),
            '/' => tokens.push(Token {
                kind: TokenKind::DIV,
            }),
            '(' => tokens.push(Token {
                kind: TokenKind::LPAREN,
            }),
            ')' => tokens.push(Token {
                kind: TokenKind::RPAREN,
            }),
            '{' => tokens.push(Token {
                kind: TokenKind::LBRACE,
            }),
            '}' => tokens.push(Token {
                kind: TokenKind::RBRACE,
            }),
            n if n.is_numeric() => {
                let mut num = 0;
                num += n.to_digit(10).unwrap();

                while let Some(n2) = chars.next_if(|n2| n2.is_numeric()) {
                    num = num * 10 + n2.to_digit(10).unwrap();
                }
                tokens.push(Token {
                    kind: TokenKind::NUM(num as i32),
                })
            }
            _ => {
                let mut identifier = String::new();
                identifier.push(c);
                while let Some(c2) = chars.next_if(|c2| c2.is_alphanumeric()) {
                    identifier.push(c2);
                }

                match identifier.as_str() {
                    "fn" => tokens.push(Token {
                        kind: TokenKind::FN,
                    }),
                    _ => tokens.push(Token {
                        kind: TokenKind::IDENTIFIER(identifier),
                    }),
                }
            }
        };
    }

    return tokens;
}

#[cfg(test)]
mod tests {
    use crate::token::{Token, TokenKind, tokenize};

    #[test]
    fn num() {
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

    #[test]
    fn function() {
        assert_eq!(
            tokenize("fn main() {}"),
            vec![
                Token {
                    kind: TokenKind::FN
                },
                Token {
                    kind: TokenKind::IDENTIFIER("main".to_owned()),
                },
                Token {
                    kind: TokenKind::LPAREN
                },
                Token {
                    kind: TokenKind::RPAREN
                },
                Token {
                    kind: TokenKind::LBRACE
                },
                Token {
                    kind: TokenKind::RBRACE
                }
            ]
        );
    }
}
