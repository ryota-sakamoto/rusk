use std::collections::{BTreeMap, HashMap};

use crate::ast::{Function, Node};
use crate::hir::{EnumVariant, Node as HirNode, StructField, Type};
use crate::scope::ScopeMap;
use crate::semantic::types::FunctionMetadata;
use crate::semantic::utils::parse_arg;

pub struct FunctionAnalyzer<'a> {
    functions: &'a HashMap<String, FunctionMetadata>,
    let_map: ScopeMap<&'a str, LetMetadata>,
    strings: &'a mut Vec<String>,
    struct_map: &'a BTreeMap<String, BTreeMap<String, StructField>>,
    enum_map: &'a HashMap<String, HashMap<String, EnumVariant>>,
    is_match_condition: bool,
    mod_name: Option<String>,
}

#[derive(Debug)]
struct LetMetadata {
    is_mut: bool,
    pub ty: Type,
}

impl<'a> FunctionAnalyzer<'a> {
    pub fn new(
        function: &'a Function,
        functions: &'a HashMap<String, FunctionMetadata>,
        strings: &'a mut Vec<String>,
        struct_map: &'a BTreeMap<String, BTreeMap<String, StructField>>,
        enum_map: &'a HashMap<String, HashMap<String, EnumVariant>>,
        impl_name: Option<String>,
        mod_name: Option<String>,
    ) -> Self {
        let mut let_map = ScopeMap::new();
        let_map.new_stack();
        for arg in &function.args {
            let a = parse_arg(arg, impl_name.clone());
            let_map.insert(
                arg.name.as_str(),
                LetMetadata {
                    is_mut: arg.is_mut,
                    ty: a.ty,
                },
            );
        }

        Self {
            functions,
            let_map,
            strings,
            struct_map,
            enum_map,
            is_match_condition: false,
            mod_name,
        }
    }

    pub fn analyze_node(&mut self, node: &'a Node) -> HirNode {
        match node {
            Node::Add(l, r) => {
                let ln = self.analyze_node(l);
                let rn = self.analyze_node(r);

                let ln_ty = self.type_of(&ln);
                let rn_ty = self.type_of(&rn);
                if ln_ty != rn_ty {
                    panic!("expected {}, found {}", ln_ty, rn_ty);
                }

                HirNode::Add(Box::new(ln), Box::new(rn), ln_ty)
            }
            Node::Sub(l, r) => HirNode::Sub(
                Box::new(self.analyze_node(l)),
                Box::new(self.analyze_node(r)),
            ),
            Node::Mul(l, r) => HirNode::Mul(
                Box::new(self.analyze_node(l)),
                Box::new(self.analyze_node(r)),
            ),
            Node::Div(l, r) => HirNode::Div(
                Box::new(self.analyze_node(l)),
                Box::new(self.analyze_node(r)),
            ),
            Node::Let(name, ty, node, is_mut) => {
                let rn = self.analyze_node(node);
                let actual_ty = ty
                    .clone()
                    .and_then(|ty| ty.parse::<Type>().ok())
                    .unwrap_or(self.type_of(&rn));

                self.let_map.insert(
                    name,
                    LetMetadata {
                        is_mut: *is_mut,
                        ty: actual_ty.clone(),
                    },
                );
                HirNode::Let(name.clone(), actual_ty, Box::new(rn), *is_mut)
            }
            Node::FieldAccess(node, field) => {
                let rn = self.analyze_node(node);
                let ty = self.type_of(&rn);

                let struct_field = self
                    .struct_map
                    .get(ty.to_string().strip_prefix("%").unwrap())
                    .and_then(|m| m.get(field))
                    .unwrap();

                HirNode::FieldAccess(Box::new(rn), struct_field.index, struct_field.ty.clone())
            }
            Node::Path(items) => {
                if items.len() == 1 {
                    let name = &items[0];
                    if self.let_map.get(&name.as_str()).is_none() {
                        panic!("{:?} is not defined", name);
                    }
                    HirNode::RLet(
                        name.clone(),
                        self.let_map.get(&name.as_str()).unwrap().ty.clone(),
                    )
                } else if items.len() == 2 {
                    let name = &items[0];
                    let variant = &items[1];

                    let m = self
                        .enum_map
                        .get(name)
                        .unwrap_or_else(|| panic!("cannot find type {:?}", name));
                    if !m.contains_key(variant) {
                        panic!("cannot find variant {:?} in {:?}", variant, name);
                    }

                    HirNode::Enum(name.clone(), variant.clone(), Vec::new())
                } else {
                    unimplemented!()
                }
            }
            Node::PathCall(identifiers, args) => {
                let name = identifiers.join("::");
                // TODO: check libc functions
                if name == "printf" {
                    return HirNode::Call(
                        name.clone(),
                        args.iter().map(|v| self.analyze_node(v)).collect(),
                        Type::Int,
                    );
                }

                if self.enum_map.contains_key(&identifiers[0]) {
                    if !self.enum_map[&identifiers[0]].contains_key(&identifiers[1]) {
                        unimplemented!()
                    }

                    let e = &self.enum_map[&identifiers[0]][&identifiers[1]];
                    if self.is_match_condition {
                        let mut fields = Vec::new();
                        for (index, a) in args.iter().enumerate() {
                            let p = self.extract_node_path(a);
                            self.let_map.insert(
                                p,
                                LetMetadata {
                                    is_mut: false,
                                    ty: e.types[index].parse().unwrap(),
                                },
                            );
                            fields.push(p.to_owned());
                        }
                        return HirNode::EnumLabel(
                            identifiers[0].clone(),
                            identifiers[1].clone(),
                            fields,
                        );
                    }

                    if e.types.len() != args.len() {
                        panic!(
                            "{:?} expects {} args, but specified {} args",
                            name,
                            e.types.len(),
                            args.len()
                        );
                    }

                    let mut fields = Vec::new();
                    for a in args {
                        fields.push(self.analyze_node(a));
                    }

                    return HirNode::Enum(identifiers[0].clone(), identifiers[1].clone(), fields);
                }

                let call_name = if identifiers.len() > 1 {
                    identifiers.join("::")
                } else {
                    format!(
                        "{}{}",
                        self.mod_name
                            .clone()
                            .map_or("".to_owned(), |mod_name| format!("{mod_name}::")),
                        name
                    )
                };

                let f = self
                    .functions
                    .get(call_name.as_str())
                    .unwrap_or_else(|| panic!("{:?} is not defined", name));

                if identifiers.len() > 1 && !f.is_public {
                    panic!("{:?} is not public", name);
                }

                if f.args.len() != args.len() {
                    panic!(
                        "{:?} expects {} args, but specified {} args",
                        name,
                        f.args.len(),
                        args.len()
                    );
                }

                HirNode::Call(
                    call_name,
                    args.iter().map(|v| self.analyze_node(v)).collect(),
                    f.ty.clone(),
                )
            }
            Node::MethodCall(node, method, args) => {
                let s = self.analyze_node(node);
                let s_ty = self.type_of(&s);
                let name = format!(
                    "{}::{}",
                    s_ty.to_string().strip_prefix("%").unwrap(),
                    method
                );

                let f = self
                    .functions
                    .get(name.as_str())
                    .unwrap_or_else(|| panic!("{:?} is not defined", name));

                if f.args.len() - 1 != args.len() {
                    panic!(
                        "{:?} expects {} args, but specified {} args",
                        name,
                        f.args.len() - 1,
                        args.len()
                    );
                }

                let mut call_args = Vec::new();
                call_args.push(s);
                call_args.extend(args.iter().map(|v| self.analyze_node(v)));

                HirNode::Call(name.clone(), call_args, f.ty.clone())
            }
            Node::Assign(s, b) => {
                let ln = self.analyze_node(s);
                let rn = self.analyze_node(b);

                let name = self.get_let_name(&ln);

                let v = self
                    .let_map
                    .get(&name.as_str())
                    .unwrap_or_else(|| panic!("{:?} is not defined", name));
                if !v.is_mut {
                    panic!("{:?} should be mut", name);
                }

                let ln_ty = self.type_of(&ln);
                let rn_ty = self.type_of(&rn);
                if ln_ty != rn_ty {
                    panic!("expected {}, found {}", ln_ty, rn_ty);
                }

                HirNode::Assign(Box::new(ln), Box::new(rn))
            }
            Node::If(l, r, e) => HirNode::If(
                Box::new(self.analyze_node(l)),
                Box::new(self.analyze_node(r)),
                e.as_ref().map(|e| Box::new(self.analyze_node(e))),
            ),
            Node::Comparison(ty, l, r) => HirNode::Comparison(
                ty.clone(),
                Box::new(self.analyze_node(l)),
                Box::new(self.analyze_node(r)),
            ),
            Node::While(l, r) => HirNode::While(
                Box::new(self.analyze_node(l)),
                Box::new(self.analyze_node(r)),
            ),
            Node::Break => HirNode::Break,
            Node::Continue => HirNode::Continue,
            Node::Block(b) => {
                self.let_map.new_stack();
                let block = HirNode::Block(b.iter().map(|v| self.analyze_node(v)).collect());
                self.let_map.drop_stack();

                block
            }
            Node::Ret(r) => HirNode::Ret(Box::new(self.analyze_node(r))),
            Node::And(l, r) => {
                let ln = self.analyze_node(l);
                let rn = self.analyze_node(r);

                let ln_ty = self.type_of(&ln);
                let rn_ty = self.type_of(&rn);
                if ln_ty != rn_ty {
                    panic!("expected {}, found {}", ln_ty, rn_ty);
                }

                HirNode::And(Box::new(ln), Box::new(rn), ln_ty)
            }
            Node::Or(l, r) => {
                let ln = self.analyze_node(l);
                let rn = self.analyze_node(r);

                let ln_ty = self.type_of(&ln);
                let rn_ty = self.type_of(&rn);
                if ln_ty != rn_ty {
                    panic!("expected {}, found {}", ln_ty, rn_ty);
                }

                HirNode::Or(Box::new(ln), Box::new(rn), ln_ty)
            }
            Node::Not(r) => HirNode::Not(Box::new(self.analyze_node(r))),
            Node::Struct(name, fields) => {
                let mut args = Vec::new();
                for (index, (_, f)) in fields.iter().enumerate() {
                    args.push((index, self.analyze_node(f)));
                }

                HirNode::Struct(name.clone(), args)
            }
            Node::Match(l, r) => HirNode::Match(
                Box::new(self.analyze_node(l)),
                r.iter()
                    .map(|(a, b)| {
                        self.is_match_condition = true;
                        let cond = self.analyze_node(a);
                        self.is_match_condition = false;

                        let mut block = Vec::new();
                        if let HirNode::EnumLabel(_, _, fields) = &cond {
                            block.extend(fields.iter().enumerate().map(|(index, f)| {
                                HirNode::Let(
                                    f.to_string(),
                                    Type::Int,
                                    Box::new(HirNode::EnumFieldAccess(
                                        Box::new(self.analyze_node(l)),
                                        index + 1,
                                    )),
                                    false,
                                )
                            }));
                        }

                        block.push(self.analyze_node(b));

                        (cond, HirNode::Block(block))
                    })
                    .collect(),
            ),
            Node::Num(n) => HirNode::Num(*n),
            Node::String(s) => {
                if !self.strings.contains(s) {
                    self.strings.push(s.clone());
                }
                HirNode::String(self.strings.len() - 1)
            }
            Node::Bool(b) => HirNode::Bool(*b),
            Node::Array(data) => {
                let nodes = data
                    .iter()
                    .map(|v| self.analyze_node(v))
                    .collect::<Vec<_>>();
                let ty = self.type_of(&nodes[0]);
                HirNode::Array(nodes, ty)
            }
            Node::ArrayAccess(node, index) => {
                let v = self.analyze_node(node);
                let ty = self.type_of(&v);
                HirNode::ArrayAccess(Box::new(v), Box::new(self.analyze_node(index)), ty.inner())
            }
            Node::Ref(node) => HirNode::Ref(Box::new(self.analyze_node(node))),
            Node::RefMut(node) => HirNode::Ref(Box::new(self.analyze_node(node))),
            Node::Deref(node) => HirNode::Deref(Box::new(self.analyze_node(node))),
            Node::Underscore => HirNode::Underscore,
            Node::Mod(_)
            | Node::StructDef(_)
            | Node::ImplDef(_)
            | Node::EnumDef(_)
            | Node::FunctionDef(_) => {
                unimplemented!()
            }
        }
    }

    fn extract_node_path(&mut self, node: &'a Node) -> &'a str {
        match node {
            Node::Path(p) => p.get(0).unwrap(),
            _ => unimplemented!(),
        }
    }

    fn get_let_name(&mut self, node: &HirNode) -> String {
        match node {
            HirNode::RLet(name, _) => name.to_string(),
            HirNode::ArrayAccess(v, _, _) => self.get_let_name(v),
            HirNode::FieldAccess(v, _, _) => self.get_let_name(v),
            HirNode::Deref(v) => self.get_let_name(v),
            _ => unimplemented!("{:?}", node),
        }
    }

    fn type_of(&self, node: &HirNode) -> Type {
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
            HirNode::Deref(v) => self.type_of(v).inner(),
            _ => panic!("{:?} should be implemented", node),
        }
    }
}
