use crate::{ast::Arg, types::Type};

#[derive(Debug)]
pub struct FunctionMetadata {
    pub args: Vec<Arg>,
    pub ty: Type,
    pub is_public: bool,
}
