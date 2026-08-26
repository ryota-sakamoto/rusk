use std::{collections::BTreeMap, fmt::Display, str::FromStr};

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
    Add(Box<Node>, Box<Node>, Type),
    Sub(Box<Node>, Box<Node>),
    Mul(Box<Node>, Box<Node>),
    Div(Box<Node>, Box<Node>),
    Num(i32),
    String(String),
    Bool(bool),
    Ret(Box<Node>),
    Let(String, Type, Box<Node>, bool),
    RLet(String, Option<String>),
    Assign(String, Box<Node>),
    Call(String, Vec<Node>, Type),
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

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Type {
    Int,
    Int8,
    Bool,
    Ptr(Box<Type>),
    Struct(String),
}

impl FromStr for Type {
    type Err = ();
    fn from_str(value: &str) -> Result<Self, ()> {
        match value {
            "i32" => Ok(Type::Int),
            "i8" => Ok(Type::Int8),
            "bool" => Ok(Type::Bool),
            _ => Ok(Type::Struct(value.to_owned())),
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Type::Int => "i32".to_owned(),
                Type::Int8 => "i8".to_owned(),
                Type::Bool => "i1".to_owned(),
                Type::Ptr(_) => "ptr".to_owned(),
                Type::Struct(name) => format!("%{name}"),
            }
        )
    }
}

impl Type {
    pub fn inner(&self) -> &Self {
        match self {
            Type::Ptr(v) => v,
            _ => panic!("not ptr"),
        }
    }
}
