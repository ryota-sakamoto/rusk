use core::panic;

use crate::token::{Token, TokenKind};

#[derive(PartialEq, Eq, Debug)]
pub struct Program {
    pub functions: Vec<Function>,
}

#[derive(PartialEq, Eq, Debug)]
pub struct Function {
    pub name: String,
    pub body: Vec<Node>,
}

#[derive(PartialEq, Eq, Debug)]
pub enum Node {
    ADD(Box<Node>, Box<Node>),
    SUB(Box<Node>, Box<Node>),
    MUL(Box<Node>, Box<Node>),
    DIV(Box<Node>, Box<Node>),
    NUM(i32),
    RET(Box<Node>),
    CALL(String, Vec<Node>),
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

    fn identifier(&mut self) -> Option<String> {
        let result = if let Some(Token {
            kind: TokenKind::IDENTIFIER(n),
        }) = self.current()
        {
            Some(n.to_owned())
        } else {
            None
        };
        if result.is_some() {
            self.pos += 1;
        }

        return result;
    }

    pub fn program(&mut self) -> Program {
        let mut functions = Vec::new();

        while self.consume(TokenKind::FN) {
            let f = self.function();
            functions.push(f);
        }

        return Program { functions };
    }

    fn function(&mut self) -> Function {
        let mut body = Vec::new();

        let name = self.identifier().expect("should be identifier");
        if !self.consume(TokenKind::LPAREN) {
            panic!("should be TokenKind::LPAREN");
        } else if !self.consume(TokenKind::RPAREN) {
            panic!("should be TokenKind::RPAREN");
        } else if !self.consume(TokenKind::LBRACE) {
            panic!("should be TokenKind::LBRACE");
        }

        while !self.consume(TokenKind::RBRACE) {
            let node = self.stmt();
            body.push(node);
        }

        return Function { name, body };
    }

    fn stmt(&mut self) -> Node {
        let return_node = self.consume(TokenKind::RET);

        let node = self.expr();
        if !self.consume(TokenKind::SEMI) {
            panic!("should be TokenKind:: SEMI");
        }

        return if return_node {
            Node::RET(Box::new(node))
        } else {
            node
        };
    }

    fn expr(&mut self) -> Node {
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
        let mut node = self.unary();

        loop {
            if self.consume(TokenKind::MUL) {
                node = Node::MUL(Box::new(node), Box::new(self.mul()));
            } else if self.consume(TokenKind::DIV) {
                node = Node::DIV(Box::new(node), Box::new(self.mul()));
            } else {
                return node;
            }
        }
    }

    fn unary(&mut self) -> Node {
        if self.consume(TokenKind::PLUS) {
            // noop
        } else if self.consume(TokenKind::MINUS) {
            return Node::SUB(Box::new(Node::NUM(0)), Box::new(self.primary()));
        }

        return self.primary();
    }

    fn primary(&mut self) -> Node {
        if let Some(identifier) = self.identifier() {
            if !self.consume(TokenKind::LPAREN) {
                panic!("should be TokenKind::LPAREN")
            }

            let mut args = Vec::new();
            if !self.consume(TokenKind::RPAREN) {
                let expr = self.expr();
                if !self.consume(TokenKind::RPAREN) {
                    panic!("should be TokenKind::RPAREN")
                }
                args.push(expr);
            }

            return Node::CALL(identifier, args);
        }

        if self.consume(TokenKind::LPAREN) {
            let node = self.expr();
            if !self.consume(TokenKind::RPAREN) {
                panic!("should be TokenKind::RPAREN")
            }
            return node;
        }

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
