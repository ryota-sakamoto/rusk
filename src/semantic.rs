use core::panic;
use std::collections::HashMap;

use crate::ast::{Function, Node, Program};

pub fn analyze(program: &Program) {
    let mut analyzer = Analyzer::new(program);
    analyzer.analyze();
}

struct Analyzer<'a> {
    program: &'a Program,
    functions: HashMap<String, &'a Function>,
}

impl<'a> Analyzer<'a> {
    fn new(program: &'a Program) -> Self {
        Self {
            program,
            functions: HashMap::new(),
        }
    }

    fn analyze(&mut self) {
        self.analyze_functions();

        for f in &self.program.functions {
            let mut function_analyzer = FunctionAnalyzer::new(f, &self.functions);
            function_analyzer.analyze();
        }
    }

    fn analyze_functions(&mut self) {
        for f in &self.program.functions {
            if self.functions.contains_key(f.name.as_str()) {
                panic!("{:?} is duplicated", f.name);
            }

            self.functions.insert(f.full_name(), f);
        }

        if !self.functions.contains_key("main") {
            panic!("{:?} is not defined", "main");
        }
    }
}

struct FunctionAnalyzer<'a> {
    function: &'a Function,
    functions: &'a HashMap<String, &'a Function>,
    map: HashMap<&'a str, bool>,
}

impl<'a> FunctionAnalyzer<'a> {
    fn new(function: &'a Function, functions: &'a HashMap<String, &'a Function>) -> Self {
        let mut map = HashMap::new();
        for arg in &function.args {
            map.insert(arg.name.as_str(), false);
        }

        Self {
            function,
            functions,
            map,
        }
    }

    fn analyze(&mut self) {
        self.analyze_node(&self.function.body);
    }

    fn analyze_node(&mut self, node: &'a Node) {
        match node {
            Node::Add(l, r) | Node::Sub(l, r) | Node::Mul(l, r) | Node::Div(l, r) => {
                self.analyze_node(l);
                self.analyze_node(r);
            }
            Node::Let(name, _, node, is_mut) => {
                self.analyze_node(node);
                self.map.insert(name, *is_mut);
            }
            Node::RLet(name, _) => {
                if !self.map.contains_key(&name.as_str()) {
                    panic!("{:?} is not defined", name);
                }
            }
            Node::Call(name, args) => {
                // TODO: check libc functions
                if name == "printf" {
                    return;
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
            }
            Node::Assign(s, b) => {
                self.analyze_node(b);
                let v = self
                    .map
                    .get(s.as_str())
                    .unwrap_or_else(|| panic!("{:?} is not defined", s));
                if !v {
                    panic!("{:?} should be mut", s);
                }
            }
            Node::If(l, r, e) => {
                self.analyze_node(l);
                self.analyze_node(r);
                if let Some(e) = e {
                    self.analyze_node(e);
                }
            }
            Node::Comparison(_, l, r) => {
                self.analyze_node(l);
                self.analyze_node(r);
            }
            Node::While(l, r) => {
                self.analyze_node(l);
                self.analyze_node(r);
            }
            Node::Block(b) => {
                for v in b {
                    self.analyze_node(v);
                }
            }
            Node::Ret(r) => {
                self.analyze_node(r);
            }
            Node::And(l, r) | Node::Or(l, r) => {
                self.analyze_node(l);
                self.analyze_node(r);
            }
            Node::Not(r) => {
                self.analyze_node(r);
            }
            Node::Struct(_, fields) => {
                for f in fields.values() {
                    self.analyze_node(f);
                }
            }
            Node::Num(_) | Node::String(_) | Node::Bool(_) => {
                // noop
            }
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
            functions: Vec::new(),
        });
    }

    #[test]
    #[should_panic(expected = r#""f" is duplicated"#)]
    fn check_duplicated_function() {
        analyze(&Program {
            mods: vec![],
            structs: vec![],
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
            functions: vec![Function {
                name: "main".to_owned(),
                args: Vec::new(),
                body: Node::Block(vec![Node::Let(
                    "a".to_owned(),
                    None,
                    Box::new(Node::Add(
                        Box::new(Node::RLet("b".to_owned(), None)),
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
    #[should_panic(expected = r#""c" is not defined"#)]
    fn check_let_existence_if() {
        analyze(&Program {
            mods: vec![],
            structs: vec![],
            functions: vec![Function {
                name: "main".to_owned(),
                args: Vec::new(),
                body: Node::Block(vec![Node::If(
                    Box::new(Node::Comparison(
                        crate::ast::ComparisonType::Eq,
                        Box::new(Node::RLet("c".to_owned(), None)),
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
            functions: vec![Function {
                name: "main".to_owned(),
                args: Vec::new(),
                body: Node::Block(vec![Node::Ret(Box::new(Node::RLet("d".to_owned(), None)))]),
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
            functions: vec![Function {
                name: "main".to_owned(),
                args: Vec::new(),
                body: Node::Block(vec![Node::Struct(
                    "Test".to_owned(),
                    BTreeMap::from([("a".to_owned(), Node::RLet("e".to_owned(), None))]),
                )]),
                ty: "void".to_owned(),
                mod_name: None,
            }],
        });
    }
}
