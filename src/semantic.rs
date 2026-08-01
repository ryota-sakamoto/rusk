use core::panic;
use std::collections::HashMap;

use crate::ast::{Function, Node, Program};

pub fn analyze(program: &Program) {
    let mut analyzer = Analyzer::new(program);
    analyzer.analyze();
}

struct Analyzer<'a> {
    program: &'a Program,
    functions: HashMap<&'a str, &'a Function>,
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

            self.functions.insert(&f.name, f);
        }

        if !self.functions.contains_key("main") {
            panic!("{:?} is not defined", "main");
        }
    }
}

struct FunctionAnalyzer<'a> {
    function: &'a Function,
    functions: &'a HashMap<&'a str, &'a Function>,
    map: Vec<&'a str>,
}

impl<'a> FunctionAnalyzer<'a> {
    fn new(function: &'a Function, functions: &'a HashMap<&'a str, &'a Function>) -> Self {
        let mut map = Vec::new();
        for arg in &function.args {
            map.push(arg.name.as_str());
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
            Node::Let(name, node) => {
                self.analyze_node(node);
                self.map.push(name);
            }
            Node::RLet(name) => {
                if !self.map.contains(&name.as_str()) {
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
            Node::Block(b) => {
                for v in b {
                    self.analyze_node(v);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{Function, Node, Program},
        semantic::analyze,
    };

    #[test]
    #[should_panic(expected = r#""main" is not defined"#)]
    fn check_main() {
        analyze(&Program {
            functions: Vec::new(),
        });
    }

    #[test]
    #[should_panic(expected = r#""f" is duplicated"#)]
    fn check_duplicated_function() {
        analyze(&Program {
            functions: vec![
                Function {
                    name: "f".to_owned(),
                    args: Vec::new(),
                    body: Node::Block(vec![]),
                    ty: "void".to_owned(),
                },
                Function {
                    name: "f".to_owned(),
                    args: Vec::new(),
                    body: Node::Block(vec![]),
                    ty: "void".to_owned(),
                },
                Function {
                    name: "main".to_owned(),
                    args: Vec::new(),
                    body: Node::Block(vec![]),
                    ty: "void".to_owned(),
                },
            ],
        });
    }

    #[test]
    #[should_panic(expected = r#""b" is not defined"#)]
    fn check_let_existence() {
        analyze(&Program {
            functions: vec![Function {
                name: "main".to_owned(),
                args: Vec::new(),
                body: Node::Block(vec![Node::Let(
                    "a".to_owned(),
                    Box::new(Node::Add(
                        Box::new(Node::RLet("b".to_owned())),
                        Box::new(Node::Num(1)),
                    )),
                )]),
                ty: "void".to_owned(),
            }],
        });
    }
}
