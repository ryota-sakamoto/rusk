use crate::token::{Token, TokenKind};

#[derive(PartialEq, Eq, Debug)]
pub enum Node {
    ADD(Box<Node>, Box<Node>),
    SUB(Box<Node>, Box<Node>),
    NUM(u32),
}

pub struct Parser<'a> {
    tokens: &'a Vec<Token>,
    pos: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn consume(&mut self, kind: TokenKind) -> bool {
        if let Some(t) = self.current()
            && t.kind == kind
        {
            self.pos += 1;
            return true;
        }

        return false;
    }

    pub fn expr(&mut self) -> Node {
        let mut node = self.mul();

        loop {
            if self.consume(TokenKind::PLUS) {
                node = Node::ADD(Box::new(node), Box::new(self.mul()));
            } else if self.consume(TokenKind::MINUS) {
                node = Node::SUB(Box::new(node), Box::new(self.mul()));
            } else {
                return node;
            }
        }
    }

    fn mul(&mut self) -> Node {
        return self.primary();
    }

    fn primary(&mut self) -> Node {
        if let Some(t) = self.current()
            && let TokenKind::NUM(n) = t.kind
        {
            self.pos += 1;
            return Node::NUM(n);
        }

        panic!("should be TokenKind::NUM")
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{Node, Parser},
        token::{Token, TokenKind},
    };

    #[test]
    fn expr() {
        let tokens = vec![
            Token {
                kind: TokenKind::NUM(12),
            },
            Token {
                kind: TokenKind::PLUS,
            },
            Token {
                kind: TokenKind::NUM(5),
            },
            Token {
                kind: TokenKind::MINUS,
            },
            Token {
                kind: TokenKind::NUM(1),
            },
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            parser.expr(),
            Node::SUB(
                Box::new(Node::ADD(Box::new(Node::NUM(12)), Box::new(Node::NUM(5)))),
                Box::new(Node::NUM(1))
            ),
        );
    }
}
