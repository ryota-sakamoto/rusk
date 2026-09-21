use crate::ast::Arg;
use crate::hir::{Arg as HirArg, Node as HirNode};
use crate::types::Type;

pub fn parse_arg(arg: &Arg, impl_name: Option<String>) -> HirArg {
    let ty = if let Some(actual_ty) = impl_name.clone()
        && arg.ty == "Self"
    {
        actual_ty.parse().unwrap()
    } else {
        arg.ty.parse().unwrap()
    };
    HirArg {
        name: arg.name.clone(),
        ty: if arg.is_pointer {
            Type::Ptr(Box::new(ty))
        } else {
            ty
        },
    }
}

pub fn type_of(node: &HirNode) -> Type {
    // TODO: fix all type
    match node {
        HirNode::Num(_) => Type::Int,
        HirNode::Bool(_) => Type::Bool,
        HirNode::Add(_, _, ty) => ty.clone(),
        HirNode::Sub(_, _) => Type::Int,
        HirNode::Mul(_, _) => Type::Int,
        HirNode::Div(_, _) => Type::Int,
        HirNode::RLet(_, ty) => ty.clone(),
        HirNode::FieldAccess(_, _, ty) => ty.clone(),
        HirNode::Call(_, _, ty) => ty.clone(),
        HirNode::Struct(name, _) => Type::Struct(name.clone()),
        HirNode::Enum(name, variant, _) => Type::Enum(name.clone(), variant.clone()),
        HirNode::Comparison(_, _, _) => Type::Bool,
        HirNode::Array(data, ty) => Type::Array(Box::new(ty.clone()), data.len()),
        HirNode::ArrayAccess(_, _, ty) => ty.clone(),
        HirNode::Deref(v) => type_of(v).inner(),
        _ => panic!("{:?} should be implemented", node),
    }
}
