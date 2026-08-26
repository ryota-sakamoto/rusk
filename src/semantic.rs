use core::panic;
use std::collections::{BTreeMap, HashMap};

use crate::ast::{Arg, Function, Node, Program};
use crate::hir::{Function as HirFunction, Node as HirNode, Program as HirProgram, Type};

pub fn analyze(program: &Program) -> HirProgram {
    let mut analyzer = Analyzer::new(program);
    analyzer.analyze()
}

struct Analyzer<'a> {
    program: &'a Program,
    functions: HashMap<String, FunctionMetadata>,
}

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

        let mut functions = Vec::new();
        for f in &self.program.functions {
            let mut function_analyzer = FunctionAnalyzer::new(f, &self.functions);

            functions.push(HirFunction {
                name: f.name.clone(),
                args: f.args.clone(),
                body: function_analyzer.analyze_node(&f.body),
                ty: f.ty.clone(),
                mod_name: f.mod_name.clone(),
            });
        }

        HirProgram {
            mods: self.program.mods.clone(),
            enums: self.program.enums.clone(),
            structs: self.program.structs.clone(),
            functions,
        }
    }

    fn analyze_functions(&mut self) {
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
}

struct LetMetadata {
    is_mut: bool,
    ty: Type,
}

impl<'a> FunctionAnalyzer<'a> {
    fn new(function: &'a Function, functions: &'a HashMap<String, FunctionMetadata>) -> Self {
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

        Self { functions, let_map }
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
            Node::RLet(name) => {
                if !self.let_map.contains_key(&name.as_str()) {
                    panic!("{:?} is not defined", name);
                }
                HirNode::RLet(name.clone())
            }
            Node::FieldAccess(node, field) => {
                HirNode::FieldAccess(Box::new(self.analyze_node(node)), field.clone())
            }
            Node::Call(name, args) => {
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
            Node::Assign(s, b) => {
                let rn = self.analyze_node(b);

                let v = self
                    .let_map
                    .get(s.as_str())
                    .unwrap_or_else(|| panic!("{:?} is not defined", s));
                if !v.is_mut {
                    panic!("{:?} should be mut", s);
                }

                let rn_ty = self.type_of(&rn);
                if rn_ty != v.ty {
                    panic!("expected {}, found {}", v.ty, rn_ty);
                }

                HirNode::Assign(s.clone(), Box::new(rn))
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
            Node::Or(l, r) => HirNode::Or(
                Box::new(self.analyze_node(l)),
                Box::new(self.analyze_node(r)),
            ),
            Node::Not(r) => HirNode::Not(Box::new(self.analyze_node(r))),
            Node::Struct(name, fields) => {
                let mut map = BTreeMap::new();
                for (k, f) in fields {
                    map.insert(k.clone(), self.analyze_node(f));
                }

                HirNode::Struct(name.clone(), map)
            }
            Node::Enum(name, variant) => HirNode::Enum(name.clone(), variant.clone()),
            Node::Match(l, r) => HirNode::Match(
                Box::new(self.analyze_node(l)),
                r.iter()
                    .map(|(a, b)| (self.analyze_node(a), self.analyze_node(b)))
                    .collect(),
            ),
            Node::Num(n) => HirNode::Num(*n),
            Node::String(s) => HirNode::String(s.clone()),
            Node::Bool(b) => HirNode::Bool(*b),
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
            HirNode::RLet(_) => Type::Int,
            HirNode::FieldAccess(_, _) => Type::Int,
            HirNode::Call(_, _, ty) => ty.clone(),
            HirNode::Struct(name, _) => Type::Struct(name.clone()),
            HirNode::Enum(_, _) => Type::Int,
            HirNode::Comparison(_, _, _) => Type::Bool,
            _ => panic!("{:?} should be implemented", node),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{
        ast::{Function, Node, Program},
        semantic::analyze,
    };

    #[test]
    #[should_panic(expected = r#""main" is not defined"#)]
    fn check_main() {
        analyze(&Program {
            mods: vec![],
            structs: vec![],
            enums: vec![],
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
            functions: vec![Function {
                name: "main".to_owned(),
                args: Vec::new(),
                body: Node::Block(vec![Node::Let(
                    "a".to_owned(),
                    None,
                    Box::new(Node::Add(
                        Box::new(Node::RLet("b".to_owned())),
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
            functions: vec![Function {
                name: "main".to_owned(),
                args: Vec::new(),
                body: Node::Block(vec![
                    Node::Let("a".to_owned(), None, Box::new(Node::Num(1)), false),
                    Node::Assign("a".to_owned(), Box::new(Node::Num(3))),
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
            functions: vec![Function {
                name: "main".to_owned(),
                args: Vec::new(),
                body: Node::Block(vec![
                    Node::Let("a".to_owned(), None, Box::new(Node::Num(1)), true),
                    Node::Assign("a".to_owned(), Box::new(Node::Bool(false))),
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
            functions: vec![Function {
                name: "main".to_owned(),
                args: Vec::new(),
                body: Node::Block(vec![Node::If(
                    Box::new(Node::Comparison(
                        crate::ast::ComparisonType::Eq,
                        Box::new(Node::RLet("c".to_owned())),
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
            functions: vec![Function {
                name: "main".to_owned(),
                args: Vec::new(),
                body: Node::Block(vec![Node::Ret(Box::new(Node::RLet("d".to_owned())))]),
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
            functions: vec![Function {
                name: "main".to_owned(),
                args: Vec::new(),
                body: Node::Block(vec![Node::Struct(
                    "Test".to_owned(),
                    BTreeMap::from([("a".to_owned(), Node::RLet("e".to_owned()))]),
                )]),
                ty: "void".to_owned(),
                mod_name: None,
            }],
        });
    }
}
