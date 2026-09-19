use std::collections::{BTreeMap, HashMap};

use crate::{ast::ComparisonType, types::Type};

#[derive(Debug)]
pub struct Program {
    pub functions: Vec<Function>,
    pub strings: Vec<String>,
    pub struct_map: BTreeMap<String, BTreeMap<String, StructField>>,
    pub enum_map: HashMap<String, HashMap<String, EnumVariant>>,
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
    String(usize),
    Bool(bool),
    Ret(Box<Node>),
    Let(String, Type, Box<Node>, bool),
    RLet(String, Type),
    FieldAccess(Box<Node>, usize, Type),
    Assign(Box<Node>, Box<Node>),
    Call(String, Vec<Node>, Type),
    Comparison(ComparisonType, Box<Node>, Box<Node>),
    And(Box<Node>, Box<Node>, Type),
    Or(Box<Node>, Box<Node>, Type),
    If(Box<Node>, Box<Node>, Option<Box<Node>>),
    While(Box<Node>, Box<Node>),
    Break,
    Continue,
    Block(Vec<Node>),
    Not(Box<Node>),
    Struct(String, Vec<(usize, Node)>),
    Enum(String, String, Vec<Node>),
    EnumLabel(String, String, Vec<String>),
    EnumFieldAccess(Box<Node>, usize),
    Match(Box<Node>, Vec<(Node, Node)>),
    Array(Vec<Node>, Type),
    ArrayAccess(Box<Node>, Box<Node>, Type),
    Ref(Box<Node>),
    Deref(Box<Node>),
    Underscore,
}

#[derive(Debug)]
pub struct StructField {
    pub ty: Type,
    pub index: usize,
}

#[derive(Debug)]
pub struct EnumVariant {
    pub index: usize,
    pub types: Vec<String>,
}

#[derive(PartialEq, Eq, Debug)]
pub struct Arg {
    pub name: String,
    pub ty: Type,
}
