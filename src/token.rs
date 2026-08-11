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
    Mod,
    Fn,
    Semi,
    Colon,
    ColonColon,
    Ret,
    Let,
    Mut,
    Assign,
    Comma,
    Eq,
    Ne,
    Gt,
    Ge,
    Lt,
    Le,
    And,
    Or,
    If,
    Else,
    Arrow,
    While,
    Not,
    Identifier(String),
    Num(i32),
    String(String),
    Bool(bool),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    let mut push_token = |kind| tokens.push(Token { kind });

    while let Some(c) = chars.next() {
        if c.is_whitespace() {
            continue;
        }

        if c == '/' && chars.next_if_eq(&'/').is_some() {
            while chars.next_if(|c2| *c2 != '\n').is_some() {}
            continue;
        }

        match c {
            '+' => push_token(TokenKind::Plus),
            '-' => {
                if chars.next_if_eq(&'>').is_some() {
                    push_token(TokenKind::Arrow);
                } else {
                    push_token(TokenKind::Minus);
                }
            }
            '*' => push_token(TokenKind::Mul),
            '/' => push_token(TokenKind::Div),
            '(' => push_token(TokenKind::LParen),
            ')' => push_token(TokenKind::RParen),
            '{' => push_token(TokenKind::LBrace),
            '}' => push_token(TokenKind::RBrace),
            ';' => push_token(TokenKind::Semi),
            ':' => {
                if chars.next_if_eq(&':').is_some() {
                    push_token(TokenKind::ColonColon);
                } else {
                    push_token(TokenKind::Colon);
                }
            }
            '=' => {
                if chars.next_if(|c2| *c2 == '=').is_some() {
                    push_token(TokenKind::Eq);
                } else {
                    push_token(TokenKind::Assign);
                }
            }
            '!' => {
                if chars.next_if_eq(&'=').is_some() {
                    push_token(TokenKind::Ne);
                } else {
                    push_token(TokenKind::Not);
                }
            }
            '<' => {
                if chars.next_if_eq(&'=').is_some() {
                    push_token(TokenKind::Le);
                } else {
                    push_token(TokenKind::Lt);
                }
            }
            '>' => {
                if chars.next_if_eq(&'=').is_some() {
                    push_token(TokenKind::Ge);
                } else {
                    push_token(TokenKind::Gt);
                }
            }
            '&' => {
                if chars.next_if_eq(&'&').is_some() {
                    push_token(TokenKind::And);
                }
            }
            '|' => {
                if chars.next_if_eq(&'|').is_some() {
                    push_token(TokenKind::Or);
                }
            }
            ',' => push_token(TokenKind::Comma),
            '"' => {
                let mut s = String::new();
                while let Some(c2) = chars.next_if(|c2| c2 != &'"') {
                    s.push(c2);
                }
                chars.next();
                push_token(TokenKind::String(s));
            }
            n if n.is_numeric() => {
                let mut num = 0;
                num += n.to_digit(10).unwrap();

                while let Some(n2) = chars.next_if(|n2| n2.is_numeric()) {
                    num = num * 10 + n2.to_digit(10).unwrap();
                }
                push_token(TokenKind::Num(num as i32));
            }
            n if n.is_alphanumeric() => {
                let mut identifier = String::new();
                identifier.push(c);
                while let Some(c2) = chars.next_if(|c2| c2.is_alphanumeric()) {
                    identifier.push(c2);
                }

                match identifier.as_str() {
                    "mod" => push_token(TokenKind::Mod),
                    "fn" => push_token(TokenKind::Fn),
                    "return" => push_token(TokenKind::Ret),
                    "let" => push_token(TokenKind::Let),
                    "mut" => push_token(TokenKind::Mut),
                    "if" => push_token(TokenKind::If),
                    "else" => push_token(TokenKind::Else),
                    "while" => push_token(TokenKind::While),
                    "true" => push_token(TokenKind::Bool(true)),
                    "false" => push_token(TokenKind::Bool(false)),
                    _ => push_token(TokenKind::Identifier(identifier)),
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

    #[test]
    fn ne() {
        assert_eq!(
            tokenize(r#"let b = a != 2;"#),
            vec![
                Token {
                    kind: TokenKind::Let,
                },
                Token {
                    kind: TokenKind::Identifier("b".to_owned())
                },
                Token {
                    kind: TokenKind::Assign,
                },
                Token {
                    kind: TokenKind::Identifier("a".to_owned())
                },
                Token {
                    kind: TokenKind::Ne,
                },
                Token {
                    kind: TokenKind::Num(2),
                },
                Token {
                    kind: TokenKind::Semi,
                },
            ]
        );
    }
}
