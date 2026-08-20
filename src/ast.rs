use core::panic;
use std::collections::BTreeMap;

use crate::token::{Token, TokenKind};

#[derive(PartialEq, Eq, Debug)]
pub struct Program {
    pub mods: Vec<String>,
    pub functions: Vec<Function>,
    pub structs: Vec<StructType>,
}

#[derive(PartialEq, Eq, Debug)]
pub struct Function {
    pub name: String,
    pub args: Vec<Arg>,
    pub body: Node,
    pub ty: String,
    pub mod_name: Option<String>,
}

impl Function {
    pub fn full_name(&self) -> String {
        format!(
            "{}{}",
            self.mod_name
                .clone()
                .map_or("".to_owned(), |mod_name| format!("{mod_name}::")),
            self.name
        )
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct StructType {
    pub name: String,
    pub fields: Vec<Arg>,
}

#[derive(PartialEq, Eq, Debug)]
pub struct Arg {
    pub name: String,
    pub ty: String,
}

#[derive(PartialEq, Eq, Debug)]
pub enum Node {
    Add(Box<Node>, Box<Node>),
    Sub(Box<Node>, Box<Node>),
    Mul(Box<Node>, Box<Node>),
    Div(Box<Node>, Box<Node>),
    Num(i32),
    String(String),
    Bool(bool),
    Ret(Box<Node>),
    Let(String, Option<String>, Box<Node>, bool),
    RLet(String, Option<String>),
    Assign(String, Box<Node>),
    Call(String, Vec<Node>),
    Comparison(ComparisonType, Box<Node>, Box<Node>),
    And(Box<Node>, Box<Node>),
    Or(Box<Node>, Box<Node>),
    If(Box<Node>, Box<Node>, Option<Box<Node>>),
    While(Box<Node>, Box<Node>),
    Block(Vec<Node>),
    Not(Box<Node>),
    Struct(String, BTreeMap<String, Node>),
}

#[derive(PartialEq, Eq, Debug)]
pub enum ComparisonType {
    Eq,
    Ne,
    Gt,
    Ge,
    Lt,
    Le,
}

pub struct Parser<'a> {
    tokens: &'a Vec<Token>,
    pos: usize,
    allow_struct: bool,
    mod_name: Option<String>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a Vec<Token>, mod_name: Option<String>) -> Self {
        Self {
            tokens,
            pos: 0,
            allow_struct: true,
            mod_name,
        }
    }

    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn consume(&mut self, kind: TokenKind) -> bool {
        if self.peek(kind) {
            self.pos += 1;
            return true;
        }
        false
    }

    fn peek(&self, kind: TokenKind) -> bool {
        if let Some(t) = self.current()
            && t.kind == kind
        {
            return true;
        }

        false
    }

    fn identifier(&mut self) -> Option<String> {
        let result = if let Some(Token {
            kind: TokenKind::Identifier(n),
        }) = self.current()
        {
            Some(n.to_owned())
        } else {
            None
        };
        if result.is_some() {
            self.pos += 1;
        }

        result
    }

    pub fn program(&mut self) -> Program {
        let mut mods = Vec::new();
        let mut functions = Vec::new();
        let mut structs = Vec::new();

        loop {
            if self.consume(TokenKind::Mod) {
                let identifier = self.identifier().expect("should be identifier");
                if !self.consume(TokenKind::Semi) {
                    panic!("should be TokenKind::Semi");
                }

                mods.push(identifier);
            } else if self.consume(TokenKind::Fn) {
                let f = self.function();
                functions.push(f);
            } else if self.consume(TokenKind::Struct) {
                structs.push(self.struct_type());
            } else {
                break;
            }
        }

        Program {
            mods,
            functions,
            structs,
        }
    }

    fn function(&mut self) -> Function {
        let name = self.identifier().expect("should be identifier");
        if !self.consume(TokenKind::LParen) {
            panic!("should be TokenKind::LPAREN");
        }

        let mut args = Vec::new();

        while !self.consume(TokenKind::RParen) {
            let name = self.identifier().expect("should be identifier");
            if !self.consume(TokenKind::Colon) {
                panic!("should be TokenKind::COLON");
            }

            let ty = self.identifier().expect("should be identifier");

            self.consume(TokenKind::Comma);

            args.push(Arg { name, ty });
        }

        let ty = if self.consume(TokenKind::Arrow) {
            self.identifier().expect("should be identifier")
        } else {
            "void".to_owned()
        };

        let body = self.block();

        Function {
            name,
            args,
            body,
            ty,
            mod_name: self.mod_name.clone(),
        }
    }

    fn struct_type(&mut self) -> StructType {
        let name = self.identifier().expect("should be identifier");
        if !self.consume(TokenKind::LBrace) {
            panic!("should be TokenKind::LBrace");
        }

        let mut fields = Vec::new();
        while !self.consume(TokenKind::RBrace) {
            let name = self.identifier().expect("should be identifier");
            if !self.consume(TokenKind::Colon) {
                panic!("should be TokenKind::COLON");
            }

            let ty = self.identifier().expect("should be identifier");

            self.consume(TokenKind::Comma);

            fields.push(Arg { name, ty });
        }

        StructType { name, fields }
    }

    fn stmt(&mut self) -> Node {
        if self.consume(TokenKind::If) {
            let prev_allow_struct = self.allow_struct;
            self.allow_struct = false;
            let node = self.expr();
            self.allow_struct = prev_allow_struct;

            let body = self.block();
            let ebody = if self.consume(TokenKind::Else) {
                if self.peek(TokenKind::If) {
                    Some(Box::new(self.stmt()))
                } else {
                    Some(Box::new(self.block()))
                }
            } else {
                None
            };

            return Node::If(Box::new(node), Box::new(body), ebody);
        }

        if self.consume(TokenKind::While) {
            let node = self.expr();
            let body = self.block();

            return Node::While(Box::new(node), Box::new(body));
        }

        if self.consume(TokenKind::Ret) {
            let node = self.expr();
            if !self.consume(TokenKind::Semi) {
                panic!("should be TokenKind::SEMI");
            }

            return Node::Ret(Box::new(node));
        }

        if self.consume(TokenKind::Let) {
            let is_mut = self.consume(TokenKind::Mut);
            let identifier = self.identifier().expect("should be identifier");

            let ty = if self.consume(TokenKind::Colon) {
                Some(self.identifier().expect("should be identifier"))
            } else {
                None
            };

            if !self.consume(TokenKind::Assign) {
                panic!("should be TokenKind::ASSIGN");
            }

            let node = self.expr();
            if !self.consume(TokenKind::Semi) {
                panic!("should be TokenKind::SEMI");
            }

            return Node::Let(identifier, ty, Box::new(node), is_mut);
        }

        let node = self.expr();
        if !self.consume(TokenKind::Semi) {
            panic!("should be TokenKind::SEMI");
        }

        node
    }

    fn block(&mut self) -> Node {
        let mut body = Vec::new();

        if !self.consume(TokenKind::LBrace) {
            panic!("should be TokenKind::LBRACE");
        }
        while !self.consume(TokenKind::RBrace) {
            let node = self.stmt();
            body.push(node);
        }

        return Node::Block(body);
    }

    fn expr(&mut self) -> Node {
        let mut node = self.equality();

        if self.consume(TokenKind::And) {
            node = Node::And(Box::new(node), Box::new(self.equality()));
        } else if self.consume(TokenKind::Or) {
            node = Node::Or(Box::new(node), Box::new(self.equality()));
        }

        node
    }

    fn equality(&mut self) -> Node {
        let mut node = self.add();

        for (k, c) in [
            (TokenKind::Eq, ComparisonType::Eq),
            (TokenKind::Ne, ComparisonType::Ne),
            (TokenKind::Gt, ComparisonType::Gt),
            (TokenKind::Ge, ComparisonType::Ge),
            (TokenKind::Lt, ComparisonType::Lt),
            (TokenKind::Le, ComparisonType::Le),
        ] {
            if self.consume(k) {
                node = Node::Comparison(c, Box::new(node), Box::new(self.add()));
                break;
            }
        }

        node
    }

    fn add(&mut self) -> Node {
        let mut node = self.mul();

        loop {
            if self.consume(TokenKind::Plus) {
                node = Node::Add(Box::new(node), Box::new(self.mul()));
            } else if self.consume(TokenKind::Minus) {
                node = Node::Sub(Box::new(node), Box::new(self.mul()));
            } else {
                return node;
            }
        }
    }

    fn mul(&mut self) -> Node {
        let mut node = self.unary();

        loop {
            if self.consume(TokenKind::Mul) {
                node = Node::Mul(Box::new(node), Box::new(self.mul()));
            } else if self.consume(TokenKind::Div) {
                node = Node::Div(Box::new(node), Box::new(self.mul()));
            } else {
                return node;
            }
        }
    }

    fn unary(&mut self) -> Node {
        if self.consume(TokenKind::Plus) {
            // noop
        } else if self.consume(TokenKind::Minus) {
            return Node::Sub(Box::new(Node::Num(0)), Box::new(self.primary()));
        } else if self.consume(TokenKind::Not) {
            return Node::Not(Box::new(self.primary()));
        }

        self.primary()
    }

    fn primary(&mut self) -> Node {
        if let Some(identifier) = self.identifier() {
            let mod_function = if self.consume(TokenKind::ColonColon) {
                Some(self.identifier().expect("should be identifier"))
            } else {
                None
            };

            if self.consume(TokenKind::LParen) {
                let mut args = Vec::new();
                while !self.consume(TokenKind::RParen) {
                    let expr = self.expr();
                    args.push(expr);
                    self.consume(TokenKind::Comma);
                }

                return Node::Call(
                    format!(
                        "{identifier}{}",
                        mod_function.map_or(String::new(), |v| format!("::{v}"))
                    ),
                    args,
                );
            }

            if self.allow_struct && self.consume(TokenKind::LBrace) {
                let mut fields = BTreeMap::new();
                while !self.consume(TokenKind::RBrace) {
                    let name = self.identifier().expect("should be identifier");
                    if !self.consume(TokenKind::Colon) {
                        panic!("should be TokenKind::COLON");
                    }

                    let value = self.expr();
                    self.consume(TokenKind::Comma);

                    fields.insert(name, value);
                }

                return Node::Struct(identifier, fields);
            }

            if self.consume(TokenKind::Assign) {
                return Node::Assign(identifier, Box::new(self.expr()));
            }

            let field = if self.consume(TokenKind::Dot) {
                Some(self.identifier().expect("should be identifier"))
            } else {
                None
            };

            return Node::RLet(identifier, field);
        }

        if self.consume(TokenKind::LParen) {
            let node = self.expr();
            if !self.consume(TokenKind::RParen) {
                panic!("should be TokenKind::RPAREN")
            }
            return node;
        }

        let parsed_node = match self.current().map(|t| &t.kind) {
            Some(TokenKind::Num(n)) => Some(Node::Num(*n)),
            Some(TokenKind::String(s)) => Some(Node::String(s.clone())),
            Some(TokenKind::Bool(b)) => Some(Node::Bool(*b)),
            _ => None,
        };

        if let Some(node) = parsed_node {
            self.pos += 1;
            return node;
        }

        panic!("should be Token, but {:?}", self.current());
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{ComparisonType, Node, Parser},
        token::{Token, TokenKind},
    };

    #[test]
    fn expr() {
        let tests = [
            (
                vec![
                    Token {
                        kind: TokenKind::Num(12),
                    },
                    Token {
                        kind: TokenKind::Plus,
                    },
                    Token {
                        kind: TokenKind::Num(5),
                    },
                    Token {
                        kind: TokenKind::Minus,
                    },
                    Token {
                        kind: TokenKind::Num(1),
                    },
                ],
                Node::Sub(
                    Box::new(Node::Add(Box::new(Node::Num(12)), Box::new(Node::Num(5)))),
                    Box::new(Node::Num(1)),
                ),
            ),
            (
                vec![
                    Token {
                        kind: TokenKind::Identifier("a".to_owned()),
                    },
                    Token {
                        kind: TokenKind::Eq,
                    },
                    Token {
                        kind: TokenKind::Num(1),
                    },
                ],
                Node::Comparison(
                    ComparisonType::Eq,
                    Box::new(Node::RLet("a".to_owned(), None)),
                    Box::new(Node::Num(1)),
                ),
            ),
        ];

        for (tokens, expected) in tests {
            let mut parser = Parser::new(&tokens, None);
            assert_eq!(parser.expr(), expected);
        }
    }

    #[test]
    fn primary() {
        let tests = [
            (
                vec![Token {
                    kind: TokenKind::Num(1),
                }],
                Node::Num(1),
            ),
            (
                vec![
                    Token {
                        kind: TokenKind::Identifier("f".to_owned()),
                    },
                    Token {
                        kind: TokenKind::LParen,
                    },
                    Token {
                        kind: TokenKind::Num(5),
                    },
                    Token {
                        kind: TokenKind::Comma,
                    },
                    Token {
                        kind: TokenKind::Num(3),
                    },
                    Token {
                        kind: TokenKind::RParen,
                    },
                ],
                Node::Call("f".to_owned(), vec![Node::Num(5), Node::Num(3)]),
            ),
        ];

        for (tokens, expected) in tests {
            let mut parser = Parser::new(&tokens, None);
            assert_eq!(parser.primary(), expected);
        }
    }
}
