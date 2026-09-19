use std::{fmt::Display, str::FromStr};

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Type {
    Int,
    Int8,
    Bool,
    Ptr(Box<Type>),
    Struct(String),
    Enum(String, String),
    Array(Box<Type>, usize),
    Void,
}

impl FromStr for Type {
    type Err = ();
    fn from_str(value: &str) -> Result<Self, ()> {
        match value {
            "i32" => Ok(Type::Int),
            "i8" => Ok(Type::Int8),
            "bool" => Ok(Type::Bool),
            "void" => Ok(Type::Void),
            _ => Ok(Type::Struct(value.to_owned())),
        }
    }
}

impl Type {
    pub fn inner(&self) -> Type {
        match self {
            Type::Array(ty, _) => *ty.clone(),
            Type::Ptr(ty) => *ty.clone(),
            _ => unimplemented!("{:?}", self),
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
                Type::Void => "void".to_owned(),
                Type::Ptr(_) => "ptr".to_owned(),
                Type::Struct(name) => format!("%{name}"),
                Type::Enum(name, _) => format!("%{name}"),
                Type::Array(ty, len) => format!("[{} x {}]", len, ty),
            }
        )
    }
}
