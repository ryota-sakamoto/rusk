use core::panic;
use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;

use crate::ast::{Arg, Function, Node, Program};
use crate::hir::{
    Function as HirFunction, Node as HirNode, Program as HirProgram, StructField, Type,
};

pub fn analyze(program: &Program) -> HirProgram {
    let mut analyzer = Analyzer::new(program);
    analyzer.analyze()
}

struct Analyzer<'a> {
    program: &'a Program,
    functions: HashMap<String, FunctionMetadata>,
}

#[derive(Debug)]
struct FunctionMetadata {
    args: Vec<Arg>,
    ty: Type,
}

impl<'a> Analyzer<'a> {
    fn new(program: &'a Program) -> Self {
        Self {
            program,
            functions: HashMap::new(),
        }
    }

    fn analyze(&mut self) -> HirProgram {
        self.analyze_functions();

        let mut struct_map = BTreeMap::new();
        for s in &self.program.structs {
            let mut fields_map = BTreeMap::new();
            for (index, field) in s.fields.iter().enumerate() {
                fields_map.insert(
                    field.name.clone(),
                    StructField {
                        ty: Type::from_str(&field.ty).unwrap(),
                        index,
                    },
                );
            }

            struct_map.insert(s.name.clone(), fields_map);
        }

        let mut enum_map = HashMap::new();
        for e in &self.program.enums {
            println!("%{} = type {{i32}}", e.name);

            let mut variants_map = HashMap::new();
            for (index, variant) in e.variants.iter().enumerate() {
                variants_map.insert(variant.name.clone(), index);
            }

            enum_map.insert(e.name.clone(), variants_map);
        }

        let mut strings = Vec::new();
        let mut functions = Vec::new();
        for i in &self.program.impls {
            for f in &i.functions {
                let mut function_analyzer =
                    FunctionAnalyzer::new(f, &self.functions, &mut strings, &struct_map, &enum_map);

                functions.push(HirFunction {
                    name: format!("{}::{}", i.name, f.name),
                    args: f.args.clone(),
                    body: function_analyzer.analyze_node(&f.body),
                    ty: f.ty.clone(),
                    mod_name: f.mod_name.clone(),
                });
            }
        }

        for f in &self.program.functions {
            let mut function_analyzer =
                FunctionAnalyzer::new(f, &self.functions, &mut strings, &struct_map, &enum_map);

            functions.push(HirFunction {
                name: f.name.clone(),
                args: f.args.clone(),
                body: function_analyzer.analyze_node(&f.body),
                ty: f.ty.clone(),
                mod_name: f.mod_name.clone(),
            });
        }

        HirProgram {
            functions,
            strings,
            struct_map,
            enum_map,
        }
    }

    fn analyze_functions(&mut self) {
        for i in &self.program.impls {
            for f in &i.functions {
                self.functions.insert(
                    format!("{}::{}", i.name, f.name),
                    FunctionMetadata {
                        args: f.args.clone(),
                        ty: f.ty.parse().unwrap(),
                    },
                );
            }
        }

        for f in &self.program.functions {
            if self.functions.contains_key(f.name.as_str()) {
                panic!("{:?} is duplicated", f.name);
            }

            self.functions.insert(
                f.full_name(),
                FunctionMetadata {
                    args: f.args.clone(),
                    ty: f.ty.parse().unwrap(),
                },
            );
        }

        if !self.functions.contains_key("main") {
            panic!("{:?} is not defined", "main");
        }
    }
}

struct FunctionAnalyzer<'a> {
    functions: &'a HashMap<String, FunctionMetadata>,
    let_map: HashMap<&'a str, LetMetadata>,
    strings: &'a mut Vec<String>,
    struct_map: &'a BTreeMap<String, BTreeMap<String, StructField>>,
    enum_map: &'a HashMap<String, HashMap<String, usize>>,
}

struct LetMetadata {
    is_mut: bool,
    pub ty: Type,
}

impl<'a> FunctionAnalyzer<'a> {
    fn new(
        function: &'a Function,
        functions: &'a HashMap<String, FunctionMetadata>,
        strings: &'a mut Vec<String>,
        struct_map: &'a BTreeMap<String, BTreeMap<String, StructField>>,
        enum_map: &'a HashMap<String, HashMap<String, usize>>,
    ) -> Self {
        let mut let_map = HashMap::new();
        for arg in &function.args {
            let_map.insert(
                arg.name.as_str(),
                LetMetadata {
                    is_mut: false,
                    ty: arg.ty.parse().unwrap(),
                },
            );
        }

        Self {
            functions,
            let_map,
            strings,
            struct_map,
            enum_map,
        }
    }

    fn analyze_node(&mut self, node: &'a Node) -> HirNode {
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
                    if !self.let_map.contains_key(&name.as_str()) {
                        panic!("{:?} is not defined", name);
                    }
                    HirNode::RLet(name.clone(), self.let_map[name.as_str()].ty.clone())
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

                    HirNode::Enum(name.clone(), variant.clone())
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

                let f = self
                    .functions
                    .get(name.as_str())
                    .unwrap_or_else(|| panic!("{:?} is not defined", name));

                if f.args.len() != args.len() {
                    panic!(
                        "{:?} expects {} args, but specified {} args",
                        name,
                        f.args.len(),
                        args.len()
                    );
                }

                HirNode::Call(
                    name.clone(),
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
                    .get(name.as_str())
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
            Node::Block(b) => HirNode::Block(b.iter().map(|v| self.analyze_node(v)).collect()),
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
                    .map(|(a, b)| (self.analyze_node(a), self.analyze_node(b)))
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
        }
    }

    fn get_let_name(&mut self, node: &HirNode) -> String {
        match node {
            HirNode::RLet(name, _) => name.to_string(),
            HirNode::ArrayAccess(v, _, _) => self.get_let_name(v),
            HirNode::FieldAccess(v, _, _) => self.get_let_name(v),
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
            HirNode::RLet(_, ty) => ty.clone(),
            HirNode::FieldAccess(_, _, ty) => ty.clone(),
            HirNode::Call(_, _, ty) => ty.clone(),
            HirNode::Struct(name, _) => Type::Struct(name.clone()),
            HirNode::Enum(_, _) => Type::Int,
            HirNode::Comparison(_, _, _) => Type::Bool,
            HirNode::Array(data, ty) => Type::Array(Box::new(ty.clone()), data.len()),
            HirNode::ArrayAccess(_, _, ty) => ty.clone(),
            _ => panic!("{:?} should be implemented", node),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{
        ast::{EnumType, EnumVariant, Function, Node, Program},
        semantic::analyze,
    };

    #[test]
    #[should_panic(expected = r#""main" is not defined"#)]
    fn check_main() {
        analyze(&Program {
            mods: vec![],
            structs: vec![],
            enums: vec![],
            impls: vec![],
            functions: Vec::new(),
        });
    }

    #[test]
    #[should_panic(expected = r#""f" is duplicated"#)]
    fn check_duplicated_function() {
        analyze(&Program {
            mods: vec![],
            structs: vec![],
            enums: vec![],
            impls: vec![],
            functions: vec![
                Function {
                    name: "f".to_owned(),
                    args: Vec::new(),
                    body: Node::Block(vec![]),
                    ty: "void".to_owned(),
                    mod_name: None,
                },
                Function {
                    name: "f".to_owned(),
                    args: Vec::new(),
                    body: Node::Block(vec![]),
                    ty: "void".to_owned(),
                    mod_name: None,
                },
                Function {
                    name: "main".to_owned(),
                    args: Vec::new(),
                    body: Node::Block(vec![]),
                    ty: "void".to_owned(),
                    mod_name: None,
                },
            ],
        });
    }

    #[test]
    #[should_panic(expected = r#""b" is not defined"#)]
    fn check_let_existence() {
        analyze(&Program {
            mods: vec![],
            structs: vec![],
            enums: vec![],
            impls: vec![],
            functions: vec![Function {
                name: "main".to_owned(),
                args: Vec::new(),
                body: Node::Block(vec![Node::Let(
                    "a".to_owned(),
                    None,
                    Box::new(Node::Add(
                        Box::new(Node::Path(vec!["b".to_owned()])),
                        Box::new(Node::Num(1)),
                    )),
                    false,
                )]),
                ty: "void".to_owned(),
                mod_name: None,
            }],
        });
    }

    #[test]
    #[should_panic(expected = r#""a" should be mut"#)]
    fn check_mut() {
        analyze(&Program {
            mods: vec![],
            structs: vec![],
            enums: vec![],
            impls: vec![],
            functions: vec![Function {
                name: "main".to_owned(),
                args: Vec::new(),
                body: Node::Block(vec![
                    Node::Let("a".to_owned(), None, Box::new(Node::Num(1)), false),
                    Node::Assign(
                        Box::new(Node::Path(vec!["a".to_owned()])),
                        Box::new(Node::Num(3)),
                    ),
                ]),
                ty: "void".to_owned(),
                mod_name: None,
            }],
        });
    }

    #[test]
    #[should_panic(expected = r#"expected i32, found i1"#)]
    fn check_assign_type() {
        analyze(&Program {
            mods: vec![],
            structs: vec![],
            enums: vec![],
            impls: vec![],
            functions: vec![Function {
                name: "main".to_owned(),
                args: Vec::new(),
                body: Node::Block(vec![
                    Node::Let("a".to_owned(), None, Box::new(Node::Num(1)), true),
                    Node::Assign(
                        Box::new(Node::Path(vec!["a".to_owned()])),
                        Box::new(Node::Bool(false)),
                    ),
                ]),
                ty: "void".to_owned(),
                mod_name: None,
            }],
        });
    }

    #[test]
    #[should_panic(expected = r#""c" is not defined"#)]
    fn check_let_existence_if() {
        analyze(&Program {
            mods: vec![],
            structs: vec![],
            enums: vec![],
            impls: vec![],
            functions: vec![Function {
                name: "main".to_owned(),
                args: Vec::new(),
                body: Node::Block(vec![Node::If(
                    Box::new(Node::Comparison(
                        crate::ast::ComparisonType::Eq,
                        Box::new(Node::Path(vec!["c".to_owned()])),
                        Box::new(Node::Num(10)),
                    )),
                    Box::new(Node::Block(vec![])),
                    None,
                )]),
                ty: "void".to_owned(),
                mod_name: None,
            }],
        });
    }

    #[test]
    #[should_panic(expected = r#""d" is not defined"#)]
    fn check_let_existence_return() {
        analyze(&Program {
            mods: vec![],
            structs: vec![],
            enums: vec![],
            impls: vec![],
            functions: vec![Function {
                name: "main".to_owned(),
                args: Vec::new(),
                body: Node::Block(vec![Node::Ret(Box::new(Node::Path(vec!["d".to_owned()])))]),
                ty: "void".to_owned(),
                mod_name: None,
            }],
        });
    }

    #[test]
    #[should_panic(expected = r#""e" is not defined"#)]
    fn check_let_existence_struct() {
        analyze(&Program {
            mods: vec![],
            structs: vec![],
            enums: vec![],
            impls: vec![],
            functions: vec![Function {
                name: "main".to_owned(),
                args: Vec::new(),
                body: Node::Block(vec![Node::Struct(
                    "Test".to_owned(),
                    BTreeMap::from([("a".to_owned(), Node::Path(vec!["e".to_owned()]))]),
                )]),
                ty: "void".to_owned(),
                mod_name: None,
            }],
        });
    }

    #[test]
    #[should_panic(expected = r#"cannot find type "Test""#)]
    fn check_enum_existence() {
        analyze(&Program {
            mods: vec![],
            structs: vec![],
            enums: vec![],
            impls: vec![],
            functions: vec![Function {
                name: "main".to_owned(),
                args: Vec::new(),
                body: Node::Block(vec![Node::Path(vec!["Test".to_owned(), "A".to_owned()])]),
                ty: "void".to_owned(),
                mod_name: None,
            }],
        });
    }

    #[test]
    #[should_panic(expected = r#"cannot find variant "B" in "Test""#)]
    fn check_enum_field_existence() {
        analyze(&Program {
            mods: vec![],
            structs: vec![],
            enums: vec![EnumType {
                name: "Test".to_owned(),
                variants: vec![EnumVariant {
                    name: "A".to_owned(),
                }],
            }],
            impls: vec![],
            functions: vec![Function {
                name: "main".to_owned(),
                args: Vec::new(),
                body: Node::Block(vec![Node::Path(vec!["Test".to_owned(), "B".to_owned()])]),
                ty: "void".to_owned(),
                mod_name: None,
            }],
        });
    }
}
