use crate::ast::Arg;
use crate::hir::Type;

#[derive(Debug)]
pub struct FunctionMetadata {
    pub args: Vec<Arg>,
    pub ty: Type,
}
