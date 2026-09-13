use crate::ast::Arg;
use crate::hir::{Arg as HirArg, Type};

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
