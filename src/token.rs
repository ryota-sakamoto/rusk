#[derive(Debug, PartialEq, Eq)]
pub enum TokenKind {
    Plus,
    Minus,
    Mul,
    Div,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Fn,
    Semi,
    Colon,
    Ret,
    Let,
    Assign,
    Comma,
    Eq,
    If,
    Else,
    Arrow,
    Identifier(String),
    Num(i32),
    String(String),
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

        if c == '/' && chars.next_if_eq(&'/').is_some() {
            while chars.next_if(|c2| *c2 != '\n').is_some() {}
            continue;
        }

        match c {
            '+' => tokens.push(Token {
                kind: TokenKind::Plus,
            }),
            '-' => {
                if chars.next_if_eq(&'>').is_some() {
                    tokens.push(Token {
                        kind: TokenKind::Arrow,
                    });
                } else {
                    tokens.push(Token {
                        kind: TokenKind::Minus,
                    })
                }
            }
            '*' => tokens.push(Token {
                kind: TokenKind::Mul,
            }),
            '/' => tokens.push(Token {
                kind: TokenKind::Div,
            }),
            '(' => tokens.push(Token {
                kind: TokenKind::LParen,
            }),
            ')' => tokens.push(Token {
                kind: TokenKind::RParen,
            }),
            '{' => tokens.push(Token {
                kind: TokenKind::LBrace,
            }),
            '}' => tokens.push(Token {
                kind: TokenKind::RBrace,
            }),
            ';' => tokens.push(Token {
                kind: TokenKind::Semi,
            }),
            ':' => tokens.push(Token {
                kind: TokenKind::Colon,
            }),
            '=' => {
                if chars.next_if(|c2| *c2 == '=').is_some() {
                    tokens.push(Token {
                        kind: TokenKind::Eq,
                    });
                } else {
                    tokens.push(Token {
                        kind: TokenKind::Assign,
                    })
                }
            }
            ',' => tokens.push(Token {
                kind: TokenKind::Comma,
            }),
            '"' => {
                let mut s = String::new();
                while let Some(c2) = chars.next_if(|c2| c2 != &'"') {
                    s.push(c2);
                }
                chars.next();

                tokens.push(Token {
                    kind: TokenKind::String(s),
                });
            }
            n if n.is_numeric() => {
                let mut num = 0;
                num += n.to_digit(10).unwrap();

                while let Some(n2) = chars.next_if(|n2| n2.is_numeric()) {
                    num = num * 10 + n2.to_digit(10).unwrap();
                }
                tokens.push(Token {
                    kind: TokenKind::Num(num as i32),
                })
            }
            n if n.is_alphanumeric() => {
                let mut identifier = String::new();
                identifier.push(c);
                while let Some(c2) = chars.next_if(|c2| c2.is_alphanumeric()) {
                    identifier.push(c2);
                }

                match identifier.as_str() {
                    "fn" => tokens.push(Token {
                        kind: TokenKind::Fn,
                    }),
                    "return" => tokens.push(Token {
                        kind: TokenKind::Ret,
                    }),
                    "let" => tokens.push(Token {
                        kind: TokenKind::Let,
                    }),
                    "if" => tokens.push(Token {
                        kind: TokenKind::If,
                    }),
                    "else" => tokens.push(Token {
                        kind: TokenKind::Else,
                    }),
                    _ => tokens.push(Token {
                        kind: TokenKind::Identifier(identifier),
                    }),
                }
            }
            _ => panic!("not allowed: {}", c),
        };
    }

    tokens
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
                    kind: TokenKind::Num(12)
                },
                Token {
                    kind: TokenKind::Plus
                },
                Token {
                    kind: TokenKind::Num(5),
                },
                Token {
                    kind: TokenKind::Minus
                },
                Token {
                    kind: TokenKind::Num(1),
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
                    kind: TokenKind::Fn
                },
                Token {
                    kind: TokenKind::Identifier("main".to_owned()),
                },
                Token {
                    kind: TokenKind::LParen
                },
                Token {
                    kind: TokenKind::RParen
                },
                Token {
                    kind: TokenKind::LBrace
                },
                Token {
                    kind: TokenKind::RBrace
                }
            ]
        );
    }

    #[test]
    fn string() {
        assert_eq!(
            tokenize(r#"let s = "let a = 1";"#),
            vec![
                Token {
                    kind: TokenKind::Let,
                },
                Token {
                    kind: TokenKind::Identifier("s".to_owned())
                },
                Token {
                    kind: TokenKind::Assign,
                },
                Token {
                    kind: TokenKind::String("let a = 1".to_owned()),
                },
                Token {
                    kind: TokenKind::Semi,
                }
            ]
        );
    }
}
