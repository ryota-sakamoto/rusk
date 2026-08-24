use std::collections::BTreeMap;

use crate::ast::{Arg, ComparisonType, EnumType, StructType};

#[derive(PartialEq, Eq, Debug)]
pub struct Program {
    pub mods: Vec<String>,
    pub functions: Vec<Function>,
    pub structs: Vec<StructType>,
    pub enums: Vec<EnumType>,
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
    Enum(String, String),
    Match(Box<Node>, Vec<(Node, Node)>),
}
