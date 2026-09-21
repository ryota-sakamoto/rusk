use std::collections::{BTreeMap, HashMap};

use crate::ast::{Node, Program};
use crate::hir::{EnumVariant, Function as HirFunction, Program as HirProgram, StructField};
use crate::semantic::function::FunctionAnalyzer;
use crate::semantic::types::FunctionMetadata;
use crate::semantic::utils::parse_arg;
use crate::types::Type;

mod function;
mod types;
mod utils;

pub fn analyze(program: &Program) -> HirProgram {
    let mut analyzer = Analyzer::new(program);
    analyzer.analyze()
}

struct Analyzer<'a> {
    program: &'a Program,
    functions: HashMap<String, FunctionMetadata>,
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
        let mut strings = Vec::new();
        let mut enum_map = HashMap::new();
        let mut const_map = HashMap::new();

        let mut functions = Vec::new();
        for m in &self.program.modules {
            for n in &m.nodes {
                match n {
                    Node::StructDef(s) => {
                        let mut fields_map = BTreeMap::new();
                        for (index, field) in s.fields.iter().enumerate() {
                            fields_map.insert(
                                field.name.clone(),
                                StructField {
                                    ty: field.ty.clone(),
                                    index,
                                },
                            );
                        }

                        struct_map.insert(s.name.clone(), fields_map);
                    }
                    Node::ImplDef(i) => {
                        for f in &i.functions {
                            let mut function_analyzer = FunctionAnalyzer::new(
                                f,
                                &self.functions,
                                &mut strings,
                                &struct_map,
                                &enum_map,
                                &const_map,
                                Some(i.name.clone()),
                                m.name.clone(),
                            );

                            functions.push(HirFunction {
                                name: format!("{}::{}", i.name, f.name),
                                args: f
                                    .args
                                    .iter()
                                    .map(|arg| parse_arg(arg, Some(i.name.clone())))
                                    .collect(),
                                body: function_analyzer.analyze_node(&f.body),
                                ty: f.ty.clone(),
                                mod_name: f.mod_name.clone(),
                            });
                        }
                    }
                    Node::EnumDef(e) => {
                        let mut variants_map = HashMap::new();
                        for (index, variant) in e.variants.iter().enumerate() {
                            variants_map.insert(
                                variant.name.clone(),
                                EnumVariant {
                                    index,
                                    types: variant.types.clone(),
                                },
                            );
                        }

                        enum_map.insert(e.name.clone(), variants_map);
                    }
                    Node::ConstDef(name, ty, value) => {
                        let v = Self::evaluate_const(value, &const_map);
                        let v_ty = match v {
                            Node::Num(_) => Type::Int,
                            Node::Bool(_) => Type::Bool,
                            _ => unimplemented!(),
                        };
                        if ty != &v_ty {
                            panic!("expected {}, found {}", ty, v_ty);
                        }

                        const_map.insert(name.clone(), v);
                    }
                    _ => {}
                }
            }
        }

        for m in &self.program.modules {
            for node in &m.nodes {
                if let Node::FunctionDef(f) = node {
                    let mut function_analyzer = FunctionAnalyzer::new(
                        f,
                        &self.functions,
                        &mut strings,
                        &struct_map,
                        &enum_map,
                        &const_map,
                        None,
                        m.name.clone(),
                    );

                    functions.push(HirFunction {
                        name: f.name.clone(),
                        args: f.args.iter().map(|arg| parse_arg(arg, None)).collect(),
                        body: function_analyzer.analyze_node(&f.body),
                        ty: f.ty.clone(),
                        mod_name: f.mod_name.clone(),
                    });
                }
            }
        }

        HirProgram {
            functions,
            strings,
            struct_map,
            enum_map,
        }
    }

    fn analyze_functions(&mut self) {
        for m in &self.program.modules {
            for node in &m.nodes {
                match node {
                    Node::ImplDef(i) => {
                        for f in &i.functions {
                            self.functions.insert(
                                format!("{}::{}", i.name, f.name),
                                FunctionMetadata {
                                    args: f.args.clone(),
                                    ty: f.ty.parse().unwrap(),
                                    is_public: f.is_public,
                                },
                            );
                        }
                    }
                    _ => {}
                }
            }
        }

        for m in &self.program.modules {
            for node in &m.nodes {
                if let Node::FunctionDef(f) = node {
                    if self.functions.contains_key(f.name.as_str()) {
                        panic!("{:?} is duplicated", f.name);
                    }

                    self.functions.insert(
                        f.full_name(),
                        FunctionMetadata {
                            args: f.args.clone(),
                            ty: f.ty.parse().unwrap(),
                            is_public: f.is_public,
                        },
                    );
                }
            }
        }

        if !self.functions.contains_key("main") {
            panic!("{:?} is not defined", "main");
        }
    }

    fn evaluate_const(node: &Node, const_map: &HashMap<String, Node>) -> Node {
        match node {
            Node::Num(n) => Node::Num(*n),
            Node::Bool(b) => Node::Bool(*b),
            Node::Add(l, r)
                if let Node::Num(a) = Self::evaluate_const(l, const_map)
                    && let Node::Num(b) = Self::evaluate_const(r, const_map) =>
            {
                Node::Num(a + b)
            }
            Node::Mul(l, r)
                if let Node::Num(a) = Self::evaluate_const(l, const_map)
                    && let Node::Num(b) = Self::evaluate_const(r, const_map) =>
            {
                Node::Num(a * b)
            }
            Node::Path(p) if p.len() == 1 => {
                let v = const_map.get(&p[0]).unwrap();
                v.clone()
            }
            _ => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, vec};

    use crate::{
        ast::{EnumType, EnumVariant, Function, Module, Node, Program},
        semantic::analyze,
        types::Type,
    };

    #[test]
    #[should_panic(expected = r#""main" is not defined"#)]
    fn check_main() {
        analyze(&Program { modules: vec![] });
    }

    #[test]
    #[should_panic(expected = r#""f" is duplicated"#)]
    fn check_duplicated_function() {
        analyze(&Program {
            modules: vec![Module {
                name: None,
                nodes: vec![
                    Node::FunctionDef(Box::new(Function {
                        name: "f".to_owned(),
                        args: Vec::new(),
                        body: Node::Block(vec![]),
                        ty: "void".to_owned(),
                        mod_name: None,
                        is_public: false,
                    })),
                    Node::FunctionDef(Box::new(Function {
                        name: "f".to_owned(),
                        args: Vec::new(),
                        body: Node::Block(vec![]),
                        ty: "void".to_owned(),
                        mod_name: None,
                        is_public: false,
                    })),
                    Node::FunctionDef(Box::new(Function {
                        name: "main".to_owned(),
                        args: Vec::new(),
                        body: Node::Block(vec![]),
                        ty: "void".to_owned(),
                        mod_name: None,
                        is_public: false,
                    })),
                ],
            }],
        });
    }

    #[test]
    #[should_panic(expected = r#""b" is not defined"#)]
    fn check_let_existence() {
        analyze(&Program {
            modules: vec![Module {
                name: None,
                nodes: vec![Node::FunctionDef(Box::new(Function {
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
                    is_public: false,
                }))],
            }],
        });
    }

    #[test]
    #[should_panic(expected = r#""a" should be mut"#)]
    fn check_mut() {
        analyze(&Program {
            modules: vec![Module {
                name: None,
                nodes: vec![Node::FunctionDef(Box::new(Function {
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
                    is_public: false,
                }))],
            }],
        });
    }

    #[test]
    #[should_panic(expected = r#"expected i32, found i1"#)]
    fn check_assign_type() {
        analyze(&Program {
            modules: vec![Module {
                name: None,
                nodes: vec![Node::FunctionDef(Box::new(Function {
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
                    is_public: false,
                }))],
            }],
        });
    }

    #[test]
    #[should_panic(expected = r#""b" is not defined"#)]
    fn check_let_existence_scope() {
        analyze(&Program {
            modules: vec![Module {
                name: None,
                nodes: vec![Node::FunctionDef(Box::new(Function {
                    name: "main".to_owned(),
                    args: Vec::new(),
                    body: Node::Block(vec![
                        Node::Let("a".to_owned(), None, Box::new(Node::Num(1)), false),
                        Node::Block(vec![Node::Let(
                            "b".to_owned(),
                            None,
                            Box::new(Node::Num(1)),
                            false,
                        )]),
                        Node::Let(
                            "c".to_owned(),
                            None,
                            Box::new(Node::Add(
                                Box::new(Node::Path(vec!["a".to_owned()])),
                                Box::new(Node::Path(vec!["b".to_owned()])),
                            )),
                            false,
                        ),
                    ]),
                    ty: "void".to_owned(),
                    mod_name: None,
                    is_public: false,
                }))],
            }],
        });
    }

    #[test]
    #[should_panic(expected = r#""c" is not defined"#)]
    fn check_let_existence_if() {
        analyze(&Program {
            modules: vec![Module {
                name: None,
                nodes: vec![Node::FunctionDef(Box::new(Function {
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
                    is_public: false,
                }))],
            }],
        });
    }

    #[test]
    #[should_panic(expected = r#""d" is not defined"#)]
    fn check_let_existence_return() {
        analyze(&Program {
            modules: vec![Module {
                name: None,
                nodes: vec![Node::FunctionDef(Box::new(Function {
                    name: "main".to_owned(),
                    args: Vec::new(),
                    body: Node::Block(vec![Node::Ret(Box::new(Node::Path(vec!["d".to_owned()])))]),
                    ty: "void".to_owned(),
                    mod_name: None,
                    is_public: false,
                }))],
            }],
        });
    }

    #[test]
    #[should_panic(expected = r#""e" is not defined"#)]
    fn check_let_existence_struct() {
        analyze(&Program {
            modules: vec![Module {
                name: None,
                nodes: vec![Node::FunctionDef(Box::new(Function {
                    name: "main".to_owned(),
                    args: Vec::new(),
                    body: Node::Block(vec![Node::Struct(
                        "Test".to_owned(),
                        BTreeMap::from([("a".to_owned(), Node::Path(vec!["e".to_owned()]))]),
                    )]),
                    ty: "void".to_owned(),
                    mod_name: None,
                    is_public: false,
                }))],
            }],
        });
    }

    #[test]
    #[should_panic(expected = r#"cannot find type "Test""#)]
    fn check_enum_existence() {
        analyze(&Program {
            modules: vec![Module {
                name: None,
                nodes: vec![Node::FunctionDef(Box::new(Function {
                    name: "main".to_owned(),
                    args: Vec::new(),
                    body: Node::Block(vec![Node::Path(vec!["Test".to_owned(), "A".to_owned()])]),
                    ty: "void".to_owned(),
                    mod_name: None,
                    is_public: false,
                }))],
            }],
        });
    }

    #[test]
    #[should_panic(expected = r#"cannot find variant "B" in "Test""#)]
    fn check_enum_field_existence() {
        analyze(&Program {
            modules: vec![Module {
                name: None,
                nodes: vec![
                    Node::EnumDef(EnumType {
                        name: "Test".to_owned(),
                        variants: vec![EnumVariant {
                            name: "A".to_owned(),
                            types: Vec::new(),
                        }],
                    }),
                    Node::FunctionDef(Box::new(Function {
                        name: "main".to_owned(),
                        args: Vec::new(),
                        body: Node::Block(vec![Node::Path(vec![
                            "Test".to_owned(),
                            "B".to_owned(),
                        ])]),
                        ty: "void".to_owned(),
                        mod_name: None,
                        is_public: false,
                    })),
                ],
            }],
        });
    }

    #[test]
    #[should_panic(expected = r#""Test::A" expects 2 args, but specified 0 args"#)]
    fn check_enum_field_len() {
        analyze(&Program {
            modules: vec![Module {
                name: None,
                nodes: vec![
                    Node::EnumDef(EnumType {
                        name: "Test".to_owned(),
                        variants: vec![EnumVariant {
                            name: "A".to_owned(),
                            types: vec!["i32".to_owned(), "bool".to_owned()],
                        }],
                    }),
                    Node::FunctionDef(Box::new(Function {
                        name: "main".to_owned(),
                        args: Vec::new(),
                        body: Node::Block(vec![Node::PathCall(
                            vec!["Test".to_owned(), "A".to_owned()],
                            vec![],
                        )]),
                        ty: "void".to_owned(),
                        mod_name: None,
                        is_public: false,
                    })),
                ],
            }],
        });
    }

    #[test]
    #[should_panic(expected = r#""test::f" is not public"#)]
    fn check_pub_fn() {
        analyze(&Program {
            modules: vec![
                Module {
                    name: None,
                    nodes: vec![Node::FunctionDef(Box::new(Function {
                        name: "main".to_owned(),
                        args: vec![],
                        body: Node::Block(vec![Node::PathCall(
                            vec!["test".to_owned(), "f".to_owned()],
                            vec![],
                        )]),
                        ty: "void".to_owned(),
                        mod_name: None,
                        is_public: false,
                    }))],
                },
                Module {
                    name: Some("test".to_owned()),
                    nodes: vec![Node::FunctionDef(Box::new(Function {
                        name: "f".to_owned(),
                        args: vec![],
                        body: Node::Block(vec![]),
                        ty: "void".to_owned(),
                        mod_name: Some("test".to_owned()),
                        is_public: false,
                    }))],
                },
            ],
        });
    }

    #[test]
    #[should_panic(expected = r#"expected i32, found i1"#)]
    fn check_const_ty() {
        analyze(&Program {
            modules: vec![Module {
                name: None,
                nodes: vec![
                    Node::ConstDef("A".to_owned(), Type::Int, Box::new(Node::Bool(false))),
                    Node::FunctionDef(Box::new(Function {
                        name: "main".to_owned(),
                        args: vec![],
                        body: Node::Block(vec![]),
                        ty: "void".to_owned(),
                        mod_name: None,
                        is_public: false,
                    })),
                ],
            }],
        });
    }
}
